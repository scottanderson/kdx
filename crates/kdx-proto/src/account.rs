//! The key that both authenticates a login and encrypts its session.

use sha1::{Digest, Sha1};

/// The 32-byte key derived from an account password.
///
/// The server stores this key for each account. The handshake proves that
/// both sides know it, and the session is encrypted with it.
#[derive(Clone, PartialEq, Eq)]
pub struct AccountKey(pub [u8; Self::LEN]);

impl AccountKey {
    /// Key length in bytes.
    pub const LEN: usize = 32;

    /// The key for an empty password, including guest logins.
    pub const EMPTY_PASSWORD: Self = Self([
        0xda, 0x39, 0xa3, 0xee, 0xca, 0xae, 0xbe, 0x6e, 0x5e, 0x6b, 0x4b, 0x0d, 0xba, 0x8e, 0x29,
        0xba, 0x32, 0x55, 0xbf, 0xef, 0x18, 0xa2, 0xb5, 0xc4, 0x95, 0x60, 0x18, 0x90, 0xaf, 0xd8,
        0x07, 0x09,
    ]);

    /// Derives the key for `password`: SHA-1, its first 8 bytes again, then a 32-bit FNV-1 hash.
    pub fn from_password(password: &[u8]) -> Self {
        if password.is_empty() {
            return Self::EMPTY_PASSWORD;
        }
        let digest: [u8; 20] = Sha1::digest(password).into();
        let mut key = [0u8; Self::LEN];
        key[..20].copy_from_slice(&digest);
        key[20..28].copy_from_slice(&digest[..8]);
        key[28..].copy_from_slice(&fnv1(1, password).to_be_bytes());
        Self(key)
    }
}

impl std::fmt::Debug for AccountKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AccountKey(..)")
    }
}

/// FNV-1 (multiply, then XOR) over `data`, starting from `seed`.
fn fnv1(seed: u32, data: &[u8]) -> u32 {
    data.iter().fold(seed, |hash, &byte| {
        hash.wrapping_mul(0x0100_0193) ^ u32::from(byte)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_password_interleaves_sha1_of_empty_input() {
        let digest: [u8; 20] = Sha1::digest(b"").into();
        let key = AccountKey::EMPTY_PASSWORD.0;
        for (word, digest_word) in [(0, 0), (2, 1), (4, 2), (6, 3), (7, 4)] {
            assert_eq!(
                key[word * 4..word * 4 + 4],
                digest[digest_word * 4..digest_word * 4 + 4]
            );
        }
    }

    #[test]
    fn from_password_layout() {
        let key = AccountKey::from_password(b"lab-only-pw!1").0;
        let digest: [u8; 20] = Sha1::digest(b"lab-only-pw!1").into();
        assert_eq!(key[..20], digest);
        assert_eq!(key[20..28], digest[..8]);
        assert_eq!(key[28..], fnv1(1, b"lab-only-pw!1").to_be_bytes());
    }
}
