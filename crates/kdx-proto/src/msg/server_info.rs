//! 105 Server Info, 106 Server Icon, 107 Server Greeting, and the server
//! settings of 171 Server Settings Info and 172 Set Server Settings.

use super::chat::ChatRecord;
use super::{len_u8, need, pad4, strings, u16_at, u32_at, u64_at, DecodeError, EncodeError};

/// What the server tells a client right after it logs in.
#[derive(Clone, PartialEq, Eq)]
pub struct ServerInfo {
    /// Account key for transfer connections; see `Connection::attach` in kdx-client.
    pub session_key: [u8; 32],
    /// The server's optional File Transfer port; 0 if it has none.
    pub file_transfer_port: u32,
    /// The client's own user ID.
    pub user_id: u32,
    /// Server version, 1620 for 1.620.
    pub version: u16,
    /// 1 if the user may change the server settings, else 0.
    pub can_modify_settings: u16,
    /// Bytes 0x2c to 0x45; 0 as sent.
    pub ignored_2c: [u8; 0x1a],
    /// The u16 at 0x46; Server 1.620 always sends 2, which stops the client's version warning.
    pub value_46: u16,
    /// Byte 0x4b; meaning unknown.
    pub ignored_4b: u8,
    /// Server name.
    pub name: Vec<u8>,
    /// Server description.
    pub description: Vec<u8>,
    /// The server's address from its settings, or empty for the address the client used.
    pub address: Vec<u8>,
}

impl Default for ServerInfo {
    fn default() -> Self {
        Self {
            session_key: [0; 32],
            file_transfer_port: 0,
            user_id: 0,
            version: 0,
            can_modify_settings: 0,
            ignored_2c: [0; 0x1a],
            value_46: 2,
            ignored_4b: 0,
            name: Vec::new(),
            description: Vec::new(),
            address: Vec::new(),
        }
    }
}

impl ServerInfo {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x4c;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [0x48, 0x49, 0x4a].map(|at| usize::from(buf[at]));
        let ([name, description, address], _) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            session_key: buf[..32].try_into().expect("32 bytes"),
            file_transfer_port: u32_at(buf, 0x20),
            user_id: u32_at(buf, 0x24),
            version: u16_at(buf, 0x28),
            can_modify_settings: u16_at(buf, 0x2a),
            ignored_2c: buf[0x2c..0x46].try_into().expect("26 bytes"),
            value_46: u16_at(buf, 0x46),
            ignored_4b: buf[0x4b],
            name,
            description,
            address,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[..32].copy_from_slice(&self.session_key);
        out[0x20..0x24].copy_from_slice(&self.file_transfer_port.to_be_bytes());
        out[0x24..0x28].copy_from_slice(&self.user_id.to_be_bytes());
        out[0x28..0x2a].copy_from_slice(&self.version.to_be_bytes());
        out[0x2a..0x2c].copy_from_slice(&self.can_modify_settings.to_be_bytes());
        out[0x2c..0x46].copy_from_slice(&self.ignored_2c);
        out[0x46..0x48].copy_from_slice(&self.value_46.to_be_bytes());
        out[0x48] = len_u8(&self.name, "server name")?;
        out[0x49] = len_u8(&self.description, "server description")?;
        out[0x4a] = len_u8(&self.address, "address")?;
        out[0x4b] = self.ignored_4b;
        for part in [&self.name, &self.description, &self.address] {
            out.extend_from_slice(part);
        }
        Ok(out)
    }
}

impl std::fmt::Debug for ServerInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerInfo")
            .field("session_key", &"<redacted>")
            .field("file_transfer_port", &self.file_transfer_port)
            .field("user_id", &self.user_id)
            .field("version", &self.version)
            .field("can_modify_settings", &self.can_modify_settings)
            .field("ignored_2c", &self.ignored_2c)
            .field("value_46", &self.value_46)
            .field("ignored_4b", &self.ignored_4b)
            .field("name", &String::from_utf8_lossy(&self.name))
            .field("description", &String::from_utf8_lossy(&self.description))
            .field("address", &String::from_utf8_lossy(&self.address))
            .finish()
    }
}

/// 106 Server Icon: the server's icon bitmap, as in the settings icon section.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerIcon(pub Vec<u8>);

