//! Checks against `captures/openkdx-chat-basic.pcapng`: Client 1.600 for Windows as `admin` in the OpenKDX lab.

use kdx_proto::msg::chat::ChatText;
use kdx_proto::session::{Header, MessageType};
use kdx_proto::AccountKey;

mod common;

/// The lab's published administrator password.
const PASSWORD: &[u8] = b"openkdx-lab";

fn session() -> Vec<(Header, Vec<u8>)> {
    common::session_with_key(
        include_bytes!("fixtures/openkdx-chat-basic.c2s.bin"),
        include_bytes!("fixtures/openkdx-chat-basic.s2c.bin"),
        &AccountKey::from_password(PASSWORD),
    )
}

fn texts(kind: MessageType) -> Vec<ChatText> {
    session()
        .iter()
        .filter(|(h, _)| h.kind == kind)
        .map(|(_, wire)| ChatText::decode(wire).unwrap())
        .collect()
}

/// Frames 24 and 25: `/me waves` goes out with the action flag and no `/me ` prefix.
#[test]
fn me_action_sets_flag_1_and_strips_the_prefix() {
    let sent = texts(MessageType::SEND_CHAT_TEXT);
    assert_eq!(sent.len(), 2);
    assert_eq!(sent[0].text, b"hello from the lab");
    assert_eq!(sent[0].flags, 0);
    assert!(!sent[0].is_action());
    assert_eq!(sent[1].text, b"waves");
    assert_eq!(sent[1].flags, ChatText::FLAG_ACTION);
    assert!(sent[1].is_action());

    let received = texts(MessageType::RECEIVE_CHAT_TEXT);
    assert_eq!(received.len(), 2);
    assert_eq!(received[1].flags, ChatText::FLAG_ACTION);
    assert_eq!(received[1].text, b"waves");
}
