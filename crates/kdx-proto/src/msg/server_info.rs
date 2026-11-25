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

/// Where the sections of a [`ServerSettings`] blob sit; not a wire field.
///
/// The server reads each section at its table offset, in any order, so a
/// blob may lay them out in any way. [`ServerSettings::decode`] records the
/// layout it found and [`ServerSettings::encode`] replays it. Each `None`
/// stands for Server 1.620's own layout: sections in table order, each
/// starting where the previous one ended, padded with zeros to 4 after every
/// section but the icon. That is also the [`Default`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SettingsLayout {
    /// Section indexes in the order they sit in the blob. Sections not
    /// listed follow in table order.
    pub order: Vec<usize>,
    /// The bytes before each section, after the previous one; `None` when
    /// they are the server's padding.
    pub gaps: [Option<Vec<u8>>; ServerSettings::SECTIONS],
    /// The bytes after the last section, before [`ServerSettings::trailing`];
    /// `None` when they are the server's padding.
    pub trailing_pad: Option<Vec<u8>>,
    /// The table offset of an empty section that has a placeholder, when it
    /// is not where the server would put it.
    pub empty_offsets: [Option<u32>; ServerSettings::SECTIONS],
    /// The tracker count at 0x6f when it is not the number of
    /// [`ServerSettings::trackers`]; the server reads the records whatever
    /// it says.
    pub tracker_count_raw: Option<u8>,
}

