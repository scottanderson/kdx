//! Framing for the hello and proof that open a connection.

use bytes::{Buf, BytesMut};
use kdx_proto::handshake::{HandshakeError, Hello, Proof};
use tokio_util::codec::{Decoder, Encoder};

use crate::Error;

/// A handshake message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Handshake {
    /// The first message from each side.
    Hello {
        /// The hello.
        hello: Hello,
        /// The XOR key that the hello is sent under.
        key: u32,
    },
    /// The second message from each side.
    Proof(Proof),
}

/// Frames the peer's [`Hello`], then its [`Proof`].
///
/// After the proof the decoder yields nothing more; switch to
/// [`SessionCodec`](crate::SessionCodec) to read the session.
#[derive(Debug, Clone, Default)]
pub struct HandshakeCodec {
    next: Step,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Step {
    #[default]
    Hello,
    Proof,
    Done,
}

impl HandshakeCodec {
    /// Creates a codec that expects a hello first.
    pub const fn new() -> Self {
        Self { next: Step::Hello }
    }

    /// Whether the peer's hello and proof have both been read.
    pub fn is_done(&self) -> bool {
        self.next == Step::Done
    }
}

impl Decoder for HandshakeCodec {
    type Item = Handshake;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Handshake>, Error> {
        match self.next {
            Step::Hello => match Hello::decode(src) {
                Ok((hello, len)) => {
                    let key = src.get_u32();
                    src.advance(len - 4);
                    self.next = Step::Proof;
                    Ok(Some(Handshake::Hello { hello, key }))
                }
                Err(HandshakeError::Incomplete(need)) => {
                    src.reserve(need.saturating_sub(src.len()));
                    Ok(None)
                }
                Err(err) => Err(err.into()),
            },
            Step::Proof => {
                if src.len() < Proof::LEN {
                    src.reserve(Proof::LEN - src.len());
                    return Ok(None);
                }
                let proof = Proof::decode(src)?;
                src.advance(Proof::LEN);
                self.next = Step::Done;
                Ok(Some(Handshake::Proof(proof)))
            }
            Step::Done => Ok(None),
        }
    }

    fn decode_eof(&mut self, src: &mut BytesMut) -> Result<Option<Handshake>, Error> {
        match self.decode(src)? {
            Some(item) => Ok(Some(item)),
            None if self.next == Step::Done || src.is_empty() && self.next == Step::Hello => {
                Ok(None)
            }
            None => Err(Error::Truncated),
        }
    }
}

impl Encoder<Handshake> for HandshakeCodec {
    type Error = Error;

    fn encode(&mut self, item: Handshake, dst: &mut BytesMut) -> Result<(), Error> {
        match item {
            Handshake::Hello { hello, key } => dst.extend_from_slice(&hello.encode(key)?),
            Handshake::Proof(proof) => dst.extend_from_slice(&proof.encode()),
        }
        Ok(())
    }
}
