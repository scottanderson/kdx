//! Tracker browsing and administration: 701 Tracker Group List, 702 Get Tracker Server List,
//! 703 Tracker Server List, 711 and 712 TrackerServ Settings, and 721 TrackerServ Statistics.
//!
//! The requests 700, 710, and 720 and the reply 713 have no payload. Layouts are in
//! `docs/protocol/tracker.md`.

use std::net::{Ipv4Addr, SocketAddrV4};

use super::server_info::ServerGreeting;
use super::{
    decode_counted, encode_counted, len_u8, need, pad4, padding, strings, u16_at, u32_at, u64_at,
    DecodeError, EncodeError,
};
use crate::AccountKey;

/// An entry of 701 Tracker Group List.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GroupListEntry {
    /// Servers listed in the group.
    pub servers: u16,
    /// Not set by the tracker; the client ignores it.
    pub value_02: u32,
    /// Group name.
    pub name: Vec<u8>,
    /// Group description.
    pub description: Vec<u8>,
    /// Bytes after the description up to the next multiple of 4, as received; empty in a new entry.
    pub padding: Vec<u8>,
}

impl GroupListEntry {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 8;

    /// Decodes the payload of 701 Tracker Group List.
    pub fn decode_list(buf: &[u8]) -> Result<Vec<Self>, DecodeError> {
        decode_counted(buf, 0, |entry| {
            need(entry, Self::HEADER_LEN)?;
            let lens = [usize::from(entry[6]), usize::from(entry[7])];
            let ([name, description], end) = strings(entry, Self::HEADER_LEN, lens)?;
            let (padding, padded) = padding(entry, end);
            let group = Self {
                servers: u16_at(entry, 0),
                value_02: u32_at(entry, 2),
                name,
                description,
                padding,
            };
            Ok((group, padded))
        })
    }

    /// Encodes the payload of 701 Tracker Group List.
    pub fn encode_list(list: &[Self]) -> Result<Vec<u8>, EncodeError> {
        let mut out = Vec::new();
        encode_counted(&mut out, list, |group| {
            let mut out = vec![0; Self::HEADER_LEN];
            out[0..2].copy_from_slice(&group.servers.to_be_bytes());
            out[2..6].copy_from_slice(&group.value_02.to_be_bytes());
            out[6] = len_u8(&group.name, "group name")?;
            out[7] = len_u8(&group.description, "group description")?;
            out.extend_from_slice(&group.name);
            out.extend_from_slice(&group.description);
            out.extend_from_slice(&group.padding);
            Ok(out)
        })?;
        Ok(out)
    }
}

/// The request of 702 Get Tracker Server List.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GetServerList {
    /// Bytes 0x00 and 0x01; meaning unknown.
    pub unknown_00: [u8; 2],
    /// Byte 0x03; meaning unknown.
    pub unknown_03: u8,
    /// Group name; an unknown group gives 0x1000c.
    pub group: Vec<u8>,
}

impl GetServerList {
    /// Size of the fixed part before the name.
    pub const HEADER_LEN: usize = 4;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let ([group], _) = strings(buf, Self::HEADER_LEN, [usize::from(buf[2])])?;
        Ok(Self {
            unknown_00: buf[..2].try_into().expect("2 bytes"),
            unknown_03: buf[3],
            group,
        })
    }

    /// Encodes the payload, without padding.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[..2].copy_from_slice(&self.unknown_00);
        out[2] = len_u8(&self.group, "group name")?;
        out[3] = self.unknown_03;
        out.extend_from_slice(&self.group);
        Ok(out)
    }
}

/// An entry of 703 Tracker Server List.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerListEntry {
    /// When the tracker first heard from the server, in milliseconds since 1900.
    pub first_seen: u64,
    /// Bytes 0x08 to 0x0d; meaning unknown.
    pub unknown_08: [u8; 6],
    /// The registration's flags.
    pub flags: u16,
    /// Users logged in.
    pub users: u16,
    /// Bytes 0x12 to 0x17; meaning unknown.
    pub unknown_12: [u8; 6],
    /// The registration's source IP and port field.
    pub address: SocketAddrV4,
    /// Server name.
    pub name: Vec<u8>,
    /// Server description.
    pub description: Vec<u8>,
    /// Opaque bytes after the description; empty from Tracker 1.110.
    pub extra: Vec<u8>,
    /// Bytes after `extra` up to the next multiple of 4, as received; empty in a new entry.
    pub padding: Vec<u8>,
}

