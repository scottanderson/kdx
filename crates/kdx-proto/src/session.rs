//! Messages exchanged after the handshake.
//!
//! Every message is a 16-byte [`Header`] block followed by the payload, padded
//! to whole 16-byte blocks. Each block is encrypted on its own with Twofish-256
//! (ECB) under the account key. The header also goes through a keyless
//! scramble before encryption.

use twofish::cipher::{Block, BlockCipherDecrypt, BlockCipherEncrypt, KeyInit};
use twofish::Twofish;

use crate::account::AccountKey;

/// Cipher block length in bytes.
pub const BLOCK_LEN: usize = 16;
/// Largest payload that Server 1.620 accepts from a client.
pub const SERVER_MAX_PAYLOAD_LEN: u32 = 0x2_0000;
/// Largest payload that Client 1.600 accepts from a server.
pub const CLIENT_MAX_PAYLOAD_LEN: u32 = 0x10_0000;

/// Errors from opening a received message.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SessionError {
    /// The header's transaction ID is zero.
    #[error("zero transaction id")]
    ZeroTxid,
    /// The header's message type is zero.
    #[error("zero message type")]
    ZeroType,
    /// A header byte that must be zero is not.
    #[error("reserved header bytes are not zero: {0:#06x}")]
    Reserved(u32),
    /// The payload is longer than the receiver accepts.
    #[error("payload too long ({0} bytes)")]
    TooLong(u32),
    /// Fewer bytes were given than the payload needs.
    #[error("need {0} payload bytes")]
    Incomplete(usize),
}

pub use crate::message_type::MessageType;

/// The plaintext message header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    /// Nonzero transaction ID. A reply reuses its request's ID.
    pub txid: u32,
    /// Nonzero message type.
    pub kind: MessageType,
    /// Flag byte; see [`Self::FLAG_REPLY`].
    pub flags: u8,
    /// Payload length in bytes, before padding.
    pub len: u32,
}

impl Header {
    /// Wire length in bytes.
    pub const LEN: usize = BLOCK_LEN;
    /// Flag bit set on replies.
    pub const FLAG_REPLY: u8 = 0x01;

    /// Whether this message is a reply.
    pub const fn is_reply(&self) -> bool {
        self.flags & Self::FLAG_REPLY != 0
    }

    /// The plaintext bytes, before scrambling.
    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut out = [0; Self::LEN];
        out[0..4].copy_from_slice(&self.txid.to_be_bytes());
        out[4..8].copy_from_slice(&self.kind.0.to_be_bytes());
        out[9] = self.flags;
        out[12..16].copy_from_slice(&self.len.to_be_bytes());
        out
    }

    /// Parses and checks plaintext bytes, after unscrambling.
    ///
    /// `max_len` is the receiver's payload limit, such as [`SERVER_MAX_PAYLOAD_LEN`].
    pub fn from_bytes(bytes: &[u8; Self::LEN], max_len: u32) -> Result<Self, SessionError> {
        let header = Self::from_bytes_unbounded(bytes)?;
        if header.len > max_len {
            return Err(SessionError::TooLong(header.len));
        }
        Ok(header)
    }

    /// Like [`Self::from_bytes`], but allows any length, as transfer data headers need.
    pub fn from_bytes_unbounded(bytes: &[u8; Self::LEN]) -> Result<Self, SessionError> {
        let word = |i: usize| u32::from_be_bytes(bytes[i..i + 4].try_into().unwrap());
        let header = Self {
            txid: word(0),
            kind: MessageType(word(4)),
            flags: bytes[9],
            len: word(12),
        };
        let reserved =
            u32::from(bytes[8]) << 16 | u32::from(u16::from_be_bytes([bytes[10], bytes[11]]));
        if header.txid == 0 {
            Err(SessionError::ZeroTxid)
        } else if header.kind.0 == 0 {
            Err(SessionError::ZeroType)
        } else if reserved != 0 {
            Err(SessionError::Reserved(reserved))
        } else {
            Ok(header)
        }
    }

    /// Length of the encrypted payload that follows this header.
    pub const fn padded_len(&self) -> usize {
        padded_len(self.len as usize)
    }
}