/// The server settings blob of 171 Server Settings Info and 172 Set Server Settings.
///
/// The sections the lab has seen non-empty are typed: 0 main, 1 trackers,
/// 2 allow/deny list, 3 greeting, 5 icon, 7 chat rooms. Sections 4, 6 and
/// 8 to 10 ([`Self::OTHER`]) are kept as bytes. A decoded blob encodes to
/// the same bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerSettings {
    /// Format version; 1 in the lab.
    pub version: u16,
    /// Bytes 0x0a and 0x0b; meaning unknown.
    pub ignored: [u8; 2],
    /// Bytes 0x64 to 0x6e; `01 01 01 01 00 01 00 02 00 00 00` from the server.
    /// The twelfth byte of the header, at 0x6f, is the tracker count.
    pub value_64: [u8; 11],
    /// Whether an empty section still gets an offset in the table
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
    /// Section 7: probably the chat rooms, as 219 Chat Info records back to back.
    pub chats: Vec<ChatRecord>,
    /// Sections [`Self::OTHER`], by index; the typed ones stay empty here.
    pub other: [Vec<u8>; Self::SECTIONS],
    /// Bytes after the last section and its padding; never seen non-empty.
    pub trailing: Vec<u8>,
    /// Where the sections sit in the blob.
    pub layout: SettingsLayout,
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
    /// Indexes of the sections kept as bytes in [`Self::other`].
    pub const OTHER: [usize; 5] = [4, 6, 8, 9, 10];
    const TABLE_AT: usize = 0x0c;
    /// The table entries the server writes for empty sections.
    const SERVER_PLACEHOLDERS: [bool; Self::SECTIONS] = [
        true, true, true, true, false, true, false, true, false, false, false,
    ];
    /// Whether the server pads a section to 4; it leaves the icon unpadded.
    const PADDED: [bool; Self::SECTIONS] = [
        true, true, true, true, true, false, true, true, true, true, true,
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
        let placeholders = table.map(|section| section.offset != 0);
        let mut order: Vec<usize> = (0..Self::SECTIONS).filter(|&i| table[i].len > 0).collect();
        order.sort_by_key(|&i| table[i].offset);
        let mut layout = SettingsLayout::default();
        let mut sections: [&[u8]; Self::SECTIONS] = [&[]; Self::SECTIONS];
        // Each section is at its table offset, in any order, as long as none overlap.
        let mut at = Self::HEADER_LEN;
        let mut prev = None;
        for &i in &order {
            let start = table[i].offset as usize;
            let end = start
                .checked_add(table[i].len as usize)
                .filter(|&end| start >= at && end <= buf.len())
                .ok_or(DecodeError::Invalid("settings section offset"))?;
            let gap = &buf[at..start];
            if gap != Self::server_gap(at, prev) {
                layout.gaps[i] = Some(gap.to_vec());
            }
            sections[i] = &buf[start..end];
            at = end;
            prev = Some(i);
        }
        let pad = Self::server_gap(at, prev);
        let trailing = if buf[at..].starts_with(&pad) {
            buf[at + pad.len()..].to_vec()
        } else {
            let padded = pad4(at).min(buf.len());
            layout.trailing_pad = Some(buf[at..padded].to_vec());
            buf[padded..].to_vec()
        };
        let main = MainSettings::decode(sections[Self::MAIN])?;
        let trackers = records(sections[Self::TRACKERS], TrackerEntry::decode)?;
        if usize::from(tracker_count) != trackers.len().min(255) {
            layout.tracker_count_raw = Some(tracker_count);
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
        let chats = records(sections[Self::CHATS], ChatRecord::decode_with_len)?;
        let mut other: [Vec<u8>; Self::SECTIONS] = Default::default();
        for i in Self::OTHER {
            other[i] = sections[i].to_vec();
        }
        let server = Self::empty_offsets(&table);
        for i in 0..Self::SECTIONS {
            if table[i].len == 0 && placeholders[i] && table[i].offset != server[i] {
                layout.empty_offsets[i] = Some(table[i].offset);
            }
        }
        // Table order needs no list.
        if !order.is_sorted() {
            layout.order = order;
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
            layout,
        })
    }

    /// Encodes the blob.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let bodies = self.bodies()?;
        let (table, order) = self.place(&bodies);
        let mut out = vec![0; Self::HEADER_LEN];
        out[..4].copy_from_slice(&Self::PROTOCOL);
        out[4..8].copy_from_slice(&Self::SUB_PROTOCOL);
        out[8..0x0a].copy_from_slice(&self.version.to_be_bytes());
        out[0x0a..Self::TABLE_AT].copy_from_slice(&self.ignored);
        for (i, section) in table.iter().enumerate() {
            let at = Self::TABLE_AT + 8 * i;
            out[at..at + 4].copy_from_slice(&section.offset.to_be_bytes());
            out[at + 4..at + 8].copy_from_slice(&section.len.to_be_bytes());
        }
        out[0x64..Self::HEADER_LEN - 1].copy_from_slice(&self.value_64);
        out[Self::HEADER_LEN - 1] = self
            .layout
            .tracker_count_raw
            .unwrap_or(self.trackers.len().min(255) as u8);
        let mut prev = None;
        for i in order {
            match &self.layout.gaps[i] {
                Some(gap) => out.extend_from_slice(gap),
                None => out.extend(Self::server_gap(out.len(), prev)),
            }
            out.extend_from_slice(&bodies[i]);
            prev = Some(i);
        }
        match &self.layout.trailing_pad {
            Some(pad) => out.extend_from_slice(pad),
            None => out.extend(Self::server_gap(out.len(), prev)),
        }
        out.extend_from_slice(&self.trailing);
        Ok(out)
    }

    /// The section table as [`Self::encode`] writes it.
    ///
    /// # Panics
    ///
    /// If a section does not encode; see [`Self::encode`].
    pub fn sections(&self) -> [SettingsSection; Self::SECTIONS] {
        let bodies = self.bodies().expect("settings sections encode");
        self.place(&bodies).0
    }

    /// The bytes of section `index`; empty when it is empty or absent.
    ///
    /// # Panics
    ///
    /// If `index` is not below [`Self::SECTIONS`], or if the section does
    /// not encode; see [`Self::encode`].
    pub fn section(&self, index: usize) -> Vec<u8> {
        self.try_section(index).expect("settings section encodes")
    }

    fn try_section(&self, index: usize) -> Result<Vec<u8>, EncodeError> {
        fn concat<T>(
            items: &[T],
            one: impl Fn(&T) -> Result<Vec<u8>, EncodeError>,
        ) -> Result<Vec<u8>, EncodeError> {
            let mut out = Vec::new();
            for item in items {
                out.extend(one(item)?);
            }
            Ok(out)
        }
        Ok(match index {
            Self::MAIN => self.main.encode()?,
            Self::TRACKERS => concat(&self.trackers, TrackerEntry::encode)?,
            Self::ALLOW_DENY => concat(&self.allow_deny, AddressRule::encode)?,
            Self::GREETING => self
                .greeting
                .as_ref()
                .map(ServerGreeting::encode)
                .unwrap_or_default(),
            Self::ICON => self.icon.clone().unwrap_or_default(),
            Self::CHATS => concat(&self.chats, ChatRecord::encode)?,
            _ => self.other[index].clone(),
        })
    }

    fn bodies(&self) -> Result<[Vec<u8>; Self::SECTIONS], EncodeError> {
        let mut bodies: [Vec<u8>; Self::SECTIONS] = Default::default();
        for (i, body) in bodies.iter_mut().enumerate() {
            *body = self.try_section(i)?;
        }
        Ok(bodies)
    }

    /// The section table and the order of the non-empty sections in the blob.
    fn place(
        &self,
        bodies: &[Vec<u8>; Self::SECTIONS],
    ) -> ([SettingsSection; Self::SECTIONS], Vec<usize>) {
        let mut order = Vec::new();
        for i in self.layout.order.iter().copied().chain(0..Self::SECTIONS) {
            if bodies.get(i).is_some_and(|b| !b.is_empty()) && !order.contains(&i) {
                order.push(i);
            }
        }
        let mut table = [SettingsSection::default(); Self::SECTIONS];
        let mut at = Self::HEADER_LEN;
        let mut prev = None;
        for &i in &order {
            at += match &self.layout.gaps[i] {
                Some(gap) => gap.len(),
                None => Self::server_gap(at, prev).len(),
            };
            table[i] = SettingsSection {
                offset: at as u32,
                len: bodies[i].len() as u32,
            };
            at += bodies[i].len();
            prev = Some(i);
        }
        let server = Self::empty_offsets(&table);
        for (i, body) in bodies.iter().enumerate() {
            if body.is_empty() && self.placeholders[i] {
                table[i].offset = self.layout.empty_offsets[i].unwrap_or(server[i]);
            }
        }
        (table, order)
    }

    /// The zeros the server writes at `at` after section `prev`.
    fn server_gap(at: usize, prev: Option<usize>) -> Vec<u8> {
        if prev.is_none_or(|i| Self::PADDED[i]) {
            vec![0; pad4(at) - at]
        } else {
            Vec::new()
        }
    }

    /// Where the server puts each empty section: after the non-empty
    /// section before it in the table, padded as the server pads it.
    fn empty_offsets(table: &[SettingsSection; Self::SECTIONS]) -> [u32; Self::SECTIONS] {
        let mut out = [0; Self::SECTIONS];
        let mut at = Self::HEADER_LEN;
        for i in 0..Self::SECTIONS {
            if table[i].len == 0 {
                out[i] = at as u32;
            } else {
                at = (table[i].offset + table[i].len) as usize;
                if Self::PADDED[i] {
                    at = pad4(at);
                }
            }
        }
        out
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
            layout: SettingsLayout::default(),
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

/// Writes the length bytes at `ats` and appends the strings in the same order.
fn put_strings(
    out: &mut Vec<u8>,
    ats: &[usize],
    strings: &[&Vec<u8>],
    what: &'static str,
) -> Result<(), EncodeError> {
    for (&at, s) in ats.iter().zip(strings) {
        out[at] = len_u8(s, what)?;
    }
    for s in strings {
        out.extend_from_slice(s);
    }
    Ok(())
}

/// The padding of a record from `end` to the next multiple of 4, or to the
/// end of `buf` when the section ends first; `None` when it is the zeros
/// the encoder writes anyway.
fn record_padding(buf: &[u8], end: usize) -> (Option<Vec<u8>>, usize) {
    let padded = pad4(end).min(buf.len());
    let pad = &buf[end..padded];
    let kept = if padded == pad4(end) && pad.iter().all(|&b| b == 0) {
        None
    } else {
        Some(pad.to_vec())
    };
    (kept, padded)
}

/// Appends `padding` if it was kept, else zeros up to the next multiple of 4.
fn pad_record(out: &mut Vec<u8>, padding: Option<&[u8]>) {
    match padding {
        Some(padding) => out.extend_from_slice(padding),
        None => out.resize(pad4(out.len()), 0),
    }
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
    /// Byte 0x54, between the length bytes; the server neither reads nor writes it.
    pub ignored_54: u8,
    /// Bytes 0x56 to 0x5b; meaning unknown.
    pub ignored_56: [u8; 6],
    /// Server name.
    pub name: Vec<u8>,
    /// Server description.
    pub description: Vec<u8>,
    /// Server address, as shown to clients.
    pub address: Vec<u8>,
    /// The bytes after the strings as read when they are not zeros to 4,
    /// which includes a section that ends before them; `None` otherwise.
    pub padding: Option<Vec<u8>>,
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
        let lens = [0x52, 0x53, 0x55].map(|at| usize::from(buf[at]));
        let ([name, description, address], end) = strings(buf, Self::HEADER_LEN, lens)?;
        let (padding, padded) = record_padding(buf, end);
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
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
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
        out[0x54] = self.ignored_54;
        out[0x56..0x5c].copy_from_slice(&self.ignored_56);
        put_strings(
            &mut out,
            &[0x52, 0x53, 0x55],
            &[&self.name, &self.description, &self.address],
            "server settings string",
        )?;
        pad_record(&mut out, self.padding.as_deref());
        Ok(out)
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
            padding: None,
        }
    }
}

