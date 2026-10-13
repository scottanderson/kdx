//! The servers that registered over UDP, and how groups list them.
//!
//! The rules are Tracker 1.110's (`docs/protocol/tracker.md`, "Tracker side").

use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::{Duration, Instant};

use kdx_proto::msg::tracker::ServerListEntry;
use kdx_proto::registration::Registration;

use crate::settings::{Group, DEFAULT_GROUP};

/// Most records one IP address may hold.
pub const MAX_RECORDS_PER_IP: usize = 5;
/// How long a record lasts without a new registration.
pub const EXPIRY: Duration = Duration::from_secs(850);
/// Least time between two sweeps for expired records.
pub const SWEEP_INTERVAL: Duration = Duration::from_secs(600);
/// Longest server name kept.
pub const MAX_NAME_LEN: usize = 64;

/// A registered server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// The registration's source address.
    pub ip: Ipv4Addr,
    /// The registration's server port.
    pub server_port: u32,
    /// The registration's flags.
    pub flags: u16,
    /// Users logged in.
    pub users: u16,
    /// Server name, at most [`MAX_NAME_LEN`] bytes.
    pub name: Vec<u8>,
    /// Server description.
    pub description: Vec<u8>,
    /// The group names the server asked for.
    pub groups: [Vec<u8>; 6],
    /// When the tracker first heard from the server, in milliseconds since 1900.
    pub first_seen: u64,
    last_seen: Instant,
}

impl Record {
    /// Whether a group takes this record.
    fn fits(&self, group: &Group) -> bool {
        (!group.trackers_only || self.flags & Registration::FROM_TRACKER != 0)
            && self.groups.contains(&group.name)
    }

    /// The record as an entry of 703 Tracker Server List.
    pub fn entry(&self) -> ServerListEntry {
        ServerListEntry {
            first_seen: self.first_seen,
            flags: self.flags,
            users: self.users,
            address: SocketAddrV4::new(self.ip, self.server_port as u16),
            name: self.name.clone(),
            description: self.description.clone(),
            extra: Vec::new(),
        }
    }
}

/// What a registration did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Registered {
    /// A new record.
    Added,
    /// A known record, now up to date.
    Updated,
    /// A known record dropped by a registration with no name.
    Removed,
    /// Nothing: no name, or the IP already holds [`MAX_RECORDS_PER_IP`] records.
    Ignored,
}

/// The registered servers.
#[derive(Debug, Clone)]
pub struct Registry {
    records: Vec<Record>,
    last_sweep: Instant,
}

impl Registry {
    /// An empty registry whose first sweep is due [`SWEEP_INTERVAL`] after `now`.
    pub const fn new(now: Instant) -> Self {
        Self {
            records: Vec::new(),
            last_sweep: now,
        }
    }

    /// Records a registration from `ip`, received at `now`, or at `wall` in milliseconds since 1900.
    ///
    /// A known source keeps its first-seen time.
    pub fn register(
        &mut self,
        ip: Ipv4Addr,
        reg: &Registration,
        now: Instant,
        wall: u64,
    ) -> Registered {
        let known = self
            .records
            .iter()
            .position(|r| r.ip == ip && r.server_port == reg.server_port);
        let record = || Record {
            ip,
            server_port: reg.server_port,
            flags: reg.flags,
            users: reg.users,
            name: reg.name[..reg.name.len().min(MAX_NAME_LEN)].to_vec(),
            description: reg.description.clone(),
            groups: reg.groups.clone(),
            first_seen: wall,
            last_seen: now,
        };
        match known {
            Some(at) if reg.name.is_empty() => {
                self.records.remove(at);
                Registered::Removed
            }
            Some(at) => {
                let first_seen = self.records[at].first_seen;
                self.records[at] = Record {
                    first_seen,
                    ..record()
                };
                Registered::Updated
            }
            None if reg.name.is_empty()
                || self.records.iter().filter(|r| r.ip == ip).count() >= MAX_RECORDS_PER_IP =>
            {
                Registered::Ignored
            }
            None => {
                self.records.push(record());
                Registered::Added
            }
        }
    }

    /// Drops records older than [`EXPIRY`], if [`SWEEP_INTERVAL`] has passed since the last sweep.
    pub fn sweep(&mut self, now: Instant) {
        if now.saturating_duration_since(self.last_sweep) <= SWEEP_INTERVAL {
            return;
        }
        self.last_sweep = now;
        self.records
            .retain(|r| now.saturating_duration_since(r.last_seen) <= EXPIRY);
    }

    /// Drops every record, as a new group list does.
    pub fn clear(&mut self) {
        self.records.clear();
    }