impl ServerIcon {
    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(Self(buf.to_vec()))
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Vec<u8> {
        self.0.clone()
    }
}

/// 107 Server Greeting, also the layout of the settings greeting section.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerGreeting {
    /// Bytes 0x00 to 0x03; 0 from the server.
    pub ignored: [u8; 4],
    /// Greeting text, to the end of the payload, with no terminator.
    pub text: Vec<u8>,
}

impl ServerGreeting {
    /// Size of the fixed part before the text.
    pub const HEADER_LEN: usize = 4;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        Ok(Self {
            ignored: buf[..Self::HEADER_LEN].try_into().expect("4 bytes"),
            text: buf[Self::HEADER_LEN..].to_vec(),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = self.ignored.to_vec();
        out.extend_from_slice(&self.text);
        out
    }
}

/// An entry of the settings section table: where a section is in the blob.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SettingsSection {
    /// Offset from the start of the blob.
    pub offset: u32,
    /// Length; 0 for an empty or absent section.
    pub len: u32,
}

/// The server settings blob of 171 Server Settings Info and 172 Set Server Settings.
///
/// The sections the lab has seen non-empty are typed: 0 main, 1 trackers,
/// 2 allow/deny list, 3 greeting, 5 icon, 7 chat rooms. Sections 4, 6 and
/// 8 to 10 are kept as bytes. A decoded blob encodes to the same bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerSettings {
    /// Format version; 1 in the lab.
    pub version: u16,
    /// Bytes 0x0a and 0x0b; meaning unknown.
    pub ignored: [u8; 2],
    /// Bytes 0x64 to 0x6e; `01 01 01 01 00 01 00 02 00 00 00` from the server.
    /// The twelfth byte of the header, at 0x6f, is the tracker count.
    pub value_64: [u8; 11],
    /// Whether an empty section still gets the running offset in the table
    /// (the server does this for 1, 2, 3, 5 and 7) or `(0, 0)`.
    pub placeholders: [bool; Self::SECTIONS],
    /// Section 0.
    pub main: MainSettings,
    /// Section 1.
    pub trackers: Vec<TrackerEntry>,
    /// Section 2.
    pub allow_deny: Vec<AddressRule>,
    /// Section 3, in the layout of 107 Server Greeting; `None` when empty.
    pub greeting: Option<ServerGreeting>,
    /// Section 5, the bytes of 106 Server Icon; `None` when empty.
    pub icon: Option<Vec<u8>>,
    /// Section 7: the chat rooms, as 219 Chat Info records back to back.
    pub chats: Vec<ChatRecord>,
    /// Sections 4, 6, 8, 9 and 10, by index; the typed ones stay empty here.
    pub other: [Vec<u8>; Self::SECTIONS],
    /// Bytes after the last section's padding; never seen non-empty.
    pub trailing: Vec<u8>,
}

impl ServerSettings {
    /// The protocol at 0x00.
    pub const PROTOCOL: [u8; 4] = *b"%KDX";
    /// The sub-protocol at 0x04.
    pub const SUB_PROTOCOL: [u8; 4] = *b"SSTG";
    /// Size of the header, which ends where the main section starts.
    pub const HEADER_LEN: usize = 0x70;
    /// Entries in the section table.
    pub const SECTIONS: usize = 11;
    /// Index of the main section.
    pub const MAIN: usize = 0;
    /// Index of the tracker section.
    pub const TRACKERS: usize = 1;
    /// Index of the allow/deny list.
    pub const ALLOW_DENY: usize = 2;
    /// Index of the greeting section.
    pub const GREETING: usize = 3;
    /// Index of the icon section.
    pub const ICON: usize = 5;
    /// Index of the chat room section.
    pub const CHATS: usize = 7;
    /// Offset of the flags byte in the main section (0x93 in the blob).
    pub const FLAGS_AT: usize = 0x23;
    /// Flag bit: the greeting is enabled.
    pub const GREETING_ENABLED: u8 = 2;
    /// Offset of the speed limit in the main section (0xa8 in the blob).
    pub const SPEED_LIMIT_AT: usize = 0x38;
    /// Offset of the server name in the main section (0xcc in the blob).
    pub const NAME_AT: usize = 0x5c;
    const TABLE_AT: usize = 0x0c;
    /// The table entries the server writes for empty sections.
    const SERVER_PLACEHOLDERS: [bool; Self::SECTIONS] = [
        true, true, true, true, false, true, false, true, false, false, false,
    ];

