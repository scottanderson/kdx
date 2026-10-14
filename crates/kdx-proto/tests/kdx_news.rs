//! Checks against `captures/kdx-news.pcapng`: the `kdx` CLI posting and deleting news on Server 1.620.

use kdx_proto::msg::news::{NewsMessage, NewsMessageChange, NewsMessageId, PostNewsMessage};
use kdx_proto::msg::Status;
use kdx_proto::session::{Header, MessageType};
use kdx_proto::AccountKey;

mod common;

macro_rules! connection {
    ($port:literal) => {
        common::session(
            include_bytes!(concat!("fixtures/kdx-news-", $port, ".c2s.bin")),
            include_bytes!(concat!("fixtures/kdx-news-", $port, ".s2c.bin")),
        )
    };
}

fn one(messages: &[(Header, Vec<u8>)], kind: MessageType) -> (Header, &[u8]) {
    let found: Vec<_> = messages.iter().filter(|(h, _)| h.kind == kind).collect();
    assert_eq!(found.len(), 1, "{kind}");
    (found[0].0, &found[0].1)
}

#[test]
fn post() {
    let session = connection!("53204");
    let (_, wire) = one(&session, MessageType::POST_NEWS_MESSAGE);
    let post = PostNewsMessage::decode(wire).unwrap();
    assert_eq!(post.group_id, 1);
    assert_eq!(post.subject, b"Third post");
    assert_eq!(post.body, b"Posted from stdin.");
    assert_eq!(post.encode().unwrap(), wire);

    // The server doesn't reply on success; the poster gets 613 like every reader.
    let (header, wire) = one(&session, MessageType::NEWS_MESSAGE_POSTED);
    assert!(!header.is_reply());
    let posted = NewsMessageChange::decode(wire).unwrap();
    assert_eq!(
        (posted.group_id, posted.message_id, posted.message_count),
        (1, 4, 2)
    );
    assert_eq!(posted.encode(), wire);
    assert!(session.iter().all(|(h, _)| h.kind != MessageType::STATUS));
}

#[test]
fn delete() {
    let session = connection!("53224");
    let (_, wire) = one(&session, MessageType::DELETE_NEWS_MESSAGE);
    let id = NewsMessageId::decode(wire).unwrap();
    assert_eq!((id.group_id, id.message_id), (1, 4));
    assert_eq!(id.encode(), wire);
    let (header, wire) = one(&session, MessageType::NEWS_MESSAGE_DELETED);
    assert!(!header.is_reply());
    let deleted = NewsMessageChange::decode(wire).unwrap();
    assert_eq!((deleted.message_id, deleted.message_count), (4, 1));
    assert!(session.iter().all(|(h, _)| h.kind != MessageType::STATUS));
}

#[test]
fn read_one() {
    let session = connection!("40426");
    let (_, wire) = one(&session, MessageType::GET_NEWS_MESSAGE);
    let id = NewsMessageId::decode(wire).unwrap();
    assert_eq!((id.group_id, id.message_id), (1, 2));
    assert_eq!(id.encode(), wire);
    let (_, wire) = one(&session, MessageType::NEWS_MESSAGE);
    let (message, _) = NewsMessage::decode(wire).unwrap();
    assert_eq!(message.message_id, 2);
    assert_eq!(message.poster, b"kdx-rs");
    assert_eq!(message.login, b"admin");
    assert_eq!(message.body, b"First line.\rSecond line.\r");
    assert_eq!(message.encode().unwrap(), wire);
}

#[test]
fn guest_may_not_post() {
    let session = common::session_with_key(
        include_bytes!("fixtures/kdx-news-40434.c2s.bin"),
        include_bytes!("fixtures/kdx-news-40434.s2c.bin"),
        &AccountKey::EMPTY_PASSWORD,
    );
    let (header, wire) = one(&session, MessageType::STATUS);
    assert!(header.is_reply());
    assert!(matches!(
        Status::decode(wire).unwrap(),
        Status::Error { code: 0x10014, .. }
    ));
}
