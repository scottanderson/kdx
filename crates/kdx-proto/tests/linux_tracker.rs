//! Checks against `captures/linux-tracker.pcapng`: Server 1.620 registering with Tracker 1.110,
//! and Client 1.620 beta 5 browsing and administering it.

use std::net::{Ipv4Addr, SocketAddrV4};

use kdx_proto::msg::tracker::{
    AdminSettings, GeneralSettings, GetServerList, GroupListEntry, ServerListEntry, SettingsGroup,
    TrackerSettings, TrackerStats,
};
use kdx_proto::msg::Status;
use kdx_proto::registration::{self, Registration};
use kdx_proto::session::{Header, MessageType};
use kdx_proto::AccountKey;

mod common;

const ADMIN_PASSWORD: &[u8] = b"lab-tracker-pw!1";

/// The first guest, port 60698: frames 27 to 61.
fn guest() -> Vec<(Header, Vec<u8>)> {
    common::session_with_key(
        include_bytes!("fixtures/linux-tracker-60698.c2s.bin"),
        include_bytes!("fixtures/linux-tracker-60698.s2c.bin"),
        &AccountKey::EMPTY_PASSWORD,
    )
}

/// The first admin, port 41900: frames 69 to 86.
fn admin() -> Vec<(Header, Vec<u8>)> {
    common::session_with_key(
        include_bytes!("fixtures/linux-tracker-41900.c2s.bin"),
        include_bytes!("fixtures/linux-tracker-41900.s2c.bin"),
        &AccountKey::from_password(ADMIN_PASSWORD),
    )
}

/// The admin who adds the group `Lab`, port 34704: frames 226 to 257.
fn group_edit() -> Vec<(Header, Vec<u8>)> {
    common::session_with_key(
        include_bytes!("fixtures/linux-tracker-34704.c2s.bin"),
        include_bytes!("fixtures/linux-tracker-34704.s2c.bin"),
        &AccountKey::from_password(ADMIN_PASSWORD),
    )
}

fn all(messages: &[(Header, Vec<u8>)], kind: MessageType) -> Vec<&[u8]> {
    messages
        .iter()
        .filter(|(h, _)| h.kind == kind)
        .map(|(_, p)| p.as_slice())
        .collect()
}

fn one(messages: &[(Header, Vec<u8>)], kind: MessageType) -> &[u8] {
    let found = all(messages, kind);
    assert_eq!(found.len(), 1, "{kind}");
    found[0]
}

fn decode_registration(wire: &[u8]) -> Registration {
    let registration = Registration::decode(wire).unwrap();
    assert_eq!(registration.encode().unwrap(), wire);
    registration
}

#[test]
fn registration() {
    let wire = include_bytes!("fixtures/linux-tracker-reg-23.bin");
    let first = decode_registration(wire);
    assert_eq!(first.flags, 0);
    assert_eq!(first.server_port, 0x29cc);
    assert_eq!(first.users, 1);
    assert!(first.string0.is_empty() && first.string4.is_empty());
    assert_eq!(first.login, b"trk-login");
    assert_eq!(first.name, b"Lab Server");
    assert!(first.description.is_empty());
    assert_eq!(first.groups[0], b"Lab, Lab2");
    assert!(first.groups[1..].iter().all(Vec::is_empty));

    // The checksum is stored big-endian.
    let mut plain = wire.to_vec();
    registration::crypt(&mut plain);
    assert_eq!(&plain[..8], b"%KDX\x00\x10\x00\x00");
    assert_eq!(plain[8..12], [0x40, 0x17, 0xb3, 0xa7]);

    // Frame 78 counts the guest; nothing else changes.
    let second = decode_registration(include_bytes!("fixtures/linux-tracker-reg-78.bin"));
    assert_eq!(second.users, 2);
    assert_eq!(Registration { users: 1, ..second }, first);

    // Frame 255, after the entry's Groups became `Lab`.
    let wire = include_bytes!("fixtures/linux-tracker-reg-255.bin");
    assert_eq!(wire.len(), 88);
    let last = decode_registration(wire);
    assert_eq!(last.groups[0], b"Lab");
    assert_eq!(last.server_id, first.server_id);
}

#[test]
fn registration_checksum_byte_order() {
    // The tracker drops a datagram hashed with the placeholder stored big-endian.
    let mut plain = include_bytes!("fixtures/linux-tracker-reg-23.bin").to_vec();
    registration::crypt(&mut plain);
    plain[8..12].copy_from_slice(&[0xcc, 0x1e, 0xd3, 0xca]);
    let sum = registration::checksum(&plain);
    plain[8..12].copy_from_slice(&sum.to_be_bytes());
    registration::crypt(&mut plain);
    assert!(Registration::decode(&plain).is_err());
}