    /// Decodes the blob.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        if buf[..4] != Self::PROTOCOL || buf[4..8] != Self::SUB_PROTOCOL {
            return Err(DecodeError::Invalid("settings protocol"));
        }
        let table: [SettingsSection; Self::SECTIONS] = std::array::from_fn(|i| SettingsSection {
            offset: u32_at(buf, Self::TABLE_AT + 8 * i),
            len: u32_at(buf, Self::TABLE_AT + 8 * i + 4),
        });
        let tracker_count = buf[Self::HEADER_LEN - 1];
        let mut placeholders = [false; Self::SECTIONS];
        let mut sections: [&[u8]; Self::SECTIONS] = [&[]; Self::SECTIONS];
        // Sections come in table order, each where the previous one, padded to 4, ended.
        let mut at = Self::HEADER_LEN;
        for (i, section) in table.iter().enumerate() {
            placeholders[i] = section.offset != 0;
            if section.len == 0 {
                continue;
            }
            let want = section.offset as usize;
            if want < at
                || want > at + 3
                || buf.get(at..want).is_none_or(|p| p.iter().any(|&b| b != 0))
            {
                return Err(DecodeError::Invalid("settings section offset"));
            }
            let end = want + section.len as usize;
            need(buf, end)?;
            sections[i] = &buf[want..end];
            at = end;
        }
        // The blob ends padded to 4 like every section before it.
        let padded = pad4(at);
        if buf.len() >= padded {
            if buf[at..padded].iter().any(|&b| b != 0) {
                return Err(DecodeError::Invalid("settings tail padding"));
            }
            at = padded;
        }
        let trailing = buf[at.min(buf.len())..].to_vec();
        let main = MainSettings::decode(sections[Self::MAIN])?;
        let trackers = records(sections[Self::TRACKERS], TrackerEntry::decode)?;
        if usize::from(tracker_count) != trackers.len().min(255) {
            return Err(DecodeError::Invalid("settings tracker count"));
        }
        let allow_deny = records(sections[Self::ALLOW_DENY], AddressRule::decode)?;
        let greeting = match sections[Self::GREETING] {
            [] => None,
            bytes => Some(ServerGreeting::decode(bytes)?),
        };
        let icon = match sections[Self::ICON] {
            [] => None,
            bytes => Some(bytes.to_vec()),
        };
        let chats = records(sections[Self::CHATS], |buf| {
            let record = ChatRecord::decode(buf)?;
            let len = record
                .encode()
                .map_err(|_| DecodeError::Invalid("chat record"))?
                .len();
            Ok((record, len))
        })?;
        let mut other: [Vec<u8>; Self::SECTIONS] = Default::default();
        for i in [4, 6, 8, 9, 10] {
            other[i] = sections[i].to_vec();
        }
        Ok(Self {
            version: u16_at(buf, 8),
            ignored: buf[0x0a..Self::TABLE_AT].try_into().expect("2 bytes"),
            value_64: buf[0x64..Self::HEADER_LEN - 1]
                .try_into()
                .expect("11 bytes"),
            placeholders,
            main,
            trackers,
            allow_deny,
            greeting,
            icon,
            chats,
            other,
            trailing,
        })
    }

    /// Encodes the blob. Strings longer than 255 bytes are cut to 255.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[..4].copy_from_slice(&Self::PROTOCOL);
        out[4..8].copy_from_slice(&Self::SUB_PROTOCOL);
        out[8..0x0a].copy_from_slice(&self.version.to_be_bytes());
        out[0x0a..Self::TABLE_AT].copy_from_slice(&self.ignored);
        for (i, section) in self.sections().iter().enumerate() {
            let at = Self::TABLE_AT + 8 * i;
            out[at..at + 4].copy_from_slice(&section.offset.to_be_bytes());
            out[at + 4..at + 8].copy_from_slice(&section.len.to_be_bytes());
        }
        out[0x64..Self::HEADER_LEN - 1].copy_from_slice(&self.value_64);
        out[Self::HEADER_LEN - 1] = self.trackers.len().min(255) as u8;
        for i in 0..Self::SECTIONS {
            let bytes = self.section(i);
            if !bytes.is_empty() {
                out.resize(pad4(out.len()), 0);
                out.extend_from_slice(&bytes);
            }
        }
        out.resize(pad4(out.len()), 0);
        out.extend_from_slice(&self.trailing);
        out
    }

    /// The section table as [`Self::encode`] writes it.
    pub fn sections(&self) -> [SettingsSection; Self::SECTIONS] {
        let mut table = [SettingsSection::default(); Self::SECTIONS];
        let mut offset = Self::HEADER_LEN;
        for (i, entry) in table.iter_mut().enumerate() {
            let len = self.section(i).len();
            if len > 0 {
                *entry = SettingsSection {
                    offset: offset as u32,
                    len: len as u32,
                };
                offset += pad4(len);
            } else if self.placeholders[i] {
                *entry = SettingsSection {
                    offset: offset as u32,
                    len: 0,
                };
            }
        }
        table
    }

    /// The bytes of section `index`; empty when it is empty or absent.
    ///
    /// # Panics
    ///
    /// If `index` is not below [`Self::SECTIONS`].
    pub fn section(&self, index: usize) -> Vec<u8> {
        match index {
            Self::MAIN => self.main.encode(),
            Self::TRACKERS => self
                .trackers
                .iter()
                .flat_map(TrackerEntry::encode)
                .collect(),
            Self::ALLOW_DENY => self
                .allow_deny
                .iter()
                .flat_map(AddressRule::encode)
                .collect(),
            Self::GREETING => self
                .greeting
                .as_ref()
                .map(ServerGreeting::encode)
                .unwrap_or_default(),
            Self::ICON => self.icon.clone().unwrap_or_default(),
            Self::CHATS => self
                .chats
                .iter()
                .flat_map(|c| c.encode().unwrap_or_default())
                .collect(),
            _ => self.other[index].clone(),
        }
    }

    /// The flags byte of the main section.
    pub const fn flags(&self) -> u8 {
        self.main.flags
    }

    /// Whether the greeting is enabled.
    pub const fn greeting_enabled(&self) -> bool {
        self.main.greeting_enabled()
    }

    /// The total outgoing speed limit in bytes per second; 0 when off.
    pub const fn speed_limit(&self) -> u32 {
        self.main.total_outgoing_speed_limit
    }

    /// The server name.
    pub fn name(&self) -> &[u8] {
        &self.main.name
    }

    /// The greeting section; `None` when no greeting is set.
    pub fn greeting(&self) -> Option<ServerGreeting> {
        self.greeting.clone()
    }

    /// The icon section; empty when the blob has no icon.
    pub fn icon(&self) -> &[u8] {
        self.icon.as_deref().unwrap_or(&[])
    }
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            version: 1,
            ignored: [0; 2],
            value_64: [1, 1, 1, 1, 0, 1, 0, 2, 0, 0, 0],
            placeholders: Self::SERVER_PLACEHOLDERS,
            main: MainSettings::default(),
            trackers: Vec::new(),
            allow_deny: Vec::new(),
            greeting: None,
            icon: None,
            chats: Vec::new(),
            other: Default::default(),
            trailing: Vec::new(),
        }
    }
}