/// A tracker the server registers with: a record of section 1.
///
/// Twelve length bytes at 0x14 to 0x1f, and the strings after the fixed part
/// in the same order. The record is padded to 4.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrackerEntry {
    /// Bytes 0x00 to 0x13; meaning unknown.
    pub ignored_00: [u8; 20],
    /// Display name; length at 0x14.
    pub name: Vec<u8>,
    /// Comments; length at 0x15.
    pub comments: Vec<u8>,
    /// Tracker address; length at 0x16.
    pub address: Vec<u8>,
    /// Login at the tracker; length at 0x17.
    pub login: Vec<u8>,
    /// Length at 0x18; meaning unknown.
    pub str_18: Vec<u8>,
    /// Length at 0x19; meaning unknown.
    pub str_19: Vec<u8>,
    /// Groups to register in, as typed; length at 0x1a.
    pub groups: Vec<u8>,
    /// Length at 0x1b; meaning unknown.
    pub str_1b: Vec<u8>,
    /// Length at 0x1c; meaning unknown.
    pub str_1c: Vec<u8>,
    /// Length at 0x1d; meaning unknown.
    pub str_1d: Vec<u8>,
    /// Length at 0x1e; meaning unknown.
    pub str_1e: Vec<u8>,
    /// Length at 0x1f; meaning unknown.
    pub str_1f: Vec<u8>,
    /// The padding as read when it is not zeros to 4, which includes the
    /// last record of a section that ends before it; `None` otherwise.
    pub padding: Option<Vec<u8>>,
}

