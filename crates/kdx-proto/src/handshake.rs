//! The two-message handshake that opens every connection and doubles as login.
//!
//! Each side sends a [`Hello`], then a [`Proof`]. The client speaks first.

use sha1::{Digest, Sha1};
use twofish::cipher::{BlockCipherEncrypt, KeyInit};
use twofish::Twofish;

use crate::account::AccountKey;
use crate::challenge::RESPONSE_LEN;
use crate::session::block_mut;
use crate::xor::xor_stream;

/// `"%TXP"`, the transport magic.
pub const MAGIC: u32 = 0x2554_5850;
/// `"%KDX"`, the service used between clients and servers.
pub const SERVICE_KDX: u32 = 0x254b_4458;
/// `"%HGC"`, the service a GConsole peer sends; the client then switches to
/// the [`GConsoleTable`](crate::challenge::GConsoleTable) and
/// [`Scramble::GConsole`](crate::session::Scramble::GConsole).
pub const SERVICE_GCONSOLE: u32 = 0x2548_4743;
/// The only hello format that current programs accept.
pub const FORMAT: u8 = 1;
/// Version byte that Client 1.600 sends.
pub const CLIENT_VERSION: u8 = 0x10;
/// Version byte that Server 1.620 sends.
pub const SERVER_VERSION: u8 = 0x20;
/// Version byte that Tracker 1.110 sends.
pub const TRACKER_VERSION: u8 = 0x30;
/// Version byte that a GConsole peer must send.
pub const GCONSOLE_VERSION: u8 = 0x40;
/// Largest hello that a receiver accepts.
pub const MAX_HELLO_LEN: usize = 0x800;
/// Longest login name that a hello can carry.
pub const MAX_NAME_LEN: usize = 31;
/// Length of a hello nonce.
pub const NONCE_LEN: usize = 12;

/// Errors from decoding or checking handshake messages.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum HandshakeError {
    /// More bytes are needed before the message can be decoded.
    #[error("need {0} bytes")]
    Incomplete(usize),
    /// The magic is not `"%TXP"`.
    #[error("bad magic {0:#010x}")]
    BadMagic(u32),
    /// The peer's hello format is older than [`FORMAT`].
    #[error("peer protocol too old (format {0})")]
    TooOld(u8),
    /// The peer's hello format is newer than [`FORMAT`].
    #[error("peer protocol too new (format {0})")]
    TooNew(u8),
    /// The hello is longer than [`MAX_HELLO_LEN`].
    #[error("hello too long ({0} bytes)")]
    TooLong(usize),
    /// The login name is longer than [`MAX_NAME_LEN`].
    #[error("login name too long ({0} bytes)")]
    NameTooLong(usize),
    /// The peer's proof reports an error.
    #[error("peer rejected the handshake: {0:?}")]
    Rejected(Status),
    /// The proof does not match the account key or the challenge.
    #[error("authentication failed")]
    AuthFailed,
}

/// The first message from each side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hello {
    /// Service tag, normally [`SERVICE_KDX`].
    pub service: u32,
    /// Hello format, normally [`FORMAT`].
    pub format: u8,
    /// Program version byte, such as [`CLIENT_VERSION`] or [`SERVER_VERSION`].
    pub version: u8,
    /// Offset into the challenge table that the peer must answer.
    pub challenge: u16,
    /// Per-connection value that the peer's proof is computed over.
    pub nonce: [u8; NONCE_LEN],
    /// Login name, empty for a guest login and from servers.
    pub name: Vec<u8>,
}

impl Hello {
    /// Length of the fixed part, including the XOR key.
    pub const HEADER_LEN: usize = 0x24;

    /// Returns the total length of the hello that starts `buf`, after checking its header.
    pub fn peek_len(buf: &[u8]) -> Result<usize, HandshakeError> {
        let header = Self::header(buf)?;
        let len = Self::total_len(&header);
        if len > MAX_HELLO_LEN {
            return Err(HandshakeError::TooLong(len));
        }
        Ok(len)
    }

    /// Decodes a hello, returning it and the number of bytes used.
    pub fn decode(buf: &[u8]) -> Result<(Self, usize), HandshakeError> {
        let len = Self::peek_len(buf)?;
        if buf.len() < len {
            return Err(HandshakeError::Incomplete(len));
        }
        let mut body = buf[4..len].to_vec();
        xor_stream(be32(buf, 0), &mut body);
        let format = body[8];
        let skip = usize::from(body[10]) + usize::from(be16(&body, 0x10));
        let name_start = Self::HEADER_LEN - 4 + skip;
        let name = body[name_start..name_start + usize::from(body[11])].to_vec();
        let hello = Self {
            service: be32(&body, 4),
            format,
            version: body[9],
            challenge: be16(&body, 0x12),
            nonce: body[0x14..0x20].try_into().unwrap(),
            name,
        };
        Ok((hello, len))
    }

