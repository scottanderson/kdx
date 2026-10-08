//! 300 Send Private Message and 301 Incoming Private Message.

use super::{need, u32_at, DecodeError};

/// A private message: to a user when sent, from a user when received.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrivateMessage {
    /// The target's ID in 300 Send Private Message; the sender's in 301.
    pub user_id: u32,
    /// Message text.
    pub text: Vec<u8>,
}

impl PrivateMessage {
    /// Size of the fixed part before the text.
    pub const HEADER_LEN: usize = 0x10;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN + 1)?;
        Ok(Self {
            user_id: u32_at(buf, 0),
            text: buf[Self::HEADER_LEN..].to_vec(),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[..4].copy_from_slice(&self.user_id.to_be_bytes());
        out.extend_from_slice(&self.text);
        out
    }
}
