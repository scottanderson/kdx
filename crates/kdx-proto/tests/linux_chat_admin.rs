//! Checks against `captures/linux-chat-admin.pcapng`: Client 1.620 for Linux as `admin`.

use kdx_proto::msg::chat::{
    ChatInvitation, ChatInvite, ChatUserChange, ChatWindowInfo, PUBLIC_CHAT,
};
use kdx_proto::session::{Header, MessageType};

mod common;

/// The private chat the client creates.
const PRIVATE_CHAT: u32 = 0xb06d_05e6;
/// Its key.
const KEY: u32 = 0x4ca9_3520;
/// The GUI client's user ID.
const CLIENT: u32 = 2;
/// The `cli-peer` listener's user ID.
const PEER: u32 = 1;

/// The GUI client's connection.
fn client() -> Vec<(Header, Vec<u8>)> {
    common::session(
        include_bytes!("fixtures/linux-chat-admin-37484.c2s.bin"),
        include_bytes!("fixtures/linux-chat-admin-37484.s2c.bin"),
    )
}

/// The `cli-peer` listener in Public Chat.
fn peer() -> Vec<(Header, Vec<u8>)> {
    common::session(
        include_bytes!("fixtures/linux-chat-admin-37478.c2s.bin"),
        include_bytes!("fixtures/linux-chat-admin-37478.s2c.bin"),
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

#[test]
fn create_private_chat() {
    let session = client();
    // Frame 84 is empty.
    let (request, wire) = one(&session, MessageType::CREATE_PRIVATE_CHAT);
    assert!(wire.is_empty());

    // Frame 85 has the 203 layout, with the creator as the only member.
    let (reply, wire) = one(&session, MessageType::CREATE_PRIVATE_CHAT_REPLY);
    assert_eq!(reply.txid, request.txid);
    let chat = ChatWindowInfo::decode(wire).unwrap();
    assert_eq!(chat.time, 0);
    assert_eq!(chat.chat_id, PRIVATE_CHAT);
    assert_eq!(chat.key, KEY);
    assert_eq!(chat.flags, 0);
    assert_eq!(chat.members, [CLIENT]);
    assert_eq!(chat.name(), b"Private 1");
    assert_eq!(chat.encode().unwrap(), wire);
}

#[test]
fn chat_invite() {
    let session = client();
    // Frames 87 and 118; user 4 had left before the second.
    let invites: Vec<_> = all(&session, MessageType::CHAT_INVITE)
        .into_iter()
        .map(|(_, wire)| {
            let invite = ChatInvite::decode(wire).unwrap();
            assert_eq!(invite.encode().unwrap(), wire);
            invite
        })
        .collect();
    assert_eq!(
        invites,
        [
            ChatInvite {
                chat_id: PRIVATE_CHAT,
                user_ids: vec![PEER],
                message: Vec::new(),
            },
            ChatInvite {
                chat_id: PRIVATE_CHAT,
                user_ids: vec![PEER, 4],
                message: Vec::new(),
            },
        ]
    );
}

#[test]
fn chat_invitation() {
    let session = peer();
    // Frames 88 and 120.
    let invitations: Vec<_> = all(&session, MessageType::CHAT_INVITATION)
        .into_iter()
        .map(|(_, wire)| {
            let invitation = ChatInvitation::decode(wire).unwrap();
            assert_eq!(invitation.encode().unwrap(), wire);
            invitation
        })
        .collect();
    let first = ChatInvitation {
        chat_id: PRIVATE_CHAT,
        key: KEY,
        user_id: CLIENT,
        unknown_0c: 1,
        unknown_10: 1,
        message: Vec::new(),
        name: b"Private 1".to_vec(),
    };
    assert_eq!(
        invitations,
        [
            first.clone(),
            ChatInvitation {
                unknown_0c: 2,
                ..first
            }
        ]
    );
}

#[test]
fn chat_user_joined() {
    let session = peer();
    // Frame 68: the client joins Public Chat.
    let (_, wire) = one(&session, MessageType::CHAT_USER_JOINED);
    let joined = ChatUserChange::decode(wire).unwrap();
    assert_eq!(
        joined,
        ChatUserChange {
            chat_id: PUBLIC_CHAT,
            user_id: CLIENT,
            unknown_08: 0
        }
    );
    assert_eq!(joined.encode(), wire);
}
