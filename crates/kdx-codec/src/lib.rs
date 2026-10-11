//! Framing for KDX sessions over async I/O, built on `kdx-proto`.
//!
//! The codecs plug into [`tokio_util::codec`]. A connection starts with
//! [`HandshakeCodec`], which frames each side's hello and proof.
//!
//! The codecs do no authentication of their own: checking a
//! [`Proof`](kdx_proto::handshake::Proof) is up to the caller.

mod handshake;

pub use handshake::{Handshake, HandshakeCodec};

use kdx_proto::handshake::HandshakeError;

/// Errors from the codecs.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The underlying I/O failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// A handshake message is malformed.
    #[error(transparent)]
    Handshake(#[from] HandshakeError),
    /// The stream ended in the middle of a frame.
    #[error("stream ended mid-frame")]
    Truncated,
}
