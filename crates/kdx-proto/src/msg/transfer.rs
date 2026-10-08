//! File transfers: 412 Download File through 418 Upload File (C).
//!
//! These run on a transfer connection; see `Connection::attach` in kdx-client.

use super::{len_u8, need, strings, u16_at, u32_at, u64_at, DecodeError, EncodeError, FilePath};

/// Option bit for a plain (unencrypted) transfer.
pub const OPTION_PLAIN: u8 = 1;

/// 412 Download File.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DownloadFile {
    /// Where to resume the data fork; the server rounds it down to 0x1000.
    pub data_offset: u64,
    /// Where to resume the resource fork; the server rounds it down to 0x1000.
    pub rsrc_offset: u64,
    /// Option bits such as [`OPTION_PLAIN`].
    pub options: u8,
    /// The file.
    pub path: FilePath,
}

impl DownloadFile {
    /// Size of the fixed part before the path.
    pub const HEADER_LEN: usize = 0x18;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        Ok(Self {
            data_offset: u64_at(buf, 0),
            rsrc_offset: u64_at(buf, 8),
            options: buf[0x17],
            path: FilePath::decode(&buf[Self::HEADER_LEN..])?,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..8].copy_from_slice(&self.data_offset.to_be_bytes());
        out[8..0x10].copy_from_slice(&self.rsrc_offset.to_be_bytes());
        out[0x17] = self.options;
        out.extend(self.path.encode()?);
        Ok(out)
    }
}

/// The name and the three strings that follow it in 413 and 415.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileStrings {
    /// File name.
    pub name: Vec<u8>,
    /// Probably a type derived from the extension, such as `exttype/bin`.
    pub second: Vec<u8>,
    /// Meaning unknown.
    pub third: Vec<u8>,
    /// Meaning unknown; at most 0x41 bytes.
    pub fourth: Vec<u8>,
}

impl FileStrings {
    /// Reads the lengths at `at` and the strings at `at + 4`; returns the end offset.
    fn decode(buf: &[u8], at: usize) -> Result<(Self, usize), DecodeError> {
        need(buf, at + 4)?;
        let lens = [0, 1, 2, 3].map(|i| usize::from(buf[at + i]));
        let ([name, second, third, fourth], end) = strings(buf, at + 4, lens)?;
        Ok((
            Self {
                name,
                second,
                third,
                fourth,
            },
            end,
        ))
    }

    /// Writes the lengths at `at` of `out` and appends the strings.
    fn encode(&self, out: &mut Vec<u8>, at: usize) -> Result<(), EncodeError> {
        let parts = [&self.name, &self.second, &self.third, &self.fourth];
        for (i, s) in parts.iter().enumerate() {
            out[at + i] = len_u8(s, "file string")?;
        }
        for s in parts {
            out.extend_from_slice(s);
        }
        Ok(())
    }
}

/// 413 Download File (R): the file's details, sent before its data.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DownloadInfo {
    /// Full size of the data fork.
    pub data_size: u64,
    /// Full size of the resource fork.
    pub rsrc_size: u64,
    /// Creation time in milliseconds.
    pub created: u64,
    /// Modification time in milliseconds.
    pub modified: u64,
    /// Meaning unknown; 1 in the lab.
    pub unknown_30: u32,
    /// Name and type strings.
    pub strings: FileStrings,
}

impl DownloadInfo {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x4c;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let (strings, _) = FileStrings::decode(buf, 0x48)?;
        Ok(Self {
            data_size: u64_at(buf, 0),
            rsrc_size: u64_at(buf, 8),
            created: u64_at(buf, 0x18),
            modified: u64_at(buf, 0x20),
            unknown_30: u32_at(buf, 0x30),
            strings,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..8].copy_from_slice(&self.data_size.to_be_bytes());
        out[8..0x10].copy_from_slice(&self.rsrc_size.to_be_bytes());
        out[0x18..0x20].copy_from_slice(&self.created.to_be_bytes());
        out[0x20..0x28].copy_from_slice(&self.modified.to_be_bytes());
        out[0x30..0x34].copy_from_slice(&self.unknown_30.to_be_bytes());
        self.strings.encode(&mut out, 0x48)?;
        Ok(out)
    }
}

/// 415 Upload File.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UploadFile {
    /// Size of the data fork.
    pub data_size: u64,
    /// Size of the resource fork; this build expects 0.
    pub rsrc_size: u64,
    /// Option bits such as [`OPTION_PLAIN`].
    pub options: u8,
    /// Name and type strings; the name must not be empty.
    pub strings: FileStrings,
    /// The destination folder.
    pub folder: FilePath,
}

impl UploadFile {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x4c;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let (strings, end) = FileStrings::decode(buf, 0x48)?;
        if strings.name.is_empty() {
            return Err(DecodeError::Invalid("empty upload name"));
        }
        let path_len = usize::from(u16_at(buf, 0x46));
        need(buf, end + path_len)?;
        Ok(Self {
            data_size: u64_at(buf, 0),
            rsrc_size: u64_at(buf, 8),
            options: buf[0x17],
            strings,
            folder: FilePath::decode(&buf[end..end + path_len])?,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let folder = self.folder.encode()?;
        let path_len = u16::try_from(folder.len()).map_err(|_| EncodeError::TooLong("path"))?;
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..8].copy_from_slice(&self.data_size.to_be_bytes());
        out[8..0x10].copy_from_slice(&self.rsrc_size.to_be_bytes());
        out[0x17] = self.options;
        out[0x46..0x48].copy_from_slice(&path_len.to_be_bytes());
        self.strings.encode(&mut out, 0x48)?;
        out.extend(folder);
        Ok(out)
    }
}

/// 416 Upload File (R): where the server will resume.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UploadReply {
    /// Data-fork bytes already on the server from a partial upload.
    pub data_offset: u64,
    /// Resource-fork offset; all ones, since resource forks don't resume.
    pub rsrc_offset: u64,
}

impl UploadReply {
    /// Payload length.
    pub const LEN: usize = 0x18;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::LEN)?;
        Ok(Self {
            data_offset: u64_at(buf, 0),
            rsrc_offset: u64_at(buf, 8),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> [u8; Self::LEN] {
        let mut out = [0; Self::LEN];
        out[0..8].copy_from_slice(&self.data_offset.to_be_bytes());
        out[8..0x10].copy_from_slice(&self.rsrc_offset.to_be_bytes());
        out
    }
}
