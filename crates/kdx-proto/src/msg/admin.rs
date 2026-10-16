//! Administration: the connection monitor, disconnects, broadcasts, server
//! history, accounts, processes, and folder access.

use crate::{AccountKey, MessageType};

use super::{
    decode_counted, encode_counted, len_u8, need, pad4, padding, strings, u16_at, u32_at, u64_at,
    DecodeError, EncodeError, FilePath,
};

/// 114 Extra Server Info.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExtraServerInfo {
    /// Bytes 0x00 to 0x0f; meaning unknown.
    pub ignored_00: [u8; 0x10],
    /// Server start time in milliseconds.
    pub start_time: u64,
    /// Current server time in milliseconds.
    pub now: u64,
    /// Uptime in seconds.
    pub uptime: u32,
    /// An operating system value; 0x1000 in the lab.
    pub os_value_24: u32,
    /// An operating system value; 1000 in the lab.
    pub os_value_28: u32,
    /// Bytes 0x2d to 0x2f; meaning unknown.
    pub ignored_2d: [u8; 3],
    /// Operating system description, as from `uname`.
    pub os: Vec<u8>,
}

impl ExtraServerInfo {
    /// Size of the fixed part before the string.
    pub const HEADER_LEN: usize = 0x30;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let ([os], _) = strings(buf, Self::HEADER_LEN, [usize::from(buf[0x2c])])?;
        Ok(Self {
            ignored_00: buf[..0x10].try_into().expect("16 bytes"),
            start_time: u64_at(buf, 0x10),
            now: u64_at(buf, 0x18),
            uptime: u32_at(buf, 0x20),
            os_value_24: u32_at(buf, 0x24),
            os_value_28: u32_at(buf, 0x28),
            ignored_2d: buf[0x2d..0x30].try_into().expect("3 bytes"),
            os,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[..0x10].copy_from_slice(&self.ignored_00);
        out[0x10..0x18].copy_from_slice(&self.start_time.to_be_bytes());
        out[0x18..0x20].copy_from_slice(&self.now.to_be_bytes());
        out[0x20..0x24].copy_from_slice(&self.uptime.to_be_bytes());
        out[0x24..0x28].copy_from_slice(&self.os_value_24.to_be_bytes());
        out[0x28..0x2c].copy_from_slice(&self.os_value_28.to_be_bytes());
        out[0x2c] = len_u8(&self.os, "OS string")?;
        out[0x2d..0x30].copy_from_slice(&self.ignored_2d);
        out.extend_from_slice(&self.os);
        Ok(out)
    }
}

/// 116 Your Address: the client's address as the server sees it, as `ip:port`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct YourAddress(pub Vec<u8>);

impl YourAddress {
    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, 1)?;
        let ([address], _) = strings(buf, 1, [usize::from(buf[0])])?;
        Ok(Self(address))
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![len_u8(&self.0, "address")?];
        out.extend_from_slice(&self.0);
        Ok(out)
    }
}

/// A record of 143 Connection Monitor Info: a connection, or one of its file transfers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MonitoredConnection {
    /// User ID.
    pub user_id: u32,
    /// Transfer ID; 0 in a connection record.
    pub transfer_id: u32,
    /// Login time in milliseconds; the start time in a transfer record.
    pub login_time: u64,
    /// A time in milliseconds; 0 in a transfer record.
    pub time_10: u64,
    /// The server's current time in milliseconds.
    pub time_18: u64,
    /// Bytes moved so far.
    pub bytes_done: u64,
    /// Bytes to move in all.
    pub bytes_total: u64,
    /// Bytes per second since the start.
    pub rate: u32,
    /// Seconds until the transfer ends.
    pub seconds_left: u32,
    /// A per-transfer value; the speed limit in `captures/linux-catalog-volumes.pcapng`.
    pub value_38: u32,
    /// Transfer state; 0x19c (412) for a download.
    pub state: u32,
    /// Bytes 0x40 and 0x41; meaning unknown.
    pub ignored_40: [u8; 2],
    /// Account class index.
    pub class_index: u8,
    /// Bytes 0x43 to 0x5f; meaning unknown.
    pub ignored_43: [u8; 0x1d],
    /// Bytes 0x64 and 0x65; meaning unknown.
    pub ignored_64: [u8; 2],
    /// Nick.
    pub nick: Vec<u8>,
    /// Login.
    pub login: Vec<u8>,
    /// Account class name.
    pub account: Vec<u8>,
    /// Address as `ip:port`; the transfer connection's in a transfer record.
    pub address: Vec<u8>,
    /// The transferred file; the root in a connection record.
    pub path: FilePath,
    /// Bytes after the path up to the next multiple of 4, as received; empty in a new record.
    pub padding: Vec<u8>,
}

impl MonitoredConnection {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x68;

    /// Decodes the payload of 143 Connection Monitor Info, which has no count.
    pub fn decode_list(mut buf: &[u8]) -> Result<Vec<Self>, DecodeError> {
        let mut list = Vec::new();
        while !buf.is_empty() {
            let (record, len) = Self::decode(buf)?;
            list.push(record);
            buf = &buf[pad4(len).min(buf.len())..];
        }
        Ok(list)
    }

    fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [0x60, 0x61, 0x62, 0x63].map(|at| usize::from(buf[at]));
        let ([nick, login, account, address], at) = strings(buf, Self::HEADER_LEN, lens)?;
        let end = at + usize::from(u16_at(buf, 0x66));
        need(buf, end)?;
        let (padding, padded) = padding(buf, end);
        let record = Self {
            user_id: u32_at(buf, 0),
            transfer_id: u32_at(buf, 4),
            login_time: u64_at(buf, 8),
            time_10: u64_at(buf, 0x10),
            time_18: u64_at(buf, 0x18),
            bytes_done: u64_at(buf, 0x20),
            bytes_total: u64_at(buf, 0x28),
            rate: u32_at(buf, 0x30),
            seconds_left: u32_at(buf, 0x34),
            value_38: u32_at(buf, 0x38),
            state: u32_at(buf, 0x3c),
            ignored_40: buf[0x40..0x42].try_into().expect("2 bytes"),
            class_index: buf[0x42],
            ignored_43: buf[0x43..0x60].try_into().expect("29 bytes"),
            ignored_64: buf[0x64..0x66].try_into().expect("2 bytes"),
            nick,
            login,
            account,
            address,
            path: FilePath::decode(&buf[at..end])?,
            padding,
        };
        Ok((record, padded))
    }

    /// Encodes records as 143 Connection Monitor Info, padding with zeros past the stored padding.
    pub fn encode_list(list: &[Self]) -> Result<Vec<u8>, EncodeError> {
        let mut out = Vec::new();
        for record in list {
            let path = record.path.encode()?;
            let path_len = u16::try_from(path.len()).map_err(|_| EncodeError::TooLong("path"))?;
            let mut fixed = vec![0; Self::HEADER_LEN];
            fixed[0..4].copy_from_slice(&record.user_id.to_be_bytes());
            fixed[4..8].copy_from_slice(&record.transfer_id.to_be_bytes());
            fixed[8..0x10].copy_from_slice(&record.login_time.to_be_bytes());
            fixed[0x10..0x18].copy_from_slice(&record.time_10.to_be_bytes());
            fixed[0x18..0x20].copy_from_slice(&record.time_18.to_be_bytes());
            fixed[0x20..0x28].copy_from_slice(&record.bytes_done.to_be_bytes());
            fixed[0x28..0x30].copy_from_slice(&record.bytes_total.to_be_bytes());
            fixed[0x30..0x34].copy_from_slice(&record.rate.to_be_bytes());
            fixed[0x34..0x38].copy_from_slice(&record.seconds_left.to_be_bytes());
            fixed[0x38..0x3c].copy_from_slice(&record.value_38.to_be_bytes());
            fixed[0x3c..0x40].copy_from_slice(&record.state.to_be_bytes());
            fixed[0x40..0x42].copy_from_slice(&record.ignored_40);
            fixed[0x42] = record.class_index;
            fixed[0x43..0x60].copy_from_slice(&record.ignored_43);
            fixed[0x64..0x66].copy_from_slice(&record.ignored_64);
            let parts = [
                &record.nick,
                &record.login,
                &record.account,
                &record.address,
            ];
            for (i, s) in parts.iter().enumerate() {
                fixed[0x60 + i] = len_u8(s, "connection string")?;
            }
            fixed[0x66..0x68].copy_from_slice(&path_len.to_be_bytes());
            out.extend(fixed);
            for s in parts {
                out.extend_from_slice(s);
            }
            out.extend(path);
            out.extend_from_slice(&record.padding);
            out.resize(pad4(out.len()), 0);
        }
        Ok(out)
    }
}

/// 150 Disconnect User.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DisconnectUser {
    /// The user to disconnect.
    pub user_id: u32,
    /// Bytes 0x04 to 0x07; meaning unknown.
    pub ignored_04: [u8; 4],
    /// How long to ban the user's address, in seconds; 0 for no ban.
    pub ban_seconds: u32,
    /// Bytes 0x0c to 0x10; meaning unknown.
    pub ignored_0c: [u8; 5],
    /// [`Self::SILENT`] or 0.
    pub options: u8,
    /// Bytes 0x12 and 0x13; meaning unknown.
    pub ignored_12: [u8; 2],
    /// Bytes 0x15 to 0x17; meaning unknown.
    pub ignored_15: [u8; 3],
    /// Message for the user.
    pub message: Vec<u8>,
}

impl DisconnectUser {
    /// Size of the fixed part before the message.
    pub const HEADER_LEN: usize = 0x18;
    /// Longest ban the server accepts.
    pub const MAX_BAN_SECONDS: u32 = 0x80000;
    /// Disconnects without sending 151 Disconnect Message.
    pub const SILENT: u8 = 1;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let ([message], _) = strings(buf, Self::HEADER_LEN, [usize::from(buf[0x14])])?;
        Ok(Self {
            user_id: u32_at(buf, 0),
            ignored_04: buf[4..8].try_into().expect("4 bytes"),
            ban_seconds: u32_at(buf, 8),
            ignored_0c: buf[0x0c..0x11].try_into().expect("5 bytes"),
            options: buf[0x11],
            ignored_12: buf[0x12..0x14].try_into().expect("2 bytes"),
            ignored_15: buf[0x15..0x18].try_into().expect("3 bytes"),
            message,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..4].copy_from_slice(&self.user_id.to_be_bytes());
        out[4..8].copy_from_slice(&self.ignored_04);
        out[8..0x0c].copy_from_slice(&self.ban_seconds.to_be_bytes());
        out[0x0c..0x11].copy_from_slice(&self.ignored_0c);
        out[0x11] = self.options;
        out[0x12..0x14].copy_from_slice(&self.ignored_12);
        out[0x14] = len_u8(&self.message, "message")?;
        out[0x15..0x18].copy_from_slice(&self.ignored_15);
        out.extend_from_slice(&self.message);
        Ok(out)
    }
}

/// 151 Disconnect Message, the notice a user gets before the server disconnects it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DisconnectMessage {
    /// The user who sent it.
    pub user_id: u32,
    /// Bytes 0x04 to 0x0b; meaning unknown.
    pub ignored: [u8; 8],
    /// [`Self::BANNED`], [`Self::SHUTDOWN`], or 0.
    pub flags: u16,
    /// Meaning unknown; 0 in the lab.
    pub value_0e: u16,
    /// Meaning unknown; 0 in the lab.
    pub value_10: u16,
    /// Message text.
    pub message: Vec<u8>,
    /// The sender's name.
    pub name: Vec<u8>,
}

