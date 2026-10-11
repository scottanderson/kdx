//! An async KDX client on tokio.
//!
//! [`Client::login`] logs in and offers typed requests. Underneath,
//! [`Connection::connect`] runs the handshake and returns a connection that
//! sends and receives raw session messages through `kdx-codec`.

use anyhow::{bail, Context, Result};
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use kdx_codec::{Frame, Handshake, HandshakeCodec, SessionCodec};
use kdx_proto::handshake::{Hello, Proof, CLIENT_VERSION, FORMAT, NONCE_LEN, SERVICE_KDX};
use kdx_proto::session::{Header, MessageType};
use kdx_proto::{AccountKey, ChallengeTable};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_util::codec::Framed;

mod client;
pub mod table;
#[cfg(test)]
mod tests;

pub use client::{Client, Event, ServerError};

/// Login details for [`Connection::connect`].
#[derive(Debug, Clone, Default)]
pub struct Login {
    /// Account login; empty for a guest login.
    pub name: Vec<u8>,
    /// Account password; empty for a guest login.
    pub password: Vec<u8>,
}

/// An authenticated connection to a KDX server.
#[derive(Debug)]
pub struct Connection {
    framed: Framed<TcpStream, SessionCodec>,
    next_txid: u32,
    /// The server's hello.
    pub server_hello: Hello,
}

impl Connection {
    /// Connects and runs the handshake.
    pub async fn connect(
        addr: impl ToSocketAddrs,
        login: &Login,
        table: &ChallengeTable,
    ) -> Result<Self> {
        let key = AccountKey::from_password(&login.password);
        Self::connect_with_key(
            addr,
            login.name.clone(),
            key,
            &Responses::Table(table.clone()),
        )
        .await
    }

    /// Opens a second connection on behalf of a logged-in user, as file
    /// transfers do.
    ///
    /// `session_key` is the first 32 bytes of the user's 105 Server Info.
    pub async fn attach(
        addr: impl ToSocketAddrs,
        user_id: u32,
        session_key: [u8; AccountKey::LEN],
        table: &ChallengeTable,
    ) -> Result<Self> {
        Self::attach_with(addr, user_id, session_key, &Responses::Table(table.clone())).await
    }

    async fn attach_with(
        addr: impl ToSocketAddrs,
        user_id: u32,
        session_key: [u8; AccountKey::LEN],
        responses: &Responses,
    ) -> Result<Self> {
        let mut name = vec![0];
        name.extend_from_slice(&user_id.to_be_bytes());
        Self::connect_with_key(addr, name, AccountKey(session_key), responses).await
    }

    async fn connect_with_key(
        addr: impl ToSocketAddrs,
        name: Vec<u8>,
        key: AccountKey,
        responses: &Responses,
    ) -> Result<Self> {
        let stream = TcpStream::connect(addr).await.context("connecting")?;
        stream.set_nodelay(true)?;
        let mut framed = Framed::new(stream, HandshakeCodec::new());

        let hello = Hello {
            service: SERVICE_KDX,
            format: FORMAT,
            version: CLIENT_VERSION,
            challenge: random_u32()? as u16 % ChallengeTable::OFFSETS,
            nonce: random_bytes()?,
            name,
        };
        framed
            .send(Handshake::Hello {
                hello: hello.clone(),
                key: random_u32()?,
            })
            .await?;

        let server_hello = match framed.next().await.context("reading server hello")?? {
            Handshake::Hello { hello, .. } => hello,
            Handshake::Proof(_) => unreachable!("the codec reads a hello first"),
        };
        let response = responses
            .response(server_hello.challenge)
            .context("server challenge out of range")?;
        framed
            .send(Handshake::Proof(Proof::new_ok(
                &key,
                &server_hello.nonce,
                response,
            )))
            .await?;

        let Handshake::Proof(proof) = framed.next().await.context("reading server proof")?? else {
            unreachable!("the codec reads a proof after the hello");
        };
        let expected = responses
            .response(hello.challenge)
            .expect("challenge in range");
        proof
            .verify(&key, &hello.nonce, &expected)
            .context("server proof")?;

        Ok(Self {
            framed: framed.map_codec(|_| SessionCodec::client(&key)),
            next_txid: random_u32()?.max(1),
            server_hello,
        })
    }

