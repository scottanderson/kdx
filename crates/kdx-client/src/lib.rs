//! A blocking KDX client connection.
//!
//! [`Connection::connect`] runs the handshake and returns a connection that
//! sends and receives session messages.

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};

use anyhow::{bail, Context, Result};
use kdx_proto::handshake::{Hello, Proof, CLIENT_VERSION, FORMAT, NONCE_LEN, SERVICE_KDX};
use kdx_proto::session::{Header, MessageType, SessionCipher};
use kdx_proto::{AccountKey, ChallengeTable};

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
    stream: TcpStream,
    cipher: SessionCipher,
    next_txid: u32,
    /// The server's hello.
    pub server_hello: Hello,
}

impl Connection {
    /// Connects and runs the handshake.
    pub fn connect(
        addr: impl ToSocketAddrs,
        login: &Login,
        table: &ChallengeTable,
    ) -> Result<Self> {
        let key = AccountKey::from_password(&login.password);
        Self::connect_with_key(addr, login.name.clone(), key, table)
    }

    /// Opens a second connection on behalf of a logged-in user, as file
    /// transfers do.
    ///
    /// `session_key` is the first 32 bytes of the user's 105 Server Info.
    pub fn attach(
        addr: impl ToSocketAddrs,
        user_id: u32,
        session_key: [u8; AccountKey::LEN],
        table: &ChallengeTable,
    ) -> Result<Self> {
        let mut name = vec![0];
        name.extend_from_slice(&user_id.to_be_bytes());
        Self::connect_with_key(addr, name, AccountKey(session_key), table)
    }

    fn connect_with_key(
        addr: impl ToSocketAddrs,
        name: Vec<u8>,
        key: AccountKey,
        table: &ChallengeTable,
    ) -> Result<Self> {
        let mut stream = TcpStream::connect(addr).context("connecting")?;
        stream.set_nodelay(true)?;

        let hello = Hello {
            service: SERVICE_KDX,
            format: FORMAT,
            version: CLIENT_VERSION,
            challenge: random_u32()? as u16 % ChallengeTable::OFFSETS,
            nonce: random_bytes()?,
            name,
        };
        stream.write_all(&hello.encode(random_u32()?)?)?;

        let mut buf = vec![0; Hello::HEADER_LEN];
        stream
            .read_exact(&mut buf)
            .context("reading server hello")?;
        let len = Hello::peek_len(&buf)?;
        buf.resize(len, 0);
        stream.read_exact(&mut buf[Hello::HEADER_LEN..])?;
        let (server_hello, _) = Hello::decode(&buf)?;

        let response = table
            .response(server_hello.challenge)
            .context("server challenge out of range")?;
        stream.write_all(&Proof::new_ok(&key, &server_hello.nonce, response).encode())?;

        let mut wire = [0; Proof::LEN];
        stream
            .read_exact(&mut wire)
            .context("reading server proof")?;
        let expected = table.response(hello.challenge).expect("challenge in range");
        Proof::decode(&wire)?
            .verify(&key, &hello.nonce, &expected)
            .context("server proof")?;

        Ok(Self {
            stream,
            cipher: SessionCipher::new(&key),
            next_txid: random_u32()?.max(1),
            server_hello,
        })
    }

    /// Sends a request and returns its transaction ID.
    pub fn send(&mut self, kind: MessageType, payload: &[u8]) -> Result<u32> {
        let txid = self.next_txid;
        self.next_txid = self.next_txid.wrapping_add(1).max(1);
        let header = Header {
            txid,
            kind,
            flags: 0,
            len: 0,
        };
        self.stream.write_all(&self.cipher.seal(header, payload))?;
        Ok(txid)
    }

    /// Receives the next message.
    pub fn recv(&mut self) -> Result<(Header, Vec<u8>)> {
        let mut block = [0; Header::LEN];
        self.stream.read_exact(&mut block)?;
        let header = self.cipher.open_header(&block)?;
        let mut data = vec![0; header.padded_len()];
        self.stream.read_exact(&mut data)?;
        Ok((header, self.cipher.open_payload(&header, &data)?))
    }

    /// The underlying socket, for timeouts.
    pub const fn stream(&self) -> &TcpStream {
        &self.stream
    }
}

/// Reads the challenge table from a saved copy or from a KDX program file.
pub fn load_table(path: &std::path::Path) -> Result<ChallengeTable> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    match ChallengeTable::from_bytes(&bytes) {
        Ok(table) => Ok(table),
        Err(_) => match ChallengeTable::extract(&bytes) {
            Ok(table) => Ok(table),
            Err(err) => bail!("{}: {err}", path.display()),
        },
    }
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
