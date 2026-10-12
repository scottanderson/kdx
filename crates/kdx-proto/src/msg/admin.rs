//! Administration: the connection monitor, disconnects, broadcasts, server
//! history, accounts, processes, and folder access.

use crate::MessageType;

use super::{
    len_u8, need, pad4, strings, u16_at, u32_at, u64_at, DecodeError, EncodeError, FilePath,
};

/// Reads a u32 count followed by that many records, each padded to 4.
fn decode_counted<T>(
    buf: &[u8],
    count_at: usize,
    mut decode: impl FnMut(&[u8]) -> Result<(T, usize), DecodeError>,
) -> Result<Vec<T>, DecodeError> {
    need(buf, count_at + 4)?;
    let count = u32_at(buf, count_at) as usize;
    let mut at = count_at + 4;
    let mut list = Vec::with_capacity(count.min(buf.len()));
    for _ in 0..count {
        let (item, len) = decode(&buf[at..])?;
        list.push(item);
        at = (at + pad4(len)).min(buf.len());
    }
    Ok(list)
}

/// Appends a u32 count, then each record padded with zeros to 4.
fn encode_counted<T>(
    out: &mut Vec<u8>,
    list: &[T],
    mut encode: impl FnMut(&T) -> Result<Vec<u8>, EncodeError>,
) -> Result<(), EncodeError> {
    let count = u32::try_from(list.len()).map_err(|_| EncodeError::TooLong("list"))?;
    out.extend_from_slice(&count.to_be_bytes());
    for item in list {
        out.extend(encode(item)?);
        out.resize(pad4(out.len()), 0);
    }
    Ok(())
}

/// 114 Extra Server Info.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExtraServerInfo {
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
            start_time: u64_at(buf, 0x10),
            now: u64_at(buf, 0x18),
            uptime: u32_at(buf, 0x20),
            os_value_24: u32_at(buf, 0x24),
            os_value_28: u32_at(buf, 0x28),
            os,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0x10..0x18].copy_from_slice(&self.start_time.to_be_bytes());
        out[0x18..0x20].copy_from_slice(&self.now.to_be_bytes());
        out[0x20..0x24].copy_from_slice(&self.uptime.to_be_bytes());
        out[0x24..0x28].copy_from_slice(&self.os_value_24.to_be_bytes());
        out[0x28..0x2c].copy_from_slice(&self.os_value_28.to_be_bytes());
        out[0x2c] = len_u8(&self.os, "OS string")?;
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
    /// Account class index.
    pub class_index: u8,
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
            class_index: buf[0x42],
            nick,
            login,
            account,
            address,
            path: FilePath::decode(&buf[at..end])?,
        };
        Ok((record, end))
    }

    /// Encodes records as 143 Connection Monitor Info, padding with zeros.
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
            fixed[0x42] = record.class_index;
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
    /// How long to ban the user's address, in seconds; 0 for no ban.
    pub ban_seconds: u32,
    /// [`Self::SILENT`] or 0.
    pub options: u8,
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
            ban_seconds: u32_at(buf, 8),
            options: buf[0x11],
            message,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..4].copy_from_slice(&self.user_id.to_be_bytes());
        out[8..0x0c].copy_from_slice(&self.ban_seconds.to_be_bytes());
        out[0x11] = self.options;
        out[0x14] = len_u8(&self.message, "message")?;
        out.extend_from_slice(&self.message);
        Ok(out)
    }
}

/// 151 Disconnect Message, the notice a user gets before the server disconnects it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DisconnectMessage {
    /// The user who sent it.
    pub user_id: u32,
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
            value_0e: u16_at(buf, 0x0e),
            text: buf[Self::HEADER_LEN..].to_vec(),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..4].copy_from_slice(&self.user_id.to_be_bytes());
        out[0x0e..0x10].copy_from_slice(&self.value_0e.to_be_bytes());
        out.extend_from_slice(&self.text);
        out
    }
}

/// 174 Shutdown Server.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShutdownServer {
    /// [`Self::EXIT`], [`Self::RESTART_COMPUTER`], or [`Self::SHUT_DOWN_COMPUTER`].
    pub mode: u8,
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
            mode: buf[1],
            message: buf[Self::HEADER_LEN..].to_vec(),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        if self.message.len() > Self::MAX_MESSAGE_LEN {
            return Err(EncodeError::TooLong("shutdown message"));
        }
        let mut out = vec![0, self.mode, 0, 0];
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
}

impl HistoryRecord {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x4c;