impl TrackerEntry {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x20;
    const LENS_AT: [usize; 12] = [
        0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
    ];

    /// Decodes one record at the start of `buf`, returning it and its padded length.
    pub fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = Self::LENS_AT.map(|at| usize::from(buf[at]));
        let (
            [name, comments, address, login, str_18, str_19, groups, str_1b, str_1c, str_1d, str_1e, str_1f],
            end,
        ) = strings(buf, Self::HEADER_LEN, lens)?;
        let (padding, padded) = record_padding(buf, end);
        Ok((
            Self {
                ignored_00: buf[0..0x14].try_into().expect("20 bytes"),
                name,
                comments,
                address,
                login,
                str_18,
                str_19,
                groups,
                str_1b,
                str_1c,
                str_1d,
                str_1e,
                str_1f,
                padding,
            },
            padded,
        ))
    }

    /// Encodes the record, padded to 4.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..0x14].copy_from_slice(&self.ignored_00);
        put_strings(
            &mut out,
            &Self::LENS_AT,
            &[
                &self.name,
                &self.comments,
                &self.address,
                &self.login,
                &self.str_18,
                &self.str_19,
                &self.groups,
                &self.str_1b,
                &self.str_1c,
                &self.str_1d,
                &self.str_1e,
                &self.str_1f,
            ],
            "tracker entry string",
        )?;
        pad_record(&mut out, self.padding.as_deref());
        Ok(out)
    }
}