#[test]
fn empty_messages() {
    let sessions = [guest(), admin(), group_edit()];
    let messages = sessions.iter().flatten();
    let mut seen = Vec::new();
    for (header, payload) in messages {
        let kind = header.kind;
        if [700, 710, 713, 720].contains(&kind.0) {
            assert!(payload.is_empty(), "{kind}");
            seen.push(kind.0);
        }
    }
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen, [700, 710, 713, 720]);
}

#[test]
fn group_list() {
    // Frame 34: a fresh tracker's eight groups, with the registration in General.
    let guest = guest();
    let wire = one(&guest, MessageType::TRACKER_GROUP_LIST);
    let groups = GroupListEntry::decode_list(wire).unwrap();
    assert_eq!(GroupListEntry::encode_list(&groups).unwrap(), wire);
    let names: Vec<_> = groups.iter().map(|g| g.name.as_slice()).collect();
    assert_eq!(
        names,
        [
            &b"Business"[..],
            b"Chat",
            b"Education",
            b"Games",
            b"General",
            b"Macintosh",
            b"Trackers",
            b"Windows"
        ]
    );
    for group in &groups {
        let servers = u16::from(group.name == b"General");
        assert_eq!(group.servers, servers);
        assert!(group.description.is_empty());
    }

    // Frame 247: the new group `Lab`, still empty.
    let edit = group_edit();
    let wire = all(&edit, MessageType::TRACKER_GROUP_LIST)[1];
    let groups = GroupListEntry::decode_list(wire).unwrap();
    assert_eq!(GroupListEntry::encode_list(&groups).unwrap(), wire);
    assert_eq!(groups.len(), 9);
    assert_eq!(groups[5].name, b"Lab");
    assert_eq!(groups[5].description, b"lab group");
    assert_eq!(groups[5].servers, 0);
}

#[test]
fn server_list() {
    // Frames 36/37 and 57/58: General holds the server, Chat nothing.
    let guest = guest();
    let requests = all(&guest, MessageType::GET_TRACKER_SERVER_LIST);
    let replies = all(&guest, MessageType::TRACKER_SERVER_LIST);
    let groups: Vec<_> = requests
        .iter()
        .map(|&wire| {
            let request = GetServerList::decode(wire).unwrap();
            assert_eq!(request.encode().unwrap(), wire);
            request.group
        })
        .collect();
    assert_eq!(groups, [b"General".to_vec(), b"Chat".to_vec()]);
    assert_eq!(replies[1], b"");
    assert_eq!(ServerListEntry::decode_list(replies[1]).unwrap(), []);
    assert_eq!(ServerListEntry::encode_list(&[]).unwrap(), b"");

    let servers = ServerListEntry::decode_list(replies[0]).unwrap();
    assert_eq!(ServerListEntry::encode_list(&servers).unwrap(), replies[0]);
    let lab = ServerListEntry {
        // 07:15:33.828 UTC, when registration 23 arrived.
        first_seen: 0x0000_03a3_80f5_a144,
        flags: 0,
        users: 1,
        address: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0x29cc),
        name: b"Lab Server".to_vec(),
        description: Vec::new(),
        extra: Vec::new(),
    };
    assert_eq!(servers, std::slice::from_ref(&lab));

    // Frame 257: group `Lab` after registration 255, which replaced the record.
    let edit = group_edit();
    let wire = *all(&edit, MessageType::TRACKER_SERVER_LIST).last().unwrap();
    let servers = ServerListEntry::decode_list(wire).unwrap();
    assert_eq!(ServerListEntry::encode_list(&servers).unwrap(), wire);
    assert_eq!(servers.len(), 1);
    // 07:36:12.034 UTC, when registration 255 arrived.
    assert_eq!(servers[0].first_seen, 0x0000_03a3_8108_8602);
    assert_eq!(
        ServerListEntry {
            first_seen: lab.first_seen,
            ..servers[0].clone()
        },
        lab
    );
}

fn decode_settings(wire: &[u8]) -> TrackerSettings {
    let settings = TrackerSettings::decode(wire).unwrap();
    assert_eq!(settings.encode(), wire);
    assert_eq!(settings.format, 1);
    for (i, section) in settings.sections.iter().enumerate() {
        let Some(bytes) = section else { continue };
        let reencoded = match i {
            TrackerSettings::GENERAL => GeneralSettings::decode(bytes).unwrap().encode().unwrap(),
            TrackerSettings::GROUPS => {
                SettingsGroup::encode_list(&SettingsGroup::decode_list(bytes).unwrap()).unwrap()
            }
            TrackerSettings::ADMIN => AdminSettings::decode(bytes).unwrap().encode().unwrap(),
            _ => continue,
        };
        assert_eq!(&reencoded, bytes, "section {i}");
    }
    settings
}

