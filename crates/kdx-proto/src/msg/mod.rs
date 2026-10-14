//! Message payloads.
//!
//! Each payload type decodes from and encodes to the bytes that follow a
//! session [`Header`](crate::session::Header). Layouts are documented in
//! `docs/protocol/KDXProtocol.md`.

pub mod admin;
pub mod chat;
pub mod dcc;
pub mod file;
pub mod gconsole;
pub mod news;
pub mod path;
pub mod private_message;
pub mod server_info;
pub mod status;
pub mod tracker;
pub mod transfer;
pub mod user;
pub mod user_info;

pub use path::FilePath;
pub use private_message::PrivateMessage;
pub use server_info::ServerInfo;
pub use status::Status;
pub use user::UserRecord;
pub use user_info::UserInfo;

/// Errors from decoding a payload.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DecodeError {
    /// The payload ends before a field or string it declares.
    #[error("payload too short: need {need} bytes, have {have}")]
    TooShort {
        /// Bytes needed.
        need: usize,
        /// Bytes available.
        have: usize,
    },
    /// A field holds a value the layout doesn't allow.
    #[error("invalid {0}")]
    Invalid(&'static str),
}

/// Errors from encoding a payload.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EncodeError {
    /// A string or list is longer than its length field can hold.
    #[error("{0} too long")]
    TooLong(&'static str),
}

/// Checks that `buf` holds at least `need` bytes.
pub(crate) const fn need(buf: &[u8], need: usize) -> Result<(), DecodeError> {
    if buf.len() < need {
        Err(DecodeError::TooShort {
            need,
            have: buf.len(),
        })
    } else {
        Ok(())
    }
}

pub(crate) fn u16_at(buf: &[u8], at: usize) -> u16 {
    u16::from_be_bytes([buf[at], buf[at + 1]])
}

pub(crate) fn u32_at(buf: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(buf[at..at + 4].try_into().expect("4 bytes"))
}

pub(crate) fn u64_at(buf: &[u8], at: usize) -> u64 {
    u64::from_be_bytes(buf[at..at + 8].try_into().expect("8 bytes"))
}

/// Splits consecutive strings off `buf` at `at`, one per length.
pub(crate) fn strings<const N: usize>(
    buf: &[u8],
    at: usize,
    lens: [usize; N],
) -> Result<([Vec<u8>; N], usize), DecodeError> {
    need(buf, at + lens.iter().sum::<usize>())?;
    let mut at = at;
    let strings = lens.map(|len| {
        let s = buf[at..at + len].to_vec();
        at += len;
        s
    });
    Ok((strings, at))
}

/// The length of `s` as a u8 length field.
pub(crate) fn len_u8(s: &[u8], what: &'static str) -> Result<u8, EncodeError> {
    u8::try_from(s.len()).map_err(|_| EncodeError::TooLong(what))
}

/// Rounds `len` up to a multiple of 4, the padding most list entries use.
pub(crate) const fn pad4(len: usize) -> usize {
    (len + 3) & !3
}

/// The bytes of `buf` from `end` to the next multiple of 4, and the length up to there.
pub(crate) fn padding(buf: &[u8], end: usize) -> (Vec<u8>, usize) {
    let padded = pad4(end).min(buf.len()).max(end);
    (buf[end..padded].to_vec(), padded)
}

/// The records of a counted list payload and the bytes after the last one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountedList<T> {
    /// The records.
    pub records: Vec<T>,
    /// Bytes after the last record, as received; empty by default.
    pub rest: Vec<u8>,
}

impl<T> Default for CountedList<T> {
    fn default() -> Self {
        Self {
            records: Vec::new(),
            rest: Vec::new(),
        }
    }
}

impl<T> From<Vec<T>> for CountedList<T> {
    fn from(records: Vec<T>) -> Self {
        Self {
            records,
            rest: Vec::new(),
        }
    }
}

impl<T, const N: usize> From<[T; N]> for CountedList<T> {
    fn from(records: [T; N]) -> Self {
        Vec::from(records).into()
    }
}

/// Reads a u32 count followed by that many records, each padded to 4, then the rest.
pub(crate) fn decode_counted<T>(
    buf: &[u8],
    count_at: usize,
    mut decode: impl FnMut(&[u8]) -> Result<(T, usize), DecodeError>,
) -> Result<CountedList<T>, DecodeError> {
    need(buf, count_at + 4)?;
    let count = u32_at(buf, count_at) as usize;
    let mut at = count_at + 4;
    let mut records = Vec::with_capacity(count.min(buf.len()));
    for _ in 0..count {
        let (item, len) = decode(&buf[at..])?;
        records.push(item);
        at = (at + pad4(len)).min(buf.len());
    }
    Ok(CountedList {
        records,
        rest: buf[at..].to_vec(),
    })
}

/// Appends a u32 count, then each record padded with zeros to 4.
pub(crate) fn encode_counted<T>(
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