    /// Number of records.
    pub const fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether there are no records.
    pub const fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// The records listed in `group`, one of `groups`.
    ///
    /// A record is listed in each group that takes it, or in [`DEFAULT_GROUP`] if none does.
    pub fn listed<'a>(
        &'a self,
        group: &'a Group,
        groups: &'a [Group],
    ) -> impl Iterator<Item = &'a Record> + 'a {
        let default = group.name == DEFAULT_GROUP;
        self.records
            .iter()
            .filter(move |r| r.fits(group) || default && !groups.iter().any(|g| r.fits(g)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reg(server_port: u32, name: &[u8], group: &[u8]) -> Registration {
        let mut reg = Registration {
            server_port,
            users: 1,
            name: name.to_vec(),
            ..Registration::default()
        };
        reg.groups[0] = group.to_vec();
        reg
    }

    fn groups() -> Vec<Group> {
        [&b"General"[..], b"Lab", b"Trackers"]
            .iter()
            .map(|name| Group::new(name, b""))
            .collect()
    }

    fn names<'a>(registry: &'a Registry, group: &[u8], groups: &'a [Group]) -> Vec<&'a [u8]> {
        let group = groups.iter().find(|g| g.name == group).unwrap();
        registry
            .listed(group, groups)
            .map(|r| &r.name[..])
            .collect()
    }

    #[test]
    fn groups_list_matching_records_and_general_the_rest() {
        let start = Instant::now();
        let groups = groups();
        let mut registry = Registry::new(start);
        let ip = Ipv4Addr::LOCALHOST;
        registry.register(ip, &reg(1, b"A", b"Lab, Lab2"), start, 1);
        registry.register(ip, &reg(2, b"B", b"Lab"), start, 1);
        registry.register(ip, &reg(3, b"C", b"Trackers"), start, 1);
        let mut tracker = reg(4, b"D", b"Trackers");
        tracker.flags = Registration::FROM_TRACKER;
        registry.register(ip, &tracker, start, 1);
        assert_eq!(names(&registry, b"General", &groups), [b"A", b"C"]);
        assert_eq!(names(&registry, b"Lab", &groups), [b"B"]);
        assert_eq!(names(&registry, b"Trackers", &groups), [b"D"]);
    }

    #[test]
    fn known_sources_keep_their_first_seen_time() {
        let start = Instant::now();
        let mut registry = Registry::new(start);
        let ip = Ipv4Addr::LOCALHOST;
        assert_eq!(
            registry.register(ip, &reg(1, b"A", b"x"), start, 1),
            Registered::Added
        );
        assert_eq!(
            registry.register(ip, &reg(1, b"B", b"y"), start, 2),
            Registered::Updated
        );
        assert_eq!(registry.records[0].first_seen, 1);
        assert_eq!(registry.records[0].name, b"B");
        assert_eq!(
            registry.register(ip, &reg(1, b"", b"y"), start, 3),
            Registered::Removed
        );
        assert!(registry.is_empty());
    }

    #[test]
    fn five_records_per_ip() {
        let start = Instant::now();
        let mut registry = Registry::new(start);
        for port in 0..5 {
            registry.register(Ipv4Addr::LOCALHOST, &reg(port, b"S", b""), start, 0);
        }
        assert_eq!(
            registry.register(Ipv4Addr::LOCALHOST, &reg(5, b"S", b""), start, 0),
            Registered::Ignored
        );
        assert_eq!(
            registry.register(Ipv4Addr::new(10, 0, 0, 1), &reg(5, b"S", b""), start, 0),
            Registered::Added
        );
    }

    #[test]
    fn long_names_are_cut() {
        let start = Instant::now();
        let mut registry = Registry::new(start);
        registry.register(Ipv4Addr::LOCALHOST, &reg(1, &[b'n'; 100], b""), start, 0);
        assert_eq!(registry.records[0].name.len(), MAX_NAME_LEN);
    }

    #[test]
    fn sweeps_are_spaced_and_drop_old_records() {
        let start = Instant::now();
        let mut registry = Registry::new(start);
        registry.register(Ipv4Addr::LOCALHOST, &reg(1, b"A", b""), start, 0);
        let late = start + EXPIRY + Duration::from_secs(1);
        registry.register(Ipv4Addr::LOCALHOST, &reg(2, b"B", b""), late, 0);

        // Not yet: the last sweep was at start.
        registry.sweep(start + SWEEP_INTERVAL);
        assert_eq!(registry.len(), 2);
        registry.sweep(late);
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.records[0].name, b"B");
    }
}
