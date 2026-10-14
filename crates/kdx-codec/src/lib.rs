//! Framing for KDX sessions over async I/O, built on `kdx-proto`.
//!
//! The codecs plug into [`tokio_util::codec`]. A connection starts with
//! [`HandshakeCodec`], which frames each side's hello and proof. Once the
//! proofs check out, switch the same [`Framed`](tokio_util::codec::Framed) to
//! [`SessionCodec`] with
//! [`map_codec`](tokio_util::codec::Framed::map_codec), which keeps any bytes
//! already read:
//!
//! ```no_run
//! # use tokio::io::{AsyncRead, AsyncWrite};
//! # async fn run(stream: impl AsyncRead + AsyncWrite, key: kdx_proto::AccountKey) {
//! use kdx_codec::{HandshakeCodec, SessionCodec};
//! use tokio_util::codec::Framed;
//!
//! let framed = Framed::new(stream, HandshakeCodec::new());
//! // Send our hello and proof, and read and verify the server's.
//! let framed = framed.map_codec(|_| SessionCodec::client(&key));
//! # }
//! ```
//!
//! The codecs do no authentication of their own: checking a
//! [`Proof`](kdx_proto::handshake::Proof) is up to the caller.

mod handshake;
mod session;

pub use handshake::{Handshake, HandshakeCodec};
pub use session::{Frame, SessionCodec};

use kdx_proto::handshake::HandshakeError;
use kdx_proto::session::SessionError;

/// Errors from the codecs.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The underlying I/O failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// A handshake message is malformed.
    #[error(transparent)]
    Handshake(#[from] HandshakeError),
    /// A session header is malformed or too long.
    #[error(transparent)]
    Session(#[from] SessionError),
    /// The stream ended in the middle of a frame.
    #[error("stream ended mid-frame")]
    Truncated,
    /// A payload is longer than a header can describe.
    #[error("payload too long ({0} bytes)")]
    TooLong(u64),
    /// A [`Frame::Data`] was sent with no data stream open.
    #[error("data sent outside a data stream")]
    NoDataStream,
    /// A [`Frame::Data`] holds more bytes than its stream has left.
    #[error("data exceeds its stream by {0} bytes")]
    DataOverrun(u64),
    /// A frame was sent while a data stream still had bytes left.
    #[error("data stream has {0} bytes left")]
    DataPending(u64),
}
