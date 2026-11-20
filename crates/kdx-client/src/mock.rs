//! An in-process KDX server for tests: a real handshake and real framing
//! from `kdx-codec`, with a scripted session behind them.
//!
//! A [`Script`] is the server's side of one connection, step by step: what
//! to expect from the client, what to reply or push, when to close. The
//! mock accepts one connection per script, in order, and records what the
//! client sent in [`Captured`]. Both sides use [`ChallengeTable::default`],
//! so no Haxial table is needed.
//!
//! ```no_run
//! # async fn run() {
//! use kdx_client::mock::{MockServer, Step};
//! use kdx_client::{Client, Login};
//! use kdx_proto::session::MessageType;
//! use kdx_proto::{AccountKey, ChallengeTable};
//!
//! let server = MockServer::start(
//!     vec![(b"admin".to_vec(), AccountKey::from_password(b"pw"))],
//!     vec![vec![
//!         Step::Expect(MessageType::SET_IDENTITY),
//!         // Send 101 User List and 105 Server Info here...
//!     ]],
//! )
//! .await;
//! let login = Login { name: b"admin".to_vec(), password: b"pw".to_vec() };
//! let me = kdx_proto::msg::UserRecord::default();
//! let client = Client::login(server.addr, &login, &me, ChallengeTable::default()).await;
//! server.finish().await;
//! # }
//! ```

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use kdx_codec::{Frame, Handshake, HandshakeCodec, SessionCodec};
use kdx_proto::handshake::{Hello, Proof, Status, SERVER_SUB_VERSION, SUB_PROTOCOL_KDX, VERSION};
use kdx_proto::session::{Header, MessageType};
use kdx_proto::{AccountKey, ChallengeTable};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio_util::codec::Framed;

/// One step of a server-side script.
#[derive(Debug, Clone)]
pub enum Step {
    /// Read one message and require this type; remember its txid and payload.
    Expect(MessageType),
    /// Reply to the last expected message (its txid, reply flag set).
    Reply(MessageType, Vec<u8>),
    /// Send a message with the server's own txid, as a push.
    Send(MessageType, Vec<u8>),
    /// A data stream such as 414 Download File (D): the header (a reply to
    /// the last expected message when `reply`), then the bytes.
    DataReply {
        /// The data stream's type.
        kind: MessageType,
        /// Whether the header answers the last expected message.
        reply: bool,
        /// The stream's bytes.
        bytes: Vec<u8>,
    },
    /// Consume one data header and its bytes; record them.
    ExpectData,
    /// Close the connection.
    Close,
    /// Wait before the next step.
    Sleep(Duration),
}

/// The server's side of one connection.
pub type Script = Vec<Step>;

/// What the client sent.
#[derive(Debug, Default)]
pub struct Captured {
    /// `(kind, txid, payload)` of each message, in order.
    pub messages: Vec<(MessageType, u32, Vec<u8>)>,
    /// `(declared length, bytes)` of each data stream.
    pub data: Vec<(u64, Vec<u8>)>,
    /// `(txid, reply flag)` of each data header.
    pub data_headers: Vec<(u32, bool)>,
    /// Script expectations the client violated.
    pub failures: Vec<String>,
}

/// How the mock presents itself and how it rejects a bad login.
#[derive(Debug, Clone)]
pub struct MockConfig {
    /// [`SERVER_SUB_VERSION`] or [`TRACKER_SUB_VERSION`](kdx_proto::handshake::TRACKER_SUB_VERSION).
    pub sub_version: u8,
    /// `None`: close the connection on a bad proof, as Server 1.620 does.
    /// `Some(status)`: answer with a failure proof carrying that status.
    pub reject: Option<Status>,
}

impl Default for MockConfig {
    fn default() -> Self {
        Self {
            sub_version: SERVER_SUB_VERSION,
            reject: None,
        }
    }
}