impl ServerListEntry {
    /// Size of the fixed part before the address.
    pub const HEADER_LEN: usize = 0x1c;
    /// The only address length: an IPv4 address and a port.
    pub const ADDR_LEN: u8 = 6;

    /// Decodes the payload of 703 Tracker Server List; an empty payload is an empty group.
    pub fn decode_list(buf: &[u8]) -> Result<Vec<Self>, DecodeError> {
        if buf.is_empty() {
            return Ok(Vec::new());
        }
        decode_counted(buf, 0, |entry| {
            need(entry, Self::HEADER_LEN)?;
            if entry[0x18] != Self::ADDR_LEN {
                return Err(DecodeError::Invalid("tracker server address length"));
            }
            let lens = [6, entry[0x19], entry[0x1a], entry[0x1b]].map(usize::from);
            let ([addr, name, description, extra], end) = strings(entry, Self::HEADER_LEN, lens)?;
            let (padding, padded) = padding(entry, end);
            let server = Self {
                first_seen: u64_at(entry, 0),
                unknown_08: entry[8..0x0e].try_into().expect("6 bytes"),
                flags: u16_at(entry, 0x0e),
                users: u16_at(entry, 0x10),
                unknown_12: entry[0x12..0x18].try_into().expect("6 bytes"),
                address: SocketAddrV4::new(
                    Ipv4Addr::new(addr[0], addr[1], addr[2], addr[3]),
                    u16_at(&addr, 4),
                ),
                name,
                description,
                extra,
                padding,
            };
            Ok((server, padded))
        })
    }

    /// Encodes the payload of 703 Tracker Server List, which is empty for no servers.
    pub fn encode_list(list: &[Self]) -> Result<Vec<u8>, EncodeError> {
        let mut out = Vec::new();
        if list.is_empty() {
            return Ok(out);
        }
        encode_counted(&mut out, list, |server| {
            let mut out = vec![0; Self::HEADER_LEN];
            out[0..8].copy_from_slice(&server.first_seen.to_be_bytes());
            out[8..0x0e].copy_from_slice(&server.unknown_08);
            out[0x0e..0x10].copy_from_slice(&server.flags.to_be_bytes());
            out[0x10..0x12].copy_from_slice(&server.users.to_be_bytes());
            out[0x12..0x18].copy_from_slice(&server.unknown_12);
            out[0x18] = Self::ADDR_LEN;
            out[0x19] = len_u8(&server.name, "server name")?;
            out[0x1a] = len_u8(&server.description, "server description")?;
            out[0x1b] = len_u8(&server.extra, "server extra")?;
            out.extend_from_slice(&server.address.ip().octets());
            out.extend_from_slice(&server.address.port().to_be_bytes());
            out.extend_from_slice(&server.name);
            out.extend_from_slice(&server.description);
            out.extend_from_slice(&server.extra);
            out.extend_from_slice(&server.padding);
            Ok(out)
        })?;
        Ok(out)
    }
}

/// 721 TrackerServ Statistics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TrackerStats {
    /// When the tracker started, in milliseconds since 1900.
    pub start_time: u64,
    /// TCP connections accepted.
    pub connections_accepted: u64,
    /// UDP datagrams received.
    pub datagrams_received: u64,
    /// Bytes 0x18 to 0x1f; meaning unknown.
    pub unknown_18: [u8; 8],
    /// Hours since start.
    pub hours: u32,
    /// TCP connections open now.
    pub connections: u32,
    /// Servers listed.
    pub servers: u32,
    /// Bytes 0x2c to 0x37; meaning unknown.
    pub unknown_2c: [u8; 0x0c],
}