    /// Sends a request and returns its transaction ID.
    pub async fn send(&mut self, kind: MessageType, payload: &[u8]) -> Result<u32> {
        let txid = self.next_txid();
        let header = Header {
            txid,
            kind,
            flags: 0,
            len: 0,
        };
        let payload = Bytes::copy_from_slice(payload);
        self.framed.send(Frame::Message(header, payload)).await?;
        Ok(txid)
    }

    /// Receives the next message.
    ///
    /// This is cancel-safe, so it can run under [`tokio::time::timeout`].
    pub async fn recv(&mut self) -> Result<(Header, Vec<u8>)> {
        match self.next_frame().await? {
            Frame::Message(header, payload) => Ok((header, payload.into())),
            Frame::DataHeader(header) => bail!("unexpected {} data stream", header.kind),
            Frame::Data(_) => bail!("unexpected transfer data"),
        }
    }

    /// Receives a transfer data header and streams its data into `sink`.
    ///
    /// The data is framed like a message, but its length can exceed the cap on
    /// normal payloads, so it comes in chunks rather than all at once.
    pub async fn recv_data(&mut self, sink: &mut (impl AsyncWrite + Unpin)) -> Result<Header> {
        let header = match self.next_frame().await? {
            Frame::DataHeader(header) => header,
            Frame::Message(header, _) => bail!("expected transfer data, got {}", header.kind),
            Frame::Data(_) => bail!("transfer data without a header"),
        };
        let mut left = u64::from(header.len);
        while left > 0 {
            let Frame::Data(chunk) = self.next_frame().await? else {
                bail!("transfer data ended {left} bytes early");
            };
            sink.write_all(&chunk).await?;
            left -= chunk.len() as u64;
        }
        sink.flush().await?;
        Ok(header)
    }

    /// Sends a transfer data header for `len` bytes, then the bytes from `source`.
    pub async fn send_data(
        &mut self,
        kind: MessageType,
        len: u32,
        source: &mut (impl AsyncRead + Unpin),
    ) -> Result<u32> {
        let txid = self.next_txid();
        let header = Header {
            txid,
            kind,
            flags: 0,
            len,
        };
        self.framed.feed(Frame::DataHeader(header)).await?;
        let mut left = u64::from(len);
        let mut chunk = vec![0; DATA_CHUNK];
        while left > 0 {
            let n = left.min(DATA_CHUNK as u64) as usize;
            source
                .read_exact(&mut chunk[..n])
                .await
                .context("reading upload source")?;
            self.framed
                .feed(Frame::Data(Bytes::copy_from_slice(&chunk[..n])))
                .await?;
            left -= n as u64;
        }
        self.framed.flush().await?;
        Ok(txid)
    }

    async fn next_frame(&mut self) -> Result<Frame> {
        Ok(self.framed.next().await.context("connection closed")??)
    }

    fn next_txid(&mut self) -> u32 {
        let txid = self.next_txid;
        self.next_txid = self.next_txid.wrapping_add(1).max(1);
        txid
    }
}

/// Bytes of upload data read from the source at a time.
const DATA_CHUNK: usize = 0x10000;

/// Where the handshake gets its challenge responses.
#[derive(Debug, Clone)]
enum Responses {
    Table(ChallengeTable),
    /// Responses derived from the offset, for tests that have no table.
    #[cfg(test)]
    Fake,
}

impl Responses {
    fn response(&self, offset: u16) -> Option<[u8; kdx_proto::challenge::RESPONSE_LEN]> {
        match self {
            Self::Table(table) => table.response(offset),
            #[cfg(test)]
            Self::Fake => (offset < ChallengeTable::OFFSETS).then(|| fake_response(offset)),
        }
    }
}

/// The response that [`Responses::Fake`] gives for `offset`.
#[cfg(test)]
fn fake_response(offset: u16) -> [u8; kdx_proto::challenge::RESPONSE_LEN] {
    std::array::from_fn(|i| (offset as usize + i) as u8)
}

fn random_bytes<const N: usize>() -> Result<[u8; N]> {
    let mut bytes = [0; N];
    getrandom::fill(&mut bytes).map_err(|err| anyhow::anyhow!("random: {err}"))?;
    Ok(bytes)
}

fn random_u32() -> Result<u32> {
    Ok(u32::from_be_bytes(random_bytes()?))
}

const _: () = assert!(NONCE_LEN == 12);
