//! The privilege bits of accounts and account classes.

/// A set of privilege bits, sent as a big-endian u128 with bit 0 as the most significant bit.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Privileges(pub u128);

impl Privileges {
    /// Encoded length.
    pub const LEN: usize = 16;

    /// The mask of privilege bit `n`.
    pub const fn bit(n: u32) -> u128 {
        1 << 127 >> n
    }

    /// Whether privilege bit `n` is set.
    pub const fn contains(self, n: u32) -> bool {
        self.0 & Self::bit(n) != 0
    }

    /// Decodes the 16 bytes at `at`; the caller checks the length.
    pub(crate) fn decode_at(buf: &[u8], at: usize) -> Self {
        Self(u128::from_be_bytes(
            buf[at..at + Self::LEN].try_into().expect("16 bytes"),
        ))
    }

    /// Encodes the bits.
    pub const fn encode(self) -> [u8; Self::LEN] {
        self.0.to_be_bytes()
    }
}

impl std::fmt::LowerHex for Privileges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Privileges;

    #[test]
    fn bit_0_is_the_first_byte_high_bit() {
        let p = Privileges::decode_at(&[0x80, 0, 0, 0, 0x08, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 0);
        assert!(p.contains(0));
        assert!(p.contains(36));
        assert!(!p.contains(1));
        assert_eq!(Privileges::bit(127), 1);
    }
}
