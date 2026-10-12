//! Checks against `captures/linux-chat-rooms.pcapng`: Client 1.620 for Linux as `admin`.

use kdx_proto::msg::chat::{ChatDeleted, ChatId, ChatRecord, ChatUserChange, PUBLIC_CHAT};
use kdx_proto::msg::Status;
use kdx_proto::session::{Header, MessageType};

mod common;

/// The chat ID of `lab-room`.
const LAB_ROOM: u32 = 0xc217_598c;
/// The chat ID of `lab-chat`.
const LAB_CHAT: u32 = 0xe8df_4d29;
/// The client's user ID.
const CLIENT: u32 = 2;

/// The GUI client's connection.
fn client() -> Vec<(Header, Vec<u8>)> {
    common::session(
        include_bytes!("fixtures/linux-chat-rooms-43718.c2s.bin"),
        include_bytes!("fixtures/linux-chat-rooms-43718.s2c.bin"),
    )
}

/// The `cli-member` peer in `lab-chat`.
fn member() -> Vec<(Header, Vec<u8>)> {
    common::session(
        include_bytes!("fixtures/linux-chat-rooms-55916.c2s.bin"),
        include_bytes!("fixtures/linux-chat-rooms-55916.s2c.bin"),
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

/// The 1 Status that answers `request`.
fn status(messages: &[(Header, Vec<u8>)], request: Header) -> Status {
    let (_, wire) = all(messages, MessageType::STATUS)
        .into_iter()
        .find(|(h, _)| h.txid == request.txid)
        .unwrap();
    Status::decode(wire).unwrap()
}

/// Decodes each chat ID request of `kind` and checks it re-encodes.
fn chat_ids(messages: &[(Header, Vec<u8>)], kind: MessageType) -> Vec<u32> {
    all(messages, kind)
        .into_iter()
        .map(|(_, wire)| {
            let id = ChatId::decode(wire).unwrap();
            assert_eq!(id.encode(), wire);
            id.0
        })
        .collect()
}

/// Decodes each chat record of `kind`, checks it re-encodes, and checks the
/// success value of its 1 Status if `status_value` is set.
fn records(
    messages: &[(Header, Vec<u8>)],
    kind: MessageType,
    status_value: Option<u32>,
) -> Vec<ChatRecord> {
    all(messages, kind)
        .into_iter()
        .map(|(header, wire)| {
            let record = ChatRecord::decode(wire).unwrap();
            assert_eq!(record.encode().unwrap(), wire);
            if let Some(value) = status_value {
                assert_eq!(status(messages, header), Status::Success { value });
            }
            record
        })
        .collect()
}

#[test]
fn create_chat() {
    let session = client();
    // Frames 42 and 85, answered at 45 and 88.
    let created = records(&session, MessageType::CREATE_CHAT, Some(217));
    let [room, chat] = &created[..] else {
        panic!("{created:?}")
    };
    assert_eq!(room.chat_id, 0);
    assert_eq!(room.name, b"lab-room");
    assert_eq!(room.description, b"lab");
    assert_eq!(room.flags, 0);
    assert_eq!(room.join_classes, 0x8000_0000);
    assert_eq!(room.send_classes, 0x8000_0000);
    assert_eq!(room.topic_classes, 0x8000_0000);
    assert_eq!(chat.chat_id, 0);
    assert_eq!(chat.name, b"lab-chat");
    assert_eq!(chat.description, b"lab chat");
    assert_eq!(chat.flags, 2);
}

#[test]
fn get_and_set_chat() {
    let session = client();
    // Frames 49, 59, and 136.
    assert_eq!(
        chat_ids(&session, MessageType::GET_CHAT),
        [LAB_ROOM, LAB_ROOM, PUBLIC_CHAT]
    );

    // Frames 50, 60, and 137.
    let infos = records(&session, MessageType::CHAT_INFO, None);
    assert_eq!(infos.len(), 3);
    assert_eq!(infos[0].description, b"lab");
    assert_eq!(infos[1].description, b"lab edited");
    assert_eq!(infos[2].name, b"Public Chat");
    assert_eq!(infos[2].join_classes, u32::MAX);

    // Frames 52 and 62, answered at 55 and 65; the second ticks Voice Chat.
    let sets = records(&session, MessageType::SET_CHAT, Some(220));
    assert_eq!(sets.len(), 2);
    assert_eq!(sets[0].chat_id, LAB_ROOM);
    assert_eq!(sets[0], infos[1]);
    assert_eq!(
        sets[1],
        ChatRecord {
            flags: 2,
            ..sets[0].clone()
        }
    );
}

#[test]
fn leave_and_delete_chat() {
    let session = client();
    // Frame 75.
    assert_eq!(chat_ids(&session, MessageType::LEAVE_CHAT), [LAB_ROOM]);

    // Frames 77 and 122, answered at 81 and 129.
    assert_eq!(
        chat_ids(&session, MessageType::DELETE_CHAT),
        [LAB_ROOM, LAB_CHAT]
    );
    for (header, _) in all(&session, MessageType::DELETE_CHAT) {
        assert_eq!(status(&session, header), Status::Success { value: 201 });
    }

    // Frame 127: only `lab-chat` had members, the deleter among them.
    let (_, wire) = one(&session, MessageType::CHAT_DELETED);
    let deleted = ChatDeleted::decode(wire).unwrap();
    assert_eq!(
        deleted,
        ChatDeleted {
            chat_id: LAB_CHAT,
            user_id: CLIENT
        }
    );
    assert_eq!(deleted.encode(), wire);
}

#[test]
fn get_chat_dcc_addresses() {
    let session = client();
    // Frames 72 and 119.
    assert_eq!(
        chat_ids(&session, MessageType::GET_CHAT_DCC_ADDRESSES),
        [LAB_ROOM, LAB_CHAT]
    );
    // Frames 73 and 120: no other member had a DCC port.
    let replies = all(&session, MessageType::CHAT_DCC_ADDRESSES);
    assert_eq!(replies.len(), 2);
    assert!(replies.iter().all(|(_, wire)| wire.is_empty()));
}

#[test]
fn member_sees_join_and_delete() {
    let session = member();
    // Frame 117: the client joins `lab-chat`.
    let (_, wire) = one(&session, MessageType::CHAT_USER_JOINED);
    let joined = ChatUserChange::decode(wire).unwrap();
    assert_eq!(
        joined,
        ChatUserChange {
            chat_id: LAB_CHAT,
            user_id: CLIENT,
            unknown_08: 0
        }
    );
    assert_eq!(joined.encode(), wire);

    // Frame 125, the same 221 the deleter got.
    let (_, wire) = one(&session, MessageType::CHAT_DELETED);
    let deleted = ChatDeleted::decode(wire).unwrap();
    assert_eq!(deleted.chat_id, LAB_CHAT);
    assert_eq!(deleted.user_id, CLIENT);
    assert_eq!(deleted.encode(), wire);
}
