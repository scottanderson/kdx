//! Server pushes decode to typed notices; unknown types stay raw.

mod common;

use kdx_client::mock::Step;
use kdx_client::Notice;
use kdx_proto::msg::admin::{DisconnectMessage, RcvBroadcast};
use kdx_proto::msg::chat::ChatText;
use kdx_proto::msg::PrivateMessage;
use kdx_proto::session::MessageType;

#[tokio::test]
async fn notices_decode_pushes_and_keep_unknown_as_other() {
    let chat = ChatText {
        chat_id: 1,
        user_id: 2,
        flags: ChatText::FLAG_ACTION,
        text: b"waves".to_vec(),
        ..ChatText::default()
    };
    let pm = PrivateMessage {
        user_id: 2,
        text: b"psst".to_vec(),
        ..PrivateMessage::default()
    };
    let broadcast = RcvBroadcast {
        user_id: 0,
        text: b"restart soon".to_vec(),
        ..RcvBroadcast::default()
    };
    let bye = DisconnectMessage {
        user_id: 1,
        flags: DisconnectMessage::BANNED,
        message: b"go away".to_vec(),
        name: b"admin".to_vec(),
        ..DisconnectMessage::default()
    };
    let (server, mut client) = common::client_with(vec![
        Step::Send(MessageType::RECEIVE_CHAT_TEXT, chat.encode().unwrap()),
        Step::Send(MessageType::INCOMING_PRIVATE_MESSAGE, pm.encode()),
        Step::Send(MessageType::RCV_BROADCAST_MESSAGE, broadcast.encode()),
        Step::Send(MessageType::DISCONNECT_MESSAGE, bye.encode().unwrap()),
        Step::Send(MessageType(9999), vec![1, 2, 3]),
    ])
    .await;

    assert_eq!(client.next_notice().await.unwrap(), Notice::ChatText(chat));
    assert_eq!(
        client.next_notice().await.unwrap(),
        Notice::PrivateMessage(pm)
    );
    assert_eq!(
        client.next_notice().await.unwrap(),
        Notice::Broadcast(broadcast)
    );
    assert_eq!(client.next_notice().await.unwrap(), Notice::Disconnect(bye));
    match client.next_notice().await.unwrap() {
        Notice::Other(header, payload) => {
            assert_eq!(header.kind, MessageType(9999));
            assert_eq!(payload, [1, 2, 3]);
        }
        other => panic!("expected Other, got {other:?}"),
    }
    drop(client);
    server.finish().await;
}

#[test]
fn a_malformed_push_is_other() {
    let header = kdx_proto::session::Header {
        txid: 1,
        kind: MessageType::RECEIVE_CHAT_TEXT,
        flags: 0,
        len: 2,
    };
    assert!(Notice::decode(&header, &[0, 1]).is_err());
}