    /// Encodes the hello with the given XOR key.
    pub fn encode(&self, key: u32) -> Result<Vec<u8>, HandshakeError> {
        if self.name.len() > MAX_NAME_LEN {
            return Err(HandshakeError::NameTooLong(self.name.len()));
        }
        let len = (Self::HEADER_LEN + self.name.len() + 3) & !3;
        let mut out = Vec::with_capacity(len);
        out.extend_from_slice(&key.to_be_bytes());
        out.extend_from_slice(&MAGIC.to_be_bytes());
        out.extend_from_slice(&self.service.to_be_bytes());
        out.extend_from_slice(&[self.format, self.version, 0, self.name.len() as u8]);
        out.extend_from_slice(&[0; 6]);
        out.extend_from_slice(&self.challenge.to_be_bytes());
        out.extend_from_slice(&self.nonce);
        out.extend_from_slice(&self.name);
        out.resize(len, 0);
        xor_stream(key, &mut out[4..]);
        Ok(out)
    }

    /// Decrypts and checks the first [`Self::HEADER_LEN`] bytes.
    fn header(buf: &[u8]) -> Result<[u8; Self::HEADER_LEN], HandshakeError> {
        let mut header: [u8; Self::HEADER_LEN] = buf
            .get(..Self::HEADER_LEN)
            .ok_or(HandshakeError::Incomplete(Self::HEADER_LEN))?
            .try_into()
            .unwrap();
        xor_stream(be32(&header, 0), &mut header[4..]);
        let magic = be32(&header, 4);
        if magic != MAGIC {
            return Err(HandshakeError::BadMagic(magic));
        }
        match header[0x0c] {
            0 => Err(HandshakeError::TooOld(0)),
            FORMAT => Ok(header),
            format => Err(HandshakeError::TooNew(format)),
        }
    }

    fn total_len(header: &[u8; Self::HEADER_LEN]) -> usize {
        let name_len = usize::from(header[0x0f]);
        let pad = usize::from(header[0x0e]);
        let ext = usize::from(be16(header, 0x14));
        (name_len + pad + ext + 0x27) & !3
    }
}

/// Result code at the end of a [`Proof`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The sender accepted our hello.
    Ok,
    /// The sender does not speak our protocol or service.
    UnknownProtocol,
    /// The sender considers us too old.
    TooOld,
    /// The sender considers us too new.
    TooNew,
    /// A value not seen in the binaries.
    Other(u8),
}

impl Status {
    /// Converts a wire byte.
    pub const fn from_byte(byte: u8) -> Self {
        match byte {
            0x10 => Self::Ok,
            2 => Self::UnknownProtocol,
            3 => Self::TooOld,
            4 => Self::TooNew,
            other => Self::Other(other),
        }
    }

    /// The wire byte.
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::Ok => 0x10,
            Self::UnknownProtocol => 2,
            Self::TooOld => 3,
            Self::TooNew => 4,
            Self::Other(byte) => byte,
        }
    }
}

/// The second message from each side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof {
    /// Proof of the account key over the receiver's nonce. Its first 4 bytes are the XOR key.
    pub auth: [u8; 20],
    /// The challenge table bytes at the offset that the receiver asked for.
    pub response: [u8; RESPONSE_LEN],
    /// Whether the sender accepted the receiver's hello.
    pub status: Status,
}

impl Proof {
    /// Wire length in bytes.
    pub const LEN: usize = 44;

    /// Builds an accepting proof for a peer with `peer_nonce`.
    pub fn new_ok(
        key: &AccountKey,
        peer_nonce: &[u8; NONCE_LEN],
        response: [u8; RESPONSE_LEN],
    ) -> Self {
        Self {
            auth: compute_auth(key, peer_nonce),
            response,
            status: Status::Ok,
        }
    }

    /// Builds the proof that rejects a peer's hello.
    pub fn new_error(peer_nonce: &[u8; NONCE_LEN], status: Status) -> Self {
        let word = u32::from_le_bytes(peer_nonce[4..8].try_into().unwrap()) ^ 0x201f_7790;
        let mut auth = [0; 20];
        auth[..4].copy_from_slice(&word.to_be_bytes());
        Self {
            auth,
            response: [0; RESPONSE_LEN],
            status,
        }
    }