/// Decodes records back to back until `buf` is used up; `one` returns a record and its length.
fn records<T>(
    buf: &[u8],
    mut one: impl FnMut(&[u8]) -> Result<(T, usize), DecodeError>,
) -> Result<Vec<T>, DecodeError> {
    let mut at = 0;
    let mut out = Vec::new();
    while at < buf.len() {
        let (record, len) = one(&buf[at..])?;
        if len == 0 {
            return Err(DecodeError::Invalid("empty settings record"));
        }
        out.push(record);
        at += len;
    }
    Ok(out)
}

/// The first 255 bytes of a string, the most a length byte can say.
fn str255(s: &[u8]) -> &[u8] {
    &s[..s.len().min(255)]
}

/// The padding of a record from `end` to the next multiple of 4; empty when it is the zeros the encoder writes anyway.
fn record_padding(buf: &[u8], end: usize) -> Result<(Vec<u8>, usize), DecodeError> {
    let padded = pad4(end);
    need(buf, padded)?;
    let pad = &buf[end..padded];
    let kept = if pad.iter().all(|&b| b == 0) {
        Vec::new()
    } else {
        pad.to_vec()
    };
    Ok((kept, padded))
}

/// Appends `padding` if it was kept, else zeros, up to the next multiple of 4 from `start`.
fn pad_record(out: &mut Vec<u8>, start: usize, padding: &[u8]) {
    if padding.is_empty() {
        let len = out.len() - start;
        out.resize(start + pad4(len), 0);
    } else {
        out.extend_from_slice(padding);
    }
}

