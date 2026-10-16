//! 152 Get User Info and its reply, 153 User Info.

use super::{len_u8, need, pad4, strings, u32_at, u64_at, DecodeError, EncodeError};

/// 152 Get User Info.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GetUserInfo {
    /// The user to describe.
    pub user_id: u32,
}

impl GetUserInfo {
    /// Payload length.
    pub const LEN: usize = 4;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::LEN)?;
        Ok(Self {
            user_id: u32_at(buf, 0),
        })
    }

    /// Encodes the payload.
    pub const fn encode(&self) -> [u8; Self::LEN] {
        self.user_id.to_be_bytes()
    }
}

/// What the server tells an administrator about one user.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UserInfo {
    /// User ID.
    pub user_id: u32,
    /// Bytes 0x04 to 0x07; meaning unknown.
    pub ignored_04: [u8; 4],
    /// Login time in milliseconds.
    pub login_time: u64,
    /// A time in milliseconds.
    pub time_10: u64,
    /// A time in milliseconds.
    pub time_18: u64,
    /// Bytes 0x20 to 0x3a; meaning unknown.
    pub ignored_20: [u8; 0x1b],
    /// Byte 0x3c; meaning unknown.
    pub ignored_3c: u8,
    /// Byte 0x3e; meaning unknown.
    pub ignored_3e: u8,
    /// Bytes 0x42 and 0x43; meaning unknown.
    pub ignored_42: [u8; 2],
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
    /// Bytes after the strings, uninitialised padding to 4 on the wire; empty in a new message.
    pub padding: Vec<u8>,
}

impl UserInfo {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x44;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [0x3b, 0x3d, 0x3f, 0x40, 0x41].map(|at| usize::from(buf[at]));
        let ([address, nick, name_3f, login, name_41], end) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            user_id: u32_at(buf, 0),
            ignored_04: buf[4..8].try_into().expect("4 bytes"),
            login_time: u64_at(buf, 8),
            time_10: u64_at(buf, 0x10),
            time_18: u64_at(buf, 0x18),
            ignored_20: buf[0x20..0x3b].try_into().expect("27 bytes"),
            ignored_3c: buf[0x3c],
            ignored_3e: buf[0x3e],
            ignored_42: buf[0x42..0x44].try_into().expect("2 bytes"),
            address,
            nick,
            name_3f,
            login,
            name_41,
            padding: buf[end..].to_vec(),
        })
    }

    /// Encodes the payload, padded to 4 with zeros past the stored padding.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..4].copy_from_slice(&self.user_id.to_be_bytes());
        out[4..8].copy_from_slice(&self.ignored_04);
        out[8..0x10].copy_from_slice(&self.login_time.to_be_bytes());
        out[0x10..0x18].copy_from_slice(&self.time_10.to_be_bytes());
        out[0x18..0x20].copy_from_slice(&self.time_18.to_be_bytes());
        out[0x20..0x3b].copy_from_slice(&self.ignored_20);
        out[0x3b] = len_u8(&self.address, "address")?;
        out[0x3c] = self.ignored_3c;
        out[0x3d] = len_u8(&self.nick, "nick")?;
        out[0x3e] = self.ignored_3e;
        out[0x3f] = len_u8(&self.name_3f, "name")?;
        out[0x40] = len_u8(&self.login, "login")?;
        out[0x41] = len_u8(&self.name_41, "name")?;
        out[0x42..0x44].copy_from_slice(&self.ignored_42);
        for s in [
            &self.address,
            &self.nick,
            &self.name_3f,
            &self.login,
            &self.name_41,
            &self.padding,
        ] {
            out.extend_from_slice(s);
        }
        out.resize(pad4(out.len()), 0);
        Ok(out)
    }
}