#[test]
fn settings() {
    let admin = admin();

    // Frame 80: a fresh tracker's settings.
    let info = decode_settings(one(&admin, MessageType::TRACKERSERV_SETTINGS_INFO));
    assert_eq!(info.present, [1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0]);
    let general = info.general().unwrap().unwrap();
    assert_eq!(
        general,
        GeneralSettings {
            port: 10800,
            name: b"New TrackerServ".to_vec(),
            ..GeneralSettings::default()
        }
    );
    assert_eq!(info.sections[TrackerSettings::TRACKERS], Some(Vec::new()));
    let groups = info.groups().unwrap().unwrap();
    assert_eq!(groups.len(), 8);
    assert!(groups
        .iter()
        .all(|g| g.flags == 0 && g.value_10 == 0 && g.value_12 == 0));
    assert_eq!(info.greeting().unwrap(), None);
    assert_eq!(
        info.admin().unwrap().unwrap(),
        AdminSettings {
            key: None,
            login: b"admin".to_vec()
        }
    );

    // Frame 82: the client leaves out the groups and doesn't mark the admin section present.
    let set = decode_settings(one(&admin, MessageType::SET_TRACKERSERV_SETTINGS));
    assert_eq!(set.present, [1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0]);
    let general = set.general().unwrap().unwrap();
    assert_eq!(general.flags, GeneralSettings::GREETING_ENABLED);
    assert_eq!(general.name, b"Lab Tracker");
    assert_eq!(general.description, b"Description");
    assert_eq!(set.sections[TrackerSettings::GROUPS], None);
    assert_eq!(set.groups().unwrap(), None);
    assert_eq!(set.greeting().unwrap().unwrap().text, b"Tracker greeting");
    assert_eq!(set.admin().unwrap(), None);
    // The ignored admin section holds the new password's account key.
    let ignored = set.sections[TrackerSettings::ADMIN].as_deref().unwrap();
    let ignored = AdminSettings::decode(ignored).unwrap();
    assert_eq!(
        ignored.key,
        Some(AccountKey::from_password(b"lab-tracker-pw!2"))
    );
    assert_eq!(ignored.login, b"admin");
}

#[test]
fn settings_with_new_group() {
    let edit = group_edit();

    // Frame 237: the settings saved at frame 82.
    let info = decode_settings(one(&edit, MessageType::TRACKERSERV_SETTINGS_INFO));
    let general = info.general().unwrap().unwrap();
    assert_eq!(general.name, b"Lab Tracker");
    assert_eq!(info.greeting().unwrap().unwrap().text, b"Tracker greeting");

    // Frame 240: all nine groups, `Lab` after General.
    let set = decode_settings(one(&edit, MessageType::SET_TRACKERSERV_SETTINGS));
    let groups = set.groups().unwrap().unwrap();
    let mut expected = info.groups().unwrap().unwrap();
    expected.insert(
        5,
        SettingsGroup {
            name: b"Lab".to_vec(),
            description: b"lab group".to_vec(),
            ..SettingsGroup::default()
        },
    );
    assert_eq!(groups, expected);
    assert_eq!(
        AdminSettings::decode(set.sections[TrackerSettings::ADMIN].as_deref().unwrap()).unwrap(),
        AdminSettings {
            key: None,
            login: b"admin".to_vec()
        }
    );
}

#[test]
fn statistics() {
    // Frame 86: the client's Tracker Stats window shows the same.
    let admin = admin();
    let wire = one(&admin, MessageType::TRACKERSERV_STATISTICS);
    let stats = TrackerStats::decode(wire).unwrap();
    assert_eq!(stats.encode(), wire);
    assert_eq!(
        stats,
        TrackerStats {
            // 07:13:51.362 UTC.
            start_time: 0x0000_03a3_80f4_1102,
            connections_accepted: 2,
            datagrams_received: 2,
            hours: 0,
            connections: 1,
            servers: 1,
        }
    );
}

#[test]
fn guest_denied() {
    // Frame 61: 710 from a guest.
    let guest = guest();
    let wire = one(&guest, MessageType::STATUS);
    let status = Status::decode(wire).unwrap();
    assert_eq!(status.encode(), wire);
    assert_eq!(
        status,
        Status::TrackerError {
            code: 0x10014,
            request: MessageType::GET_TRACKERSERV_SETTINGS,
        }
    );
}
