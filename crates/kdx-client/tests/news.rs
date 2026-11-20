//! News requests against the mock.

mod common;

use kdx_client::mock::Step;
use kdx_client::to_cr_lines;
use kdx_proto::msg::news::{
    NewsMessage, NewsMessageChange, NewsMessages, NewsgroupEntry, NewsgroupInfo, PostNewsMessage,
};
use kdx_proto::msg::Status;
use kdx_proto::session::MessageType;

fn barrier() -> [Step; 2] {
    [
        Step::Expect(MessageType::GET_MY_ADDRESS),
        Step::Reply(MessageType::YOUR_ADDRESS, vec![127, 0, 0, 1]),
    ]
}

#[tokio::test]
async fn newsgroups_round_trip() {
    let group = NewsgroupEntry {
        group_id: 1,
        message_count: 2,
        name: b"General News".to_vec(),
        padding: vec![0; 3],
        ..NewsgroupEntry::default()
    };
    let (server, mut client) = common::client_with(vec![
        Step::Expect(MessageType::GET_NEWSGROUP_LIST),
        Step::Reply(
            MessageType::NEWSGROUP_LIST,
            NewsgroupEntry::encode_list(&[group]).unwrap(),
        ),
    ])
    .await;
    assert_eq!(client.newsgroups().await.unwrap()[0].name, b"General News");
    drop(client);
    server.finish().await;
}

#[tokio::test]
async fn post_waits_for_613_and_returns_the_message_id() {
    let other = NewsMessageChange {
        group_id: 7,
        message_id: 0,
        message_count: 1,
    };
    let ours = NewsMessageChange {
        group_id: 1,
        message_id: 42,
        message_count: 3,
    };
    let mut steps = vec![
        Step::Expect(MessageType::POST_NEWS_MESSAGE),
        Step::Send(MessageType::NEWS_MESSAGE_POSTED, other.encode().to_vec()),
        Step::Send(MessageType::NEWS_MESSAGE_POSTED, ours.encode().to_vec()),
    ];
    steps.extend(barrier());
    let (server, mut client) = common::client_with(steps).await;
    let id = client
        .post_news_id(1, b"subject", b"line one\rline two")
        .await
        .unwrap();
    assert_eq!(id, 42);
    // The other group's 613 is not ours and stays queued.
    assert_eq!(
        client.next_event().await.unwrap().0.kind,
        MessageType::NEWS_MESSAGE_POSTED
    );
    drop(client);
    server.finish().await;
    let sent =
        PostNewsMessage::decode(&server.payloads(MessageType::POST_NEWS_MESSAGE)[0]).unwrap();
    assert_eq!(
        (sent.group_id, &sent.subject[..], &sent.body[..]),
        (1, &b"subject"[..], &b"line one\rline two"[..])
    );
}

#[tokio::test]
async fn post_converts_newlines_to_cr() {
    let mut steps = vec![Step::Expect(MessageType::POST_NEWS_MESSAGE)];
    steps.extend(barrier());
    let (server, mut client) = common::client_with(steps).await;
    client.post_news(1, b"s", b"a\nb\r\nc").await.unwrap();
    drop(client);
    server.finish().await;
    let sent =
        PostNewsMessage::decode(&server.payloads(MessageType::POST_NEWS_MESSAGE)[0]).unwrap();
    assert_eq!(sent.body, b"a\rb\rc");
    assert_eq!(to_cr_lines(b"x\r\n\ny"), b"x\r\ry");
}

#[tokio::test]
async fn post_without_613_returns_zero() {
    let mut steps = vec![Step::Expect(MessageType::POST_NEWS_MESSAGE)];
    steps.extend(barrier());
    let (server, mut client) = common::client_with(steps).await;
    assert_eq!(client.post_news_id(1, b"s", b"b").await.unwrap(), 0);
    drop(client);
    server.finish().await;
}

#[tokio::test]
async fn news_messages_and_one_message() {
    let msg = NewsMessage {
        group_id: 1,
        message_id: 5,
        poster: b"admin".to_vec(),
        subject: b"hi".to_vec(),
        body: b"body".to_vec(),
        ..NewsMessage::default()
    };
    let list = NewsMessages {
        group_id: 1,
        permissions: 7,
        messages: vec![msg.clone()],
        ..NewsMessages::default()
    };
    let (server, mut client) = common::client_with(vec![
        Step::Expect(MessageType::GET_NEWS_MESSAGES),
        Step::Reply(MessageType::NEWS_MESSAGES, list.encode().unwrap()),
        Step::Expect(MessageType::GET_NEWS_MESSAGE),
        Step::Reply(MessageType::NEWS_MESSAGE, msg.encode().unwrap()),
    ])
    .await;
    assert_eq!(client.news(1).await.unwrap().messages[0].subject, b"hi");
    assert_eq!(client.news_message(1, 5).await.unwrap().body, b"body");
    drop(client);
    server.finish().await;
}

#[tokio::test]
async fn newsgroup_create_set_and_delete() {
    let group = NewsgroupInfo {
        name: b"Lab".to_vec(),
        read_classes: u32::MAX,
        post_classes: u32::MAX,
        ..NewsgroupInfo::default()
    };
    let made = NewsgroupInfo {
        group_id: 3,
        ..group.clone()
    };
    let (server, mut client) = common::client_with(vec![
        Step::Expect(MessageType::GET_NEWSGROUP),
        Step::Reply(MessageType::NEWSGROUP_INFO, made.encode().unwrap()),
        // Server 1.620 answers 600 and 604 with a success Status.
        Step::Expect(MessageType::CREATE_NEWSGROUP),
        Step::Reply(
            MessageType::STATUS,
            Status::Success { value: 0x258 }.encode(),
        ),
        Step::Expect(MessageType::SET_NEWSGROUP),
        Step::Reply(MessageType::NEWSGROUP_INFO, made.encode().unwrap()),
        Step::Expect(MessageType::DELETE_NEWSGROUP),
        Step::Reply(
            MessageType::STATUS,
            Status::Success { value: 0x259 }.encode(),
        ),
    ])
    .await;
    assert_eq!(client.newsgroup(3).await.unwrap(), made);
    assert_eq!(client.create_newsgroup(&group).await.unwrap(), None);
    assert_eq!(
        client.set_newsgroup(&made).await.unwrap(),
        Some(made.clone())
    );
    client.delete_newsgroup(3).await.unwrap();
    drop(client);
    server.finish().await;
    assert_eq!(
        server.payloads(MessageType::GET_NEWSGROUP)[0],
        3u32.to_be_bytes()
    );
    assert_eq!(
        server.payloads(MessageType::DELETE_NEWSGROUP)[0],
        3u32.to_be_bytes()
    );
}