    /// Decodes the records of 181 Server History Join (R).
    pub fn decode_list(buf: &[u8]) -> Result<Vec<Self>, DecodeError> {
        decode_counted(buf, 4, Self::decode)
    }

    /// Decodes 183 Server History Added, one record.
    pub fn decode_added(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(Self::decode(buf)?.0)
    }

    fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [0x46, 0x47, 0x48, 0x49].map(|at| usize::from(buf[at]));
        let ([address, nick, login, detail], at) = strings(buf, Self::HEADER_LEN, lens)?;
        let end = at + usize::from(u16_at(buf, 0x42));
        need(buf, end)?;
        let record = Self {
            time: u64_at(buf, 0),
            kind: MessageType(u32_at(buf, 8)),
            user_id: u32_at(buf, 0x0c),
            address,
            nick,
            login,
            detail,
            path: FilePath::decode(&buf[at..end])?,
        };
        Ok((record, end))
    }

    /// Encodes 181 Server History Join (R), padding each record with zeros to 4.
    pub fn encode_list(list: &[Self]) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; 4];
        encode_counted(&mut out, list, Self::encode)?;
        Ok(out)
    }

    /// Encodes 183 Server History Added, padded with zeros to 4.
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
        out[0x42..0x44].copy_from_slice(&path_len.to_be_bytes());
        let parts = [&self.address, &self.nick, &self.login, &self.detail];
        for (i, s) in parts.iter().enumerate() {
            out[0x46 + i] = len_u8(s, "history string")?;
        }
        for s in parts {
            out.extend_from_slice(s);
        }
        out.extend(path);
        Ok(out)
    }
}

/// An account class, as in 503 Account Class Info, 504 Set Account Class and 506 Account Class List.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccountClass {
    /// Privilege bits, most significant bit of byte 0 = bit 0.
    pub privileges: [u8; 16],
    /// A time in milliseconds.
    pub time: u64,
    /// Meaning unknown; 0x50 for Administrator and 0x28 for Guest in 503, 0 in 506.
    pub unknown_25: u8,
    /// Class index.
    pub index: u8,
    /// The three bytes after the index; 0xff in the first for Administrator.
    pub unknown_after_index: [u8; 3],
    /// Class name.
    pub name: Vec<u8>,
}

impl AccountClass {
    /// Offset of the index in 503 Account Class Info.
    const INFO_INDEX_AT: usize = 0x30;
    /// Offset of the index in an entry of 506 Account Class List.
    const LIST_INDEX_AT: usize = 0x28;

    /// Decodes 503 Account Class Info.
    pub fn decode_info(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(Self::decode_at(buf, Self::INFO_INDEX_AT)?.0)
    }

    /// Decodes 504 Set Account Class, which has the layout of 503.
    pub fn decode_set(buf: &[u8]) -> Result<Self, DecodeError> {
        Self::decode_info(buf)
    }

    /// Decodes 506 Account Class List.
    pub fn decode_list(buf: &[u8]) -> Result<Vec<Self>, DecodeError> {
        decode_counted(buf, 0, |entry| Self::decode_at(entry, Self::LIST_INDEX_AT))
    }

    /// Encodes 503 Account Class Info, padded with zeros to 4.
    pub fn encode_info(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = self.encode_at(Self::INFO_INDEX_AT)?;
        out.resize(pad4(out.len()), 0);
        Ok(out)
    }