impl DisconnectMessage {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x14;
    /// The user's address is banned.
    pub const BANNED: u16 = 2;
    /// The server is shutting down or upgrading.
    pub const SHUTDOWN: u16 = 4;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [usize::from(buf[0x12]), usize::from(buf[0x13])];
        let ([message, name], _) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            user_id: u32_at(buf, 0),
            ignored: buf[4..0x0c].try_into().expect("8 bytes"),
            flags: u16_at(buf, 0x0c),
            value_0e: u16_at(buf, 0x0e),
            value_10: u16_at(buf, 0x10),
            message,
            name,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..4].copy_from_slice(&self.user_id.to_be_bytes());
        out[4..0x0c].copy_from_slice(&self.ignored);
        out[0x0c..0x0e].copy_from_slice(&self.flags.to_be_bytes());
        out[0x0e..0x10].copy_from_slice(&self.value_0e.to_be_bytes());
        out[0x10..0x12].copy_from_slice(&self.value_10.to_be_bytes());
        out[0x12] = len_u8(&self.message, "message")?;
        out[0x13] = len_u8(&self.name, "name")?;
        out.extend_from_slice(&self.message);
        out.extend_from_slice(&self.name);
        Ok(out)
    }
}

/// 154 Send Broadcast Message.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SendBroadcast {
    /// Meaning unknown; 0 in the lab.
    pub value_0: u16,
    /// Copied to 0x0e of 155 Rcv Broadcast Message; 0 in the lab.
    pub value_2: u16,
    /// Message text, to the end of the payload.
    pub text: Vec<u8>,
}

impl SendBroadcast {
    /// Size of the fixed part before the text.
    pub const HEADER_LEN: usize = 4;
    /// Longest text the server accepts.
    pub const MAX_TEXT_LEN: usize = 0x800;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        Ok(Self {
            value_0: u16_at(buf, 0),
            value_2: u16_at(buf, 2),
            text: buf[Self::HEADER_LEN..].to_vec(),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        if self.text.len() > Self::MAX_TEXT_LEN {
            return Err(EncodeError::TooLong("broadcast text"));
        }
        let mut out = Vec::with_capacity(Self::HEADER_LEN + self.text.len());
        out.extend_from_slice(&self.value_0.to_be_bytes());
        out.extend_from_slice(&self.value_2.to_be_bytes());
        out.extend_from_slice(&self.text);
        Ok(out)
    }
}

/// 155 Rcv Broadcast Message.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RcvBroadcast {
    /// The sender; 0 for the server's own notices.
    pub user_id: u32,
    /// Bytes 0x04 to 0x0d; meaning unknown.
    pub ignored: [u8; 0x0a],
    /// Copied from 0x02 of 154 Send Broadcast Message.
    pub value_0e: u16,
    /// Message text, to the end of the payload.
    pub text: Vec<u8>,
}

impl RcvBroadcast {
    /// Size of the fixed part before the text.
    pub const HEADER_LEN: usize = 0x10;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        Ok(Self {
            user_id: u32_at(buf, 0),
            ignored: buf[4..0x0e].try_into().expect("10 bytes"),
            value_0e: u16_at(buf, 0x0e),
            text: buf[Self::HEADER_LEN..].to_vec(),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..4].copy_from_slice(&self.user_id.to_be_bytes());
        out[4..0x0e].copy_from_slice(&self.ignored);
        out[0x0e..0x10].copy_from_slice(&self.value_0e.to_be_bytes());
        out.extend_from_slice(&self.text);
        out
    }
}

/// 174 Shutdown Server.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShutdownServer {
    /// Byte 0x00; meaning unknown.
    pub ignored_00: u8,
    /// [`Self::EXIT`], [`Self::RESTART_COMPUTER`], or [`Self::SHUT_DOWN_COMPUTER`].
    pub mode: u8,
    /// Bytes 0x02 and 0x03; meaning unknown.
    pub ignored_02: [u8; 2],
    /// Message for the users, to the end of the payload.
    pub message: Vec<u8>,
}

impl ShutdownServer {
    /// Size of the fixed part before the message.
    pub const HEADER_LEN: usize = 4;
    /// Longest message the server accepts.
    pub const MAX_MESSAGE_LEN: usize = 0xff;
    /// Exits the server program.
    pub const EXIT: u8 = 1;
    /// Runs `/sbin/shutdown -r now` before exiting.
    pub const RESTART_COMPUTER: u8 = 2;
    /// Runs `/sbin/shutdown -h now` before exiting.
    pub const SHUT_DOWN_COMPUTER: u8 = 4;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        Ok(Self {
            ignored_00: buf[0],
            mode: buf[1],
            ignored_02: buf[2..4].try_into().expect("2 bytes"),
            message: buf[Self::HEADER_LEN..].to_vec(),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        if self.message.len() > Self::MAX_MESSAGE_LEN {
            return Err(EncodeError::TooLong("shutdown message"));
        }
        let [a, b] = self.ignored_02;
        let mut out = vec![self.ignored_00, self.mode, a, b];
        out.extend_from_slice(&self.message);
        Ok(out)
    }
}

/// A server history record, as in 181 Server History Join (R) and 183 Server History Added.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryRecord {
    /// When it happened, in milliseconds.
    pub time: u64,
    /// One of 184 to 191, or the type of the logged request.
    pub kind: MessageType,
    /// The user it is about; 0 for the server itself.
    pub user_id: u32,
    /// Bytes 0x10 to 0x41; meaning unknown.
    pub ignored_10: [u8; 0x32],
    /// Bytes 0x44 and 0x45; meaning unknown.
    pub ignored_44: [u8; 2],
    /// Bytes 0x4a and 0x4b; meaning unknown.
    pub ignored_4a: [u8; 2],
    /// The user's address as `ip:port`.
    pub address: Vec<u8>,
    /// The user's nick.
    pub nick: Vec<u8>,
    /// The user's login.
    pub login: Vec<u8>,
    /// A detail of the event, such as the new nick of 100 or the chat name of 202.
    pub detail: Vec<u8>,
    /// The file of a file event; the root otherwise.
    pub path: FilePath,
    /// Bytes after the path (to the next multiple of 4 in 181), as received; empty in a new record.
    pub padding: Vec<u8>,
}

impl HistoryRecord {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x4c;

