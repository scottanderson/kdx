//! The tracker's settings, in the `%KDXTSTG` format of 711 and 712.
//!
//! [`Settings::apply`] reads a blob the way Tracker 1.110 does in
//! `FUN_0804d63c`, for both 712 Set TrackerServ Settings and the settings
//! file. The file is the same blob with the secrets, not XOR'd.

use std::path::Path;

use anyhow::{bail, Context, Result};
use kdx_proto::msg::tracker::{AdminSettings, GeneralSettings, TrackerSettings};
use kdx_proto::AccountKey;

/// The default tracker port, for both TCP and UDP.
pub const DEFAULT_TRACKER_PORT: u16 = 10800;
/// Where a registration that matches no group is listed.
pub const DEFAULT_GROUP: &[u8] = b"General";
/// The group added back with [`DEFAULT_GROUP`] when a group list leaves it out.
pub const TRACKERS_GROUP: &[u8] = b"Trackers";
/// The name of a fresh Tracker 1.110.
const FRESH_NAME: &[u8] = b"New TrackerServ";
/// The groups of a fresh Tracker 1.110.
const FRESH_GROUPS: [&[u8]; 8] = [
    b"Business",
    b"Chat",
    b"Education",
    b"Games",
    b"General",
    b"Macintosh",
    b"Trackers",
    b"Windows",
];
/// Longest name and admin login kept.
const MAX_NAME_LEN: usize = 63;
/// Longest greeting section kept.
const MAX_GREETING_LEN: usize = 0xffff;
/// Size of an upstream tracker entry before its strings.
const UPSTREAM_HEADER_LEN: usize = 0x20;
/// Index of the address among an upstream tracker entry's strings.
const UPSTREAM_ADDRESS: usize = 2;
/// Size of a group record before its strings.
const GROUP_HEADER_LEN: usize = 0x1c;

/// A group that servers are listed in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    /// Group name.
    pub name: Vec<u8>,
    /// Group description.
    pub description: Vec<u8>,
    /// Whether the group takes only registrations from trackers.
    pub trackers_only: bool,
}

impl Group {
    /// Creates a group; it takes only trackers when its name contains `tracker` in any case.
    pub fn new(name: &[u8], description: &[u8]) -> Self {
        Self {
            name: name.to_vec(),
            description: description.to_vec(),
            trackers_only: name.windows(7).any(|w| w.eq_ignore_ascii_case(b"tracker")),
        }
    }
}

/// Everything the tracker keeps across restarts.
#[derive(Clone, PartialEq, Eq)]
pub struct Settings {
    /// Tracker name, at most 63 bytes.
    pub name: Vec<u8>,
    /// Tracker description.
    pub description: Vec<u8>,
    /// Tracker port, for both TCP and UDP.
    pub tracker_port: u16,
    /// Register with Trackers; kept, but this tracker registers nowhere.
    pub register: bool,
    /// Enable Greeting.
    pub greeting_enabled: bool,
    /// The four limits of the general section, read from the file only.
    pub limits: [u16; 4],
    /// Server ID for upstream registrations, read from the file only.
    pub server_id: [u8; 12],
    /// The upstream tracker entries, as their section bytes.
    pub upstream: Vec<u8>,
    /// Groups, sorted by name.
    pub groups: Vec<Group>,
    /// The greeting section: a u32 0, then the text; empty for none.
    pub greeting: Vec<u8>,
    /// Admin login.
    pub admin_login: Vec<u8>,
    /// Admin password key.
    pub admin_key: AccountKey,
}

impl std::fmt::Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Settings")
            .field("name", &String::from_utf8_lossy(&self.name))
            .field("tracker_port", &self.tracker_port)
            .field("groups", &self.groups.len())
            .field("admin_login", &String::from_utf8_lossy(&self.admin_login))
            .finish_non_exhaustive()
    }
}

/// What [`Settings::apply`] changed that the running tracker must act on.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Applied {
    /// The group list was replaced, which drops every registration.
    pub groups: bool,
}

