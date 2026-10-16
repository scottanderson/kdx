//! The user record of 100 Set Identity, 101 User List, 102 User Changed and
//! 103 User Joined, and the small user messages 104, 110, 111 and 118.

use super::{len_u8, need, pad4, padding, strings, u16_at, u32_at, DecodeError, EncodeError};

/// One user, as the client describes itself and as the server lists it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UserRecord {
    /// Server-assigned user ID; 0 in 100 Set Identity.
    pub user_id: u32,
    /// Status value, also set by 110 Set Mini User Icon.
    pub status: u8,
    /// Bytes 0x05 to 0x09; meaning unknown.
    pub ignored_05: [u8; 5],
    /// TCP port of the client's client-to-client (DCC) listener; 0 if it has none.
    pub client_port: u16,
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
    /// Byte 0x1b; meaning unknown.
    pub ignored_1b: u8,
    /// Icon bitmap; empty when the server leaves it out.
    pub icon: Vec<u8>,
    /// Extra data, skipped by the server.
    pub extra: Vec<u8>,
    /// Nickname.
    pub nick: Vec<u8>,
    /// Info text.
    pub info: Vec<u8>,
    /// Bytes after the info text up to the next multiple of 4, as received; empty in a new record.
    pub padding: Vec<u8>,
}

impl UserRecord {
    /// Size of the fixed part before the icon.
    pub const HEADER_LEN: usize = 0x1c;
    /// Longest icon the server accepts.
    pub const MAX_ICON_LEN: usize = 0x708;

    /// Decodes one record with its padding to 4 and returns it with its length.
    pub fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [
            usize::from(u16_at(buf, 0x0c)),
            usize::from(buf[0x18]),
            usize::from(buf[0x19]),
            usize::from(buf[0x1a]),
        ];
        let ([icon, extra, nick, info], end) = strings(buf, Self::HEADER_LEN, lens)?;
        let (padding, padded) = padding(buf, end);
        let record = Self {
            user_id: u32_at(buf, 0),
            status: buf[4],
            ignored_05: buf[5..0x0a].try_into().expect("5 bytes"),
            client_port: u16_at(buf, 0x0a),
            colors: buf[0x0e..0x14].try_into().expect("6 bytes"),
            server_14: buf[0x14],
            server_15: buf[0x15],
            server_16: buf[0x16],
            flags: buf[0x17],
            ignored_1b: buf[0x1b],
            icon,
            extra,
            nick,
            info,
            padding,
        };
        Ok((record, padded))
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

    /// Encodes the record with its stored padding; without any, as the client sends it.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        if self.icon.len() > Self::MAX_ICON_LEN {
            return Err(EncodeError::TooLong("icon"));
        }
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..4].copy_from_slice(&self.user_id.to_be_bytes());
        out[4] = self.status;
        out[5..0x0a].copy_from_slice(&self.ignored_05);
        out[0x0a..0x0c].copy_from_slice(&self.client_port.to_be_bytes());
        out[0x0c..0x0e].copy_from_slice(&(self.icon.len() as u16).to_be_bytes());
        out[0x0e..0x14].copy_from_slice(&self.colors);
        out[0x14] = self.server_14;
        out[0x15] = self.server_15;
        out[0x16] = self.server_16;
        out[0x17] = self.flags;
        out[0x18] = len_u8(&self.extra, "extra data")?;
        out[0x19] = len_u8(&self.nick, "nick")?;
        out[0x1a] = len_u8(&self.info, "info text")?;
        out[0x1b] = self.ignored_1b;
        for part in [
            &self.icon,
            &self.extra,
            &self.nick,
            &self.info,
            &self.padding,
        ] {
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

/// 104 User Departed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UserDeparted {
    /// The user who left.
    pub user_id: u32,
    /// Meaning unknown; 7 in every capture.
    pub value_4: u8,
    /// Bytes 0x05 to 0x07; meaning unknown.
    pub ignored: [u8; 3],
}

impl UserDeparted {
    /// Payload length.
    pub const LEN: usize = 8;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::LEN)?;
        Ok(Self {
            user_id: u32_at(buf, 0),
            value_4: buf[4],
            ignored: buf[5..8].try_into().expect("3 bytes"),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> [u8; Self::LEN] {
        let mut out = [0; Self::LEN];
        out[..4].copy_from_slice(&self.user_id.to_be_bytes());
        out[4] = self.value_4;
        out[5..].copy_from_slice(&self.ignored);
        out
    }
}

/// 110 Set Mini User Icon.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MiniUserIcon {
    /// Status: 1 after `/away`, 0 after `/back`.
    pub status: u8,
    /// Bytes 0x01 to 0x03; meaning unknown.
    pub ignored: [u8; 3],
}

impl MiniUserIcon {
    /// Payload length.
    pub const LEN: usize = 4;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::LEN)?;
        Ok(Self {
            status: buf[0],
            ignored: buf[1..4].try_into().expect("3 bytes"),
        })
    }

    /// Encodes the payload.
    pub const fn encode(&self) -> [u8; Self::LEN] {
        let [a, b, c] = self.ignored;
        [self.status, a, b, c]
    }
}

/// 111 Mini User Icon Changed: a user ID, then that user's 110 payload.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MiniUserIconChanged {
    /// The user whose status changed.
    pub user_id: u32,
    /// The 110 payload.
    pub icon: MiniUserIcon,
}

impl MiniUserIconChanged {
    /// Payload length.
    pub const LEN: usize = 8;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::LEN)?;
        Ok(Self {
            user_id: u32_at(buf, 0),
            icon: MiniUserIcon::decode(&buf[4..])?,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> [u8; Self::LEN] {
        let mut out = [0; Self::LEN];
        out[..4].copy_from_slice(&self.user_id.to_be_bytes());
        out[4..].copy_from_slice(&self.icon.encode());
        out
    }
}

/// 118, which reports whether the client is registered. Provisional name.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RegistrationNotice {
    /// The length of the client's purchase certificate; 0 without one.
    pub value: u32,
}

impl RegistrationNotice {
    /// Payload length.
    pub const LEN: usize = 4;

    /// Whether the server counts the client as registered.
    pub const fn is_registered(&self) -> bool {
        self.value != 0
    }

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::LEN)?;
        Ok(Self {
            value: u32_at(buf, 0),
        })
    }

    /// Encodes the payload.
    pub const fn encode(&self) -> [u8; Self::LEN] {
        self.value.to_be_bytes()
    }
}
