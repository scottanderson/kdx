//! Session framing, and a client and a server talking through `Framed` over an in-memory pipe.

use bytes::{Bytes, BytesMut};
use futures_util::{SinkExt, StreamExt};
use kdx_codec::{Error, Frame, Handshake, HandshakeCodec, SessionCodec};
use kdx_proto::handshake::{
    Hello, Proof, CLIENT_SUB_VERSION, SERVER_SUB_VERSION, SUB_PROTOCOL_KDX, VERSION,
};
use kdx_proto::session::{Header, MessageType, SessionError, SERVER_MAX_PAYLOAD_LEN};
use kdx_proto::AccountKey;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{Decoder, Encoder, Framed};

const RESPONSE: [u8; 20] = [7; 20];

fn hello(sub_version: u8, nonce: u8, name: &[u8]) -> Hello {
    Hello {
        sub_protocol: SUB_PROTOCOL_KDX,
        version: VERSION,
        sub_version,
        challenge: 0,
        nonce: [nonce; 12],
        name: name.to_vec(),
    }
}

const fn header(txid: u32, kind: MessageType, len: u32) -> Header {
    Header {
        txid,
        kind,
        flags: 0,
        len,
    }
}

/// Sends `own` and a proof, then reads and checks the peer's.
async fn handshake<S: AsyncRead + AsyncWrite + Unpin>(
    framed: &mut Framed<S, HandshakeCodec>,
    own: Hello,
    key: &AccountKey,
) -> Hello {
    framed
        .send(Handshake::Hello {
            hello: own.clone(),
            key: 0x1234_5678,
        })
        .await
        .unwrap();
    let Some(Ok(Handshake::Hello { hello: peer, .. })) = framed.next().await else {
        panic!("no hello");
    };
    framed
        .send(Handshake::Proof(Proof::new_ok(key, &peer.nonce, RESPONSE)))
        .await
        .unwrap();
    let Some(Ok(Handshake::Proof(proof))) = framed.next().await else {
        panic!("no proof");
    };
    proof.verify(key, &own.nonce, &RESPONSE).unwrap();
    peer
}

#[tokio::test]
async fn login_message_and_upload() {
    let key = AccountKey::from_password(b"pw");
    let (a, b) = tokio::io::duplex(1000);
    let upload: Bytes = (0..200_000u32).map(|i| i as u8).collect();

    let server = {
        let key = key.clone();
        let upload = upload.clone();
        tokio::spawn(async move {
            let mut framed = Framed::new(b, HandshakeCodec::new());
            let peer = handshake(&mut framed, hello(SERVER_SUB_VERSION, 2, b""), &key).await;
            assert_eq!(peer.name, b"admin");
            let mut framed = framed.map_codec(|_| SessionCodec::server(&key));

            let Some(Ok(Frame::Message(h, p))) = framed.next().await else {
                panic!("no message");
            };
            assert_eq!((h.kind, &p[..]), (MessageType::SET_IDENTITY, &b"me"[..]));
            let reply = Header {
                flags: Header::FLAG_REPLY,
                ..header(h.txid, MessageType::SERVER_INFO, 0)
            };
            framed
                .send(Frame::Message(reply, Bytes::from_static(b"info")))
                .await
                .unwrap();

            let Some(Ok(Frame::DataHeader(h))) = framed.next().await else {
                panic!("no data header");
            };
            assert_eq!((h.kind, h.len), (MessageType::UPLOAD_FILE_DATA, 200_000));
            let mut got = Vec::new();
            while got.len() < upload.len() {
                let Some(Ok(Frame::Data(d))) = framed.next().await else {
                    panic!("data ended at {}", got.len());
                };
                got.extend_from_slice(&d);
            }
            assert!(got == upload);
            assert!(framed.next().await.is_none());
        })
    };

    let mut framed = Framed::new(a, HandshakeCodec::new());
    handshake(&mut framed, hello(CLIENT_SUB_VERSION, 1, b"admin"), &key).await;
    let mut framed = framed.map_codec(|_| SessionCodec::client(&key));
    let set_identity = header(9, MessageType::SET_IDENTITY, 0);
    framed
        .send(Frame::Message(set_identity, Bytes::from_static(b"me")))
        .await
        .unwrap();
    let Some(Ok(Frame::Message(h, p))) = framed.next().await else {
        panic!("no reply");
    };
    assert!(h.is_reply() && h.txid == 9 && &p[..] == b"info");

    let data = header(10, MessageType::UPLOAD_FILE_DATA, upload.len() as u32);
    framed.send(Frame::DataHeader(data)).await.unwrap();
    // Odd sizes, so blocks straddle sends.
    for piece in upload.chunks(9999) {
        framed
            .feed(Frame::Data(Bytes::copy_from_slice(piece)))
            .await
            .unwrap();
    }
    framed.flush().await.unwrap();
    drop(framed);
    server.await.unwrap();
}

#[test]
fn encoder_checks_data_streams() {
    let mut codec = SessionCodec::client(&AccountKey::EMPTY_PASSWORD);
    let mut dst = BytesMut::new();
    let data = |n: usize| Frame::Data(Bytes::from(vec![0; n]));
    assert!(matches!(
        codec.encode(data(1), &mut dst),
        Err(Error::NoDataStream)
    ));
    let start = header(1, MessageType::DOWNLOAD_FILE_DATA, 20);
    codec.encode(Frame::DataHeader(start), &mut dst).unwrap();
    codec.encode(data(5), &mut dst).unwrap();
    assert_eq!(dst.len(), 16);
    assert!(matches!(
        codec.encode(data(16), &mut dst),
        Err(Error::DataOverrun(1))
    ));
    let message = Frame::Message(header(2, MessageType::PING, 0), Bytes::new());
    assert!(matches!(
        codec.encode(message.clone(), &mut dst),
        Err(Error::DataPending(15))
    ));
    codec.encode(data(15), &mut dst).unwrap();
    assert_eq!(dst.len(), 16 + 32);
    assert!(codec.is_idle());
    codec.encode(message, &mut dst).unwrap();
}

#[test]
fn decoder_enforces_the_payload_limit() {
    let key = AccountKey::EMPTY_PASSWORD;
    let mut wire = BytesMut::new();
    let big = header(1, MessageType::SEND_CHAT_TEXT, SERVER_MAX_PAYLOAD_LEN + 1);
    SessionCodec::client(&key)
        .encode(Frame::DataHeader(big), &mut wire)
        .unwrap();
    assert!(matches!(
        SessionCodec::server(&key).decode(&mut wire.clone()),
        Err(Error::Session(SessionError::TooLong(_)))
    ));
    // The client's limit is higher.
    assert!(SessionCodec::client(&key)
        .decode(&mut wire)
        .unwrap()
        .is_none());
}

#[test]
fn decoder_reports_truncation() {
    let key = AccountKey::EMPTY_PASSWORD;
    let mut wire = BytesMut::new();
    let mut codec = SessionCodec::client(&key);
    codec
        .encode(
            Frame::Message(header(1, MessageType::PING, 0), Bytes::from_static(b"hi")),
            &mut wire,
        )
        .unwrap();
    wire.truncate(20);
    assert!(codec.decode(&mut wire).unwrap().is_none());
    assert!(matches!(codec.decode_eof(&mut wire), Err(Error::Truncated)));
}