impl Settings {
    /// The settings of a fresh tracker with the given admin login and password.
    pub fn new(admin_login: &[u8], admin_password: &[u8]) -> Self {
        Self {
            name: FRESH_NAME.to_vec(),
            description: Vec::new(),
            tracker_port: DEFAULT_TRACKER_PORT,
            register: false,
            greeting_enabled: false,
            limits: [0; 4],
            server_id: [0; 12],
            upstream: Vec::new(),
            groups: FRESH_GROUPS
                .iter()
                .map(|name| Group::new(name, b""))
                .collect(),
            greeting: Vec::new(),
            admin_login: truncated(admin_login, MAX_NAME_LEN),
            admin_key: AccountKey::from_password(admin_password),
        }
    }

    /// Reads a settings file written by [`Self::save`].
    pub fn load(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
        let blob =
            TrackerSettings::decode(&bytes).with_context(|| format!("{}", path.display()))?;
        if blob.admin()?.and_then(|admin| admin.key).is_none() {
            bail!("{}: no admin password", path.display());
        }
        let mut settings = Self::new(b"", b"");
        settings.apply(&blob, true);
        Ok(settings)
    }

    /// Writes the settings, with the secrets, to `path`.
    pub fn save(&self, path: &Path) -> Result<()> {
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, self.to_blob(true).encode())
            .with_context(|| format!("writing {}", tmp.display()))?;
        std::fs::rename(&tmp, path).with_context(|| format!("writing {}", path.display()))
    }

    /// The text of 107 Server Greeting to send after login, if any.
    pub fn greeting_text(&self) -> Option<&[u8]> {
        (self.greeting_enabled && self.greeting.len() > 4).then(|| &self.greeting[..])
    }

    /// The blob of 711 TrackerServ Settings Info, or of the file when `secrets` is set.
    ///
    /// 711 leaves out the limits, the server ID, and the admin key.
    pub fn to_blob(&self, secrets: bool) -> TrackerSettings {
        let general = GeneralSettings {
            limits: if secrets { self.limits } else { [0; 4] },
            flags: (u32::from(self.register) * GeneralSettings::REGISTER)
                | (u32::from(self.greeting_enabled) * GeneralSettings::GREETING_ENABLED),
            tracker_port: u32::from(self.tracker_port),
            server_id: if secrets { self.server_id } else { [0; 12] },
            name: self.name.clone(),
            description: self.description.clone(),
            ..GeneralSettings::default()
        };
        let admin = AdminSettings {
            key: secrets.then(|| self.admin_key.clone()),
            login: self.admin_login.clone(),
            ..AdminSettings::default()
        };
        let mut blob = TrackerSettings::default();
        blob.set_section(
            TrackerSettings::GENERAL,
            general.encode().expect("lengths kept in range"),
        );
        blob.set_section(TrackerSettings::TRACKERS, self.upstream.clone());
        blob.set_section(TrackerSettings::GROUPS, encode_groups(&self.groups));
        blob.set_section(TrackerSettings::GREETING, self.greeting.clone());
        blob.set_section(
            TrackerSettings::ADMIN,
            admin.encode().expect("lengths kept in range"),
        );
        blob
    }

    /// Applies a blob as Tracker 1.110 does; `from_file` also takes the limits and server ID.
    ///
    /// Each section is read only when its present byte is 1. A missing or
    /// empty upstream section clears Register with Trackers, and a missing or
    /// short greeting section clears the greeting.
    pub fn apply(&mut self, blob: &TrackerSettings, from_file: bool) -> Applied {
        let mut applied = Applied::default();
        if let Some(general) = present(blob, TrackerSettings::GENERAL)
            .filter(|s| s.len() >= GeneralSettings::HEADER_LEN)
            .and_then(|s| GeneralSettings::decode(s).ok())
        {
            self.register = general.flags & GeneralSettings::REGISTER != 0;
            self.greeting_enabled = general.flags & GeneralSettings::GREETING_ENABLED != 0;
            self.tracker_port = match general.tracker_port {
                port @ 1..=0xfffe => port as u16,
                _ => DEFAULT_TRACKER_PORT,
            };
            if from_file {
                self.limits = general.limits;
                self.server_id = general.server_id;
            }
            self.name = truncated(&general.name, MAX_NAME_LEN);
            self.description = general.description;
            if self.name.is_empty() {
                self.register = false;
            }
        }

        let mut has_upstream = false;
        if let Some(section) = present(blob, TrackerSettings::TRACKERS) {
            let entries = upstream_entries(section);
            has_upstream = entries.iter().any(|&(at, len)| {
                !upstream_string(&section[at..at + len], UPSTREAM_ADDRESS).is_empty()
            });
            self.upstream = entries
                .iter()
                .flat_map(|&(at, len)| &section[at..at + len])
                .copied()
                .collect();
        }
        if !has_upstream {
            self.register = false;
        }

        if let Some(section) =
            present(blob, TrackerSettings::GROUPS).filter(|section| !section.is_empty())
        {
            self.groups = decode_groups(section);
            for name in [DEFAULT_GROUP, TRACKERS_GROUP] {
                insert_group(&mut self.groups, Group::new(name, b""));
            }
            applied.groups = true;
        }

        if let Some(admin) = present(blob, TrackerSettings::ADMIN)
            .filter(|s| s.len() >= AdminSettings::HEADER_LEN)
            .and_then(|s| AdminSettings::decode(s).ok())
        {
            if let Some(key) = admin.key {
                self.admin_key = key;
            }
            if !admin.login.is_empty() {
                self.admin_login = truncated(&admin.login, MAX_NAME_LEN);
            }
        }

        self.greeting = present(blob, TrackerSettings::GREETING)
            .filter(|section| section.len() > 4)
            .map(|section| truncated(section, MAX_GREETING_LEN))
            .unwrap_or_default();
        applied
    }

    /// Sets the admin login, cut to 63 bytes, and returns whether it changed.
    pub fn set_admin_login(&mut self, login: &[u8]) -> bool {
        let login = truncated(login, MAX_NAME_LEN);
        let changed = login != self.admin_login;
        self.admin_login = login;
        changed
    }

    /// Whether `login` is the admin login, compared ignoring ASCII case.
    pub fn is_admin_login(&self, login: &[u8]) -> bool {
        login.eq_ignore_ascii_case(&self.admin_login)
    }
}

