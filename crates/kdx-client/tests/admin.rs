//! Administration requests against the mock.

mod common;

use kdx_client::mock::Step;
use kdx_client::set_server_name;
use kdx_proto::msg::admin::{
    DisconnectMessage, DisconnectUser, ExtraServerInfo, HistoryList, HistoryRecord,
    MonitoredConnection, SendBroadcast, ShutdownServer, YourAddress,
};
use kdx_proto::msg::chat::ChatRecord;
use kdx_proto::msg::server_info::ServerSettings;
use kdx_proto::msg::{FilePath, Status};
use kdx_proto::session::MessageType;

fn settings() -> ServerSettings {
    let mut s = ServerSettings::default();
    s.main.flags = 2;
    s.chats = vec![ChatRecord {
        chat_id: 2,
        name: b"Lounge".to_vec(),
        ..ChatRecord::default()
    }];
    s
}

#[test]
fn set_server_name_rewrites_only_section_0_and_keeps_the_rest() {
    let before = settings();
    let mut s = before.clone();
    set_server_name(&mut s, b"Renamed Lab").unwrap();
    assert_eq!(s.main.name, b"Renamed Lab");
    let bytes = s.encode().unwrap();
    assert_eq!(
        bytes[0x70 + 0x52],
        11,
        "the length byte at 0x52 follows the name"
    );
    assert_eq!(
        s.sections()[ServerSettings::CHATS].len,
        before.sections()[ServerSettings::CHATS].len
    );
    assert_eq!(s.chats, before.chats);
    assert!(s.greeting_enabled());
    assert!(set_server_name(&mut s, b"").is_err());
    assert!(set_server_name(&mut s, b"a\0b").is_err());
    assert!(set_server_name(&mut s, &[b'x'; 256]).is_err());
}

#[tokio::test]
async fn server_settings_round_trip_and_set() {
    let (server, mut client) = common::client_with(vec![
        Step::Expect(MessageType::GET_SERVER_SETTINGS),
        Step::Reply(
            MessageType::SERVER_SETTINGS_INFO,
            settings().encode().unwrap(),
        ),
        Step::Expect(MessageType::SET_SERVER_SETTINGS),
        Step::Reply(MessageType::STATUS, Status::Success { value: 172 }.encode()),
    ])
    .await;
    let mut s = client.server_settings().await.unwrap();
    assert_eq!(s.main.name, b"New Server");
    set_server_name(&mut s, b"Lab").unwrap();
    client.set_server_settings(&s).await.unwrap();
    drop(client);
    server.finish().await;
    let sent =
        ServerSettings::decode(&server.payloads(MessageType::SET_SERVER_SETTINGS)[0]).unwrap();
    assert_eq!(sent.main.name, b"Lab");
    assert_eq!(sent.chats[0].name, b"Lounge");
}

fn record(nick: &str) -> HistoryRecord {
    HistoryRecord {
        time: 0,
        kind: MessageType::HISTORY_LOGIN,
        user_id: 2,
        ignored_10: [0; 0x32],
        ignored_44: [0; 2],
        ignored_4a: [0; 2],
        address: b"127.0.0.1".to_vec(),
        nick: nick.as_bytes().to_vec(),
        login: Vec::new(),
        detail: Vec::new(),
        path: FilePath::default(),
        padding: Vec::new(),
    }
}

#[tokio::test]
async fn history_join_returns_list_then_events() {
    let list = HistoryList {
        ignored: [0; 4],
        records: vec![record("bob")],
    };
    let (server, mut client) = common::client_with(vec![
        Step::Expect(MessageType::SERVER_HISTORY_JOIN),
        Step::Reply(
            MessageType::SERVER_HISTORY_JOIN_REPLY,
            list.encode().unwrap(),
        ),
        Step::Send(
            MessageType::SERVER_HISTORY_ADDED,
            record("carol").encode_added().unwrap(),
        ),
        Step::Expect(MessageType::SERVER_HISTORY_LEAVE),
    ])
    .await;
    let list = client.history_join().await.unwrap();
    assert_eq!(list.records[0].nick, b"bob");
    match client.next_notice().await.unwrap() {
        kdx_client::Notice::HistoryAdded(added) => {
            assert_eq!(
                (added.kind, &added.nick[..]),
                (MessageType::HISTORY_LOGIN, &b"carol"[..])
            );
        }
        other => panic!("expected HistoryAdded, got {other:?}"),
    }
    client.history_leave().await.unwrap();
    drop(client);
    server.finish().await;
}