    /// Encodes 504 Set Account Class: the 503 layout, without padding.
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
        let ([name], end) = strings(buf, names_at, [usize::from(buf[index_at + 4])])?;
        let class = Self {
            privileges: buf[..16].try_into().expect("16 bytes"),
            time: u64_at(buf, 0x10),
            unknown_25: buf[0x25],
            index: buf[index_at],
            unknown_after_index: buf[index_at + 1..index_at + 4].try_into().expect("3 bytes"),
            name,
        };
        Ok((class, end))
    }

    fn encode_at(&self, index_at: usize) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; index_at + 8];
        out[..16].copy_from_slice(&self.privileges);
        out[0x10..0x18].copy_from_slice(&self.time.to_be_bytes());
        out[0x25] = self.unknown_25;
        out[index_at] = self.index;
        out[index_at + 1..index_at + 4].copy_from_slice(&self.unknown_after_index);
        out[index_at + 4] = len_u8(&self.name, "class name")?;
        out.extend_from_slice(&self.name);
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
    /// Class index.
    pub class_index: u8,
    /// The three bytes after the class index, stored as given.
    pub unknown_77: [u8; 3],
    /// Login.
    pub login: Vec<u8>,
    /// Account name.
    pub name: Vec<u8>,
    /// A third string, stored as given; empty in the lab.
    pub third: Vec<u8>,
    /// In 511, the login of the account to change; in 510, the login of whoever last saved it.
    pub editor: Vec<u8>,
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
            .field("class_index", &self.class_index)
            .field("unknown_77", &self.unknown_77)
            .field("login", &self.login)
            .field("name", &self.name)
            .field("third", &self.third)
            .field("editor", &self.editor)
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
        let ([login, name, third, editor], _) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            privileges: buf[..0x10].try_into().expect("16 bytes"),
            privilege_mask: buf[0x10..0x20].try_into().expect("16 bytes"),
            key: buf[0x20..0x40].try_into().expect("32 bytes"),
            created: u64_at(buf, 0x40),
            modified: u64_at(buf, 0x48),
            last_login: u64_at(buf, 0x50),
            class_index: buf[0x76],
            unknown_77: buf[0x77..0x7a].try_into().expect("3 bytes"),
            login,
            name,
            third,
            editor,
        })
    }

    /// Encodes the payload, padded with zeros to 4.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[..0x10].copy_from_slice(&self.privileges);
        out[0x10..0x20].copy_from_slice(&self.privilege_mask);
        out[0x20..0x40].copy_from_slice(&self.key);
        out[0x40..0x48].copy_from_slice(&self.created.to_be_bytes());
        out[0x48..0x50].copy_from_slice(&self.modified.to_be_bytes());
        out[0x50..0x58].copy_from_slice(&self.last_login.to_be_bytes());
        out[0x76] = self.class_index;
        out[0x77..0x7a].copy_from_slice(&self.unknown_77);
        out[0x7a] = len_u8(&self.login, "login")?;
        out[0x7b] = len_u8(&self.name, "account name")?;
        out[0x7c] = len_u8(&self.third, "third string")?;
        out[0x7d] = len_u8(&self.editor, "editor login")?;
        for s in [&self.login, &self.name, &self.third, &self.editor] {
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
    /// Meaning unknown; 1 for admin and 0 for guest in the lab.
    pub unknown_1b: u8,
    /// Class index.
    pub class_index: u8,
    /// Meaning unknown; 0xff for admin, as in the class.
    pub unknown_21: u8,
    /// Login.
    pub login: Vec<u8>,
    /// Account name.
    pub name: Vec<u8>,
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
            let account = Self {
                created: u64_at(entry, 0),
                last_login: u64_at(entry, 8),
                unknown_1b: entry[0x1b],
                class_index: entry[0x20],
                unknown_21: entry[0x21],
                login,
                name,
            };
            Ok((account, end))
        })
    }

    /// Encodes the payload of 513 Account List.
    pub fn encode_list(list: &[Self]) -> Result<Vec<u8>, EncodeError> {
        let mut out = Vec::new();
        encode_counted(&mut out, list, |account| {
            let mut entry = vec![0; Self::HEADER_LEN];
            entry[0..8].copy_from_slice(&account.created.to_be_bytes());
            entry[8..0x10].copy_from_slice(&account.last_login.to_be_bytes());
            entry[0x1b] = account.unknown_1b;
            entry[0x20] = account.class_index;
            entry[0x21] = account.unknown_21;
            entry[0x24] = len_u8(&account.login, "login")?;
            entry[0x25] = len_u8(&account.name, "account name")?;
            entry.extend_from_slice(&account.login);
            entry.extend_from_slice(&account.name);
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
    /// The processes.
    pub processes: Vec<Process>,
}

/// A process in 803 Process List.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Process {
    /// Process ID.
    pub pid: u32,
    /// Start time in milliseconds.
    pub start_time: u64,
    /// Command line, cut at 0xff bytes.
    pub command: Vec<u8>,
    /// User name.
    pub user: Vec<u8>,
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
            let lens = [usize::from(entry[0x3a]), usize::from(entry[0x39])];
            let ([command, user], end) = strings(entry, Self::ENTRY_LEN, lens)?;
            let process = Process {
                pid: u32_at(entry, 0),
                start_time: u64_at(entry, 8),
                command,
                user,
            };
            Ok((process, end))
        })?;
        Ok(Self {
            memory_00: u64_at(buf, 0),
            memory_08: u64_at(buf, 8),
            processes,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..8].copy_from_slice(&self.memory_00.to_be_bytes());
        out[8..0x10].copy_from_slice(&self.memory_08.to_be_bytes());
        encode_counted(&mut out, &self.processes, |process| {
            let mut entry = vec![0; Self::ENTRY_LEN];
            entry[0..4].copy_from_slice(&process.pid.to_be_bytes());
            entry[8..0x10].copy_from_slice(&process.start_time.to_be_bytes());
            entry[0x39] = len_u8(&process.user, "user")?;
            entry[0x3a] = len_u8(&process.command, "command")?;
            entry.extend_from_slice(&process.command);
            entry.extend_from_slice(&process.user);
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

/// An item of 1107 Folder Access List.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FolderAccessItem {
    /// Meaning unknown; 1, 2 and 0 in the lab.
    pub unknown_00: u32,
    /// Item ID, as in a 401 File List entry; 0 for `[Default]`.
    pub item_id: u32,
    /// Meaning unknown; 0 in the lab.
    pub unknown_10: u8,
    /// Item name.
    pub name: Vec<u8>,
    /// A second string; empty in the lab.
    pub second: Vec<u8>,
    /// Description.
    pub description: Vec<u8>,
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
            let item = Self {
                unknown_00: u32_at(entry, 0),
                item_id: u32_at(entry, 4),
                unknown_10: entry[0x10],
                name,
                second,
                description,
            };
            Ok((item, end))
        })
    }

    /// Encodes the payload of 1107 Folder Access List.
    pub fn encode_list(list: &[Self]) -> Result<Vec<u8>, EncodeError> {
        let mut out = Vec::new();
        encode_counted(&mut out, list, |item| {
            let mut entry = vec![0; Self::HEADER_LEN];
            entry[0..4].copy_from_slice(&item.unknown_00.to_be_bytes());
            entry[4..8].copy_from_slice(&item.item_id.to_be_bytes());
            entry[0x10] = item.unknown_10;
            entry[0x11] = len_u8(&item.name, "item name")?;
            entry[0x12] = len_u8(&item.second, "second string")?;
            entry[0x13] = len_u8(&item.description, "description")?;
            for s in [&item.name, &item.second, &item.description] {
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
    pub unknown_00: u32,
    /// [`Self::BUILT_IN`] and [`Self::OWNER_IN_NAME`].
    pub flags: u32,
    /// Icon, such as 0x2c for Dropbox.
    pub icon: u32,
    /// Probably the owner's rule; permission bits in the top u16.
    pub rule: u32,
    /// One rule per account class: permission bits in the top u16, bit 0 first.
    pub class_rules: [u32; 32],
    /// Meaning unknown; 0 in the lab.
    pub unknown_b0: u8,
    /// Item name, the tag in a folder name, such as `DB`.
    pub name: Vec<u8>,
    /// Owner login.
    pub owner: Vec<u8>,
    /// Description.
    pub description: Vec<u8>,
    /// In 1105, the name of the item to change.
    pub current_name: Vec<u8>,
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
        let ([name, owner, description, current_name], _) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            unknown_00: u32_at(buf, 0),
            flags: u32_at(buf, 4),
            icon: u32_at(buf, 8),
            rule: u32_at(buf, 0x0c),
            class_rules: std::array::from_fn(|i| u32_at(buf, 0x10 + 4 * i)),
            unknown_b0: buf[0xb0],
            name,
            owner,
            description,
            current_name,
        })
    }

    /// Encodes the payload, padded with zeros to 4.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..4].copy_from_slice(&self.unknown_00.to_be_bytes());
        out[4..8].copy_from_slice(&self.flags.to_be_bytes());
        out[8..0x0c].copy_from_slice(&self.icon.to_be_bytes());
        out[0x0c..0x10].copy_from_slice(&self.rule.to_be_bytes());
        for (i, rule) in self.class_rules.iter().enumerate() {
            out[0x10 + 4 * i..0x14 + 4 * i].copy_from_slice(&rule.to_be_bytes());
        }
        out[0xad] = len_u8(&self.name, "item name")?;
        out[0xae] = len_u8(&self.owner, "owner login")?;
        out[0xaf] = len_u8(&self.description, "description")?;
        out[0xb0] = self.unknown_b0;
        out[0xb1] = len_u8(&self.current_name, "current name")?;
        for s in [
            &self.name,
            &self.owner,
            &self.description,
            &self.current_name,
        ] {
            out.extend_from_slice(s);
        }
        out.resize(pad4(out.len()), 0);
        Ok(out)
    }
}
