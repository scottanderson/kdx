//! The programs' clock: milliseconds since 1900, as the files store it.

use crate::wire::{DecodeError, Reader, Writer};

/// Milliseconds since 1900-01-01 00:00 UTC, the time base of every file and
/// of the `u64` time fields on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Time1900(pub u64);

impl Time1900 {
    /// Milliseconds from 1900-01-01 to the Unix epoch.
    pub const UNIX_EPOCH_OFFSET_MS: i64 = 2_208_988_800_000;

    /// Milliseconds since the Unix epoch.
    pub const fn to_unix_ms(self) -> i64 {
        self.0 as i64 - Self::UNIX_EPOCH_OFFSET_MS
    }

    /// From milliseconds since the Unix epoch.
    pub const fn from_unix_ms(ms: i64) -> Self {
        Time1900((ms + Self::UNIX_EPOCH_OFFSET_MS) as u64)
    }

    /// Reads eight big-endian bytes.
    pub fn decode(r: &mut Reader) -> Result<Self, DecodeError> {
        Ok(Time1900(r.u64()?))
    }

    /// Writes eight big-endian bytes.
    pub fn encode(self, w: &mut Writer) {
        w.u64(self.0);
    }
}
