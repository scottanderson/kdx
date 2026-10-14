//! Checks against `captures/linux-dcc-voice-failed.pcapng`: two copies of
//! Client 1.620 for Linux, B connecting to A's DCC listener.

use kdx_proto::msg::chat::DccAddress;
use kdx_proto::msg::dcc::Skpk;
use kdx_proto::session::{MessageType, SessionCipher};
use kdx_proto::AccountKey;

mod common;

const PRIMARY_C2S: &[u8] = include_bytes!("fixtures/linux-dcc-voice-failed-45702.c2s.bin");
const PRIMARY_S2C: &[u8] = include_bytes!("fixtures/linux-dcc-voice-failed-45702.s2c.bin");
const SERVER_S2C: &[u8] = include_bytes!("fixtures/linux-dcc-voice-failed-43114.s2c.bin");

/// The 117 SKPK payload on one stream of the primary connection, which logs in with no password.
fn skpk_wire(stream: &[u8]) -> Vec<u8> {
    let cipher = SessionCipher::new(&AccountKey::EMPTY_PASSWORD);
    common::messages(stream, &cipher)
        .into_iter()
        .find(|(header, _)| header.kind == MessageType::SKPK)
        .map(|(_, payload)| payload)
        .unwrap()
}

#[test]
fn skpk() {
    // Frame 22 from B, frame 25 from A.
    for (stream, connection_id) in [(PRIMARY_C2S, 4), (PRIMARY_S2C, 2)] {
        let wire = skpk_wire(stream);
        let skpk = Skpk::decode(&wire).unwrap();
        assert_eq!(skpk.dcc_port, 10600);
        assert_eq!(skpk.connection_id, connection_id);
        assert_eq!(skpk.client_version, 1620);
        assert_eq!(skpk.encode(), wire);
    }
}

#[test]
fn chat_dcc_addresses() {
    // Frames 109 and 155: the server lists A's listener for B. Padding is 0x22.
    let cipher = SessionCipher::new(&AccountKey::EMPTY_PASSWORD);
    let replies: Vec<_> = common::messages(SERVER_S2C, &cipher)
        .into_iter()
        .filter(|(header, _)| header.kind == MessageType::CHAT_DCC_ADDRESSES)
        .collect();
    assert_eq!(replies.len(), 2);
    for (_, wire) in replies {
        let list = DccAddress::decode_list(&wire).unwrap();
        assert_eq!(
            list,
            [DccAddress {
                user_id: 1,
                address: b"127.0.0.1:10600".to_vec(),
                padding: vec![0x22],
                ..Default::default()
            }]
        );
        assert_eq!(DccAddress::encode_list(&list).unwrap(), wire);
    }
}