    /// Decodes 183 Server History Added, one record.
    pub fn decode_added(buf: &[u8]) -> Result<Self, DecodeError> {
        let (mut record, end) = Self::decode(buf)?;
        record.padding.extend_from_slice(&buf[end..]);
        Ok(record)
    }

    fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [0x46, 0x47, 0x48, 0x49].map(|at| usize::from(buf[at]));
        let ([address, nick, login, detail], at) = strings(buf, Self::HEADER_LEN, lens)?;
        let end = at + usize::from(u16_at(buf, 0x42));
        need(buf, end)?;
        let (padding, padded) = padding(buf, end);
        let record = Self {
            time: u64_at(buf, 0),
            kind: MessageType(u32_at(buf, 8)),
            user_id: u32_at(buf, 0x0c),
            ignored_10: buf[0x10..0x42].try_into().expect("50 bytes"),
            ignored_44: buf[0x44..0x46].try_into().expect("2 bytes"),
            ignored_4a: buf[0x4a..0x4c].try_into().expect("2 bytes"),
            address,
            nick,
            login,
            detail,
            path: FilePath::decode(&buf[at..end])?,
            padding,
        };
        Ok((record, padded))
    }

    /// Encodes 183 Server History Added, padded to 4 with zeros past the stored padding.
    pub fn encode_added(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = self.encode()?;
        out.resize(pad4(out.len()), 0);
        Ok(out)
    }

    fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let path = self.path.encode()?;
        let path_len = u16::try_from(path.len()).map_err(|_| EncodeError::TooLong("path"))?;
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..8].copy_from_slice(&self.time.to_be_bytes());
        out[8..0x0c].copy_from_slice(&self.kind.0.to_be_bytes());
        out[0x0c..0x10].copy_from_slice(&self.user_id.to_be_bytes());
        out[0x10..0x42].copy_from_slice(&self.ignored_10);
        out[0x42..0x44].copy_from_slice(&path_len.to_be_bytes());
        out[0x44..0x46].copy_from_slice(&self.ignored_44);
        out[0x4a..0x4c].copy_from_slice(&self.ignored_4a);
        let parts = [&self.address, &self.nick, &self.login, &self.detail];
        for (i, s) in parts.iter().enumerate() {
            out[0x46 + i] = len_u8(s, "history string")?;
        }
        for s in parts {
            out.extend_from_slice(s);
        }
        out.extend(path);
        out.extend_from_slice(&self.padding);
        Ok(out)
    }
}

/// 181 Server History Join (R).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HistoryList {
    /// Bytes 0x00 to 0x03; 0 from the server.
    pub ignored: [u8; 4],
    /// The records.
    pub records: Vec<HistoryRecord>,
}

impl HistoryList {
    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        let records = decode_counted(buf, 4, HistoryRecord::decode)?;
        Ok(Self {
            ignored: buf[..4].try_into().expect("4 bytes"),
            records,
        })
    }

    /// Encodes the payload, padding each record to 4 with zeros past its stored padding.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = self.ignored.to_vec();
        encode_counted(&mut out, &self.records, HistoryRecord::encode)?;
        Ok(out)
    }
}

/// An account login with no length byte: 508 Delete Account, 509 Get Account and 515 Account Deleted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccountLogin(pub Vec<u8>);

impl AccountLogin {
    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(Self(buf.to_vec()))
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Vec<u8> {
        self.0.clone()
    }
}

/// A class index: 502 Get Account Class.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AccountClassIndex(pub u8);

impl AccountClassIndex {
    /// Payload length.
    pub const LEN: usize = 1;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::LEN)?;
        Ok(Self(buf[0]))
    }

    /// Encodes the payload.
    pub const fn encode(&self) -> [u8; Self::LEN] {
        [self.0]
    }
}

/// The key of the new password: 550 Change Own Password.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeOwnPassword(pub AccountKey);

impl ChangeOwnPassword {
    /// Payload length.
    pub const LEN: usize = 32;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::LEN)?;
        Ok(Self(AccountKey(
            buf[..Self::LEN].try_into().expect("32 bytes"),
        )))
    }

    /// Encodes the payload.
    pub const fn encode(&self) -> [u8; Self::LEN] {
        self.0 .0
    }
}

/// An account class, as in 503 Account Class Info, 504 Set Account Class and 506 Account Class List.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccountClass {
    /// Privilege bits, most significant bit of byte 0 = bit 0.
    pub privileges: [u8; 16],
    /// A time in milliseconds.
    pub time: u64,
    /// Bytes 0x18 to 0x24; meaning unknown.
    pub ignored_18: [u8; 0x0d],
    /// Flood protection in commands per 2 minutes, such as 0x50 for Administrator in 503; 0 in 506.
    pub ignored_25: u8,
    /// Bytes 0x26 and 0x27; stored by the server as given.
    pub ignored_26: [u8; 2],
    /// The 8 bytes at 0x28 of 503 and 504, which 506 leaves out.
    pub ignored_28: [u8; 8],
    /// Class index.
    pub index: u8,
    /// The three bytes after the index; 0xff in the first for Administrator.
    pub ignored_after_index: [u8; 3],
    /// The byte after the editor login length; meaning unknown.
    pub ignored_after_lengths: u8,
    /// Class name.
    pub name: Vec<u8>,
    /// A second string; empty in the lab.
    pub second: Vec<u8>,
    /// The login of whoever last saved the class; the server sets it on 504.
    pub editor: Vec<u8>,
    /// Bytes after the strings (to the next multiple of 4 in 506), uninitialised on the wire; empty in a new record.
    pub padding: Vec<u8>,
}

impl AccountClass {
    /// Offset of the index in 503 Account Class Info.
    const INFO_INDEX_AT: usize = 0x30;
    /// Offset of the index in an entry of 506 Account Class List.
    const LIST_INDEX_AT: usize = 0x28;