/// A length byte the originals leave at 0.
fn unpinned_zero(bytes: &[u8]) -> Result<(), DecodeError> {
    if bytes.iter().any(|&b| b != 0) {
        return Err(DecodeError::Invalid("settings length byte"));
    }
    Ok(())
}

/// Section 0 of the settings: ports, history, flags, speed limit, name, description, address.
///
/// Offsets are from the section's start; the strings follow the 0x5c-byte
/// header in order and the section is padded to 4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MainSettings {
    /// Bytes 0x00 to 0x11; meaning unknown.
    pub ignored_00: [u8; 18],
    /// 0 when not set.
    pub file_transfer_port: u16,
    /// Bytes 0x14 and 0x15; meaning unknown.
    pub ignored_14: [u8; 2],
    /// 0 when not set.
    pub alternate_port: u16,
    /// Server History, "Record Server History to File": bit 0 login, 1
    /// disconnected, 2 download, 3 upload, 4 delete, 5 file list, 6 other
    /// file, 7 modify account, 8 other account, 9 server settings, 10 other.
    pub history_kinds: u32,
    /// "Start new file when size exceeds", in bytes; 1 MiB by default.
    pub history_max_file_size: u32,
    /// Bytes 0x20 to 0x22; meaning unknown.
    pub ignored_20: [u8; 3],
    /// Bit 0: register with trackers. Bit 1: the greeting is on.
    pub flags: u8,
    /// Bytes 0x24 and 0x25; meaning unknown.
    pub ignored_24: [u8; 2],
    /// The main port, 10700 by default.
    pub port: u16,
    /// Bytes 0x28 to 0x2b; meaning unknown.
    pub ignored_28: [u8; 4],
    /// Twelve bytes that differ per server directory; probably an identifier.
    pub ignored_2c: [u8; 12],
    /// Bytes per second; 0 when off.
    pub total_outgoing_speed_limit: u32,
    /// Bytes 0x3c to 0x51; meaning unknown.
    pub ignored_3c: [u8; 22],
    /// Byte 0x54, between the length bytes; 0.
    pub ignored_54: u8,
    /// Bytes 0x56 to 0x5b; meaning unknown.
    pub ignored_56: [u8; 6],
    /// Server name.
    pub name: Vec<u8>,
    /// Server description.
    pub description: Vec<u8>,
    /// Server address, as shown to clients.
    pub address: Vec<u8>,
    /// Padding after the strings when it is not zeros; empty otherwise.
    pub padding: Vec<u8>,
}

impl MainSettings {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x5c;
    /// Flag bit: register with the trackers.
    pub const REGISTER: u8 = 1;
    /// Flag bit: the greeting is enabled.
    pub const GREETING_ENABLED: u8 = 2;

    /// Whether the greeting is enabled.
    pub const fn greeting_enabled(&self) -> bool {
        self.flags & Self::GREETING_ENABLED != 0
    }

    /// Whether the server registers with its trackers.
    pub const fn register_with_trackers(&self) -> bool {
        self.flags & Self::REGISTER != 0
    }

