//! Checks against `captures/linux-chat-admin.pcapng`: Client 1.620 for Linux as `admin`.

use kdx_proto::msg::admin::{
    DisconnectMessage, DisconnectUser, HistoryList, RcvBroadcast, SendBroadcast,
};
use kdx_proto::msg::chat::{
    ChatInvitation, ChatInvite, ChatUserChange, ChatWindowInfo, PUBLIC_CHAT,
};
use kdx_proto::msg::user::UserDeparted;
use kdx_proto::msg::user_info::GetUserInfo;
use kdx_proto::msg::UserInfo;
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
    assert_eq!(chat.name, b"Private 1");
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
                ..ChatInvite::default()
            },
            ChatInvite {
                chat_id: PRIVATE_CHAT,
                user_ids: vec![PEER, 4],
                message: Vec::new(),
                ..ChatInvite::default()
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
        ignored_0c: 1,
        ignored_10: 1,
        message: Vec::new(),
        name: b"Private 1".to_vec(),
        ..ChatInvitation::default()
    };
    assert_eq!(
        invitations,
        [
            first.clone(),
            ChatInvitation {
                ignored_0c: 2,
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
            ignored_08: 0,
            ..ChatUserChange::default()
        }
    );
    assert_eq!(joined.encode(), wire);
}

#[test]
fn user_departed() {
    // Frames 59 and 116: cli-peer2 left twice, as users 3 and 4.
    let session = client();
    let departed: Vec<_> = all(&session, MessageType::USER_DEPARTED)
        .into_iter()
        .map(|(_, wire)| {
            let departed = UserDeparted::decode(wire).unwrap();
            assert_eq!(departed.encode(), wire);
            departed
        })
        .collect();
    assert_eq!(
        departed,
        [3, 4].map(|user_id| UserDeparted {
            user_id,
            value_4: 7,
            ..UserDeparted::default()
        })
    );
}

#[test]
fn get_user_info() {
    // Frames 122/124 and 126/127.
    let session = client();
    let requests = all(&session, MessageType::GET_USER_INFO);
    assert_eq!(requests.len(), 2);
    for (header, wire) in requests {
        let request = GetUserInfo::decode(wire).unwrap();
        assert_eq!(request, GetUserInfo { user_id: CLIENT });
        assert_eq!(request.encode(), wire);
        let (_, reply) = all(&session, MessageType::USER_INFO)
            .into_iter()
            .find(|(h, _)| h.txid == header.txid)
            .unwrap();
        let info = UserInfo::decode(reply).unwrap();
        assert_eq!(info.user_id, CLIENT);
        assert_eq!(info.nick, b"kdx-lab-linux");
    }
}

#[test]
fn broadcast() {
    let session = client();
    // Frame 129.
    let (_, wire) = one(&session, MessageType::SEND_BROADCAST_MESSAGE);
    let sent = SendBroadcast::decode(wire).unwrap();
    assert_eq!(
        sent,
        SendBroadcast {
            value_0: 0,
            value_2: 0,
            text: b"test broadcast".to_vec()
        }
    );
    assert_eq!(sent.encode().unwrap(), wire);

    // Frame 132: back to the sender, too.
    let (_, wire) = one(&session, MessageType::RCV_BROADCAST_MESSAGE);
    let received = RcvBroadcast::decode(wire).unwrap();
    assert_eq!(
        received,
        RcvBroadcast {
            user_id: CLIENT,
            value_0e: sent.value_2,
            text: sent.text,
            ..RcvBroadcast::default()
        }
    );
    assert_eq!(received.encode(), wire);
}

#[test]
fn disconnect_user() {
    let session = client();
    // Frame 159: the GUI disconnects itself, with no ban.
    let (_, wire) = one(&session, MessageType::DISCONNECT_USER);
    let request = DisconnectUser::decode(wire).unwrap();
    assert_eq!(
        request,
        DisconnectUser {
            user_id: CLIENT,
            ban_seconds: 0,
            options: 0,
            message: b"test".to_vec(),
            ..DisconnectUser::default()
        }
    );
    assert_eq!(request.encode().unwrap(), wire);

    // Frame 161.
    let (_, wire) = one(&session, MessageType::DISCONNECT_MESSAGE);
    let notice = DisconnectMessage::decode(wire).unwrap();
    assert_eq!(
        notice,
        DisconnectMessage {
            user_id: CLIENT,
            flags: 0,
            value_0e: 0,
            value_10: 0,
            message: b"test".to_vec(),
            name: b"kdx-lab-linux".to_vec(),
            ..DisconnectMessage::default()
        }
    );
    assert_eq!(notice.encode().unwrap(), wire);
}

#[test]
fn server_history() {
    // Frame 135: 0x12 records.
    let session = client();
    let (_, wire) = one(&session, MessageType::SERVER_HISTORY_JOIN_REPLY);
    let list = HistoryList::decode(wire).unwrap();
    let records = &list.records;
    assert_eq!(records.len(), 0x12);
    assert_eq!(records[0].kind, MessageType::HISTORY_SERVER_STARTED);
    assert_eq!(list.encode().unwrap(), wire);
}
