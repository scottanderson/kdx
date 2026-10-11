//! Handshake framing, checked against `captures/first-connect.pcapng`.

use bytes::BytesMut;
use kdx_codec::{Error, Handshake, HandshakeCodec};
use kdx_proto::handshake::{
    HandshakeError, Hello, Proof, CLIENT_VERSION, FORMAT, SERVER_VERSION, SERVICE_KDX,
};
use kdx_proto::AccountKey;
use tokio_util::codec::{Decoder, Encoder};

const C2S: &[u8] = include_bytes!("../../kdx-proto/tests/fixtures/first-connect.c2s.bin");
const S2C: &[u8] = include_bytes!("../../kdx-proto/tests/fixtures/first-connect.s2c.bin");

const RESPONSE: [u8; 20] = [7; 20];

fn hello(version: u8, nonce: u8, name: &[u8]) -> Hello {
    Hello {
        service: SERVICE_KDX,
        format: FORMAT,
        version,
        challenge: 0,
        nonce: [nonce; 12],
        name: name.to_vec(),
    }
}

#[test]
fn capture_round_trip() {
    for (stream, hello_len, name) in [(C2S, 44, &b"admin"[..]), (S2C, 36, b"")] {
        let len = hello_len + Proof::LEN;
        // Feed a byte at a time, so each message is incomplete until its last byte.
        let mut codec = HandshakeCodec::new();
        let mut buf = BytesMut::new();
        let mut items = Vec::new();
        for &byte in &stream[..len] {
            buf.extend_from_slice(&[byte]);
            items.extend(codec.decode(&mut buf).unwrap());
        }
        assert!(codec.is_done() && buf.is_empty());
        let [Handshake::Hello { hello, .. }, Handshake::Proof(_)] = items.as_slice() else {
            panic!("got {items:?}");
        };
        assert_eq!(hello.name, name);

        let mut wire = BytesMut::new();
        for item in items {
            codec.encode(item, &mut wire).unwrap();
        }
        assert_eq!(wire, stream[..len]);
    }
}

#[test]
fn handshake_reports_truncation() {
    let mut codec = HandshakeCodec::new();
    assert!(codec.decode_eof(&mut BytesMut::new()).unwrap().is_none());
    let mut partial = BytesMut::from(&hello(CLIENT_VERSION, 1, b"x").encode(5).unwrap()[..30]);
    assert!(matches!(
        codec.decode_eof(&mut partial),
        Err(Error::Truncated)
    ));
}

#[test]
fn handshake_rejects_bad_magic() {
    let mut wire = BytesMut::from(&[0u8; 64][..]);
    assert!(matches!(
        HandshakeCodec::new().decode(&mut wire),
        Err(Error::Handshake(HandshakeError::BadMagic(_)))
    ));
}

#[test]
fn handshake_leaves_session_bytes() {
    let mut wire = BytesMut::new();
    let mut codec = HandshakeCodec::new();
    let hello = hello(SERVER_VERSION, 3, b"");
    codec
        .encode(
            Handshake::Hello {
                hello: hello.clone(),
                key: 1,
            },
            &mut wire,
        )
        .unwrap();
    let proof = Proof::new_ok(&AccountKey::EMPTY_PASSWORD, &[0; 12], RESPONSE);
    codec
        .encode(Handshake::Proof(proof.clone()), &mut wire)
        .unwrap();
    wire.extend_from_slice(b"session");
    assert_eq!(
        codec.decode(&mut wire).unwrap(),
        Some(Handshake::Hello { hello, key: 1 })
    );
    assert_eq!(
        codec.decode(&mut wire).unwrap(),
        Some(Handshake::Proof(proof))
    );
    assert!(codec.is_done());
    assert_eq!(codec.decode(&mut wire).unwrap(), None);
    assert_eq!(&wire[..], b"session");
}