    /// Decodes the section.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        unpinned_zero(&buf[0x54..0x55])?;
        let lens = [0x52, 0x53, 0x55].map(|at| usize::from(buf[at]));
        let ([name, description, address], end) = strings(buf, Self::HEADER_LEN, lens)?;
        let (padding, padded) = record_padding(buf, end)?;
        if padded != buf.len() {
            return Err(DecodeError::Invalid("main settings length"));
        }
        Ok(Self {
            ignored_00: buf[0..0x12].try_into().expect("18 bytes"),
            file_transfer_port: u16_at(buf, 0x12),
            ignored_14: buf[0x14..0x16].try_into().expect("2 bytes"),
            alternate_port: u16_at(buf, 0x16),
            history_kinds: u32_at(buf, 0x18),
            history_max_file_size: u32_at(buf, 0x1c),
            ignored_20: buf[0x20..0x23].try_into().expect("3 bytes"),
            flags: buf[0x23],
            ignored_24: buf[0x24..0x26].try_into().expect("2 bytes"),
            port: u16_at(buf, 0x26),
            ignored_28: buf[0x28..0x2c].try_into().expect("4 bytes"),
            ignored_2c: buf[0x2c..0x38].try_into().expect("12 bytes"),
            total_outgoing_speed_limit: u32_at(buf, 0x38),
            ignored_3c: buf[0x3c..0x52].try_into().expect("22 bytes"),
            ignored_54: buf[0x54],
            ignored_56: buf[0x56..0x5c].try_into().expect("6 bytes"),
            name,
            description,
            address,
            padding,
        })
    }

    /// Encodes the section, padded to 4.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..0x12].copy_from_slice(&self.ignored_00);
        out[0x12..0x14].copy_from_slice(&self.file_transfer_port.to_be_bytes());
        out[0x14..0x16].copy_from_slice(&self.ignored_14);
        out[0x16..0x18].copy_from_slice(&self.alternate_port.to_be_bytes());
        out[0x18..0x1c].copy_from_slice(&self.history_kinds.to_be_bytes());
        out[0x1c..0x20].copy_from_slice(&self.history_max_file_size.to_be_bytes());
        out[0x20..0x23].copy_from_slice(&self.ignored_20);
        out[0x23] = self.flags;
        out[0x24..0x26].copy_from_slice(&self.ignored_24);
        out[0x26..0x28].copy_from_slice(&self.port.to_be_bytes());
        out[0x28..0x2c].copy_from_slice(&self.ignored_28);
        out[0x2c..0x38].copy_from_slice(&self.ignored_2c);
        out[0x38..0x3c].copy_from_slice(&self.total_outgoing_speed_limit.to_be_bytes());
        out[0x3c..0x52].copy_from_slice(&self.ignored_3c);
        let (name, description, address) = (
            str255(&self.name),
            str255(&self.description),
            str255(&self.address),
        );
        out[0x52] = name.len() as u8;
        out[0x53] = description.len() as u8;
        out[0x54] = self.ignored_54;
        out[0x55] = address.len() as u8;
        out[0x56..0x5c].copy_from_slice(&self.ignored_56);
        out.extend_from_slice(name);
        out.extend_from_slice(description);
        out.extend_from_slice(address);
        pad_record(&mut out, 0, &self.padding);
        out
    }
}

impl Default for MainSettings {
    fn default() -> Self {
        Self {
            ignored_00: [0; 18],
            file_transfer_port: 0,
            ignored_14: [0; 2],
            alternate_port: 0,
            history_kinds: 0,
            history_max_file_size: 0x10_0000,
            ignored_20: [0; 3],
            flags: 0,
            ignored_24: [0; 2],
            port: 10700,
            ignored_28: [0; 4],
            ignored_2c: [0; 12],
            total_outgoing_speed_limit: 0,
            ignored_3c: [0; 22],
            ignored_54: 0,
            ignored_56: [0; 6],
            name: b"New Server".to_vec(),
            description: Vec::new(),
            address: Vec::new(),
            padding: Vec::new(),
        }
    }
}

/// A tracker the server registers with: a record of section 1.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrackerEntry {
    /// Bytes 0x00 to 0x13; meaning unknown.
    pub ignored_00: [u8; 20],
    /// Bytes 0x18 and 0x19; 0.
    pub ignored_18: [u8; 2],
    /// Byte 0x1b; 0.
    pub ignored_1b: u8,
    /// Bytes 0x1c to 0x1f; meaning unknown.
    pub ignored_1c: [u8; 4],
    /// Display name.
    pub name: Vec<u8>,
    /// Comments.
    pub comments: Vec<u8>,
    /// Tracker address.
    pub address: Vec<u8>,
    /// Login at the tracker.
    pub login: Vec<u8>,
    /// Groups to register in, as typed.
    pub groups: Vec<u8>,
    /// Padding to 4 when it is not zeros; empty otherwise.
    pub padding: Vec<u8>,
}

