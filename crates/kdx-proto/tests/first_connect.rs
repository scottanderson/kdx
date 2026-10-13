//! Checks against `captures/first-connect.pcapng`: Client 1.600 logs in as `admin` to Server 1.620.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use kdx_proto::handshake::{
    compute_auth, Hello, Proof, Status, CLIENT_VERSION, SERVER_VERSION, SERVICE_KDX,
};
use kdx_proto::session::{Header, MessageType, SessionCipher};
use kdx_proto::{AccountKey, ChallengeTable};
use std::path::PathBuf;

mod common;
use common::{messages, PASSWORD};

const C2S: &[u8] = include_bytes!("fixtures/first-connect.c2s.bin");
const S2C: &[u8] = include_bytes!("fixtures/first-connect.s2c.bin");

struct Side {
    hello: Hello,
    hello_len: usize,
    proof: Proof,
}

fn side(stream: &[u8]) -> Side {
    let (hello, hello_len) = Hello::decode(stream).unwrap();
    let proof = Proof::decode(&stream[hello_len..]).unwrap();
    Side {
        hello,
        hello_len,
        proof,
    }
}

#[test]
fn hellos() {
    let client = side(C2S);
    assert_eq!(client.hello.service, SERVICE_KDX);
    assert_eq!(client.hello.version, CLIENT_VERSION);
    assert_eq!(client.hello.name, b"admin");
    assert_eq!(client.hello_len, 44);
    assert_eq!(
        client
            .hello
            .encode(u32::from_be_bytes(C2S[..4].try_into().unwrap()))
            .unwrap(),
        C2S[..44]
    );

    let server = side(S2C);
    assert_eq!(server.hello.version, SERVER_VERSION);
    assert!(server.hello.name.is_empty());
    assert_eq!(
        server
            .hello
            .encode(u32::from_be_bytes(S2C[..4].try_into().unwrap()))
            .unwrap(),
        S2C[..36]
    );
}

#[test]
fn proofs() {
    let key = AccountKey::from_password(PASSWORD);
    let (client, server) = (side(C2S), side(S2C));
    for (sender, receiver, wire) in [
        (&client, &server, &C2S[44..88]),
        (&server, &client, &S2C[36..80]),
    ] {
        assert_eq!(sender.proof.status, Status::Ok);
        assert_eq!(sender.proof.auth, compute_auth(&key, &receiver.hello.nonce));
        assert_eq!(sender.proof.encode(), wire);
        sender
            .proof
            .verify(&key, &receiver.hello.nonce, &sender.proof.response)
            .unwrap();
    }
}

#[test]
fn challenge_responses() {
    // The table is not in the repository; skip unless the environment, lab binaries, or `kdx table fetch` provide it.
    let Some(table) = env_table().or_else(vendor_table).or_else(cached_table) else {
        let missing = "no challenge table in KDX_CHALLENGE_TABLE, vendor/haxial, or the kdx cache";
        // CI sets KDX_REQUIRE_TABLE so a missing table fails instead of skipping.
        assert!(std::env::var_os("KDX_REQUIRE_TABLE").is_none(), "{missing}");
        eprintln!("skipping: {missing}");
        return;
    };
    let (client, server) = (side(C2S), side(S2C));
    assert_eq!(
        table.response(server.hello.challenge),
        Some(client.proof.response)
    );
    assert_eq!(
        table.response(client.hello.challenge),
        Some(server.proof.response)
    );
}

#[test]
fn session() {
    let cipher = SessionCipher::new(&AccountKey::from_password(PASSWORD));
    let c2s = messages(C2S, &cipher);
    let s2c = messages(S2C, &cipher);
    let kinds = |list: &[(Header, Vec<u8>)]| list.iter().map(|(h, _)| h.kind).collect::<Vec<_>>();
    assert!(kinds(&c2s).contains(&MessageType::SET_IDENTITY));
    assert!(kinds(&c2s).contains(&MessageType::GET_NEWS_MESSAGES));
    assert!(kinds(&s2c).contains(&MessageType::USER_LIST));
    assert!(kinds(&s2c).contains(&MessageType::NEWS_MESSAGES));

    // Sealing the decrypted messages again must give the captured bytes, padding included.
    for (stream, list) in [(C2S, &c2s), (S2C, &s2c)] {
        let resealed: Vec<u8> = list.iter().flat_map(|(h, p)| cipher.seal(*h, p)).collect();
        assert_eq!(resealed, stream[side(stream).hello_len + Proof::LEN..]);
    }
}