/// A running mock server.
#[derive(Debug)]
pub struct MockServer {
    /// Where it listens.
    pub addr: SocketAddr,
    /// What clients have sent so far.
    pub captured: Arc<Mutex<Captured>>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl MockServer {
    /// Starts a server that accepts one connection per script, in order.
    ///
    /// `accounts` maps login names to account keys; an unknown login or a
    /// bad proof is rejected.
    pub async fn start(accounts: Vec<(Vec<u8>, AccountKey)>, scripts: Vec<Script>) -> Self {
        Self::start_as(SERVER_SUB_VERSION, accounts, scripts).await
    }

    /// [`Self::start`] with the given hello sub-version.
    pub async fn start_as(
        sub_version: u8,
        accounts: Vec<(Vec<u8>, AccountKey)>,
        scripts: Vec<Script>,
    ) -> Self {
        let config = MockConfig {
            sub_version,
            ..MockConfig::default()
        };
        Self::start_with(config, accounts, scripts).await
    }

    /// [`Self::start`] with a full configuration.
    pub async fn start_with(
        config: MockConfig,
        accounts: Vec<(Vec<u8>, AccountKey)>,
        scripts: Vec<Script>,
    ) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind the mock");
        let addr = listener.local_addr().expect("mock address");
        let captured = Arc::new(Mutex::new(Captured::default()));
        let cap = captured.clone();
        let handle = tokio::spawn(async move {
            let mut tasks = Vec::new();
            for script in scripts {
                let (stream, _) = listener.accept().await.expect("accept");
                let accounts = accounts.clone();
                let cap = cap.clone();
                let config = config.clone();
                tasks.push(tokio::spawn(serve(stream, config, accounts, script, cap)));
            }
            for task in tasks {
                let _ = task.await;
            }
        });
        Self {
            addr,
            captured,
            handle: Mutex::new(Some(handle)),
        }
    }

    /// Waits for every script to finish, then [`Self::check`]s; panics after 5
    /// seconds. The captured messages stay readable afterwards.
    pub async fn finish(&self) {
        self.wait().await;
        self.check();
    }

    /// Waits for every script to finish without checking it; panics after 5
    /// seconds. For a test that expects the client to break the script.
    pub async fn wait(&self) {
        let handle = self.handle.lock().expect("handle").take();
        if let Some(handle) = handle {
            tokio::time::timeout(Duration::from_secs(5), handle)
                .await
                .expect("mock server finished")
                .expect("mock server task");
        }
    }

    /// Panics if the client violated a script.
    pub fn check(&self) {
        let captured = self.captured.lock().expect("captured");
        let failures = &captured.failures;
        assert!(failures.is_empty(), "mock script violated: {failures:?}");
    }

    /// The payloads of every message of `kind` the client sent, in order.
    pub fn payloads(&self, kind: MessageType) -> Vec<Vec<u8>> {
        self.captured
            .lock()
            .expect("captured")
            .messages
            .iter()
            .filter(|(k, _, _)| *k == kind)
            .map(|(_, _, payload)| payload.clone())
            .collect()
    }
}

type Session = Framed<TcpStream, SessionCodec>;