impl TrackerStats {
    /// Payload length.
    pub const LEN: usize = 0x38;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::LEN)?;
        Ok(Self {
            start_time: u64_at(buf, 0),
            connections_accepted: u64_at(buf, 8),
            datagrams_received: u64_at(buf, 0x10),
            unknown_18: buf[0x18..0x20].try_into().expect("8 bytes"),
            hours: u32_at(buf, 0x20),
            connections: u32_at(buf, 0x24),
            servers: u32_at(buf, 0x28),
            unknown_2c: buf[0x2c..Self::LEN].try_into().expect("12 bytes"),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::LEN];
        out[0..8].copy_from_slice(&self.start_time.to_be_bytes());
        out[8..0x10].copy_from_slice(&self.connections_accepted.to_be_bytes());
        out[0x10..0x18].copy_from_slice(&self.datagrams_received.to_be_bytes());
        out[0x18..0x20].copy_from_slice(&self.unknown_18);
        out[0x20..0x24].copy_from_slice(&self.hours.to_be_bytes());
        out[0x24..0x28].copy_from_slice(&self.connections.to_be_bytes());
        out[0x28..0x2c].copy_from_slice(&self.servers.to_be_bytes());
        out[0x2c..].copy_from_slice(&self.unknown_2c);
        out
    }
}

/// The `%KDXTSTG` settings of 711 TrackerServ Settings Info and 712 Set TrackerServ Settings.
///
/// Sections are kept as raw bytes, so a decoded blob encodes to the same bytes. The typed
/// accessors read a section only where the tracker would.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackerSettings {
    /// Format; 1 from Tracker 1.110 and Client 1.620 beta 5.
    pub format: u16,
    /// Bytes 0x0a and 0x0b; meaning unknown.
    pub unknown_0a: [u8; 2],
    /// The present bytes at 0x6c, one per section; the tracker reads a section only when 1.
    pub present: [u8; Self::SECTIONS],
    /// Each section's bytes, or `None` for offset and size 0.
    pub sections: [Option<Vec<u8>>; Self::SECTIONS],
}

impl Default for TrackerSettings {
    fn default() -> Self {
        Self {
            format: Self::FORMAT,
            unknown_0a: [0; 2],
            present: [0; Self::SECTIONS],
            sections: Default::default(),
        }
    }
}

impl TrackerSettings {
    /// The magic at the start of the blob.
    pub const MAGIC: &'static [u8; 8] = b"%KDXTSTG";
    /// The format Tracker 1.110 writes.
    pub const FORMAT: u16 = 1;
    /// Size of the header, where the first section starts.
    pub const HEADER_LEN: usize = 0x78;
    /// Entries in the section table.
    pub const SECTIONS: usize = 12;
    /// Index of the general section, [`GeneralSettings`].
    pub const GENERAL: usize = 0;
    /// Index of the upstream trackers section.
    pub const TRACKERS: usize = 1;
    /// Index of the groups section, a run of [`SettingsGroup`] records.
    pub const GROUPS: usize = 2;
    /// Index of the greeting section, in the layout of 107 Server Greeting.
    pub const GREETING: usize = 4;
    /// Index of the admin section, [`AdminSettings`].
    pub const ADMIN: usize = 5;
    /// The order sections are laid out in by both programs.
    const LAYOUT: [usize; Self::SECTIONS] = [0, 1, 2, 3, 5, 4, 6, 7, 8, 9, 10, 11];
    const TABLE_AT: usize = 0x0c;
    const PRESENT_AT: usize = 0x6c;