#[test]
fn payloads() {
    use kdx_proto::msg::news::{NewsMessages, NewsgroupEntry};
    use kdx_proto::msg::{PrivateMessage, ServerInfo, UserRecord};

    let cipher = SessionCipher::new(&AccountKey::from_password(PASSWORD));
    let mut seen = Vec::new();
    for (header, payload) in messages(C2S, &cipher)
        .into_iter()
        .chain(messages(S2C, &cipher))
    {
        let payload = payload.as_slice();
        match header.kind {
            MessageType::SET_IDENTITY => {
                let (me, len) = UserRecord::decode(payload).unwrap();
                assert_eq!(len, payload.len());
                assert_eq!(me.user_id, 0);
                assert_eq!(me.nick, b"kdx@gitbran.ch");
                assert_eq!(me.encode().unwrap(), payload);
            }
            MessageType::USER_LIST => {
                let list = UserRecord::decode_list(payload).unwrap();
                assert_eq!(list.len(), 1);
                assert_eq!(list[0].nick, b"kdx@gitbran.ch");
                assert_ne!(list[0].user_id, 0);
                assert_eq!(UserRecord::encode_list(&list).unwrap(), payload);
            }
            MessageType::SERVER_INFO => {
                let info = ServerInfo::decode(payload).unwrap();
                assert_eq!(info.name, b"New Server");
                assert_eq!(info.version, 0x0654);
                assert!(!format!("{info:?}").contains(&format!("{:?}", info.session_key)));
                assert_eq!(info.encode().unwrap(), payload);
            }
            MessageType::SEND_PRIVATE_MESSAGE | MessageType::INCOMING_PRIVATE_MESSAGE => {
                let message = PrivateMessage::decode(payload).unwrap();
                assert!([&b"test"[..], b"hi"].contains(&message.text.as_slice()));
                assert_eq!(message.encode(), payload);
            }
            MessageType::NEWSGROUP_LIST => {
                let list = NewsgroupEntry::decode_list(payload).unwrap();
                assert!(!list.is_empty());
                assert_eq!(NewsgroupEntry::encode_list(&list).unwrap(), payload);
            }
            MessageType::NEWS_MESSAGES => {
                let news = NewsMessages::decode(payload).unwrap();
                assert!(!news.messages.is_empty());
                assert_eq!(news.encode().unwrap(), payload);
            }
            _ => continue,
        }
        seen.push(header.kind);
    }
    for kind in [
        MessageType::SET_IDENTITY,
        MessageType::USER_LIST,
        MessageType::SERVER_INFO,
        MessageType::SEND_PRIVATE_MESSAGE,
        MessageType::INCOMING_PRIVATE_MESSAGE,
        MessageType::NEWSGROUP_LIST,
        MessageType::NEWS_MESSAGES,
    ] {
        assert!(seen.contains(&kind), "{kind} not in the capture");
    }
}

/// The table in `KDX_CHALLENGE_TABLE`, as base64, which CI sets from a secret.
fn env_table() -> Option<ChallengeTable> {
    let b64 = std::env::var("KDX_CHALLENGE_TABLE")
        .ok()
        .filter(|v| !v.trim().is_empty())?;
    let bytes = STANDARD
        .decode(b64.trim())
        .expect("KDX_CHALLENGE_TABLE is not base64");
    Some(ChallengeTable::from_bytes(&bytes).unwrap())
}

/// The table extracted from the server binary that `vendor/fetch` downloads.
fn vendor_table() -> Option<ChallengeTable> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../vendor/haxial/KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe"
    );
    Some(ChallengeTable::extract(&std::fs::read(path).ok()?).unwrap())
}

/// The table `kdx table fetch` saved, at the path `kdx_client::table::cache_path` uses.
fn cached_table() -> Option<ChallengeTable> {
    let dir = std::env::var_os("XDG_CACHE_HOME")
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    let bytes = std::fs::read(dir.join("kdx/table/kdx.bin")).ok()?;
    Some(ChallengeTable::from_bytes(&bytes).unwrap())
}
