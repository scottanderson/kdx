//! Checks against `captures/linux-server-settings.pcapng`: Client 1.620 for Linux as `admin`.

use kdx_proto::msg::admin::HistoryRecord;
use kdx_proto::msg::server_info::{ServerGreeting, ServerIcon, ServerSettings};
use kdx_proto::msg::transfer::NotifySpeedLimit;
use kdx_proto::msg::user::{MiniUserIcon, MiniUserIconChanged};
use kdx_proto::msg::FilePath;
use kdx_proto::session::{Header, MessageType};

mod common;

/// The first login, port 45150: frames 20 to 62.
fn first() -> Vec<(Header, Vec<u8>)> {
    common::session(
        include_bytes!("fixtures/linux-server-settings-45150.c2s.bin"),
        include_bytes!("fixtures/linux-server-settings-45150.s2c.bin"),
    )
}

/// The second login, port 40560: frames 72 to 228.
fn second() -> Vec<(Header, Vec<u8>)> {
    common::session(
        include_bytes!("fixtures/linux-server-settings-40560.c2s.bin"),
        include_bytes!("fixtures/linux-server-settings-40560.s2c.bin"),
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

#[test]
fn server_settings() {
    let first = first();
    let second = second();
    let infos: Vec<_> = all(&first, MessageType::SERVER_SETTINGS_INFO)
        .into_iter()
        .chain(all(&second, MessageType::SERVER_SETTINGS_INFO))
        .collect();
    let sets: Vec<_> = all(&first, MessageType::SET_SERVER_SETTINGS)
        .into_iter()
        .chain(all(&second, MessageType::SET_SERVER_SETTINGS))
        .collect();
    // 171 at frames 36, 52, 221; 172 at frames 40, 56, 225.
    assert_eq!(
        infos.iter().map(|w| w.len()).collect::<Vec<_>>(),
        [216, 764, 764]
    );
    assert_eq!(
        sets.iter().map(|w| w.len()).collect::<Vec<_>>(),
        [764, 240, 240]
    );
    let decode = |wire: &[u8]| {
        let settings = ServerSettings::decode(wire).unwrap();
        assert_eq!(settings.encode(), wire);
        assert_eq!(settings.version, 1);
        assert_eq!(
            settings.sections[ServerSettings::MAIN],
            kdx_proto::msg::server_info::SettingsSection {
                offset: 0x70,
                len: 0x68
            }
        );
        settings
    };
    let [fresh, enabled, unlimited] = [infos[0], infos[1], infos[2]].map(decode);
    let [saved, enable, unlimit] = [sets[0], sets[1], sets[2]].map(decode);

    // Before the first save: the default name, no greeting, no icon, no limit.
    assert_eq!(fresh.name(), b"New Server");
    assert_eq!(fresh.greeting(), None);
    assert!(fresh.icon().is_empty());
    assert_eq!(fresh.speed_limit(), 0);

    // Frame 40 sets the name, greeting, icon and limit, with the greeting disabled.
    assert_eq!(saved.name(), b"Lab Server");
    assert_eq!(saved.greeting().unwrap().text, b"Welcome to the lab.");
    assert!(!saved.greeting_enabled());
    assert_eq!(saved.icon().len(), 524);
    assert_eq!(saved.speed_limit(), 0x5000);

    // Frame 56 enables the greeting and leaves out the icon.
    assert_eq!(enabled.flags(), 0);
    assert_eq!(enable.flags(), ServerSettings::GREETING_ENABLED);
    assert!(enable.icon().is_empty());

    // The server kept the icon (frame 221); frame 225 turns the limit off.
    assert!(unlimited.greeting_enabled());
    assert_eq!(unlimited.icon(), saved.icon());
    assert_eq!(unlimited.speed_limit(), 0x5000);
    assert_eq!(unlimit.speed_limit(), 0);
    assert_eq!(unlimit.name(), b"Lab Server");
}

#[test]
fn greeting_and_icon() {
    let first = first();
    // Frames 46 and 62 answer 112 before and after the greeting is enabled.
    let greetings: Vec<_> = all(&first, MessageType::SERVER_GREETING)
        .into_iter()
        .map(|wire| {
            let greeting = ServerGreeting::decode(wire).unwrap();
            assert_eq!(greeting.encode(), wire);
            greeting.text
        })
        .collect();
    assert_eq!(
        greetings,
        [
            b"(Greeting disabled.)".to_vec(),
            b"Welcome to the lab.".to_vec()
        ]
    );

    // Frames 82 and 83, at login.
    let second = second();
    let wire = one(&second, MessageType::SERVER_GREETING);
    let greeting = ServerGreeting::decode(wire).unwrap();
    assert_eq!(greeting.text, b"Welcome to the lab.");
    assert_eq!(greeting.encode(), wire);
    let wire = one(&second, MessageType::SERVER_ICON);
    let icon = ServerIcon::decode(wire).unwrap();
    assert_eq!(icon.0.len(), 524);
    assert!(icon.0.starts_with(b"\x01\x20\x20\x04"));
    assert_eq!(icon.encode(), wire);
    let settings = ServerSettings::decode(all(&second, MessageType::SERVER_SETTINGS_INFO)[0]);
    assert_eq!(settings.unwrap().icon(), icon.0);
}

#[test]
fn notify_speed_limit() {
    // Frame 110, after the 413 on the transfer connection.
    let session = second();
    let wire = one(&session, MessageType::NOTIFY_SPEED_LIMIT);
    let limit = NotifySpeedLimit::decode(wire).unwrap();
    assert_eq!(limit.bytes_per_second, 20480);
    assert_eq!(limit.encode(), wire);
}

#[test]
fn mini_user_icon() {
    let session = second();
    // Frames 210 and 215: /away, then /back.
    let sets: Vec<_> = all(&session, MessageType::SET_MINI_USER_ICON);
    let changes: Vec<_> = all(&session, MessageType::MINI_USER_ICON_CHANGED);
    assert_eq!((sets.len(), changes.len()), (2, 2));
    for ((set, changed), status) in sets.into_iter().zip(changes).zip([1, 0]) {
        let icon = MiniUserIcon::decode(set).unwrap();
        assert_eq!(icon, MiniUserIcon { status });
        assert_eq!(icon.encode(), set);

        // Frames 213 and 218, back to the sender, user 3.
        let notice = MiniUserIconChanged::decode(changed).unwrap();
        assert_eq!(notice, MiniUserIconChanged { user_id: 3, icon });
        assert_eq!(notice.encode(), changed);
    }
}

#[test]
fn server_history() {
    // Frame 33: 190 Server Started, then the peer's and the client's records.
    let first = first();
    let wire = one(&first, MessageType::SERVER_HISTORY_JOIN_REPLY);
    let records = HistoryRecord::decode_list(wire).unwrap();
    assert_eq!(
        records.iter().map(|r| r.kind.0).collect::<Vec<_>>(),
        [190, 184, 202, 184, 180]
    );
    assert_eq!(records[0].user_id, 0);
    assert!(records[0].address.is_empty());
    let join = &records[2];
    assert_eq!(join.user_id, 1);
    assert_eq!(join.address, b"127.0.0.1:52888");
    assert_eq!(join.nick, b"cli-peer");
    assert_eq!(join.login, b"admin");
    assert_eq!(join.detail, b"Public Chat");
    assert_eq!(HistoryRecord::encode_list(&records).unwrap(), wire);

    // Frame 89 has 12 records.
    let second = second();
    let wire = one(&second, MessageType::SERVER_HISTORY_JOIN_REPLY);
    let records = HistoryRecord::decode_list(wire).unwrap();
    assert_eq!(records.len(), 12);
    assert_eq!(HistoryRecord::encode_list(&records).unwrap(), wire);

    // Each 183 is one record padded to 4.
    let added: Vec<_> = all(&first, MessageType::SERVER_HISTORY_ADDED)
        .into_iter()
        .chain(all(&second, MessageType::SERVER_HISTORY_ADDED))
        .map(|wire| {
            let record = HistoryRecord::decode_added(wire).unwrap();
            assert_eq!(record.encode_added().unwrap(), wire);
            record
        })
        .collect();
    assert_eq!(
        added.iter().map(|r| r.kind.0).collect::<Vec<_>>(),
        [170, 172, 170, 172, 400, 412, 188, 100, 202, 170, 172]
    );
    // Frame 108: the download, with its path.
    assert_eq!(added[5].kind, MessageType::DOWNLOAD_FILE);
    assert_eq!(added[5].path, FilePath::new(["speed.bin"]));
    assert_eq!(added[6].path, added[5].path);
    // Frame 197: the new nick of 100 Set Identity.
    assert_eq!(added[7].nick, b"kdx-lab-linux");
    assert_eq!(added[7].detail, b"lab-linux");
    assert!(added.windows(2).all(|w| w[0].time <= w[1].time));
}
