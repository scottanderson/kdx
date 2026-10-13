//! Framing for the encrypted messages that follow the handshake.

use bytes::{Bytes, BytesMut};
use kdx_proto::session::{
    padded_len, padding_fill, Header, MessageType, Scramble, SessionCipher, BLOCK_LEN,
    CLIENT_MAX_PAYLOAD_LEN, SERVER_MAX_PAYLOAD_LEN,
};
use kdx_proto::AccountKey;
use tokio_util::codec::{Decoder, Encoder};

use crate::Error;

/// Most transfer data that one [`Frame::Data`] carries when decoded.
const DATA_CHUNK: usize = 0x10000;

/// A session frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    /// A whole message.
    ///
    /// When encoding, the header's `len` is set from the payload.
    Message(Header, Bytes),
    /// The header of a transfer data stream, such as 414 Download File (D).
    ///
    /// Its `len` bytes follow as [`Frame::Data`], which can exceed any limit
    /// on messages.
    DataHeader(Header),
    /// The next bytes of the open data stream.
    Data(Bytes),
}

/// Encrypts and frames session messages for one connection.
///
/// Headers of 414 Download File (D) and 417 Upload File (D) open a data
/// stream: the decoder yields [`Frame::DataHeader`], then the data in
/// [`Frame::Data`] chunks of at most 64 KiB, so a large file is never held
/// whole. Every other header is decoded with its payload as one
/// [`Frame::Message`]. The encoder pads the last block of data with the
/// header's txid fill, as Client 1.620 does.
#[derive(Debug, Clone)]
pub struct SessionCodec {
    cipher: SessionCipher,
    max_len: u32,
    read: Read,
    write: Option<Write>,
}

#[derive(Debug, Clone, Copy)]
enum Read {
    Header,
    Payload(Header),
    Data { left: u32, padded: usize },
}

#[derive(Debug, Clone)]
struct Write {
    fill: [u8; 4],
    left: u32,
    tail: BytesMut,
}

impl SessionCodec {
    /// Creates a codec for a session keyed with `key` that accepts payloads up to `max_len`.
    pub fn new(key: &AccountKey, max_len: u32) -> Self {
        Self {
            cipher: SessionCipher::new(key),
            max_len,
            read: Read::Header,
            write: None,
        }
    }

    /// Creates a codec for a client, which accepts what Client 1.600 accepts from a server.
    pub fn client(key: &AccountKey) -> Self {
        Self::new(key, CLIENT_MAX_PAYLOAD_LEN)
    }

    /// Creates a codec for a server, which accepts what Server 1.620 accepts from a client.
    pub fn server(key: &AccountKey) -> Self {
        Self::new(key, SERVER_MAX_PAYLOAD_LEN)
    }

    /// Uses `scramble` for headers instead of the `%KDX` one.
    #[must_use]
    pub fn with_scramble(self, scramble: Scramble) -> Self {
        Self {
            cipher: self.cipher.with_scramble(scramble),
            ..self
        }
    }

    /// Whether a header of this type opens a data stream.
    pub const fn is_data(kind: MessageType) -> bool {
        matches!(
            kind,
            MessageType::DOWNLOAD_FILE_DATA | MessageType::UPLOAD_FILE_DATA
        )
    }

    /// Whether the codec is between frames in both directions.
    pub const fn is_idle(&self) -> bool {
        matches!(self.read, Read::Header) && self.write.is_none()
    }
}

impl Decoder for SessionCodec {
    type Item = Frame;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Frame>, Error> {
        loop {
            match self.read {
                Read::Header => {
                    let Some(block) = src.get(..Header::LEN) else {
                        src.reserve(Header::LEN - src.len());
                        return Ok(None);
                    };
                    let header = self.cipher.open_data_header(block.try_into().unwrap())?;
                    let _ = src.split_to(Header::LEN);
                    if Self::is_data(header.kind) {
                        if header.len > 0 {
                            self.read = Read::Data {
                                left: header.len,
                                padded: header.padded_len(),
                            };
                        }
                        return Ok(Some(Frame::DataHeader(header)));
                    }
                    if header.len > self.max_len {
                        return Err(kdx_proto::session::SessionError::TooLong(header.len).into());
                    }
                    self.read = Read::Payload(header);
                }
                Read::Payload(header) => {
                    let padded = header.padded_len();
                    if src.len() < padded {
                        src.reserve(padded - src.len());
                        return Ok(None);
                    }
                    let mut payload = src.split_to(padded);
                    self.cipher.decrypt_blocks(&mut payload);
                    payload.truncate(header.len as usize);
                    self.read = Read::Header;
                    return Ok(Some(Frame::Message(header, payload.freeze())));
                }
                Read::Data { left, padded } => {
                    let n = (src.len() / BLOCK_LEN * BLOCK_LEN)
                        .min(padded)
                        .min(DATA_CHUNK);
                    if n == 0 {
                        src.reserve(padded.min(DATA_CHUNK));
                        return Ok(None);
                    }
                    let mut chunk = src.split_to(n);
                    self.cipher.decrypt_blocks(&mut chunk);
                    let keep = left.min(n as u32);
                    chunk.truncate(keep as usize);
                    self.read = if padded == n {
                        Read::Header
                    } else {
                        Read::Data {
                            left: left - keep,
                            padded: padded - n,
                        }
                    };
                    return Ok(Some(Frame::Data(chunk.freeze())));
                }
            }
        }
    }

    fn decode_eof(&mut self, src: &mut BytesMut) -> Result<Option<Frame>, Error> {
        match self.decode(src)? {
            Some(frame) => Ok(Some(frame)),
            None if src.is_empty() && matches!(self.read, Read::Header) => Ok(None),
            None => Err(Error::Truncated),
        }
    }
}

impl Encoder<Frame> for SessionCodec {
    type Error = Error;

    fn encode(&mut self, frame: Frame, dst: &mut BytesMut) -> Result<(), Error> {
        match frame {
            Frame::Message(header, payload) => {
                self.check_no_stream()?;
                u32::try_from(payload.len()).map_err(|_| Error::TooLong(payload.len()))?;
                dst.extend_from_slice(&self.cipher.seal(header, &payload));
            }
            Frame::DataHeader(header) => {
                self.check_no_stream()?;
                dst.extend_from_slice(&self.cipher.seal_header(header));
                if header.len > 0 {
                    self.write = Some(Write {
                        fill: padding_fill(header.txid),
                        left: header.len,
                        tail: BytesMut::new(),
                    });
                }
            }
            Frame::Data(data) => {
                let write = self.write.as_mut().ok_or(Error::NoDataStream)?;
                let len = u32::try_from(data.len())
                    .ok()
                    .filter(|&len| len <= write.left)
                    .ok_or_else(|| Error::DataOverrun(data.len() - write.left as usize))?;
                write.left -= len;
                write.tail.extend_from_slice(&data);
                if write.left == 0 {
                    let end = padded_len(write.tail.len());
                    let fill = write.fill;
                    write
                        .tail
                        .extend((write.tail.len()..end).map(|i| fill[i % 4]));
                }
                let mut blocks = write
                    .tail
                    .split_to(write.tail.len() / BLOCK_LEN * BLOCK_LEN);
                self.cipher.encrypt_blocks(&mut blocks);
                dst.extend_from_slice(&blocks);
                if write.left == 0 {
                    self.write = None;
                }
            }
        }
        Ok(())
    }
}

impl SessionCodec {
    const fn check_no_stream(&self) -> Result<(), Error> {
        match &self.write {
            Some(write) => Err(Error::DataPending(write.left)),
            None => Ok(()),
        }
    }
}
