//! Checks against `captures/cli-disconnect-ban.pcapng`: the command-line client of the lab.

use kdx_proto::handshake::{Hello, Proof, Status};
use kdx_proto::msg::admin::DisconnectMessage;
use kdx_proto::session::{Header, MessageType};
use kdx_proto::AccountKey;

mod common;

/// Frame 9: the server's proof to a wrong password has status 1 and only the auth word set.
#[test]
fn wrong_password_gets_proof_status_1_and_auth_word() {
    let c2s = include_bytes!("fixtures/cli-disconnect-ban-54320.c2s.bin");
    let s2c = include_bytes!("fixtures/cli-disconnect-ban-54320.s2c.bin");
    let (client_hello, hello_len) = Hello::decode(c2s).unwrap();
    let (_, server_hello_len) = Hello::decode(s2c).unwrap();
    let proof = Proof::decode(&s2c[server_hello_len..]).unwrap();
    assert_eq!(proof.status, Status::Failed);
    assert_eq!(Status::Failed.to_byte(), 1);
    assert_eq!(Status::from_byte(1), Status::Failed);
    let nonce = client_hello.nonce;
    let word = u32::from_le_bytes(nonce[4..8].try_into().unwrap()) ^ 0x201f_7790;
    assert_eq!(&proof.auth[..4], &word.to_be_bytes());
    assert!(proof.auth[4..].iter().all(|&b| b == 0));
    assert!(proof.response.iter().all(|&b| b == 0));
    assert_eq!(
        s2c.len(),
        server_hello_len + Proof::LEN,
        "the server closes after the proof"
    );
    assert_eq!(c2s.len(), hello_len + Proof::LEN);
}

/// Frame 64: the victim, a guest, gets a 151 with the banned flag before the close.
#[test]
fn ban_message_carries_flag_2() {
    let session = common::session_with_key(
        include_bytes!("fixtures/cli-disconnect-ban-54332.c2s.bin"),
        include_bytes!("fixtures/cli-disconnect-ban-54332.s2c.bin"),
        &AccountKey::EMPTY_PASSWORD,
    );
    let (_, wire): &(Header, Vec<u8>) = session
        .iter()
        .find(|(h, _)| h.kind == MessageType::DISCONNECT_MESSAGE)
        .expect("a 151");
    let message = DisconnectMessage::decode(wire).unwrap();
    assert_eq!(message.flags, DisconnectMessage::BANNED);
}