#[tokio::test]
async fn monitor_info_round_trip_and_disconnect_user() {
    let conn = MonitoredConnection {
        user_id: 2,
        nick: b"bob".to_vec(),
        ..MonitoredConnection::default()
    };
    let (server, mut client) = common::client_with(vec![
        Step::Expect(MessageType::OPEN_CONNECTION_MONITOR),
        Step::Expect(MessageType::GET_CONNECTION_MONITOR_INFO),
        Step::Reply(
            MessageType::CONNECTION_MONITOR_INFO,
            MonitoredConnection::encode_list(std::slice::from_ref(&conn)).unwrap(),
        ),
        Step::Expect(MessageType::CLOSE_CONNECTION_MONITOR),
        Step::Expect(MessageType::DISCONNECT_USER),
        Step::Reply(MessageType::STATUS, Status::Success { value: 150 }.encode()),
        Step::Expect(MessageType::SEND_BROADCAST_MESSAGE),
        // No reply to 154: the barrier's 115 answers instead.
        Step::Expect(MessageType::GET_MY_ADDRESS),
        Step::Reply(
            MessageType::YOUR_ADDRESS,
            YourAddress(b"a".to_vec()).encode().unwrap(),
        ),
        Step::Expect(MessageType::SHUTDOWN_SERVER),
    ])
    .await;
    client.monitor_open().await.unwrap();
    let info = client.monitor_info().await.unwrap();
    assert_eq!(info[0].nick, b"bob");
    client.monitor_close().await.unwrap();
    client.disconnect_user(2, 10, b"bye", true).await.unwrap();
    client.broadcast(b"hello all").await.unwrap();
    client.shutdown(1, b"going down").await.unwrap();
    drop(client);
    server.finish().await;
    let d = DisconnectUser::decode(&server.payloads(MessageType::DISCONNECT_USER)[0]).unwrap();
    assert_eq!(
        (
            d.user_id,
            d.ban_seconds,
            d.options & DisconnectUser::SILENT,
            &d.message[..]
        ),
        (2, 10, DisconnectUser::SILENT, &b"bye"[..])
    );
    let b =
        SendBroadcast::decode(&server.payloads(MessageType::SEND_BROADCAST_MESSAGE)[0]).unwrap();
    assert_eq!(b.text, b"hello all");
    let s = ShutdownServer::decode(&server.payloads(MessageType::SHUTDOWN_SERVER)[0]).unwrap();
    assert_eq!((s.mode, &s.message[..]), (1, &b"going down"[..]));
}

#[tokio::test]
async fn extra_server_info_and_my_address() {
    let extra = ExtraServerInfo {
        uptime: 42,
        os: b"Linux".to_vec(),
        ..ExtraServerInfo::default()
    };
    let (server, mut client) = common::client_with(vec![
        Step::Expect(MessageType::GET_EXTRA_SERVER_INFO),
        Step::Reply(MessageType::EXTRA_SERVER_INFO, extra.encode().unwrap()),
        Step::Expect(MessageType::GET_MY_ADDRESS),
        Step::Reply(
            MessageType::YOUR_ADDRESS,
            YourAddress(b"10.0.0.7:10700".to_vec()).encode().unwrap(),
        ),
    ])
    .await;
    assert_eq!(client.extra_server_info().await.unwrap().os, b"Linux");
    assert_eq!(client.my_address().await.unwrap(), b"10.0.0.7:10700");
    drop(client);
    server.finish().await;
}

#[tokio::test]
async fn a_disconnect_before_the_close_names_the_reason() {
    let bye = DisconnectMessage {
        user_id: 1,
        message: b"server full".to_vec(),
        name: b"admin".to_vec(),
        ..DisconnectMessage::default()
    };
    let (server, mut client) = common::client_with(vec![
        Step::Expect(MessageType::GET_MY_ADDRESS),
        Step::Send(MessageType::DISCONNECT_MESSAGE, bye.encode().unwrap()),
        Step::Close,
    ])
    .await;
    let err = client.my_address().await.unwrap_err();
    assert!(
        err.to_string().contains("disconnected: server full"),
        "{err:#}"
    );
    drop(client);
    server.finish().await;
}
