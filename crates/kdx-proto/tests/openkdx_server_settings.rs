//! Checks against `captures/openkdx-server-settings.pcapng`: the OpenKDX client reads and renames.

use kdx_proto::msg::server_info::ServerSettings;
use kdx_proto::session::{Header, MessageType};
use kdx_proto::AccountKey;

mod common;

fn session() -> Vec<(Header, Vec<u8>)> {
    common::session_with_key(
        include_bytes!("fixtures/openkdx-server-settings-43622.c2s.bin"),
        include_bytes!("fixtures/openkdx-server-settings-43622.s2c.bin"),
        &AccountKey::from_password(b"openkdx-lab"),
    )
}

fn all(kind: MessageType) -> Vec<Vec<u8>> {
    session()
        .into_iter()
        .filter(|(h, _)| h.kind == kind)
        .map(|(_, p)| p)
        .collect()
}

/// Frames 17 and 19: the typed sections of a fresh server, re-encoded byte for byte.
#[test]
fn settings_171_decode_typed_and_reencode_byte_exact() {
    let infos = all(MessageType::SERVER_SETTINGS_INFO);
    assert!(infos.len() >= 2);
    let settings = ServerSettings::decode(&infos[0]).unwrap();
    assert_eq!(settings.encode(), infos[0]);
    assert_eq!(settings.main.name, b"New Server");
    assert_eq!(settings.main.port, 10700);
    assert_eq!(settings.name(), b"New Server");
    assert_eq!(settings.trackers, []);
    assert_eq!(settings.allow_deny, []);
    assert_eq!(settings.greeting, None);
    assert_eq!(settings.icon, None);
    // Server 1.620 sends no chat rooms in 171; the file has them (`kdx-legacy`).
    assert!(settings.chats.is_empty());
    assert!(settings.placeholders[ServerSettings::CHATS]);
    assert_eq!(settings.sections()[ServerSettings::MAIN].offset, 0x70);
    assert_eq!(infos[0].len(), ServerSettings::HEADER_LEN + 0x68);
}

/// Frame 20: the OpenKDX client's 172 echoes the 171 with the new name.
#[test]
fn set_172_reencodes_with_the_new_name() {
    let sets = all(MessageType::SET_SERVER_SETTINGS);
    let settings = ServerSettings::decode(&sets[0]).unwrap();
    assert_eq!(settings.encode(), sets[0]);
    assert_eq!(settings.main.name, b"OpenKDX Lab");
}

/// Frame 23: the 171 after the rename carries the new name and its length byte.
#[test]
fn name_length_byte_0x52_is_the_name_length() {
    let infos = all(MessageType::SERVER_SETTINGS_INFO);
    let after = ServerSettings::decode(&infos[2]).unwrap();
    assert_eq!(after.main.name, b"OpenKDX Lab");
    assert_eq!(
        after.section(ServerSettings::MAIN)[0x52],
        after.main.name.len() as u8
    );
    assert_eq!(after.encode(), infos[2]);
}
