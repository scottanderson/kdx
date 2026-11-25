//! Chat requests against the mock.

mod common;

use kdx_client::mock::Step;
use kdx_proto::msg::chat::{
    ChatInvite, ChatListEntry, ChatRecord, ChatText, ChatWindowInfo, SetChatTopic,
};
use kdx_proto::msg::Status;
use kdx_proto::session::MessageType;

#[tokio::test]
async fn say_sets_action_flag_and_strips_nothing() {
    let (server, mut client) = common::client_with(vec![
        Step::Expect(MessageType::SEND_CHAT_TEXT),
        Step::Expect(MessageType::SEND_CHAT_TEXT),
    ])
    .await;
    client.say(1, b"hello", false).await.unwrap();
    client.say(1, b"waves", true).await.unwrap();
    drop(client);
    server.finish().await;
    let sent: Vec<ChatText> = server
        .payloads(MessageType::SEND_CHAT_TEXT)
        .iter()
        .map(|p| ChatText::decode(p).unwrap())
        .collect();
    assert_eq!(sent.len(), 2);
    assert_eq!(
        (sent[0].flags, &sent[0].text[..], sent[0].user_id),
        (0, &b"hello"[..], 0)
    );
    assert_eq!(
        (sent[1].flags, &sent[1].text[..]),
        (ChatText::FLAG_ACTION, &b"waves"[..])
    );
}

#[tokio::test]
async fn say_rejects_text_over_the_limit() {
    let (server, mut client) = common::client_with(vec![]).await;
    let err = client
        .say(1, &vec![b'x'; ChatText::MAX_TEXT_LEN + 1], false)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("chat text"), "{err:?}");
    drop(client);
    server.finish().await;
}

#[tokio::test]
async fn join_returns_window_info() {
    let info = ChatWindowInfo {
        chat_id: 1,
        members: vec![1, 2],
        name: b"Public Chat".to_vec(),
        topic: b"hi".to_vec(),
        ..ChatWindowInfo::default()
    };
    let (server, mut client) = common::client_with(vec![
        Step::Expect(MessageType::JOIN_CHAT),
        Step::Reply(MessageType::CHAT_WINDOW_INFO, info.encode().unwrap()),
    ])
    .await;
    assert_eq!(client.join_chat(1, 0).await.unwrap(), info);
    drop(client);
    server.finish().await;
}

#[tokio::test]
async fn send_pm_rejects_empty() {
    let (server, mut client) = common::client_with(vec![]).await;
    let err = client.send_private_message(2, b"").await.unwrap_err();
    assert!(err.to_string().contains("empty"), "{err:?}");
    drop(client);
    server.finish().await;
}

#[tokio::test]
async fn chat_list_round_trip() {
    let entry = ChatListEntry {
        chat_id: 1,
        member_count: 1,
        name: b"Public Chat".to_vec(),
        padding: vec![0; 3],
        ..ChatListEntry::default()
    };
    let (server, mut client) = common::client_with(vec![
        Step::Expect(MessageType::GET_CHAT_LIST),
        Step::Reply(
            MessageType::CHAT_LIST,
            ChatListEntry::encode_list(&[entry]).unwrap(),
        ),
    ])
    .await;
    assert_eq!(client.chat_list().await.unwrap()[0].chat_id, 1);
    drop(client);
    server.finish().await;
}

#[tokio::test]
async fn topic_invite_create_and_delete() {
    let room = ChatRecord {
        name: b"Lounge".to_vec(),
        join_classes: u32::MAX,
        ..ChatRecord::default()
    };
    let created = ChatRecord {
        chat_id: 7,
        key: 0x1234,
        ..room.clone()
    };
    let (server, mut client) = common::client_with(vec![
        Step::Expect(MessageType::SET_CHAT_TOPIC),
        Step::Expect(MessageType::CHAT_INVITE),
        Step::Expect(MessageType::CREATE_CHAT),
        Step::Reply(MessageType::CHAT_INFO, created.encode().unwrap()),
        Step::Expect(MessageType::DELETE_CHAT),
        Step::Reply(
            MessageType::STATUS,
            Status::Success { value: 0xc9 }.encode(),
        ),
    ])
    .await;
    client.set_topic(1, b"new topic").await.unwrap();
    client.invite(1, &[2, 3], b"come").await.unwrap();
    assert_eq!(client.create_chat(&room).await.unwrap(), created);
    client.delete_chat(7).await.unwrap();
    let err = client.invite(1, &[0; 21], b"").await.unwrap_err();
    assert!(err.to_string().contains("invitees"), "{err:?}");
    drop(client);
    server.finish().await;

    let topic = SetChatTopic::decode(&server.payloads(MessageType::SET_CHAT_TOPIC)[0]).unwrap();
    assert_eq!((topic.chat_id, &topic.topic[..]), (1, &b"new topic"[..]));
    let invite = ChatInvite::decode(&server.payloads(MessageType::CHAT_INVITE)[0]).unwrap();
    assert_eq!(
        (invite.user_ids, &invite.message[..]),
        (vec![2, 3], &b"come"[..])
    );
    assert_eq!(
        server.payloads(MessageType::DELETE_CHAT)[0],
        7u32.to_be_bytes()
    );
}
