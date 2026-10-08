//! 153 User Info, the reply to 152 Get User Info.

use super::{len_u8, need, pad4, strings, u32_at, u64_at, DecodeError, EncodeError};

/// What the server tells an administrator about one user.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UserInfo {
    /// User ID.
    pub user_id: u32,
    /// Login time in milliseconds.
    pub login_time: u64,
    /// A time in milliseconds.
    pub time_10: u64,
    /// A time in milliseconds.
    pub time_18: u64,
    /// Address as `ip:port`.
    pub address: Vec<u8>,
    /// Nick.
    pub nick: Vec<u8>,
    /// Probably the account or class name.
    pub name_3f: Vec<u8>,
    /// Login.
    pub login: Vec<u8>,
    /// Probably the account or class name.
    pub name_41: Vec<u8>,
}

impl UserInfo {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x44;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [0x3b, 0x3d, 0x3f, 0x40, 0x41].map(|at| usize::from(buf[at]));
        let ([address, nick, name_3f, login, name_41], _) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            user_id: u32_at(buf, 0),
            login_time: u64_at(buf, 8),
            time_10: u64_at(buf, 0x10),
            time_18: u64_at(buf, 0x18),
            address,
            nick,
            name_3f,
            login,
            name_41,
        })
    }

    /// Encodes the payload, padded with zeros to 4 bytes.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..4].copy_from_slice(&self.user_id.to_be_bytes());
        out[8..0x10].copy_from_slice(&self.login_time.to_be_bytes());
        out[0x10..0x18].copy_from_slice(&self.time_10.to_be_bytes());
        out[0x18..0x20].copy_from_slice(&self.time_18.to_be_bytes());
        out[0x3b] = len_u8(&self.address, "address")?;
        out[0x3d] = len_u8(&self.nick, "nick")?;
        out[0x3f] = len_u8(&self.name_3f, "name")?;
        out[0x40] = len_u8(&self.login, "login")?;
        out[0x41] = len_u8(&self.name_41, "name")?;
        for s in [
            &self.address,
            &self.nick,
            &self.name_3f,
            &self.login,
            &self.name_41,
        ] {
            out.extend_from_slice(s);
        }
        out.resize(pad4(out.len()), 0);
        Ok(out)
    }
}
