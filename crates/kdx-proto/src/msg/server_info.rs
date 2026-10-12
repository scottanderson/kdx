//! 105 Server Info, 106 Server Icon, 107 Server Greeting, and the server
//! settings of 171 Server Settings Info and 172 Set Server Settings.

use super::{len_u8, need, strings, u16_at, u32_at, DecodeError, EncodeError};

/// What the server tells a client right after it logs in.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct ServerInfo {
    /// Account key for transfer connections; see `Connection::attach` in kdx-client.
    pub session_key: [u8; 32],
    /// A server value of unknown meaning; 0 in the lab.
    pub server_value: u32,
    /// The client's own user ID.
    pub user_id: u32,
    /// Server version, 1620 for 1.620.
    pub version: u16,
    /// Whether the user may change the server settings.
    pub can_modify_settings: bool,
    /// Server name.
    pub name: Vec<u8>,
    /// Server description.
    pub description: Vec<u8>,
    /// A third string of unknown meaning.
    pub third: Vec<u8>,
}

impl ServerInfo {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x4c;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [0x48, 0x49, 0x4a].map(|at| usize::from(buf[at]));
        let ([name, description, third], _) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            session_key: buf[..32].try_into().expect("32 bytes"),
            server_value: u32_at(buf, 0x20),
            user_id: u32_at(buf, 0x24),
            version: u16_at(buf, 0x28),
            can_modify_settings: u16_at(buf, 0x2a) != 0,
            name,
            description,
            third,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[..32].copy_from_slice(&self.session_key);
        out[0x20..0x24].copy_from_slice(&self.server_value.to_be_bytes());
        out[0x24..0x28].copy_from_slice(&self.user_id.to_be_bytes());
        out[0x28..0x2a].copy_from_slice(&self.version.to_be_bytes());
        out[0x2b] = self.can_modify_settings.into();
        out[0x47] = 2;
        out[0x48] = len_u8(&self.name, "server name")?;
        out[0x49] = len_u8(&self.description, "server description")?;
        out[0x4a] = len_u8(&self.third, "third string")?;
        for part in [&self.name, &self.description, &self.third] {
            out.extend_from_slice(part);
        }
        Ok(out)
    }
}

impl std::fmt::Debug for ServerInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerInfo")
            .field("session_key", &"<redacted>")
            .field("server_value", &self.server_value)
            .field("user_id", &self.user_id)
            .field("version", &self.version)
            .field("can_modify_settings", &self.can_modify_settings)
            .field("name", &String::from_utf8_lossy(&self.name))
            .field("description", &String::from_utf8_lossy(&self.description))
            .field("third", &String::from_utf8_lossy(&self.third))
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
            text: buf[Self::HEADER_LEN..].to_vec(),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::HEADER_LEN];
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
/// Only the fields the spec names are decoded; the rest is kept as is, so a
/// decoded blob encodes to the same bytes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerSettings {
    /// Format version; 1 in the lab.
    pub version: u16,
    /// The section table at 0x0c.
    pub sections: [SettingsSection; Self::SECTIONS],
    /// The 12 bytes after the table, at 0x64; meaning unknown.
    pub value_64: [u8; 12],
    /// Everything after the header, from [`Self::HEADER_LEN`]: the sections and their padding.
    pub body: Vec<u8>,
}

impl ServerSettings {
    /// The magic at the start of the blob.
    pub const MAGIC: &'static [u8; 8] = b"%KDXSSTG";
    /// Size of the header, which ends where the main section starts in the lab.
    pub const HEADER_LEN: usize = 0x70;
    /// Entries in the section table.
    pub const SECTIONS: usize = 11;
    /// Index of the main section.
    pub const MAIN: usize = 0;
    /// Index of the greeting section, in the layout of 107 Server Greeting.
    pub const GREETING: usize = 3;
    /// Index of the icon section, the bytes of 106 Server Icon.
    pub const ICON: usize = 5;
    /// Offset of the flags byte in the main section (0x93 in the blob).
    pub const FLAGS_AT: usize = 0x23;
    /// Flag bit: the greeting is enabled.
    pub const GREETING_ENABLED: u8 = 2;
    /// Offset of the speed limit in the main section (0xa8 in the blob).
    pub const SPEED_LIMIT_AT: usize = 0x38;
    /// Offset of the server name in the main section (0xcc in the blob).
    pub const NAME_AT: usize = 0x5c;
    const TABLE_AT: usize = 0x0c;

