//! Checks against `captures/linux-accounts-news.pcapng`: Client 1.620 for Linux as `admin` and as guest.

use kdx_proto::handshake::{compute_auth, Hello, Proof, Status as ProofStatus};
use kdx_proto::msg::admin::AccountClass;
use kdx_proto::msg::news::{NewsgroupId, NewsgroupInfo};
use kdx_proto::msg::Status;
use kdx_proto::session::{Header, MessageType};
use kdx_proto::AccountKey;

mod common;

fn session() -> Vec<(Header, Vec<u8>)> {
    common::session(
        include_bytes!("fixtures/linux-accounts-news-51382.c2s.bin"),
        include_bytes!("fixtures/linux-accounts-news-51382.s2c.bin"),
    )
}

fn all(messages: &[(Header, Vec<u8>)], kind: MessageType) -> Vec<(Header, &[u8])> {
    messages
        .iter()
        .filter(|(h, _)| h.kind == kind)
        .map(|(h, p)| (*h, p.as_slice()))
        .collect()
}

fn one(messages: &[(Header, Vec<u8>)], kind: MessageType) -> (Header, &[u8]) {
    let found = all(messages, kind);
    assert_eq!(found.len(), 1, "{kind}");
    found[0]
}

/// The 1 Status reply to `request`.
fn status(messages: &[(Header, Vec<u8>)], request: &Header) -> Status {
    let (_, wire) = all(messages, MessageType::STATUS)
        .into_iter()
        .find(|(h, _)| h.txid == request.txid)
        .unwrap();
    Status::decode(wire).unwrap()
}

#[test]
fn set_account_class() {
    let session = session();
    // Frames 17 and 27.
    for (_, wire) in all(&session, MessageType::GET_ACCOUNT_CLASS) {
        assert_eq!(wire, [2]);
    }
    // Frames 20 and 30: Upload (bit 11) off, then on again; each 61 bytes, unpadded.
    let sets = all(&session, MessageType::SET_ACCOUNT_CLASS);
    let bytes: Vec<_> = sets
        .iter()
        .map(|(request, wire)| {
            let class = AccountClass::decode_set(wire).unwrap();
            assert_eq!((class.index, class.name.as_slice()), (2, &b"Guest"[..]));
            assert_eq!(class.encode_set().unwrap(), *wire);
            // Frames 23 and 33.
            assert_eq!(status(&session, request), Status::Success { value: 0x1f8 });
            class.privileges.encode()[1]
        })
        .collect();
    assert_eq!(bytes, [0x20, 0x30]);
}

#[test]
fn create_set_and_delete_newsgroup() {
    let session = session();
    // Frame 58: 82 bytes, unpadded, with the ID left 0.
    let (request, wire) = one(&session, MessageType::CREATE_NEWSGROUP);
    let create = NewsgroupInfo::decode(wire).unwrap();
    assert_eq!(create.group_id, 0);
    assert_eq!(create.name, b"test newsgroup");
    assert_eq!(create.description, b"desc");
    assert_eq!(create.size_limit_kib, 420);
    assert_eq!(
        (create.read_classes, create.post_classes),
        (1 << 31, 1 << 31)
    );
    assert_eq!(create.ignored_20, [0; 4]);
    assert_eq!(create.encode().unwrap(), wire);
    // Frame 61.
    assert_eq!(status(&session, &request), Status::Success { value: 0x258 });

    // Frames 65 and 78; 68.
    for kind in [MessageType::GET_NEWS_MESSAGES, MessageType::GET_NEWSGROUP] {
        for (_, wire) in all(&session, kind) {
            assert_eq!(NewsgroupId::decode(wire).unwrap(), NewsgroupId(2));
            assert_eq!(NewsgroupId(2).encode(), wire);
        }
    }

    // Frame 69: the reply is not padded either.
    let (_, wire) = one(&session, MessageType::NEWSGROUP_INFO);
    assert_eq!(wire.len(), 82);
    let info = NewsgroupInfo::decode(wire).unwrap();
    assert_eq!(
        (info.group_id, info.name.as_slice()),
        (2, &b"test newsgroup"[..])
    );
    assert_eq!(info.encode().unwrap(), wire);

    // Frame 71: 91 bytes.
    let (request, wire) = one(&session, MessageType::SET_NEWSGROUP);
    let set = NewsgroupInfo::decode(wire).unwrap();
    assert_eq!(set.group_id, 2);
    assert_eq!(set.name, b"Lab Group");
    assert_eq!(set.description, b"edited description");
    assert_eq!(set.size_limit_kib, 50);
    assert_eq!(set.encode().unwrap(), wire);
    // Frame 74.
    assert_eq!(status(&session, &request), Status::Success { value: 0x25c });

    // Frames 81 and 84.
    let (request, wire) = one(&session, MessageType::DELETE_NEWSGROUP);
    assert_eq!(NewsgroupId::decode(wire).unwrap(), NewsgroupId(2));
    assert_eq!(NewsgroupId(2).encode(), wire);
    assert_eq!(status(&session, &request), Status::Success { value: 0x259 });
}

#[test]
fn guest_login() {
    let c2s = include_bytes!("fixtures/linux-accounts-news-36078.c2s.bin");
    let s2c = include_bytes!("fixtures/linux-accounts-news-36078.s2c.bin");
    let (client, client_len) = Hello::decode(c2s).unwrap();
    let (server, server_len) = Hello::decode(s2c).unwrap();
    // Frame 94: no login name.
    assert!(client.name.is_empty());
    // Frames 98 and 99: both proofs use the empty password's key.
    let key = AccountKey::EMPTY_PASSWORD;
    for (wire, receiver) in [(&c2s[client_len..], &server), (&s2c[server_len..], &client)] {
        let proof = Proof::decode(wire).unwrap();
        assert_eq!(proof.status, ProofStatus::Ok);
        assert_eq!(proof.auth, compute_auth(&key, &receiver.nonce));
    }
    // Frame 100 and on decrypt with the same key.
    let session = common::session_with_key(c2s, s2c, &key);
    one(&session, MessageType::SET_IDENTITY);
    one(&session, MessageType::SERVER_INFO);
}