    /// Decodes 503 Account Class Info.
    pub fn decode_info(buf: &[u8]) -> Result<Self, DecodeError> {
        let (mut class, end) = Self::decode_at(buf, Self::INFO_INDEX_AT)?;
        class.padding.extend_from_slice(&buf[end..]);
        Ok(class)
    }

    /// Decodes 504 Set Account Class, which has the layout of 503.
    pub fn decode_set(buf: &[u8]) -> Result<Self, DecodeError> {
        Self::decode_info(buf)
    }

    /// Decodes 506 Account Class List.
    pub fn decode_list(buf: &[u8]) -> Result<Vec<Self>, DecodeError> {
        decode_counted(buf, 0, |entry| Self::decode_at(entry, Self::LIST_INDEX_AT))
    }

    /// Encodes 503 Account Class Info, padded to 4 with zeros past the stored padding.
    pub fn encode_info(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = self.encode_at(Self::INFO_INDEX_AT)?;
        out.resize(pad4(out.len()), 0);
        Ok(out)
    }

    /// Encodes 504 Set Account Class: the 503 layout with only the stored padding.
    pub fn encode_set(&self) -> Result<Vec<u8>, EncodeError> {
        self.encode_at(Self::INFO_INDEX_AT)
    }

    /// Encodes 506 Account Class List.
    pub fn encode_list(list: &[Self]) -> Result<Vec<u8>, EncodeError> {
        let mut out = Vec::new();
        encode_counted(&mut out, list, |c| c.encode_at(Self::LIST_INDEX_AT))?;
        Ok(out)
    }

    fn decode_at(buf: &[u8], index_at: usize) -> Result<(Self, usize), DecodeError> {
        let names_at = index_at + 8;
        need(buf, names_at)?;
        let lens = [4, 5, 6].map(|i| usize::from(buf[index_at + i]));
        let ([name, second, editor], end) = strings(buf, names_at, lens)?;
        let (padding, padded) = padding(buf, end);
        let class = Self {
            privileges: buf[..16].try_into().expect("16 bytes"),
            time: u64_at(buf, 0x10),
            ignored_18: buf[0x18..0x25].try_into().expect("13 bytes"),
            ignored_25: buf[0x25],
            ignored_26: buf[0x26..0x28].try_into().expect("2 bytes"),
            ignored_28: if index_at == Self::INFO_INDEX_AT {
                buf[0x28..0x30].try_into().expect("8 bytes")
            } else {
                [0; 8]
            },
            index: buf[index_at],
            ignored_after_index: buf[index_at + 1..index_at + 4].try_into().expect("3 bytes"),
            ignored_after_lengths: buf[index_at + 7],
            name,
            second,
            editor,
            padding,
        };
        Ok((class, padded))
    }

    fn encode_at(&self, index_at: usize) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; index_at + 8];
        out[..16].copy_from_slice(&self.privileges);
        out[0x10..0x18].copy_from_slice(&self.time.to_be_bytes());
        out[0x18..0x25].copy_from_slice(&self.ignored_18);
        out[0x25] = self.ignored_25;
        out[0x26..0x28].copy_from_slice(&self.ignored_26);
        if index_at == Self::INFO_INDEX_AT {
            out[0x28..0x30].copy_from_slice(&self.ignored_28);
        }
        out[index_at] = self.index;
        out[index_at + 1..index_at + 4].copy_from_slice(&self.ignored_after_index);
        out[index_at + 4] = len_u8(&self.name, "class name")?;
        out[index_at + 5] = len_u8(&self.second, "second string")?;
        out[index_at + 6] = len_u8(&self.editor, "editor login")?;
        out[index_at + 7] = self.ignored_after_lengths;
        for s in [&self.name, &self.second, &self.editor, &self.padding] {
            out.extend_from_slice(s);
        }
        Ok(out)
    }
}

/// An account record: the request of 507 Create Account and 511 Set Account, and 510 Account Info.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Account {
    /// Values of the privileges set in the mask.
    pub privileges: [u8; 16],
    /// Privileges the account sets itself, overriding its class.
    pub privilege_mask: [u8; 16],
    /// The account key; all zero in 511 to keep the current one.
    ///
    /// 510 Account Info never holds the key: byte 0 is 1 if the account has a password.
    pub key: [u8; 32],
    /// Creation time in milliseconds; the server sets it.
    pub created: u64,
    /// Modification time in milliseconds; the server sets it.
    pub modified: u64,
    /// Last login time in milliseconds; 0 if never.
    pub last_login: u64,
    /// Bytes 0x58 to 0x75; meaning unknown.
    pub ignored_58: [u8; 0x1e],
    /// Class index.
    pub class_index: u8,
    /// The three bytes after the class index, stored as given.
    pub ignored_77: [u8; 3],
    /// Bytes 0x7e and 0x7f; meaning unknown.
    pub ignored_7e: [u8; 2],
    /// Login.
    pub login: Vec<u8>,
    /// Account name.
    pub name: Vec<u8>,
    /// A third string, stored as given; empty in the lab.
    pub third: Vec<u8>,
    /// In 511, the login of the account to change; in 510, the login of whoever last saved it.
    pub editor: Vec<u8>,
    /// Bytes after the strings up to the next multiple of 4, uninitialised on the wire; empty in a new record.
    pub padding: Vec<u8>,
}

impl std::fmt::Debug for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Account")
            .field("privileges", &self.privileges)
            .field("privilege_mask", &self.privilege_mask)
            .field("key", &"..")
            .field("created", &self.created)
            .field("modified", &self.modified)
            .field("last_login", &self.last_login)
            .field("ignored_58", &self.ignored_58)
            .field("class_index", &self.class_index)
            .field("ignored_77", &self.ignored_77)
            .field("ignored_7e", &self.ignored_7e)
            .field("login", &self.login)
            .field("name", &self.name)
            .field("third", &self.third)
            .field("editor", &self.editor)
            .field("padding", &self.padding)
            .finish()
    }
}

impl Account {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x80;

