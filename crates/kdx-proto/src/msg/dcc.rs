//! Client-to-client (DCC) messages (`docs/protocol/dcc.md`).

use super::{need, u16_at, u32_at, DecodeError};
use crate::AccountKey;

/// 117 SKPK: what each client sends first on a primary connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skpk {
    /// The sender's key for secondary connections to it.
    pub key: AccountKey,
    /// The sender's DCC listener.
    pub dcc_port: u32,
    /// The sender's ID for this connection, which a secondary connection logs in with.
    pub connection_id: u32,
    /// The sender's client version, 1620 for 1.620.
    pub client_version: u16,
    /// Bytes 0x2a to 0x2f; meaning unknown.
    pub unknown_2a: [u8; 6],
}

impl Skpk {
    /// Payload length.
    pub const LEN: usize = 0x30;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::LEN)?;
        Ok(Self {
            key: AccountKey(buf[..AccountKey::LEN].try_into().expect("32 bytes")),
            dcc_port: u32_at(buf, 0x20),
            connection_id: u32_at(buf, 0x24),
            client_version: u16_at(buf, 0x28),
            unknown_2a: buf[0x2a..Self::LEN].try_into().expect("6 bytes"),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::LEN];
        out[..AccountKey::LEN].copy_from_slice(&self.key.0);
        out[0x20..0x24].copy_from_slice(&self.dcc_port.to_be_bytes());
        out[0x24..0x28].copy_from_slice(&self.connection_id.to_be_bytes());
        out[0x28..0x2a].copy_from_slice(&self.client_version.to_be_bytes());
        out[0x2a..].copy_from_slice(&self.unknown_2a);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let skpk = Skpk {
            key: AccountKey([7; AccountKey::LEN]),
            dcc_port: 10600,
            connection_id: 2,
            client_version: 1620,
            unknown_2a: [0; 6],
        };
        let wire = skpk.encode();
        assert_eq!(wire.len(), Skpk::LEN);
        assert_eq!(&wire[0x20..0x2a], b"\0\0\x29\x68\0\0\0\x02\x06\x54");
        assert_eq!(Skpk::decode(&wire).unwrap(), skpk);
    }

    #[test]
    fn debug_hides_key() {
        let skpk = Skpk {
            key: AccountKey([7; AccountKey::LEN]),
            dcc_port: 0,
            connection_id: 0,
            client_version: 0,
            unknown_2a: [0; 6],
        };
        assert!(format!("{skpk:?}").contains("key: AccountKey(..)"));
    }
}