async fn serve(
    stream: TcpStream,
    config: MockConfig,
    accounts: Vec<(Vec<u8>, AccountKey)>,
    script: Script,
    captured: Arc<Mutex<Captured>>,
) {
    let Some(mut session) = handshake(stream, &config, &accounts).await else {
        return;
    };
    let mut last_txid = 0;
    let mut own_txid = 0xc3b7_1d00u32;
    for step in script {
        match step {
            Step::Expect(kind) => match session.next().await {
                Some(Ok(Frame::Message(header, payload))) => {
                    let got = header.kind;
                    let mut cap = captured.lock().expect("captured");
                    cap.messages.push((got, header.txid, payload.to_vec()));
                    if got != kind {
                        cap.failures.push(format!("expected {kind}, got {got}"));
                        return;
                    }
                    last_txid = header.txid;
                }
                other => {
                    let failure = format!("expected {kind}, got {other:?}");
                    captured.lock().expect("captured").failures.push(failure);
                    return;
                }
            },
            Step::Reply(kind, payload) => {
                let header = header(last_txid, kind, true, 0);
                send(&mut session, Frame::Message(header, payload.into())).await;
            }
            Step::Send(kind, payload) => {
                own_txid += 1;
                let header = header(own_txid, kind, false, 0);
                send(&mut session, Frame::Message(header, payload.into())).await;
            }
            Step::DataReply { kind, reply, bytes } => {
                let header = if reply {
                    header(last_txid, kind, true, bytes.len() as u64)
                } else {
                    own_txid += 1;
                    header(own_txid, kind, false, bytes.len() as u64)
                };
                send(&mut session, Frame::DataHeader(header)).await;
                if !bytes.is_empty() {
                    send(&mut session, Frame::Data(Bytes::from(bytes))).await;
                }
            }
            Step::ExpectData => {
                let header = match session.next().await {
                    Some(Ok(Frame::DataHeader(header))) => header,
                    other => {
                        let failure = format!("expected a data header, got {other:?}");
                        captured.lock().expect("captured").failures.push(failure);
                        return;
                    }
                };
                let mut got = Vec::new();
                let mut cut_short = None;
                while (got.len() as u64) < header.len {
                    match session.next().await {
                        Some(Ok(Frame::Data(chunk))) => got.extend_from_slice(&chunk),
                        other => {
                            cut_short = Some(format!("expected data, got {other:?}"));
                            break;
                        }
                    }
                }
                // A stream cut short is recorded as far as it got, then fails.
                let mut cap = captured.lock().expect("captured");
                cap.data_headers.push((header.txid, header.is_reply()));
                cap.data.push((header.len, got));
                if let Some(failure) = cut_short {
                    cap.failures.push(failure);
                    return;
                }
            }
            Step::Close => return,
            Step::Sleep(duration) => tokio::time::sleep(duration).await,
        }
    }
    // Keep the connection open until the client goes away.
    while let Some(Ok(_)) = session.next().await {}
}

/// Runs the server side of the handshake; `None` when the login is rejected.
async fn handshake(
    stream: TcpStream,
    config: &MockConfig,
    accounts: &[(Vec<u8>, AccountKey)],
) -> Option<Session> {
    let table = ChallengeTable::default();
    let mut framed = Framed::new(stream, HandshakeCodec::new());
    let Some(Ok(Handshake::Hello { hello: client, .. })) = framed.next().await else {
        return None;
    };
    let hello = Hello {
        sub_protocol: SUB_PROTOCOL_KDX,
        version: VERSION,
        sub_version: config.sub_version,
        challenge: 3,
        nonce: [7; 12],
        name: Vec::new(),
    };
    let ours = Handshake::Hello {
        hello: hello.clone(),
        key: 0x1234_5678,
    };
    framed.send(ours).await.ok()?;
    let Some(Ok(Handshake::Proof(proof))) = framed.next().await else {
        return None;
    };
    let key = accounts
        .iter()
        .find(|(login, _)| *login == client.name)
        .map(|(_, key)| key.clone());
    let expected = table.response(hello.challenge).expect("in range");
    let key = key.filter(|key| proof.verify(key, &hello.nonce, &expected).is_ok());
    let Some(key) = key else {
        if let Some(status) = config.reject {
            let rejection = Proof::new_error(&client.nonce, status);
            let _ = framed.send(Handshake::Proof(rejection)).await;
        }
        return None;
    };
    let response = table.response(client.challenge)?;
    let ours = Proof::new_ok(&key, &client.nonce, response);
    framed.send(Handshake::Proof(ours)).await.ok()?;
    Some(framed.map_codec(|_| SessionCodec::server(&key)))
}

const fn header(txid: u32, kind: MessageType, reply: bool, len: u64) -> Header {
    Header {
        txid,
        kind,
        flags: if reply { Header::FLAG_REPLY } else { 0 },
        len,
    }
}

async fn send(session: &mut Session, frame: Frame) {
    session.send(frame).await.expect("mock send");
}
