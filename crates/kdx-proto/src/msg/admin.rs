//! Server, account and process administration replies.

use super::{len_u8, need, pad4, strings, u32_at, u64_at, DecodeError, EncodeError};

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

/// A connection record of 143 Connection Monitor Info.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MonitoredConnection {
    /// User ID.
    pub user_id: u32,
    /// Login time in milliseconds.
    pub login_time: u64,
    /// A time in milliseconds.
    pub time_10: u64,
    /// A time in milliseconds, probably the last activity.
    pub time_18: u64,
    /// Nick.
    pub nick: Vec<u8>,
    /// Login.
    pub login: Vec<u8>,
    /// Account name.
    pub account: Vec<u8>,
    /// Address as `ip:port`.
    pub address: Vec<u8>,
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
        let ([nick, login, account, address], end) = strings(buf, Self::HEADER_LEN, lens)?;
        let record = Self {
            user_id: u32_at(buf, 0),
            login_time: u64_at(buf, 8),
            time_10: u64_at(buf, 0x10),
            time_18: u64_at(buf, 0x18),
            nick,
            login,
            account,
            address,
        };
        Ok((record, end))
    }

    /// Encodes records as 143 Connection Monitor Info, padding with zeros.
    pub fn encode_list(list: &[Self]) -> Result<Vec<u8>, EncodeError> {
        let mut out = Vec::new();
        for record in list {
            let mut fixed = vec![0; Self::HEADER_LEN];
            fixed[0..4].copy_from_slice(&record.user_id.to_be_bytes());
            fixed[8..0x10].copy_from_slice(&record.login_time.to_be_bytes());
            fixed[0x10..0x18].copy_from_slice(&record.time_10.to_be_bytes());
            fixed[0x18..0x20].copy_from_slice(&record.time_18.to_be_bytes());
            let parts = [
                &record.nick,
                &record.login,
                &record.account,
                &record.address,
            ];
            for (i, s) in parts.iter().enumerate() {
                fixed[0x60 + i] = len_u8(s, "connection string")?;
            }
            out.extend(fixed);
            for s in parts {
                out.extend_from_slice(s);
            }
            out.resize(pad4(out.len()), 0);
        }
        Ok(out)
    }
}

/// An account class, as in 503 Account Class Info and 506 Account Class List.
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

/// 510 Account Info.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccountInfo {
    /// Probably a mask of the privileges set on the account itself.
    pub privilege_mask: [u8; 16],
    /// Probably the values of those privileges.
    pub privileges: [u8; 16],
    /// Meaning unknown; 1 in the lab.
    pub unknown_20: u8,
    /// Creation time in milliseconds.
    pub created: u64,
    /// Modification time in milliseconds.
    pub modified: u64,
    /// Last login time in milliseconds.
    pub last_login: u64,
    /// Login.
    pub login: Vec<u8>,
    /// Account name.
    pub name: Vec<u8>,
}

impl AccountInfo {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x80;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [usize::from(buf[0x7a]), usize::from(buf[0x7b])];
        let ([login, name], _) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            privilege_mask: buf[..0x10].try_into().expect("16 bytes"),
            privileges: buf[0x10..0x20].try_into().expect("16 bytes"),
            unknown_20: buf[0x20],
            created: u64_at(buf, 0x40),
            modified: u64_at(buf, 0x48),
            last_login: u64_at(buf, 0x50),
            login,
            name,
        })
    }

    /// Encodes the payload, padded with zeros to 4.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[..0x10].copy_from_slice(&self.privilege_mask);
        out[0x10..0x20].copy_from_slice(&self.privileges);
        out[0x20] = self.unknown_20;
        out[0x40..0x48].copy_from_slice(&self.created.to_be_bytes());
        out[0x48..0x50].copy_from_slice(&self.modified.to_be_bytes());
        out[0x50..0x58].copy_from_slice(&self.last_login.to_be_bytes());
        out[0x7a] = len_u8(&self.login, "login")?;
        out[0x7b] = len_u8(&self.name, "account name")?;
        out.extend_from_slice(&self.login);
        out.extend_from_slice(&self.name);
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