/// A section whose present byte is 1; absent sections read as empty.
fn present(blob: &TrackerSettings, index: usize) -> Option<&[u8]> {
    (blob.present[index] == 1).then(|| blob.sections[index].as_deref().unwrap_or_default())
}

/// The offset and length of each whole upstream tracker entry.
fn upstream_entries(section: &[u8]) -> Vec<(usize, usize)> {
    let mut entries = Vec::new();
    let mut at = 0;
    while let Some(entry) = section.get(at..).filter(|e| e.len() >= UPSTREAM_HEADER_LEN) {
        let strings: usize = entry[0x14..0x20].iter().map(|&len| usize::from(len)).sum();
        let len = (UPSTREAM_HEADER_LEN + strings + 3) & !3;
        if len > entry.len() {
            break;
        }
        entries.push((at, len));
        at += len;
    }
    entries
}

/// String `index` of an upstream tracker entry.
fn upstream_string(entry: &[u8], index: usize) -> &[u8] {
    let start = UPSTREAM_HEADER_LEN
        + entry[0x14..0x14 + index]
            .iter()
            .map(|&len| usize::from(len))
            .sum::<usize>();
    &entry[start..start + usize::from(entry[0x14 + index])]
}

/// Reads group records, sorted by name, skipping duplicates.
fn decode_groups(mut section: &[u8]) -> Vec<Group> {
    let mut groups = Vec::new();
    while section.len() > GROUP_HEADER_LEN {
        let lens = &section[0x18..0x1c];
        let len =
            (GROUP_HEADER_LEN + 3 + lens.iter().map(|&len| usize::from(len)).sum::<usize>()) & !3;
        if len > section.len() {
            break;
        }
        let name_end = GROUP_HEADER_LEN + usize::from(lens[0]);
        let name = &section[GROUP_HEADER_LEN..name_end];
        let description = &section[name_end..name_end + usize::from(lens[1])];
        insert_group(&mut groups, Group::new(name, description));
        section = &section[len..];
    }
    groups
}

