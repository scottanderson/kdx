//! The challenge table that every KDX program embeds.
//!
//! The table is Haxial's copyrighted text, so this crate does not contain it.
//! Callers obtain it from a KDX file with [`ChallengeTable::extract`] or load a
//! saved copy with [`ChallengeTable::from_bytes`].

use sha2::{Digest, Sha256};

/// Bytes that start the table in every known KDX build.
pub const MARKER: &[u8] = b"This program including the following story";

/// SHA-256 of the table.
pub const SHA256: [u8; 32] = [
    0x4f, 0xca, 0x76, 0x36, 0xe7, 0x7e, 0x85, 0xfd, 0xde, 0x95, 0x86, 0x2a, 0x6b, 0xe6, 0x6e, 0x12,
    0xe5, 0xe0, 0x31, 0xad, 0x61, 0xad, 0xd6, 0x5c, 0x5e, 0x97, 0x5d, 0x3d, 0xaf, 0xf4, 0xb9, 0xe7,
];

/// Length of a challenge response in bytes.
pub const RESPONSE_LEN: usize = 20;

/// Errors from loading a challenge table.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ChallengeError {
    /// The input does not contain the table's starting bytes.
    #[error("no KDX challenge table found")]
    NotFound,
    /// The input is not exactly [`ChallengeTable::LEN`] bytes.
    #[error("challenge table must be {expected} bytes, got {0}", expected = ChallengeTable::LEN)]
    WrongLength(usize),
    /// The bytes are not the known table.
    #[error("challenge table checksum does not match")]
    WrongChecksum,
}

/// The verified 1739-byte challenge table.
#[derive(Clone, PartialEq, Eq)]
pub struct ChallengeTable(Box<[u8]>);

impl ChallengeTable {
    /// Table length in bytes.
    pub const LEN: usize = 1739;

    /// Number of valid challenge offsets: `0..OFFSETS`.
    pub const OFFSETS: u16 = (Self::LEN - RESPONSE_LEN + 1) as u16;

    /// Loads a saved table and verifies its checksum.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ChallengeError> {
        if bytes.len() != Self::LEN {
            return Err(ChallengeError::WrongLength(bytes.len()));
        }
        if Sha256::digest(bytes)[..] != SHA256 {
            return Err(ChallengeError::WrongChecksum);
        }
        Ok(Self(bytes.into()))
    }

    /// Finds and verifies the table inside an uncompressed KDX program file.
    pub fn extract(file: &[u8]) -> Result<Self, ChallengeError> {
        let start = file
            .windows(MARKER.len())
            .position(|window| window == MARKER)
            .ok_or(ChallengeError::NotFound)?;
        let end = (start + Self::LEN).min(file.len());
        Self::from_bytes(&file[start..end])
    }

    /// The table bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// The response to a challenge, or `None` if `offset` is out of range.
    pub fn response(&self, offset: u16) -> Option<[u8; RESPONSE_LEN]> {
        let start = usize::from(offset);
        self.0.get(start..start + RESPONSE_LEN)?.try_into().ok()
    }
}

impl std::fmt::Debug for ChallengeTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ChallengeTable(..)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_input() {
        assert_eq!(
            ChallengeTable::extract(b"nothing here"),
            Err(ChallengeError::NotFound)
        );
        assert_eq!(
            ChallengeTable::from_bytes(&[0; 10]),
            Err(ChallengeError::WrongLength(10))
        );
        assert_eq!(
            ChallengeTable::from_bytes(&[0; ChallengeTable::LEN]),
            Err(ChallengeError::WrongChecksum)
        );
    }

    #[test]
    fn offsets_cover_the_whole_table() {
        assert_eq!(
            usize::from(ChallengeTable::OFFSETS) - 1 + RESPONSE_LEN,
            ChallengeTable::LEN
        );
    }
}
