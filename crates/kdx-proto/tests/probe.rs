//! Checks against `captures/probe.pcapng`: kdx-probe sessions as `admin` on Server 1.620.

use kdx_proto::msg::chat::{
    ChatListEntry, ChatRecord, ChatText, ChatTopicChanged, ChatWindowInfo, JoinChat, SetChatTopic,
    PUBLIC_CHAT,
};
use kdx_proto::msg::file::{FileInfo, FileListEntry, ATTR_FOLDER};
use kdx_proto::msg::{FilePath, UserInfo};
use kdx_proto::session::{Header, MessageType};

mod common;

macro_rules! connection {
    ($port:literal) => {
        common::session(
            include_bytes!(concat!("fixtures/probe-", $port, ".c2s.bin")),
            include_bytes!(concat!("fixtures/probe-", $port, ".s2c.bin")),
        )
    };
}

/// The payloads of every message of one type.
fn payloads(messages: &[(Header, Vec<u8>)], kind: MessageType) -> Vec<&[u8]> {
    let found: Vec<&[u8]> = messages
        .iter()
        .filter(|(h, _)| h.kind == kind)
        .map(|(_, p)| p.as_slice())
        .collect();
    assert!(!found.is_empty(), "no {kind}");
    found
}

fn one(messages: &[(Header, Vec<u8>)], kind: MessageType) -> &[u8] {
    let found = payloads(messages, kind);
    assert_eq!(found.len(), 1, "{kind}");
    found[0]
}

#[test]
fn chat() {
    let session = connection!("35844");

    let join = one(&session, MessageType::JOIN_CHAT);
    assert_eq!(JoinChat::decode(join).unwrap().chat_id, PUBLIC_CHAT);
    assert_eq!(JoinChat::decode(join).unwrap().encode(), join);

    let window = one(&session, MessageType::CHAT_WINDOW_INFO);
    let info = ChatWindowInfo::decode(window).unwrap();
    assert_eq!(info.name(), b"Public Chat");
    assert_eq!(info.members, [3]);
    assert_eq!(info.encode().unwrap(), window);

    for kind in [MessageType::SEND_CHAT_TEXT, MessageType::RECEIVE_CHAT_TEXT] {
        let wire = one(&session, kind);
        let text = ChatText::decode(wire).unwrap();
        assert_eq!(text.text, b"Hello");
        assert_eq!(text.encode().unwrap(), wire);
    }
    let received = ChatText::decode(one(&session, MessageType::RECEIVE_CHAT_TEXT)).unwrap();
    assert_eq!(received.user_id, 3);

    // The probe put the topic at 0x10, so the server read four zeros and "H".
    let set = one(&session, MessageType::SET_CHAT_TOPIC);
    assert_eq!(SetChatTopic::decode(set).unwrap().topic, b"\0\0\0\0H");
    let changed = one(&session, MessageType::CHAT_TOPIC_CHANGED);
    let topic = ChatTopicChanged::decode(changed).unwrap();
    assert_eq!(topic.topic, b"\0\0\0\0H");
    assert_eq!(topic.nick, b"probe");
    assert_eq!(topic.user_id, 3);
    assert_eq!(topic.encode().unwrap(), changed);

    let record = one(&session, MessageType::CHAT_INFO);
    let chat = ChatRecord::decode(record).unwrap();
    assert_eq!(chat.name, b"Public Chat");
    assert_eq!(chat.description, b"For all users");
    assert_eq!(chat.join_classes, u32::MAX);
    assert_eq!(chat.encode().unwrap(), record);
}

#[test]
fn chat_list() {
    let session = connection!("50770");
    let wire = one(&session, MessageType::CHAT_LIST);
    let list = ChatListEntry::decode_list(wire).unwrap();
    let names: Vec<&[u8]> = list.iter().map(|c| c.name.as_slice()).collect();
    assert_eq!(names, [&b"Public Chat"[..], b"Voice Chat", b"Admin Lounge"]);
    assert_eq!(list[1].flags, 2);
    assert_eq!(ChatListEntry::encode_list(&list).unwrap(), wire);
}

#[test]
fn files() {
    for session in [connection!("32980"), connection!("54328")] {
        for wire in payloads(&session, MessageType::FILE_LIST) {
            let list = FileListEntry::decode_list(wire).unwrap();
            let mut expected = wire.to_vec();
            if let [first, second] = &list[..] {
                assert_eq!(first.name, b"Admin Dropbox [DB]");
                assert_eq!(second.name, b"Uploads [UL]");
                assert!(list
                    .iter()
                    .all(|e| e.attributes == ATTR_FOLDER && e.is_folder()));
                assert_eq!((first.access_item, second.access_item), (0x2c, 0x2b));
                // The server leaves the padding after the first name uninitialised.
                let pad = 4 + FileListEntry::HEADER_LEN + first.name.len();
                expected[pad..pad + 2].fill(0);
            } else {
                assert!(list.is_empty());
            }
            assert_eq!(FileListEntry::encode_list(&list).unwrap(), expected);
        }
    }

    let session = connection!("54328");
    let request = one(&session, MessageType::GET_FILE_FOLDER_INFO);
    assert_eq!(
        FilePath::decode(request).unwrap(),
        FilePath::new(["Uploads [UL]"])
    );
    let wire = one(&session, MessageType::FILE_FOLDER_INFO);
    let info = FileInfo::decode(wire).unwrap();
    assert_eq!(info.name, b"Uploads [UL]");
    assert_eq!(info.kind, b"folder");
    assert_eq!(info.encode().unwrap(), wire);
}

#[test]
fn user_info() {
    let session = connection!("50984");
    let wire = one(&session, MessageType::USER_INFO);
    let info = UserInfo::decode(wire).unwrap();
    assert_eq!(info.address, b"127.0.0.1:50984");
    assert_eq!(info.nick, b"probe");
    assert_eq!(info.login, b"admin");
    assert_eq!(info.name_3f, b"Administrator");
    assert_eq!(info.encode().unwrap(), wire);
}