impl TrackerEntry {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x20;

    /// Decodes one record at the start of `buf`, returning it and its padded length.
    pub fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        unpinned_zero(&[buf[0x18], buf[0x19], buf[0x1b]])?;
        let lens = [0x14, 0x15, 0x16, 0x17, 0x1a].map(|at| usize::from(buf[at]));
        let ([name, comments, address, login, groups], end) = strings(buf, Self::HEADER_LEN, lens)?;
        let (padding, padded) = record_padding(buf, end)?;
        Ok((
            Self {
                ignored_00: buf[0..0x14].try_into().expect("20 bytes"),
                ignored_18: buf[0x18..0x1a].try_into().expect("2 bytes"),
                ignored_1b: buf[0x1b],
                ignored_1c: buf[0x1c..0x20].try_into().expect("4 bytes"),
                name,
                comments,
                address,
                login,
                groups,
                padding,
            },
            padded,
        ))
    }

    /// Encodes the record, padded to 4.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..0x14].copy_from_slice(&self.ignored_00);
        let fields = [
            &self.name,
            &self.comments,
            &self.address,
            &self.login,
            &self.groups,
        ]
        .map(|s| str255(s));
        for (at, s) in [0x14, 0x15, 0x16, 0x17, 0x1a].into_iter().zip(fields) {
            out[at] = s.len() as u8;
        }
        out[0x18..0x1a].copy_from_slice(&self.ignored_18);
        out[0x1b] = self.ignored_1b;
        out[0x1c..0x20].copy_from_slice(&self.ignored_1c);
        for s in fields {
            out.extend_from_slice(s);
        }
        pad_record(&mut out, 0, &self.padding);
        out
    }
}

/// An allow or deny rule: a record of section 2.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AddressRule {
    /// When the rule expires, in milliseconds since 1900; 0 for never.
    pub expires: u64,
    /// Bytes 0x08 to 0x0f; meaning unknown.
    pub ignored_08: [u8; 8],
    /// Allow (true) or deny.
    pub allow: bool,
    /// Bytes 0x12 to 0x17; meaning unknown.
    pub ignored_12: [u8; 6],
    /// Byte 0x1b; 0.
    pub ignored_1b: u8,
    /// Address or pattern.
    pub address: Vec<u8>,
    /// Display name.
    pub name: Vec<u8>,
    /// Comments.
    pub comments: Vec<u8>,
    /// Padding to 4 when it is not zeros; empty otherwise.
    pub padding: Vec<u8>,
}

impl AddressRule {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x1c;

    /// Decodes one record at the start of `buf`, returning it and its padded length.
    pub fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let allow = match u16_at(buf, 0x10) {
            0 => false,
            1 => true,
            _ => return Err(DecodeError::Invalid("allow/deny flag")),
        };
        unpinned_zero(&buf[0x1b..0x1c])?;
        let lens = [0x18, 0x19, 0x1a].map(|at| usize::from(buf[at]));
        let ([address, name, comments], end) = strings(buf, Self::HEADER_LEN, lens)?;
        let (padding, padded) = record_padding(buf, end)?;
        Ok((
            Self {
                expires: u64_at(buf, 0),
                ignored_08: buf[8..0x10].try_into().expect("8 bytes"),
                allow,
                ignored_12: buf[0x12..0x18].try_into().expect("6 bytes"),
                ignored_1b: buf[0x1b],
                address,
                name,
                comments,
                padding,
            },
            padded,
        ))
    }

    /// Encodes the record, padded to 4.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..8].copy_from_slice(&self.expires.to_be_bytes());
        out[8..0x10].copy_from_slice(&self.ignored_08);
        out[0x10..0x12].copy_from_slice(&u16::from(self.allow).to_be_bytes());
        out[0x12..0x18].copy_from_slice(&self.ignored_12);
        let fields = [&self.address, &self.name, &self.comments].map(|s| str255(s));
        for (at, s) in [0x18, 0x19, 0x1a].into_iter().zip(fields) {
            out[at] = s.len() as u8;
        }
        out[0x1b] = self.ignored_1b;
        for s in fields {
            out.extend_from_slice(s);
        }
        pad_record(&mut out, 0, &self.padding);
        out
    }
}