/// An allow or deny rule: a record of section 2.
///
/// Four length bytes at 0x18 to 0x1b, and the strings after the fixed part
/// in the same order. The record is padded to 4.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AddressRule {
    /// When the rule expires, in milliseconds since 1900; 0 for never.
    pub expires: u64,
    /// Bytes 0x08 to 0x0f; meaning unknown.
    pub ignored_08: [u8; 8],
    /// 1 to allow, else deny; the server keeps the value as it is.
    pub allow_raw: u16,
    /// Bytes 0x12 to 0x17; meaning unknown.
    pub ignored_12: [u8; 6],
    /// Address or pattern; length at 0x18.
    pub address: Vec<u8>,
    /// Display name; length at 0x19.
    pub name: Vec<u8>,
    /// Comments; length at 0x1a.
    pub comments: Vec<u8>,
    /// Length at 0x1b, which the server reads but leaves at 0; meaning unknown.
    pub str_1b: Vec<u8>,
    /// The padding as read when it is not zeros to 4, which includes the
    /// last record of a section that ends before it; `None` otherwise.
    pub padding: Option<Vec<u8>>,
}

impl AddressRule {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x1c;

    /// Whether the rule allows rather than denies.
    pub const fn allow(&self) -> bool {
        self.allow_raw == 1
    }

    /// Decodes one record at the start of `buf`, returning it and its padded length.
    pub fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [0x18, 0x19, 0x1a, 0x1b].map(|at| usize::from(buf[at]));
        let ([address, name, comments, str_1b], end) = strings(buf, Self::HEADER_LEN, lens)?;
        let (padding, padded) = record_padding(buf, end);
        Ok((
            Self {
                expires: u64_at(buf, 0),
                ignored_08: buf[8..0x10].try_into().expect("8 bytes"),
                allow_raw: u16_at(buf, 0x10),
                ignored_12: buf[0x12..0x18].try_into().expect("6 bytes"),
                address,
                name,
                comments,
                str_1b,
                padding,
            },
            padded,
        ))
    }

    /// Encodes the record, padded to 4.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..8].copy_from_slice(&self.expires.to_be_bytes());
        out[8..0x10].copy_from_slice(&self.ignored_08);
        out[0x10..0x12].copy_from_slice(&self.allow_raw.to_be_bytes());
        out[0x12..0x18].copy_from_slice(&self.ignored_12);
        put_strings(
            &mut out,
            &[0x18, 0x19, 0x1a, 0x1b],
            &[&self.address, &self.name, &self.comments, &self.str_1b],
            "allow/deny string",
        )?;
        pad_record(&mut out, self.padding.as_deref());
        Ok(out)
    }
}