/// The bytes the original programs pad a payload with: an LCG step of the txid, repeated.
pub const fn padding_fill(txid: u32) -> [u8; 4] {
    txid.wrapping_mul(0x41c6_4e6d)
        .wrapping_add(0x3039)
        .to_le_bytes()
}

/// Rounds a payload length up to whole cipher blocks.
pub const fn padded_len(len: usize) -> usize {
    len.div_ceil(BLOCK_LEN) * BLOCK_LEN
}

/// Applies the header scramble that runs before encryption.
pub fn scramble(header: &mut [u8; Header::LEN]) {
    let [h0, h1, h2, h3] = words(header);
    set_words(
        header,
        [
            h1.rotate_left(13) ^ 0x6e7d_fd34,
            h2.rotate_right(6) ^ 0xe152_da04,
            h0.rotate_left(7) ^ 0xa95a_759b,
            h3.rotate_right(3) ^ 0x0699_2e25,
        ],
    );
}

/// Reverses [`scramble`] after decryption.
pub fn unscramble(header: &mut [u8; Header::LEN]) {
    let [w0, w1, w2, w3] = words(header);
    set_words(
        header,
        [
            (w2 ^ 0xa95a_759b).rotate_right(7),
            (w0 ^ 0x6e7d_fd34).rotate_right(13),
            (w1 ^ 0xe152_da04).rotate_left(6),
            (w3 ^ 0x0699_2e25).rotate_left(3),
        ],
    );
}

/// Views a 16-byte slice as a cipher block.
pub(crate) fn block_mut(bytes: &mut [u8]) -> &mut Block<Twofish> {
    bytes.try_into().expect("16-byte block")
}

fn words(block: &[u8; BLOCK_LEN]) -> [u32; 4] {
    std::array::from_fn(|i| u32::from_be_bytes(block[i * 4..i * 4 + 4].try_into().unwrap()))
}

fn set_words(block: &mut [u8; BLOCK_LEN], words: [u32; 4]) {
    for (chunk, word) in block.chunks_exact_mut(4).zip(words) {
        chunk.copy_from_slice(&word.to_be_bytes());
    }
}

/// Encrypts and decrypts session messages for one connection.
#[derive(Clone)]
pub struct SessionCipher(Twofish);

impl SessionCipher {
    /// Creates the cipher for a session authenticated with `key`.
    pub fn new(key: &AccountKey) -> Self {
        Self(Twofish::new_from_slice(&key.0).expect("32-byte key"))
    }

    /// Encrypts a whole message. The header's `len` is set from `payload`.
    pub fn seal(&self, header: Header, payload: &[u8]) -> Vec<u8> {
        let header = Header {
            len: payload.len() as u32,
            ..header
        };
        let mut out = Vec::with_capacity(Header::LEN + header.padded_len());
        out.extend_from_slice(&self.seal_header(header));
        let fill = padding_fill(header.txid);
        out.extend((0..header.padded_len()).map(|i| fill[i % 4]));
        out[Header::LEN..Header::LEN + payload.len()].copy_from_slice(payload);
        self.encrypt_blocks(&mut out[Header::LEN..]);
        out
    }

    /// Encrypts a header as given, for a transfer data stream whose bytes follow separately.
    pub fn seal_header(&self, header: Header) -> [u8; Header::LEN] {
        let mut block = header.to_bytes();
        scramble(&mut block);
        self.0.encrypt_block(block_mut(&mut block));
        block
    }

    /// Decrypts and checks a header block against the receiver's payload limit.
    pub fn open_header(
        &self,
        block: &[u8; Header::LEN],
        max_len: u32,
    ) -> Result<Header, SessionError> {
        Header::from_bytes(&self.decrypt_header(block), max_len)
    }

    /// Decrypts a transfer data header, whose length may exceed any payload limit.
    pub fn open_data_header(&self, block: &[u8; Header::LEN]) -> Result<Header, SessionError> {
        Header::from_bytes_unbounded(&self.decrypt_header(block))
    }

    fn decrypt_header(&self, block: &[u8; Header::LEN]) -> [u8; Header::LEN] {
        let mut plain = *block;
        self.0.decrypt_block(block_mut(&mut plain));
        unscramble(&mut plain);
        plain
    }

