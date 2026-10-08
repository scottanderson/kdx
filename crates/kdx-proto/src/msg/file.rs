//! File listings: 401 File List and 410 File/Folder Info.

use super::{len_u8, need, pad4, strings, u32_at, u64_at, DecodeError, EncodeError};

/// The attributes value of a file.
pub const ATTR_FILE: u32 = 1;
/// The attributes value of a folder.
pub const ATTR_FOLDER: u32 = 0x0100_0002;

/// An entry of 401 File List.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileListEntry {
    /// Size in bytes; 0x1000 for folders.
    pub size: u64,
    /// Creation time in milliseconds.
    pub created: u64,
    /// Modification time in milliseconds.
    pub modified: u64,
    /// [`ATTR_FILE`] or [`ATTR_FOLDER`].
    pub attributes: u32,
    /// The folder access item that applies, as in 1107 Folder Access List.
    pub access_item: u16,
    /// Meaning unknown; 4 or 5 in the lab.
    pub unknown_34: u8,
    /// Item name.
    pub name: Vec<u8>,
}

impl FileListEntry {
    /// Size of the fixed part before the name.
    pub const HEADER_LEN: usize = 0x38;

    /// Whether the entry is a folder.
    pub const fn is_folder(&self) -> bool {
        self.attributes & 2 != 0
    }

    /// Decodes the payload of 401 File List; an empty payload is an empty folder.
    pub fn decode_list(buf: &[u8]) -> Result<Vec<Self>, DecodeError> {
        if buf.is_empty() {
            return Ok(Vec::new());
        }
        need(buf, 4)?;
        let count = u32_at(buf, 0) as usize;
        let mut at = 4;
        let mut list = Vec::with_capacity(count.min(buf.len()));
        for _ in 0..count {
            let entry = &buf[at..];
            need(entry, Self::HEADER_LEN)?;
            let ([name], end) = strings(entry, Self::HEADER_LEN, [usize::from(entry[0x35])])?;
            list.push(Self {
                size: u64_at(entry, 0),
                created: u64_at(entry, 8),
                modified: u64_at(entry, 0x10),
                attributes: u32_at(entry, 0x20),
                access_item: u16::from_be_bytes([entry[0x30], entry[0x31]]),
                unknown_34: entry[0x34],
                name,
            });
            at += pad4(end).min(entry.len());
        }
        Ok(list)
    }

    /// Encodes the payload of 401 File List, padding with zeros.
    pub fn encode_list(list: &[Self]) -> Result<Vec<u8>, EncodeError> {
        if list.is_empty() {
            return Ok(Vec::new());
        }
        let count = u32::try_from(list.len()).map_err(|_| EncodeError::TooLong("file list"))?;
        let mut out = count.to_be_bytes().to_vec();
        for entry in list {
            let mut fixed = [0; Self::HEADER_LEN];
            fixed[0..8].copy_from_slice(&entry.size.to_be_bytes());
            fixed[8..0x10].copy_from_slice(&entry.created.to_be_bytes());
            fixed[0x10..0x18].copy_from_slice(&entry.modified.to_be_bytes());
            fixed[0x20..0x24].copy_from_slice(&entry.attributes.to_be_bytes());
            fixed[0x2e..0x30].copy_from_slice(&[0xff, 0xff]);
            fixed[0x30..0x32].copy_from_slice(&entry.access_item.to_be_bytes());
            fixed[0x34] = entry.unknown_34;
            fixed[0x35] = len_u8(&entry.name, "file name")?;
            out.extend_from_slice(&fixed);
            out.extend_from_slice(&entry.name);
            out.resize(pad4(out.len()), 0);
        }
        Ok(out)
    }
}

/// 410 File/Folder Info.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileInfo {
    /// Meaning unknown; 0x39 in the lab.
    pub unknown_00: u32,
    /// Meaning unknown; 0xbf in the lab.
    pub unknown_04: u32,
    /// Size in bytes.
    pub size: u64,
    /// Creation time in milliseconds.
    pub created: u64,
    /// Modification time in milliseconds.
    pub modified: u64,
    /// Meaning unknown; 2 for a folder in the lab.
    pub unknown_28: u32,
    /// Item name.
    pub name: Vec<u8>,
    /// Kind, such as `folder`.
    pub kind: Vec<u8>,
}

impl FileInfo {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x40;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [usize::from(buf[0x3d]), usize::from(buf[0x3e])];
        let ([name, kind], _) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            unknown_00: u32_at(buf, 0),
            unknown_04: u32_at(buf, 4),
            size: u64_at(buf, 8),
            created: u64_at(buf, 0x10),
            modified: u64_at(buf, 0x18),
            unknown_28: u32_at(buf, 0x28),
            name,
            kind,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..4].copy_from_slice(&self.unknown_00.to_be_bytes());
        out[4..8].copy_from_slice(&self.unknown_04.to_be_bytes());
        out[8..0x10].copy_from_slice(&self.size.to_be_bytes());
        out[0x10..0x18].copy_from_slice(&self.created.to_be_bytes());
        out[0x18..0x20].copy_from_slice(&self.modified.to_be_bytes());
        out[0x28..0x2c].copy_from_slice(&self.unknown_28.to_be_bytes());
        out[0x3d] = len_u8(&self.name, "file name")?;
        out[0x3e] = len_u8(&self.kind, "kind")?;
        out.extend_from_slice(&self.name);
        out.extend_from_slice(&self.kind);
        Ok(out)
    }
}