    /// Decodes the blob.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        if &buf[..8] != Self::MAGIC {
            return Err(DecodeError::Invalid("tracker settings magic"));
        }
        let mut sections: [Option<Vec<u8>>; Self::SECTIONS] = Default::default();
        for (i, section) in sections.iter_mut().enumerate() {
            let at = Self::TABLE_AT + 8 * i;
            let offset = u32_at(buf, at) as usize;
            let len = u32_at(buf, at + 4) as usize;
            if offset == 0 && len == 0 {
                continue;
            }
            if offset < Self::HEADER_LEN {
                return Err(DecodeError::Invalid("tracker settings section offset"));
            }
            need(buf, offset + len)?;
            *section = Some(buf[offset..offset + len].to_vec());
        }
        Ok(Self {
            format: u16_at(buf, 8),
            unknown_0a: buf[0x0a..Self::TABLE_AT].try_into().expect("2 bytes"),
            present: buf[Self::PRESENT_AT..Self::HEADER_LEN]
                .try_into()
                .expect("12 bytes"),
            sections,
        })
    }

    /// Encodes the blob, with the sections back to back in the order the programs use.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[..8].copy_from_slice(Self::MAGIC);
        out[8..0x0a].copy_from_slice(&self.format.to_be_bytes());
        out[0x0a..Self::TABLE_AT].copy_from_slice(&self.unknown_0a);
        out[Self::PRESENT_AT..].copy_from_slice(&self.present);
        for i in Self::LAYOUT {
            let Some(section) = &self.sections[i] else {
                continue;
            };
            let at = Self::TABLE_AT + 8 * i;
            let offset = out.len() as u32;
            out[at..at + 4].copy_from_slice(&offset.to_be_bytes());
            out[at + 4..at + 8].copy_from_slice(&(section.len() as u32).to_be_bytes());
            out.extend_from_slice(section);
        }
        out
    }

    /// The bytes of section `index` if the tracker would read it: present is 1 and size not 0.
    ///
    /// # Panics
    ///
    /// If `index` is not below [`Self::SECTIONS`].
    pub fn section(&self, index: usize) -> Option<&[u8]> {
        let section = self.sections[index].as_deref()?;
        (self.present[index] == 1 && !section.is_empty()).then_some(section)
    }

    /// Sets section `index` and marks it present.
    ///
    /// # Panics
    ///
    /// If `index` is not below [`Self::SECTIONS`].
    pub fn set_section(&mut self, index: usize, bytes: Vec<u8>) {
        self.sections[index] = Some(bytes);
        self.present[index] = 1;
    }

    /// The general section.
    pub fn general(&self) -> Result<Option<GeneralSettings>, DecodeError> {
        self.section(Self::GENERAL)
            .map(GeneralSettings::decode)
            .transpose()
    }

    /// The groups section, which replaces the tracker's whole list.
    pub fn groups(&self) -> Result<Option<Vec<SettingsGroup>>, DecodeError> {
        self.section(Self::GROUPS)
            .map(SettingsGroup::decode_list)
            .transpose()
    }

    /// The greeting section.
    pub fn greeting(&self) -> Result<Option<ServerGreeting>, DecodeError> {
        self.section(Self::GREETING)
            .map(ServerGreeting::decode)
            .transpose()
    }

    /// The admin section.
    pub fn admin(&self) -> Result<Option<AdminSettings>, DecodeError> {
        self.section(Self::ADMIN)
            .map(AdminSettings::decode)
            .transpose()
    }
}

/// The general section of [`TrackerSettings`]. Offsets below are from the section start.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GeneralSettings {
    /// Four limits at 0x00, taken from the settings file only; 0 in 711.
    pub limits: [u16; 4],
    /// Bytes 0x08 to 0x1f; meaning unknown.
    pub unknown_08: [u8; 0x18],
    /// [`Self::REGISTER`] and [`Self::GREETING_ENABLED`].
    pub flags: u32,
    /// Tracker port, for both TCP and UDP; 0 or above 0xfffe means 10800.
    pub tracker_port: u32,
    /// Bytes 0x28 to 0x2b; meaning unknown.
    pub unknown_28: [u8; 4],
    /// Server ID for upstream registrations, taken from the settings file only.
    pub server_id: [u8; 12],
    /// Bytes 0x38 to 0x45; meaning unknown.
    pub unknown_38: [u8; 0x0e],
    /// Tracker name.
    pub name: Vec<u8>,
    /// Tracker description.
    pub description: Vec<u8>,
    /// Bytes after the strings, padding to 4 as received; empty in a new section.
    pub padding: Vec<u8>,
}

impl GeneralSettings {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x4c;
    /// Flag bit: Register with Trackers.
    pub const REGISTER: u32 = 1;
    /// Flag bit: Enable Greeting.
    pub const GREETING_ENABLED: u32 = 2;

    /// Decodes the section.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [usize::from(buf[0x46]), usize::from(buf[0x47])];
        let ([name, description], end) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            limits: std::array::from_fn(|i| u16_at(buf, 2 * i)),
            unknown_08: buf[8..0x20].try_into().expect("24 bytes"),
            flags: u32_at(buf, 0x20),
            tracker_port: u32_at(buf, 0x24),
            unknown_28: buf[0x28..0x2c].try_into().expect("4 bytes"),
            server_id: buf[0x2c..0x38].try_into().expect("12 bytes"),
            unknown_38: buf[0x38..0x46].try_into().expect("14 bytes"),
            name,
            description,
            padding: buf[end..].to_vec(),
        })
    }

    /// Encodes the section, padded to 4 with zeros past the stored padding.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        for (i, limit) in self.limits.iter().enumerate() {
            out[2 * i..2 * i + 2].copy_from_slice(&limit.to_be_bytes());
        }
        out[8..0x20].copy_from_slice(&self.unknown_08);
        out[0x20..0x24].copy_from_slice(&self.flags.to_be_bytes());
        out[0x24..0x28].copy_from_slice(&self.tracker_port.to_be_bytes());
        out[0x28..0x2c].copy_from_slice(&self.unknown_28);
        out[0x2c..0x38].copy_from_slice(&self.server_id);
        out[0x38..0x46].copy_from_slice(&self.unknown_38);
        out[0x46] = len_u8(&self.name, "tracker name")?;
        out[0x47] = len_u8(&self.description, "tracker description")?;
        out.extend_from_slice(&self.name);
        out.extend_from_slice(&self.description);
        out.extend_from_slice(&self.padding);
        out.resize(pad4(out.len()), 0);
        Ok(out)
    }
}

