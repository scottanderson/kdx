//! The user record of 100 Set Identity, 101 User List, 102 User Changed and
//! 103 User Joined.

use super::{len_u8, need, pad4, strings, u16_at, u32_at, DecodeError, EncodeError};

/// One user, as the client describes itself and as the server lists it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UserRecord {
    /// Server-assigned user ID; 0 in 100 Set Identity.
    pub user_id: u32,
    /// Status value, also set by 110 Set Mini User Icon.
    pub status: u8,
    /// UDP port for voice chat.
    pub voice_port: u16,
    /// Six bytes the client chooses, probably two RGB colours.
    pub colors: [u8; 6],
    /// Set by the server from the account; 0xff for admin in the lab.
    pub server_14: u8,
    /// Set by the server; meaning unknown.
    pub server_15: u8,
    /// Set by the server; meaning unknown.
    pub server_16: u8,
    /// Flag bits: bit 2 comes from the client, bit 1 is the server's hidden-marker flag.
    pub flags: u8,
    /// Icon bitmap; empty when the server leaves it out.
    pub icon: Vec<u8>,
    /// Extra data, skipped by the server.
    pub extra: Vec<u8>,
    /// Nickname.
    pub nick: Vec<u8>,
    /// Info text.
    pub info: Vec<u8>,
}

impl UserRecord {
    /// Size of the fixed part before the icon.
    pub const HEADER_LEN: usize = 0x1c;
    /// Longest icon the server accepts.
    pub const MAX_ICON_LEN: usize = 0x708;

    /// Decodes one record and returns it with its unpadded length.
    pub fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [
            usize::from(u16_at(buf, 0x0c)),
            usize::from(buf[0x18]),
            usize::from(buf[0x19]),
            usize::from(buf[0x1a]),
        ];
        let ([icon, extra, nick, info], end) = strings(buf, Self::HEADER_LEN, lens)?;
        let record = Self {
            user_id: u32_at(buf, 0),
            status: buf[4],
            voice_port: u16_at(buf, 0x0a),
            colors: buf[0x0e..0x14].try_into().expect("6 bytes"),
            server_14: buf[0x14],
            server_15: buf[0x15],
            server_16: buf[0x16],
            flags: buf[0x17],
            icon,
            extra,
            nick,
            info,
        };
        Ok((record, end))
    }

    /// Decodes the records of 101 User List, each padded to 4 bytes.
    pub fn decode_list(mut buf: &[u8]) -> Result<Vec<Self>, DecodeError> {
        let mut list = Vec::new();
        while !buf.is_empty() {
            let (record, len) = Self::decode(buf)?;
            list.push(record);
            buf = &buf[pad4(len).min(buf.len())..];
        }
        Ok(list)
    }

    /// Encodes the record without padding, as the client sends it.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        if self.icon.len() > Self::MAX_ICON_LEN {
            return Err(EncodeError::TooLong("icon"));
        }
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..4].copy_from_slice(&self.user_id.to_be_bytes());
        out[4] = self.status;
        out[0x0a..0x0c].copy_from_slice(&self.voice_port.to_be_bytes());
        out[0x0c..0x0e].copy_from_slice(&(self.icon.len() as u16).to_be_bytes());
        out[0x0e..0x14].copy_from_slice(&self.colors);
        out[0x14] = self.server_14;
        out[0x15] = self.server_15;
        out[0x16] = self.server_16;
        out[0x17] = self.flags;
        out[0x18] = len_u8(&self.extra, "extra data")?;
        out[0x19] = len_u8(&self.nick, "nick")?;
        out[0x1a] = len_u8(&self.info, "info text")?;
        for part in [&self.icon, &self.extra, &self.nick, &self.info] {
            out.extend_from_slice(part);
        }
        Ok(out)
    }

    /// Encodes records as the server lists them, each padded with zeros to 4 bytes.
    pub fn encode_list(records: &[Self]) -> Result<Vec<u8>, EncodeError> {
        let mut out = Vec::new();
        for record in records {
            out.extend(record.encode()?);
            out.resize(pad4(out.len()), 0);
        }
        Ok(out)
    }
}
