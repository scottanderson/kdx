//! One browsing connection: the handshake, then requests until it closes or idles out.

use std::net::SocketAddr;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use kdx_codec::{Frame, Handshake, HandshakeCodec, SessionCodec};
use kdx_proto::handshake::{
    Hello, Proof, Status, CLIENT_SUB_VERSION, NONCE_LEN, SUB_PROTOCOL_KDX, TRACKER_SUB_VERSION,
    VERSION,
};
use kdx_proto::session::{Header, MessageType};
use kdx_proto::AccountKey;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_util::codec::Framed;

use crate::{status, Reply, Tracker, MAX_PAYLOAD_LEN};

/// How long a connection may go without a request.
pub const IDLE_LIMIT: Duration = Duration::from_secs(1200);
/// The proof status Tracker 1.110 sends for a failed login.
const LOGIN_FAILED: Status = Status::Other(1);

/// Runs one connection to the end.
pub(crate) async fn serve(tracker: &Tracker, stream: TcpStream, peer: SocketAddr) -> Result<()> {
    stream.set_nodelay(true)?;
    let mut framed = Framed::new(stream, HandshakeCodec::new());

    let Handshake::Hello { hello: client, .. } = next(&mut framed).await? else {
        unreachable!("the codec reads a hello first");
    };
    let ours = Hello {
        sub_protocol: SUB_PROTOCOL_KDX,
        version: VERSION,
        sub_version: TRACKER_SUB_VERSION,
        challenge: random_u32() as u16 % tracker.table.offsets(),
        nonce: random(),
        name: Vec::new(),
    };
    framed
        .send(Handshake::Hello {
            hello: ours.clone(),
            key: random_u32(),
        })
        .await?;
    if let Some(status) = refuse(&client) {
        framed
            .send(Handshake::Proof(Proof::new_error(&client.nonce, status)))
            .await?;
        bail!(
            "refused sub-protocol \"{}\" sub-version {:#04x}",
            client.sub_protocol.escape_ascii(),
            client.sub_version
        );
    }

    let Handshake::Proof(proof) = next(&mut framed).await? else {
        unreachable!("the codec reads a proof after the hello");
    };
    let (key, admin) = if client.name.is_empty() {
        (Some(AccountKey::EMPTY_PASSWORD), false)
    } else {
        let settings = tracker.state().settings.clone();
        let admin = settings.is_admin_login(&client.name);
        (admin.then_some(settings.admin_key), admin)
    };
    let expected = tracker
        .table
        .response(ours.challenge)
        .expect("challenge in range");
    let response = tracker.table.response(client.challenge);
    let (Some(key), Some(response)) = (
        key.filter(|key| proof.verify(key, &ours.nonce, &expected).is_ok())
            .filter(|_| client.nonce != ours.nonce),
        response,
    ) else {
        framed
            .send(Handshake::Proof(Proof::new_error(
                &client.nonce,
                LOGIN_FAILED,
            )))
            .await?;
        bail!("login {:?} failed", String::from_utf8_lossy(&client.name));
    };
    framed
        .send(Handshake::Proof(Proof::new_ok(
            &key,
            &client.nonce,
            response,
        )))
        .await?;
    let who = if admin { "admin" } else { "guest" };
    eprintln!("{peer}: logged in as {who}");

    let mut framed = framed.map_codec(|_| SessionCodec::new(&key, MAX_PAYLOAD_LEN));
    let greeting = tracker.state().settings.greeting_text().map(<[u8]>::to_vec);
    if let Some(greeting) = greeting {
        let header = Header {
            txid: random_u32().max(1),
            kind: MessageType::SERVER_GREETING,
            flags: 0,
            len: 0,
        };
        framed.send(Frame::Message(header, greeting.into())).await?;
    }

    loop {
        let frame = match timeout(IDLE_LIMIT, framed.next()).await {
            Err(_) => bail!("idle"),
            Ok(None) => return Ok(()),
            Ok(Some(frame)) => frame?,
        };
        let Frame::Message(header, payload) = frame else {
            bail!("transfer data on a tracker connection");
        };
        if header.is_reply() {
            continue;
        }
        let (kind, payload) = match tracker.handle(header.kind, &payload, admin, peer) {
            Reply::None => continue,
            Reply::Message(kind, payload) => (kind, payload),
            Reply::Error(code) => (MessageType::STATUS, status(code, header.kind)),
        };
        let reply = Header {
            txid: header.txid,
            kind,
            flags: Header::FLAG_REPLY,
            len: 0,
        };
        framed
            .send(Frame::Message(reply, Bytes::from(payload)))
            .await?;
    }
}

/// The proof status that refuses a client's hello, as `FUN_0804bb4c` decides.
fn refuse(client: &Hello) -> Option<Status> {
    if client.sub_protocol != SUB_PROTOCOL_KDX {
        Some(Status::UnknownProtocol)
    } else if client.sub_version < CLIENT_SUB_VERSION {
        Some(Status::TooOld)
    } else if client.sub_version > CLIENT_SUB_VERSION {
        Some(Status::TooNew)
    } else {
        None
    }
}

async fn next(framed: &mut Framed<TcpStream, HandshakeCodec>) -> Result<Handshake> {
    let item = timeout(IDLE_LIMIT, framed.next())
        .await
        .map_err(|_| anyhow!("idle during the handshake"))?;
    Ok(item.context("closed during the handshake")??)
}

fn random<const N: usize>() -> [u8; N] {
    let mut bytes = [0; N];
    getrandom::fill(&mut bytes).expect("system random source");
    bytes
}

fn random_u32() -> u32 {
    u32::from_be_bytes(random())
}

const _: () = assert!(NONCE_LEN == 12);