    /// Decodes a proof.
    pub fn decode(buf: &[u8]) -> Result<Self, HandshakeError> {
        let mut wire: [u8; Self::LEN] = buf
            .get(..Self::LEN)
            .ok_or(HandshakeError::Incomplete(Self::LEN))?
            .try_into()
            .unwrap();
        xor_stream(be32(&wire, 0), &mut wire[4..]);
        Ok(Self {
            auth: wire[..20].try_into().unwrap(),
            response: wire[20..40].try_into().unwrap(),
            status: Status::from_byte(wire[40]),
        })
    }

    /// Encodes the proof.
    pub fn encode(&self) -> [u8; Self::LEN] {
        let mut wire = [0; Self::LEN];
        wire[..20].copy_from_slice(&self.auth);
        wire[20..40].copy_from_slice(&self.response);
        wire[40] = self.status.to_byte();
        xor_stream(be32(&wire, 0), &mut wire[4..]);
        wire
    }

    /// Checks a peer's proof against our nonce and the challenge response we expect.
    pub fn verify(
        &self,
        key: &AccountKey,
        own_nonce: &[u8; NONCE_LEN],
        expected_response: &[u8; RESPONSE_LEN],
    ) -> Result<(), HandshakeError> {
        if self.status != Status::Ok {
            return Err(HandshakeError::Rejected(self.status));
        }
        if self.auth != compute_auth(key, own_nonce) || &self.response != expected_response {
            return Err(HandshakeError::AuthFailed);
        }
        Ok(())
    }
}

/// Computes a proof's `auth` field for a peer with `peer_nonce`.
pub fn compute_auth(key: &AccountKey, peer_nonce: &[u8; NONCE_LEN]) -> [u8; 20] {
    let (s, n) = (&key.0, peer_nonce);
    let mut sha = Sha1::new();
    for part in [
        &s[0..4],
        &n[0..4],
        &s[4..12],
        &n[4..8],
        &s[12..20],
        &n[8..12],
        &s[20..32],
    ] {
        sha.update(part);
    }
    let mut auth: [u8; 20] = sha.finalize().into();
    let cipher = Twofish::new_from_slice(&key.0).expect("32-byte key");
    cipher.encrypt_block(block_mut(&mut auth[..16]));
    auth
}

fn be32(buf: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(buf[at..at + 4].try_into().unwrap())
}

fn be16(buf: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(buf[at..at + 2].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hello() -> Hello {
        Hello {
            service: SERVICE_KDX,
            format: FORMAT,
            version: CLIENT_VERSION,
            challenge: 123,
            nonce: *b"0123456789ab",
            name: b"admin".to_vec(),
        }
    }

    #[test]
    fn hello_round_trip() {
        let wire = hello().encode(0xdead_beef).unwrap();
        assert_eq!(wire.len(), 44);
        assert_eq!(Hello::decode(&wire), Ok((hello(), 44)));
        assert_eq!(
            Hello::decode(&wire[..40]),
            Err(HandshakeError::Incomplete(44))
        );
    }

    #[test]
    fn hello_rejects_bad_format() {
        let mut bad = hello();
        bad.format = 2;
        let wire = bad.encode(7).unwrap();
        assert_eq!(Hello::decode(&wire), Err(HandshakeError::TooNew(2)));
    }

    #[test]
    fn proof_round_trip_and_verify() {
        let key = AccountKey::from_password(b"pw");
        let nonce = *b"0123456789ab";
        let proof = Proof::new_ok(&key, &nonce, [7; 20]);
        let decoded = Proof::decode(&proof.encode()).unwrap();
        assert_eq!(decoded, proof);
        assert_eq!(decoded.verify(&key, &nonce, &[7; 20]), Ok(()));
        assert_eq!(
            decoded.verify(&key, &nonce, &[8; 20]),
            Err(HandshakeError::AuthFailed)
        );
        let other = AccountKey::from_password(b"other");
        assert_eq!(
            decoded.verify(&other, &nonce, &[7; 20]),
            Err(HandshakeError::AuthFailed)
        );
    }

    #[test]
    fn error_proof_round_trip() {
        let proof = Proof::new_error(b"0123456789ab", Status::TooOld);
        let decoded = Proof::decode(&proof.encode()).unwrap();
        assert_eq!(decoded, proof);
        assert_eq!(
            decoded.verify(&AccountKey::EMPTY_PASSWORD, b"0123456789ab", &[0; 20]),
            Err(HandshakeError::Rejected(Status::TooOld))
        );
    }
}