/// A group record of the groups section of [`TrackerSettings`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SettingsGroup {
    /// Bytes 0x00 to 0x0b; meaning unknown.
    pub unknown_00: [u8; 0x0c],
    /// Set when the name contains `tracker`; such a group lists only trackers. 0 in 711.
    pub flags: u32,
    /// Never set; 0.
    pub value_10: u16,
    /// Never set; 0.
    pub value_12: u16,
    /// Bytes 0x14 to 0x17; meaning unknown.
    pub unknown_14: [u8; 4],
    /// Bytes 0x1a and 0x1b; meaning unknown.
    pub unknown_1a: [u8; 2],
    /// Group name.
    pub name: Vec<u8>,
    /// Group description.
    pub description: Vec<u8>,
    /// Bytes after the description up to the next multiple of 4, as received; empty in a new record.
    pub padding: Vec<u8>,
}

impl SettingsGroup {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x1c;

    /// Decodes a groups section: records back to back, each padded to 4.
    pub fn decode_list(mut buf: &[u8]) -> Result<Vec<Self>, DecodeError> {
        let mut list = Vec::new();
        while !buf.is_empty() {
            need(buf, Self::HEADER_LEN)?;
            let lens = [usize::from(buf[0x18]), usize::from(buf[0x19])];
            let ([name, description], end) = strings(buf, Self::HEADER_LEN, lens)?;
            let (padding, padded) = padding(buf, end);
            list.push(Self {
                unknown_00: buf[..0x0c].try_into().expect("12 bytes"),
                flags: u32_at(buf, 0x0c),
                value_10: u16_at(buf, 0x10),
                value_12: u16_at(buf, 0x12),
                unknown_14: buf[0x14..0x18].try_into().expect("4 bytes"),
                unknown_1a: buf[0x1a..0x1c].try_into().expect("2 bytes"),
                name,
                description,
                padding,
            });
            buf = &buf[padded..];
        }
        Ok(list)
    }

    /// Encodes a groups section.
    pub fn encode_list(list: &[Self]) -> Result<Vec<u8>, EncodeError> {
        let mut out = Vec::new();
        for group in list {
            let mut fixed = [0; Self::HEADER_LEN];
            fixed[..0x0c].copy_from_slice(&group.unknown_00);
            fixed[0x0c..0x10].copy_from_slice(&group.flags.to_be_bytes());
            fixed[0x10..0x12].copy_from_slice(&group.value_10.to_be_bytes());
            fixed[0x12..0x14].copy_from_slice(&group.value_12.to_be_bytes());
            fixed[0x14..0x18].copy_from_slice(&group.unknown_14);
            fixed[0x18] = len_u8(&group.name, "group name")?;
            fixed[0x19] = len_u8(&group.description, "group description")?;
            fixed[0x1a..0x1c].copy_from_slice(&group.unknown_1a);
            out.extend_from_slice(&fixed);
            out.extend_from_slice(&group.name);
            out.extend_from_slice(&group.description);
            out.extend_from_slice(&group.padding);
            out.resize(pad4(out.len()), 0);
        }
        Ok(out)
    }
}

/// The admin section of [`TrackerSettings`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminSettings {
    /// Bytes 0x00 to 0x06; meaning unknown.
    pub unknown_00: [u8; 7],
    /// The bits of the flags byte at 0x07 other than the key's.
    pub flags_07: u8,
    /// The admin password's key, when flags bit 0 at 0x07 is set; 711 leaves it out.
    pub key: Option<AccountKey>,
    /// Bytes 0x28 to 0x8f; meaning unknown.
    pub unknown_28: [u8; 0x68],
    /// Bytes 0x91 to 0x93; meaning unknown.
    pub unknown_91: [u8; 3],
    /// Admin login.
    pub login: Vec<u8>,
    /// Bytes after the login, padding to 4 as received; empty in a new section.
    pub padding: Vec<u8>,
}

