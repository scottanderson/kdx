//! 105 Server Info.

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
