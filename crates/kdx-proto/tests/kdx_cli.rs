//! Checks against `captures/kdx-cli.pcapng`: the `kdx` CLI as `admin`.

use kdx_proto::msg::chat::{ChatUserChange, PUBLIC_CHAT};
use kdx_proto::session::{Header, MessageType};

mod common;

/// The `listener` connection in Public Chat.
fn listener() -> Vec<(Header, Vec<u8>)> {
    common::session(
        include_bytes!("fixtures/kdx-cli-51784.c2s.bin"),
        include_bytes!("fixtures/kdx-cli-51784.s2c.bin"),
    )
}

fn one(messages: &[(Header, Vec<u8>)], kind: MessageType) -> &[u8] {
    let found: Vec<_> = messages.iter().filter(|(h, _)| h.kind == kind).collect();
    assert_eq!(found.len(), 1, "{kind}");
    &found[0].1
}

#[test]
fn chat_user_joined_and_departed() {
    let session = listener();
    // Frame 417.
    let wire = one(&session, MessageType::CHAT_USER_JOINED);
    let joined = ChatUserChange::decode(wire).unwrap();
    assert_eq!(
        joined,
        ChatUserChange {
            chat_id: PUBLIC_CHAT,
            user_id: 0x14,
            unknown_08: 0,
            ..ChatUserChange::default()
        }
    );
    assert_eq!(joined.encode(), wire);

    // Frame 424, after user 0x14 sent 204 Leave Chat.
    let wire = one(&session, MessageType::CHAT_USER_DEPARTED);
    let departed = ChatUserChange::decode(wire).unwrap();
    assert_eq!(
        departed,
        ChatUserChange {
            unknown_08: 1,
            ..joined
        }
    );
    assert_eq!(departed.encode(), wire);
}
