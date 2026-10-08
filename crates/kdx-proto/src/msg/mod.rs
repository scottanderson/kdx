//! Message payloads.
//!
//! Each payload type decodes from and encodes to the bytes that follow a
//! session [`Header`](crate::session::Header). Layouts are documented in
//! `docs/protocol/KDXProtocol.md`.

pub mod news;
pub mod path;
pub mod private_message;
pub mod server_info;
pub mod status;
pub mod user;

pub use path::FilePath;
pub use private_message::PrivateMessage;
pub use server_info::ServerInfo;
pub use status::Status;
pub use user::UserRecord;

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