    /// Encrypts whole blocks in place; `data.len()` must be a multiple of [`BLOCK_LEN`].
    pub fn encrypt_blocks(&self, data: &mut [u8]) {
        assert_eq!(data.len() % BLOCK_LEN, 0, "partial block");
        for block in data.chunks_exact_mut(BLOCK_LEN) {
            self.0.encrypt_block(block_mut(block));
        }
    }

    /// Decrypts whole blocks in place; `data.len()` must be a multiple of [`BLOCK_LEN`].
    pub fn decrypt_blocks(&self, data: &mut [u8]) {
        assert_eq!(data.len() % BLOCK_LEN, 0, "partial block");
        for block in data.chunks_exact_mut(BLOCK_LEN) {
            self.0.decrypt_block(block_mut(block));
        }
    }

    /// Decrypts the payload that follows `header`. `data` must hold at least `header.padded_len()` bytes.
    pub fn open_payload(&self, header: &Header, data: &[u8]) -> Result<Vec<u8>, SessionError> {
        let padded = header.padded_len();
        let mut out = data
            .get(..padded)
            .ok_or(SessionError::Incomplete(padded))?
            .to_vec();
        self.decrypt_blocks(&mut out);
        out.truncate(header.len as usize);
        Ok(out)
    }
}

impl std::fmt::Debug for SessionCipher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SessionCipher(..)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scramble_round_trip() {
        let original: [u8; 16] = std::array::from_fn(|i| i as u8 * 17);
        let mut block = original;
        scramble(&mut block);
        assert_ne!(block, original);
        unscramble(&mut block);
        assert_eq!(block, original);
    }

    #[test]
    fn seal_and_open() {
        let cipher = SessionCipher::new(&AccountKey::from_password(b"pw"));
        let header = Header {
            txid: 5,
            kind: MessageType::SEND_PRIVATE_MESSAGE,
            flags: 0,
            len: 0,
        };
        let wire = cipher.seal(header, b"hello there, seventeen+");
        assert_eq!(wire.len(), 16 + 32);
        let opened = cipher
            .open_header(wire[..16].try_into().unwrap(), SERVER_MAX_PAYLOAD_LEN)
            .unwrap();
        assert_eq!(opened, Header { len: 23, ..header });
        assert_eq!(
            cipher.open_payload(&opened, &wire[16..]).unwrap(),
            b"hello there, seventeen+"
        );
    }

    #[test]
    fn header_checks() {
        let mut bytes = Header {
            txid: 1,
            kind: MessageType(1),
            flags: 1,
            len: 0,
        }
        .to_bytes();
        assert!(Header::from_bytes(&bytes, SERVER_MAX_PAYLOAD_LEN)
            .unwrap()
            .is_reply());
        bytes[11] = 1;
        assert_eq!(
            Header::from_bytes(&bytes, SERVER_MAX_PAYLOAD_LEN),
            Err(SessionError::Reserved(1))
        );
    }

    #[test]
    fn length_limits() {
        let header = |len| {
            Header {
                txid: 1,
                kind: MessageType(1),
                flags: 1,
                len,
            }
            .to_bytes()
        };
        let reply = header(SERVER_MAX_PAYLOAD_LEN + 1);
        assert_eq!(
            Header::from_bytes(&reply, SERVER_MAX_PAYLOAD_LEN),
            Err(SessionError::TooLong(SERVER_MAX_PAYLOAD_LEN + 1))
        );
        assert!(Header::from_bytes(&reply, CLIENT_MAX_PAYLOAD_LEN).is_ok());
        assert!(
            Header::from_bytes(&header(CLIENT_MAX_PAYLOAD_LEN), CLIENT_MAX_PAYLOAD_LEN).is_ok()
        );
        assert_eq!(
            Header::from_bytes(&header(CLIENT_MAX_PAYLOAD_LEN + 1), CLIENT_MAX_PAYLOAD_LEN),
            Err(SessionError::TooLong(CLIENT_MAX_PAYLOAD_LEN + 1))
        );
    }
}