    /// Whether 510 Account Info says the account has a password.
    pub const fn has_password(&self) -> bool {
        self.key[0] != 0
    }

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [0x7a, 0x7b, 0x7c, 0x7d].map(|at| usize::from(buf[at]));
        let ([login, name, third, editor], end) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            privileges: buf[..0x10].try_into().expect("16 bytes"),
            privilege_mask: buf[0x10..0x20].try_into().expect("16 bytes"),
            key: buf[0x20..0x40].try_into().expect("32 bytes"),
            created: u64_at(buf, 0x40),
            modified: u64_at(buf, 0x48),
            last_login: u64_at(buf, 0x50),
            ignored_58: buf[0x58..0x76].try_into().expect("30 bytes"),
            class_index: buf[0x76],
            ignored_77: buf[0x77..0x7a].try_into().expect("3 bytes"),
            ignored_7e: buf[0x7e..0x80].try_into().expect("2 bytes"),
            login,
            name,
            third,
            editor,
            padding: buf[end..].to_vec(),
        })
    }

    /// Encodes the payload, padded to 4 with zeros past the stored padding.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[..0x10].copy_from_slice(&self.privileges);
        out[0x10..0x20].copy_from_slice(&self.privilege_mask);
        out[0x20..0x40].copy_from_slice(&self.key);
        out[0x40..0x48].copy_from_slice(&self.created.to_be_bytes());
        out[0x48..0x50].copy_from_slice(&self.modified.to_be_bytes());
        out[0x50..0x58].copy_from_slice(&self.last_login.to_be_bytes());
        out[0x58..0x76].copy_from_slice(&self.ignored_58);
        out[0x76] = self.class_index;
        out[0x77..0x7a].copy_from_slice(&self.ignored_77);
        out[0x7e..0x80].copy_from_slice(&self.ignored_7e);
        out[0x7a] = len_u8(&self.login, "login")?;
        out[0x7b] = len_u8(&self.name, "account name")?;
        out[0x7c] = len_u8(&self.third, "third string")?;
        out[0x7d] = len_u8(&self.editor, "editor login")?;
        for s in [
            &self.login,
            &self.name,
            &self.third,
            &self.editor,
            &self.padding,
        ] {
            out.extend_from_slice(s);
        }
        out.resize(pad4(out.len()), 0);
        Ok(out)
    }
}

/// An entry of 513 Account List.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccountListEntry {
    /// Creation time in milliseconds.
    pub created: u64,
    /// Last login time in milliseconds; 0 if never.
    pub last_login: u64,
    /// Bytes 0x10 to 0x1a; meaning unknown.
    pub ignored_10: [u8; 0x0b],
    /// Meaning unknown; 1 for admin and 0 for guest in the lab.
    pub ignored_1b: u8,
    /// Bytes 0x1c to 0x1f; meaning unknown.
    pub ignored_1c: [u8; 4],
    /// Class index.
    pub class_index: u8,
    /// Meaning unknown; 0xff for admin, as in the class.
    pub ignored_21: u8,
    /// Bytes 0x22 and 0x23; meaning unknown.
    pub ignored_22: [u8; 2],
    /// Bytes 0x26 and 0x27; meaning unknown.
    pub ignored_26: [u8; 2],
    /// Login.
    pub login: Vec<u8>,
    /// Account name.
    pub name: Vec<u8>,
    /// Bytes after the name up to the next multiple of 4, as received; empty in a new entry.
    pub padding: Vec<u8>,
}

impl AccountListEntry {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x28;

    /// Decodes the payload of 513 Account List.
    pub fn decode_list(buf: &[u8]) -> Result<Vec<Self>, DecodeError> {
        decode_counted(buf, 0, |entry| {
            need(entry, Self::HEADER_LEN)?;
            let lens = [usize::from(entry[0x24]), usize::from(entry[0x25])];
            let ([login, name], end) = strings(entry, Self::HEADER_LEN, lens)?;
            let (padding, padded) = padding(entry, end);
            let account = Self {
                created: u64_at(entry, 0),
                last_login: u64_at(entry, 8),
                ignored_10: entry[0x10..0x1b].try_into().expect("11 bytes"),
                ignored_1b: entry[0x1b],
                ignored_1c: entry[0x1c..0x20].try_into().expect("4 bytes"),
                class_index: entry[0x20],
                ignored_21: entry[0x21],
                ignored_22: entry[0x22..0x24].try_into().expect("2 bytes"),
                ignored_26: entry[0x26..0x28].try_into().expect("2 bytes"),
                login,
                name,
                padding,
            };
            Ok((account, padded))
        })
    }

    /// Encodes the payload of 513 Account List.
    pub fn encode_list(list: &[Self]) -> Result<Vec<u8>, EncodeError> {
        let mut out = Vec::new();
        encode_counted(&mut out, list, |account| {
            let mut entry = vec![0; Self::HEADER_LEN];
            entry[0..8].copy_from_slice(&account.created.to_be_bytes());
            entry[8..0x10].copy_from_slice(&account.last_login.to_be_bytes());
            entry[0x10..0x1b].copy_from_slice(&account.ignored_10);
            entry[0x1b] = account.ignored_1b;
            entry[0x1c..0x20].copy_from_slice(&account.ignored_1c);
            entry[0x20] = account.class_index;
            entry[0x21] = account.ignored_21;
            entry[0x22..0x24].copy_from_slice(&account.ignored_22);
            entry[0x24] = len_u8(&account.login, "login")?;
            entry[0x25] = len_u8(&account.name, "account name")?;
            entry[0x26..0x28].copy_from_slice(&account.ignored_26);
            entry.extend_from_slice(&account.login);
            entry.extend_from_slice(&account.name);
            entry.extend_from_slice(&account.padding);
            Ok(entry)
        })?;
        Ok(out)
    }
}

/// 803 Process List.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProcessList {
    /// Probably the host's memory in bytes.
    pub memory_00: u64,
    /// Probably the host's memory in bytes.
    pub memory_08: u64,
    /// Bytes 0x10 to 0x1b; meaning unknown.
    pub ignored: [u8; 0x0c],
    /// The processes.
    pub processes: Vec<Process>,
}

