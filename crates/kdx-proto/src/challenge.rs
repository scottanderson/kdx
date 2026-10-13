//! The challenge table that every KDX program embeds.
//!
//! The table is Haxial's copyrighted text, so this crate does not contain it.
//! Callers obtain it from a KDX file with [`ChallengeTable::extract`] or load a
//! saved copy with [`ChallengeTable::from_bytes`].
//!
//! GConsole connections answer from a second, longer text, the
//! [`GConsoleTable`], which only the clients contain.

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
    /// The input is not exactly as long as the table.
    #[error("challenge table must be {expected} bytes, got {got}")]
    WrongLength {
        /// The table's length.
        expected: usize,
        /// The input's length.
        got: usize,
    },
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
        verify(bytes, Self::LEN, &SHA256).map(Self)
    }

    /// Finds and verifies the table inside an uncompressed KDX program file.
    pub fn extract(file: &[u8]) -> Result<Self, ChallengeError> {
        Self::from_bytes(find(file, MARKER, Self::LEN)?)
    }

    /// The table bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// The response to a challenge, or `None` if `offset` is out of range.
    pub fn response(&self, offset: u16) -> Option<[u8; RESPONSE_LEN]> {
        response(&self.0, offset)
    }
}

/// Bytes that start the GConsole table in every known client.
pub const GCONSOLE_MARKER: &[u8] = b"The following story is Copyright 2004";

/// SHA-256 of the GConsole table.
pub const GCONSOLE_SHA256: [u8; 32] = [
    0xb4, 0x3f, 0x22, 0x22, 0x29, 0x71, 0xa5, 0x7a, 0x0d, 0x8b, 0xdb, 0x3a, 0x5a, 0xf2, 0x74, 0x7c,
    0x7f, 0xf5, 0x11, 0xea, 0x88, 0xf0, 0xba, 0x8c, 0x24, 0xe3, 0x8e, 0x9e, 0xdc, 0x02, 0x9c, 0x36,
];

/// The verified 3193-byte challenge table of GConsole connections, its trailing NUL included.
#[derive(Clone, PartialEq, Eq)]
pub struct GConsoleTable(Box<[u8]>);

impl GConsoleTable {
    /// Table length in bytes.
    pub const LEN: usize = 3193;

    /// Number of valid challenge offsets: `0..OFFSETS`.
    pub const OFFSETS: u16 = (Self::LEN - RESPONSE_LEN + 1) as u16;

    /// Loads a saved table and verifies its checksum.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ChallengeError> {
        verify(bytes, Self::LEN, &GCONSOLE_SHA256).map(Self)
    }

    /// Finds and verifies the table inside an uncompressed KDX client file.
    pub fn extract(file: &[u8]) -> Result<Self, ChallengeError> {
        Self::from_bytes(find(file, GCONSOLE_MARKER, Self::LEN)?)
    }

    /// The table bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// The response to a challenge, or `None` if `offset` is out of range.
    pub fn response(&self, offset: u16) -> Option<[u8; RESPONSE_LEN]> {
        response(&self.0, offset)
    }
}

impl std::fmt::Debug for GConsoleTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("GConsoleTable(..)")
    }
}

fn verify(bytes: &[u8], len: usize, sha256: &[u8; 32]) -> Result<Box<[u8]>, ChallengeError> {
    if bytes.len() != len {
        return Err(ChallengeError::WrongLength {
            expected: len,
            got: bytes.len(),
        });
    }
    if Sha256::digest(bytes)[..] != sha256[..] {
        return Err(ChallengeError::WrongChecksum);
    }
    Ok(bytes.into())
}

/// The `len` bytes that start at `marker`, or fewer at the end of `file`.
fn find<'a>(file: &'a [u8], marker: &[u8], len: usize) -> Result<&'a [u8], ChallengeError> {
    let start = file
        .windows(marker.len())
        .position(|window| window == marker)
        .ok_or(ChallengeError::NotFound)?;
    Ok(&file[start..(start + len).min(file.len())])
}

fn response(table: &[u8], offset: u16) -> Option<[u8; RESPONSE_LEN]> {
    let start = usize::from(offset);
    table.get(start..start + RESPONSE_LEN)?.try_into().ok()
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
            Err(ChallengeError::WrongLength {
                expected: ChallengeTable::LEN,
                got: 10
            })
        );
        assert_eq!(
            ChallengeTable::from_bytes(&[0; ChallengeTable::LEN]),
            Err(ChallengeError::WrongChecksum)
        );
    }

    #[test]
    fn gconsole_table_from_the_linux_client() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../vendor/haxial/KDXClient-Lnx-beta5/extracted/KDXClientBETA5-Lnx/KDXClient.lexe"
        );
        let Ok(file) = std::fs::read(path) else {
            eprintln!("skipping: no {path}");
            return;
        };
        let table = GConsoleTable::extract(&file).unwrap();
        assert_eq!(table.as_bytes().last(), Some(&0));
    }

    #[test]
    fn offsets_cover_the_whole_table() {
        assert_eq!(
            usize::from(ChallengeTable::OFFSETS) - 1 + RESPONSE_LEN,
            ChallengeTable::LEN
        );
        assert_eq!(
            usize::from(GConsoleTable::OFFSETS) - 1 + RESPONSE_LEN,
            GConsoleTable::LEN
        );
    }
}