    /// Decodes the blob.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        if &buf[..8] != Self::MAGIC {
            return Err(DecodeError::Invalid("settings magic"));
        }
        let sections = std::array::from_fn(|i| SettingsSection {
            offset: u32_at(buf, Self::TABLE_AT + 8 * i),
            len: u32_at(buf, Self::TABLE_AT + 8 * i + 4),
        });
        let settings = Self {
            version: u16_at(buf, 8),
            sections,
            value_64: buf[0x64..Self::HEADER_LEN].try_into().expect("12 bytes"),
            body: buf[Self::HEADER_LEN..].to_vec(),
        };
        for i in 0..Self::SECTIONS {
            settings.try_section(i)?;
        }
        Ok(settings)
    }

    /// Encodes the blob.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[..8].copy_from_slice(Self::MAGIC);
        out[8..0x0a].copy_from_slice(&self.version.to_be_bytes());
        for (i, section) in self.sections.iter().enumerate() {
            let at = Self::TABLE_AT + 8 * i;
            out[at..at + 4].copy_from_slice(&section.offset.to_be_bytes());
            out[at + 4..at + 8].copy_from_slice(&section.len.to_be_bytes());
        }
        out[0x64..Self::HEADER_LEN].copy_from_slice(&self.value_64);
        out.extend_from_slice(&self.body);
        out
    }

    /// The bytes of section `index`; empty when its length is 0.
    ///
    /// # Panics
    ///
    /// If `index` is not below [`Self::SECTIONS`].
    pub fn section(&self, index: usize) -> &[u8] {
        self.try_section(index).unwrap_or_default()
    }

    fn try_section(&self, index: usize) -> Result<&[u8], DecodeError> {
        let SettingsSection { offset, len } = self.sections[index];
        if len == 0 {
            return Ok(&[]);
        }
        let start = (offset as usize)
            .checked_sub(Self::HEADER_LEN)
            .ok_or(DecodeError::Invalid("settings section offset"))?;
        let end = start + len as usize;
        need(&self.body, end).map_err(|_| DecodeError::TooShort {
            need: Self::HEADER_LEN + end,
            have: Self::HEADER_LEN + self.body.len(),
        })?;
        Ok(&self.body[start..end])
    }

    /// The flags byte of the main section; 0 if the section is too short.
    pub fn flags(&self) -> u8 {
        self.section(Self::MAIN)
            .get(Self::FLAGS_AT)
            .copied()
            .unwrap_or(0)
    }

    /// Whether the greeting is enabled.
    pub fn greeting_enabled(&self) -> bool {
        self.flags() & Self::GREETING_ENABLED != 0
    }

    /// The total outgoing speed limit in bytes per second; 0 when off.
    pub fn speed_limit(&self) -> u32 {
        let main = self.section(Self::MAIN);
        match main.get(Self::SPEED_LIMIT_AT..Self::SPEED_LIMIT_AT + 4) {
            Some(_) => u32_at(main, Self::SPEED_LIMIT_AT),
            None => 0,
        }
    }

    /// The server name, up to its NUL.
    pub fn name(&self) -> &[u8] {
        let rest = self.section(Self::MAIN).get(Self::NAME_AT..).unwrap_or(&[]);
        let end = rest.iter().position(|&b| b == 0).unwrap_or(rest.len());
        &rest[..end]
    }

    /// The greeting section; `None` when no greeting is set.
    pub fn greeting(&self) -> Option<ServerGreeting> {
        ServerGreeting::decode(self.section(Self::GREETING)).ok()
    }

    /// The icon section; empty when the blob has no icon.
    pub fn icon(&self) -> &[u8] {
        self.section(Self::ICON)
    }
}