/// A process in 803 Process List.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Process {
    /// Process ID.
    pub pid: u32,
    /// Bytes 0x04 to 0x07; meaning unknown.
    pub ignored_04: [u8; 4],
    /// Start time in milliseconds.
    pub start_time: u64,
    /// Bytes 0x10 to 0x38; meaning unknown.
    pub ignored_10: [u8; 0x29],
    /// Command line, cut at 0xff bytes.
    pub command: Vec<u8>,
    /// Third string, between the command and the user name; empty in captures.
    pub string_3b: Vec<u8>,
    /// User name.
    pub user: Vec<u8>,
    /// Bytes after the user name up to the next multiple of 4, as received; empty in a new entry.
    pub padding: Vec<u8>,
}

impl Default for Process {
    fn default() -> Self {
        Self {
            pid: 0,
            ignored_04: [0; 4],
            start_time: 0,
            ignored_10: [0; 0x29],
            command: Vec::new(),
            string_3b: Vec::new(),
            user: Vec::new(),
            padding: Vec::new(),
        }
    }
}

impl ProcessList {
    /// Size of the fixed part before the count.
    pub const HEADER_LEN: usize = 0x1c;
    const ENTRY_LEN: usize = 0x3c;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let processes = decode_counted(buf, Self::HEADER_LEN, |entry| {
            need(entry, Self::ENTRY_LEN)?;
            let lens = [
                usize::from(entry[0x3a]),
                usize::from(entry[0x3b]),
                usize::from(entry[0x39]),
            ];
            let ([command, string_3b, user], end) = strings(entry, Self::ENTRY_LEN, lens)?;
            let (padding, padded) = padding(entry, end);
            let process = Process {
                pid: u32_at(entry, 0),
                ignored_04: entry[4..8].try_into().expect("4 bytes"),
                start_time: u64_at(entry, 8),
                ignored_10: entry[0x10..0x39].try_into().expect("41 bytes"),
                command,
                string_3b,
                user,
                padding,
            };
            Ok((process, padded))
        })?;
        Ok(Self {
            memory_00: u64_at(buf, 0),
            memory_08: u64_at(buf, 8),
            ignored: buf[0x10..0x1c].try_into().expect("12 bytes"),
            processes,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..8].copy_from_slice(&self.memory_00.to_be_bytes());
        out[8..0x10].copy_from_slice(&self.memory_08.to_be_bytes());
        out[0x10..0x1c].copy_from_slice(&self.ignored);
        encode_counted(&mut out, &self.processes, |process| {
            let mut entry = vec![0; Self::ENTRY_LEN];
            entry[0..4].copy_from_slice(&process.pid.to_be_bytes());
            entry[4..8].copy_from_slice(&process.ignored_04);
            entry[8..0x10].copy_from_slice(&process.start_time.to_be_bytes());
            entry[0x10..0x39].copy_from_slice(&process.ignored_10);
            entry[0x39] = len_u8(&process.user, "user")?;
            entry[0x3a] = len_u8(&process.command, "command")?;
            entry[0x3b] = len_u8(&process.string_3b, "string_3b")?;
            entry.extend_from_slice(&process.command);
            entry.extend_from_slice(&process.string_3b);
            entry.extend_from_slice(&process.user);
            entry.extend_from_slice(&process.padding);
            Ok(entry)
        })?;
        Ok(out)
    }
}

/// 804 Exit/Terminate Process.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExitTerminateProcess {
    /// [`Self::KILL`] or [`Self::EXIT`], optionally with [`Self::EXTRA`].
    pub flags: u32,
    /// Process ID.
    pub pid: u32,
    /// Must be 0, or the server does nothing.
    pub value_08: u32,
}

impl ExitTerminateProcess {
    /// Payload length.
    pub const LEN: usize = 0x0c;
    /// Sends SIGKILL on Linux.
    pub const KILL: u32 = 1;
    /// Sends SIGTERM on Linux.
    pub const EXIT: u32 = 2;
    /// Meaning unknown; unsupported (0x10005) on Linux.
    pub const EXTRA: u32 = 4;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::LEN)?;
        Ok(Self {
            flags: u32_at(buf, 0),
            pid: u32_at(buf, 4),
            value_08: u32_at(buf, 8),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> [u8; Self::LEN] {
        let mut out = [0; Self::LEN];
        out[..4].copy_from_slice(&self.flags.to_be_bytes());
        out[4..8].copy_from_slice(&self.pid.to_be_bytes());
        out[8..].copy_from_slice(&self.value_08.to_be_bytes());
        out
    }
}

/// 1101, 1102 and 1103: an item name with no length byte or padding.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FolderAccessName(pub Vec<u8>);

impl FolderAccessName {
    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(Self(buf.to_vec()))
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Vec<u8> {
        self.0.clone()
    }
}

/// 1106 Get Folder Access List.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GetFolderAccessList {
    /// The optional u32; Client 1.620 sends 1 when it opens the file window.
    pub flags: Option<u32>,
    /// Bytes after the u32, or all of a payload under 4 bytes (under 4 long when `flags` is `None`).
    pub rest: Vec<u8>,
}

impl GetFolderAccessList {
    /// Length of the u32 when present.
    pub const FLAGS_LEN: usize = 4;

    /// Decodes the payload: a u32 if there are 4 bytes, then the rest.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        if buf.len() < Self::FLAGS_LEN {
            return Ok(Self {
                flags: None,
                rest: buf.to_vec(),
            });
        }
        Ok(Self {
            flags: Some(u32_at(buf, 0)),
            rest: buf[Self::FLAGS_LEN..].to_vec(),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        if let Some(flags) = self.flags {
            out.extend_from_slice(&flags.to_be_bytes());
        }
        out.extend_from_slice(&self.rest);
        out
    }
}