/// Encodes group records: two unset u16 values, the lengths, and the strings.
fn encode_groups(groups: &[Group]) -> Vec<u8> {
    let mut out = Vec::new();
    for group in groups {
        let mut fixed = [0; GROUP_HEADER_LEN];
        fixed[0x18] = group.name.len() as u8;
        fixed[0x19] = group.description.len() as u8;
        out.extend_from_slice(&fixed);
        out.extend_from_slice(&group.name);
        out.extend_from_slice(&group.description);
        out.resize((out.len() + 3) & !3, 0);
    }
    out
}

/// Inserts `group` in name order unless a group has its name.
fn insert_group(groups: &mut Vec<Group>, group: Group) {
    if let Err(at) = groups.binary_search_by(|g| g.name.cmp(&group.name)) {
        groups.insert(at, group);
    }
}

fn truncated(s: &[u8], max: usize) -> Vec<u8> {
    s[..s.len().min(max)].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_settings_round_trip_through_the_file_format() {
        let mut settings = Settings::new(b"admin", b"pw");
        settings.name = b"T".to_vec();
        settings.limits = [1, 2, 3, 4];
        settings.server_id = [7; 12];
        settings.greeting = b"\0\0\0\0hi".to_vec();
        settings.greeting_enabled = true;
        let blob = settings.to_blob(true);
        let mut loaded = Settings::new(b"", b"");
        loaded.apply(&TrackerSettings::decode(&blob.encode()).unwrap(), true);
        assert_eq!(loaded, settings);
        assert_eq!(loaded.greeting_text(), Some(&b"\0\0\0\0hi"[..]));
    }

    #[test]
    fn remote_settings_keep_file_only_fields() {
        let mut settings = Settings::new(b"admin", b"pw");
        settings.limits = [1, 2, 3, 4];
        let mut remote = settings.clone();
        remote.limits = [9; 4];
        remote.tracker_port = 0;
        let mut blob = remote.to_blob(false);
        // Client 1.620 beta 5 leaves the admin section unmarked.
        blob.present[TrackerSettings::ADMIN] = 0;
        settings.apply(&blob, false);
        assert_eq!(settings.limits, [1, 2, 3, 4]);
        assert_eq!(settings.tracker_port, DEFAULT_TRACKER_PORT);
        assert_eq!(settings.admin_key, AccountKey::from_password(b"pw"));
    }

    #[test]
    fn groups_are_sorted_and_keep_the_defaults() {
        let mut settings = Settings::new(b"admin", b"pw");
        let mut blob = TrackerSettings::default();
        let groups = [
            Group::new(b"Lab", b"lab group"),
            Group::new(b"Lab", b"again"),
        ];
        blob.set_section(TrackerSettings::GROUPS, encode_groups(&groups));
        assert!(settings.apply(&blob, false).groups);
        let names: Vec<_> = settings.groups.iter().map(|g| &g.name[..]).collect();
        assert_eq!(names, [&b"General"[..], b"Lab", b"Trackers"]);
        assert_eq!(settings.groups[1].description, b"lab group");
        assert!(settings.groups[2].trackers_only);
    }

    #[test]
    fn register_needs_a_name_and_an_upstream_address() {
        let mut entry = vec![0; UPSTREAM_HEADER_LEN];
        entry[0x14 + UPSTREAM_ADDRESS] = 9;
        entry.extend_from_slice(b"127.0.0.1");
        entry.resize(0x2c, 0);
        let mut settings = Settings::new(b"admin", b"pw");
        settings.name = b"T".to_vec();
        settings.register = true;
        settings.upstream = entry;
        let blob = settings.to_blob(false);
        settings.apply(&blob, false);
        assert!(settings.register);

        settings.upstream.clear();
        settings.apply(&settings.to_blob(false), false);
        assert!(!settings.register);
    }
}
