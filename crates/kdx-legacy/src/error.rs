//! The error type every reader and writer returns.

use std::path::PathBuf;

use crate::wire::DecodeError;

/// Why a file could not be read or written.
#[derive(Debug, thiserror::Error)]
pub enum LegacyError {
    /// Reading or writing the file failed.
    #[error("{path}: {source}")]
    Io {
        /// The file.
        path: PathBuf,
        /// The error.
        #[source]
        source: std::io::Error,
    },
    /// A file-level structure did not decode.
    #[error("decode: {0}")]
    Decode(#[from] DecodeError),
    /// An embedded wire record did not decode.
    #[error("decode: {0}")]
    Proto(#[from] kdx_proto::msg::DecodeError),
    /// An embedded wire record did not encode (a string too long for its
    /// length byte).
    #[error("encode: {0}")]
    Encode(#[from] kdx_proto::msg::EncodeError),
    /// The file does not start with the expected magic.
    #[error("bad magic: expected {expected:02x?}, found {found:02x?}")]
    BadMagic {
        /// What the format starts with.
        expected: [u8; 8],
        /// What the file starts with.
        found: [u8; 8],
    },
    /// The file's version is not one this crate reads.
    #[error("{file}: unsupported version {version}")]
    UnsupportedVersion {
        /// The file name.
        file: &'static str,
        /// The version found.
        version: u16,
    },
    /// The file is well-formed but says something this crate rejects.
    #[error("{0}")]
    Invalid(&'static str),
    /// The scrambled file does not unscramble to its magic.
    #[error("file is not scrambled in the expected way")]
    NotScrambled,
}

/// A result with [`LegacyError`].
pub type Result<T> = std::result::Result<T, LegacyError>;

impl LegacyError {
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        LegacyError::Io {
            path: path.into(),
            source,
        }
    }
}