/// An item of 1107 Folder Access List.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FolderAccessItem {
    /// [`FolderAccess::BUILT_IN`] and [`FolderAccess::OWNER_IN_NAME`]; 1, 2 and 0 in the lab.
    pub flags: u32,
    /// Icon, as in a 401 File List entry; 0 for `[Default]`.
    pub icon: u32,
    /// Bytes 0x08 to 0x0f; meaning unknown.
    pub ignored_08: [u8; 8],
    /// Meaning unknown; 0 in the lab.
    pub ignored_10: u8,
    /// Item name.
    pub name: Vec<u8>,
    /// A second string; empty in the lab.
    pub second: Vec<u8>,
    /// Description.
    pub description: Vec<u8>,
    /// Bytes after the description up to the next multiple of 4, as received; empty in a new item.
    pub padding: Vec<u8>,
}

impl FolderAccessItem {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x14;

    /// Decodes the payload of 1107 Folder Access List.
    pub fn decode_list(buf: &[u8]) -> Result<Vec<Self>, DecodeError> {
        decode_counted(buf, 0, |entry| {
            need(entry, Self::HEADER_LEN)?;
            let lens = [0x11, 0x12, 0x13].map(|at| usize::from(entry[at]));
            let ([name, second, description], end) = strings(entry, Self::HEADER_LEN, lens)?;
            let (padding, padded) = padding(entry, end);
            let item = Self {
                flags: u32_at(entry, 0),
                icon: u32_at(entry, 4),
                ignored_08: entry[8..0x10].try_into().expect("8 bytes"),
                ignored_10: entry[0x10],
                name,
                second,
                description,
                padding,
            };
            Ok((item, padded))
        })
    }

    /// Encodes the payload of 1107 Folder Access List.
    pub fn encode_list(list: &[Self]) -> Result<Vec<u8>, EncodeError> {
        let mut out = Vec::new();
        encode_counted(&mut out, list, |item| {
            let mut entry = vec![0; Self::HEADER_LEN];
            entry[0..4].copy_from_slice(&item.flags.to_be_bytes());
            entry[4..8].copy_from_slice(&item.icon.to_be_bytes());
            entry[8..0x10].copy_from_slice(&item.ignored_08);
            entry[0x10] = item.ignored_10;
            entry[0x11] = len_u8(&item.name, "item name")?;
            entry[0x12] = len_u8(&item.second, "second string")?;
            entry[0x13] = len_u8(&item.description, "description")?;
            for s in [&item.name, &item.second, &item.description, &item.padding] {
                entry.extend_from_slice(s);
            }
            Ok(entry)
        })?;
        Ok(out)
    }
}

/// A folder access record: 1100 Create Folder Access, 1104 Folder Access Info and 1105 Set Folder Access.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FolderAccess {
    /// Meaning unknown; 0 in the lab.
    pub ignored_00: u32,
    /// [`Self::BUILT_IN`] and [`Self::OWNER_IN_NAME`].
    pub flags: u32,
    /// Icon, such as 0x2c for Dropbox.
    pub icon: u32,
    /// Probably the owner's rule; permission bits in the top u16.
    pub rule: u32,
    /// One rule per account class: permission bits in the top u16, bit 0 first.
    pub class_rules: [u32; 32],
    /// Bytes 0x90 to 0xac; meaning unknown.
    pub ignored_90: [u8; 0x1d],
    /// Meaning unknown; 0 in the lab.
    pub ignored_b0: u8,
    /// Bytes 0xb2 to 0xb7; meaning unknown.
    pub ignored_b2: [u8; 6],
    /// Item name, the tag in a folder name, such as `DB`.
    pub name: Vec<u8>,
    /// Owner login.
    pub owner: Vec<u8>,
    /// Description.
    pub description: Vec<u8>,
    /// In 1105, the name of the item to change.
    pub current_name: Vec<u8>,
    /// Bytes after the strings, uninitialised padding to 4 on the wire; empty in a new record.
    pub padding: Vec<u8>,
}

impl FolderAccess {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0xb8;
    /// In [`Self::flags`], the item is built in, as `[Default]` is.
    pub const BUILT_IN: u32 = 1;
    /// In [`Self::flags`], folder names carry the owner login.
    pub const OWNER_IN_NAME: u32 = 2;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [0xad, 0xae, 0xaf, 0xb1].map(|at| usize::from(buf[at]));
        let ([name, owner, description, current_name], end) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            ignored_00: u32_at(buf, 0),
            flags: u32_at(buf, 4),
            icon: u32_at(buf, 8),
            rule: u32_at(buf, 0x0c),
            class_rules: std::array::from_fn(|i| u32_at(buf, 0x10 + 4 * i)),
            ignored_90: buf[0x90..0xad].try_into().expect("29 bytes"),
            ignored_b0: buf[0xb0],
            ignored_b2: buf[0xb2..0xb8].try_into().expect("6 bytes"),
            name,
            owner,
            description,
            current_name,
            padding: buf[end..].to_vec(),
        })
    }

    /// Encodes the payload, padded to 4 with zeros past the stored padding.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..4].copy_from_slice(&self.ignored_00.to_be_bytes());
        out[4..8].copy_from_slice(&self.flags.to_be_bytes());
        out[8..0x0c].copy_from_slice(&self.icon.to_be_bytes());
        out[0x0c..0x10].copy_from_slice(&self.rule.to_be_bytes());
        for (i, rule) in self.class_rules.iter().enumerate() {
            out[0x10 + 4 * i..0x14 + 4 * i].copy_from_slice(&rule.to_be_bytes());
        }
        out[0x90..0xad].copy_from_slice(&self.ignored_90);
        out[0xad] = len_u8(&self.name, "item name")?;
        out[0xae] = len_u8(&self.owner, "owner login")?;
        out[0xaf] = len_u8(&self.description, "description")?;
        out[0xb0] = self.ignored_b0;
        out[0xb1] = len_u8(&self.current_name, "current name")?;
        out[0xb2..0xb8].copy_from_slice(&self.ignored_b2);
        for s in [
            &self.name,
            &self.owner,
            &self.description,
            &self.current_name,
            &self.padding,
        ] {
            out.extend_from_slice(s);
        }
        out.resize(pad4(out.len()), 0);
        Ok(out)
    }
}