impl Default for AdminSettings {
    fn default() -> Self {
        Self {
            unknown_00: [0; 7],
            flags_07: 0,
            key: None,
            unknown_28: [0; 0x68],
            unknown_91: [0; 3],
            login: Vec::new(),
            padding: Vec::new(),
        }
    }
}

impl AdminSettings {
    /// Size of the fixed part before the login; the tracker reads no shorter section.
    pub const HEADER_LEN: usize = 0x94;
    const KEY_PRESENT: u8 = 1;

    /// Decodes the section.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let ([login], end) = strings(buf, Self::HEADER_LEN, [usize::from(buf[0x90])])?;
        let key = (buf[7] & Self::KEY_PRESENT != 0)
            .then(|| AccountKey(buf[8..0x28].try_into().expect("32 bytes")));
        Ok(Self {
            unknown_00: buf[..7].try_into().expect("7 bytes"),
            flags_07: buf[7] & !Self::KEY_PRESENT,
            key,
            unknown_28: buf[0x28..0x90].try_into().expect("104 bytes"),
            unknown_91: buf[0x91..0x94].try_into().expect("3 bytes"),
            login,
            padding: buf[end..].to_vec(),
        })
    }

    /// Encodes the section, padded to 4 with zeros past the stored padding.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[..7].copy_from_slice(&self.unknown_00);
        out[7] = self.flags_07 & !Self::KEY_PRESENT;
        if let Some(key) = &self.key {
            out[7] |= Self::KEY_PRESENT;
            out[8..0x28].copy_from_slice(&key.0);
        }
        out[0x28..0x90].copy_from_slice(&self.unknown_28);
        out[0x90] = len_u8(&self.login, "admin login")?;
        out[0x91..0x94].copy_from_slice(&self.unknown_91);
        out.extend_from_slice(&self.login);
        out.extend_from_slice(&self.padding);
        out.resize(pad4(out.len()), 0);
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_from_parts() {
        let mut settings = TrackerSettings::default();
        let general = GeneralSettings {
            tracker_port: 10800,
            name: b"T".to_vec(),
            ..GeneralSettings::default()
        };
        settings.set_section(TrackerSettings::GENERAL, general.encode().unwrap());
        settings.set_section(TrackerSettings::TRACKERS, Vec::new());
        let admin = AdminSettings {
            key: Some(AccountKey::from_password(b"pw")),
            login: b"admin".to_vec(),
            ..AdminSettings::default()
        };
        settings.set_section(TrackerSettings::ADMIN, admin.encode().unwrap());

        let wire = settings.encode();
        assert_eq!(wire.len(), 0x78 + 0x50 + 0x9c);
        let decoded = TrackerSettings::decode(&wire).unwrap();
        assert_eq!(decoded, settings);
        // Decoding keeps the zero padding the encoder added: 3 bytes after each string.
        let general = GeneralSettings {
            padding: vec![0; 3],
            ..general
        };
        assert_eq!(decoded.general().unwrap(), Some(general));
        let admin = AdminSettings {
            padding: vec![0; 3],
            ..admin
        };
        assert_eq!(decoded.admin().unwrap(), Some(admin));
        // Present but empty, and absent, are both skipped.
        assert_eq!(decoded.section(TrackerSettings::TRACKERS), None);
        assert_eq!(decoded.groups().unwrap(), None);
    }

    #[test]
    fn server_list_address_length() {
        let server = ServerListEntry {
            first_seen: 0,
            flags: 0,
            users: 0,
            address: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 10700),
            name: b"S".to_vec(),
            description: Vec::new(),
            extra: Vec::new(),
            unknown_08: [0; 6],
            unknown_12: [0; 6],
            padding: Vec::new(),
        };
        let mut wire = ServerListEntry::encode_list(&[server]).unwrap();
        wire[4 + 0x18] = 18;
        assert_eq!(
            ServerListEntry::decode_list(&wire),
            Err(DecodeError::Invalid("tracker server address length"))
        );
    }
}
