//! Re-encodes the fixture payloads whose padding or unknown bytes the codecs must keep, and
//! synthetic payloads with every unmodelled byte set.

use std::net::{Ipv4Addr, SocketAddrV4};

use kdx_proto::msg::admin::{
    Account, AccountClass, AccountListEntry, DisconnectMessage, DisconnectUser, ExtraServerInfo,
    FolderAccess, FolderAccessItem, FolderAccessName, GetFolderAccessList, HistoryList,
    HistoryRecord, MonitoredConnection, Process, ProcessList, RcvBroadcast, ShutdownServer,
};
use kdx_proto::msg::chat::{
    ChatInvitation, ChatInvite, ChatListEntry, ChatRecord, ChatText, ChatTopicChanged,
    ChatUserChange, ChatWindowInfo, SetChatTopic,
};
use kdx_proto::msg::dcc::Skpk;
use kdx_proto::msg::file::{
    CreateFolder, FileInfo, FileListEntry, FileSearch, FileSearchEntry, LaunchFile, SetFileInfo,
    SetVolumeInfo, VolumeInfo,
};
use kdx_proto::msg::gconsole::{Draw, DrawRecord};
use kdx_proto::msg::news::{
    NewsMessage, NewsMessageId, NewsMessages, NewsgroupEntry, NewsgroupId, NewsgroupInfo,
    PostNewsMessage,
};
use kdx_proto::msg::server_info::{ServerGreeting, ServerSettings};
use kdx_proto::msg::tracker::{
    AdminSettings, GeneralSettings, GetServerList, GroupListEntry, ServerListEntry, SettingsGroup,
    TrackerSettings, TrackerStats,
};
use kdx_proto::msg::transfer::{DownloadFile, DownloadInfo, UploadFile, UploadReply};
use kdx_proto::msg::user::{MiniUserIconChanged, UserDeparted};
use kdx_proto::msg::{FilePath, PrivateMessage, ServerInfo, Status, UserInfo, UserRecord};
use kdx_proto::session::MessageType;
use kdx_proto::AccountKey;

mod common;

macro_rules! fixture {
    ($name:literal) => {
        (
            $name,
            include_bytes!(concat!("fixtures/", $name, ".c2s.bin")).as_slice(),
            include_bytes!(concat!("fixtures/", $name, ".s2c.bin")).as_slice(),
        )
    };
}

/// The decoded and re-encoded payload, or `None` for a type this test skips.
fn reencode(kind: MessageType, p: &[u8]) -> Option<Vec<u8>> {
    type M = MessageType;
    let out = match kind {
        M::USER_CHANGED | M::USER_JOINED => UserRecord::decode(p).unwrap().0.encode(),
        M::CONNECTION_MONITOR_INFO => {
            MonitoredConnection::encode_list(&MonitoredConnection::decode_list(p).unwrap())
        }
        M::SET_CHAT_TOPIC => SetChatTopic::decode(p).unwrap().encode(),
        M::FILE_LIST => FileListEntry::encode_list(&FileListEntry::decode_list(p).unwrap()),
        M::FILE_FOLDER_INFO => FileInfo::decode(p).unwrap().encode(),
        M::ACCOUNT_CLASS_INFO => AccountClass::decode_info(p).unwrap().encode_info(),
        M::ACCOUNT_INFO => Account::decode(p).unwrap().encode(),
        M::ACCOUNT_LIST => {
            AccountListEntry::encode_list(&AccountListEntry::decode_list(p).unwrap())
        }
        M::NEWS_MESSAGES => NewsMessages::decode(p).unwrap().encode(),
        M::NEWS_MESSAGE => NewsMessage::decode(p).unwrap().0.encode(),
        M::POST_NEWS_MESSAGE => PostNewsMessage::decode(p).unwrap().encode(),
        M::DELETE_NEWSGROUP | M::GET_NEWSGROUP | M::GET_NEWS_MESSAGES => {
            Ok(NewsgroupId::decode(p).unwrap().encode().to_vec())
        }
        M::GET_NEWS_MESSAGE | M::DELETE_NEWS_MESSAGE => {
            Ok(NewsMessageId::decode(p).unwrap().encode().to_vec())
        }
        M::PROCESS_LIST => ProcessList::decode(p).unwrap().encode(),
        M::CREATE_FOLDER_ACCESS | M::FOLDER_ACCESS_INFO => {
            FolderAccess::decode(p).unwrap().encode()
        }
        _ => return None,
    };
    Some(out.unwrap())
}

#[test]
fn padding_and_unknown_bytes_round_trip() {
    let mut seen = std::collections::BTreeSet::new();
    for (name, c2s, s2c) in [
        fixture!("first-connect"),
        fixture!("kdx-accounts-58290"),
        fixture!("kdx-cli-51784"),
        fixture!("kdx-news-40426"),
        fixture!("kdx-news-53204"),
        fixture!("kdx-news-53224"),
        fixture!("linux-accounts-news-51382"),
        fixture!("linux-catalog-volumes-58160"),
        fixture!("linux-chat-admin-37478"),
        fixture!("linux-chat-admin-37484"),
        fixture!("linux-chat-rooms-43718"),
        fixture!("linux-files-59246"),
        fixture!("linux-server-settings-40560"),
        fixture!("probe-35844"),
        fixture!("probe-54328"),
        fixture!("probe-59006"),
    ] {
        for (header, wire) in common::session(c2s, s2c) {
            if let Some(out) = reencode(header.kind, &wire) {
                assert_eq!(out, wire, "{name} {}", header.kind);
                seen.insert(header.kind.0);
            }
        }
    }
    let expected = [
        102, 103, 143, 209, 401, 410, 503, 510, 513, 601, 602, 607, 608, 609, 610, 611, 612, 803,
        1100, 1104,
    ];
    assert_eq!(seen.into_iter().collect::<Vec<_>>(), expected);
}

/// Fills `gaps` of `wire` with nonzero bytes and checks that `reencode` gives it back.
fn gaps(
    name: &str,
    mut wire: Vec<u8>,
    gaps: &[std::ops::Range<usize>],
    reencode: impl Fn(&[u8]) -> Vec<u8>,
) {
    let mut value = 0x80u8;
    for gap in gaps {
        assert!(
            gap.end <= wire.len(),
            "{name}: gap {gap:?} past {}",
            wire.len()
        );
        for byte in &mut wire[gap.clone()] {
            *byte = value;
            value = value.wrapping_add(0x13) | 0x80;
        }
    }
    assert_eq!(reencode(&wire), wire, "{name}");
}

/// `wire` with `n` more zero bytes, where a decoder keeps padding or trailing bytes.
fn with(mut wire: Vec<u8>, n: usize) -> Vec<u8> {
    wire.resize(wire.len() + n, 0);
    wire
}

/// Shifts each range by `by`, for a record after a list count.
fn at(by: usize, ranges: &[std::ops::Range<usize>]) -> Vec<std::ops::Range<usize>> {
    ranges.iter().map(|r| r.start + by..r.end + by).collect()
}

#[test]
#[allow(clippy::single_range_in_vec_init)] // Some types have one gap.
fn nonzero_gaps_round_trip() {
    let a = || b"a".to_vec();
    let file = || FilePath::new(["f"]);

    // Transfers.
    let wire = DownloadFile {
        path: file(),
        ..DownloadFile::default()
    }
    .encode()
    .unwrap();
    gaps("412", wire, &[0x10..0x17], |p| {
        DownloadFile::decode(p).unwrap().encode().unwrap()
    });
    let wire = DownloadInfo::default().encode().unwrap();
    gaps("413", wire, &[0x10..0x18, 0x28..0x30, 0x34..0x48], |p| {
        DownloadInfo::decode(p).unwrap().encode().unwrap()
    });
    let mut upload = UploadFile::default();
    upload.strings.name = b"f".to_vec();
    let wire = with(upload.encode().unwrap(), 3);
    gaps("415", wire, &[0x10..0x17, 0x18..0x46, 0x4d..0x50], |p| {
        UploadFile::decode(p).unwrap().encode().unwrap()
    });
    let wire = UploadReply::default().encode().to_vec();
    gaps("416", wire, &[0x10..0x18], |p| {
        UploadReply::decode(p).unwrap().encode().to_vec()
    });

    // Users.
    let user = UserRecord {
        nick: a(),
        ..UserRecord::default()
    };
    let wire = with(user.encode().unwrap(), 3);
    gaps("102", wire, &[5..0x0a, 0x1b..0x1c, 0x1d..0x20], |p| {
        UserRecord::decode(p).unwrap().0.encode().unwrap()
    });
    let wire = UserDeparted::default().encode().to_vec();
    gaps("104", wire, &[5..8], |p| {
        UserDeparted::decode(p).unwrap().encode().to_vec()
    });
    let wire = MiniUserIconChanged::default().encode().to_vec();
    gaps("111", wire, &[5..8], |p| {
        MiniUserIconChanged::decode(p).unwrap().encode().to_vec()
    });
    let wire = UserInfo {
        address: a(),
        ..UserInfo::default()
    }
    .encode()
    .unwrap();
    gaps(
        "153",
        wire,
        &[
            4..8,
            0x20..0x3b,
            0x3c..0x3d,
            0x3e..0x3f,
            0x42..0x44,
            0x45..0x48,
        ],
        |p| UserInfo::decode(p).unwrap().encode().unwrap(),
    );
    let wire = PrivateMessage {
        text: a(),
        ..PrivateMessage::default()
    }
    .encode();
    gaps("300", wire, &[4..0x10], |p| {
        PrivateMessage::decode(p).unwrap().encode()
    });
    let skpk = Skpk {
        key: AccountKey([0; 32]),
        dcc_port: 0,
        connection_id: 0,
        client_version: 0,
        unknown_2a: [0; 6],
    };
    gaps("117", skpk.encode(), &[0x2a..0x30], |p| {
        Skpk::decode(p).unwrap().encode()
    });

    // Status and server information.
    let error = Status::Error {
        code: 1,
        request: MessageType(2),
        module: 0,
        line: 0,
        extra: 0,
        value_14: 0,
    };
    gaps("1", error.encode(), &[0x14..0x18], |p| {
        Status::decode(p).unwrap().encode()
    });
    gaps(
        "105",
        ServerInfo::default().encode().unwrap(),
        &[0x2c..0x46, 0x4b..0x4c],
        |p| ServerInfo::decode(p).unwrap().encode().unwrap(),
    );
    gaps("107", ServerGreeting::default().encode(), &[0..4], |p| {
        ServerGreeting::decode(p).unwrap().encode()
    });
    gaps(
        "171",
        ServerSettings::default().encode(),
        &[0x0a..0x0c],
        |p| ServerSettings::decode(p).unwrap().encode(),
    );
    gaps(
        "114",
        ExtraServerInfo::default().encode().unwrap(),
        &[0..0x10, 0x2d..0x30],
        |p| ExtraServerInfo::decode(p).unwrap().encode().unwrap(),
    );

    // Administration.
    let wire = DisconnectUser::default().encode().unwrap();
    gaps(
        "150",
        wire,
        &[4..8, 0x0c..0x11, 0x12..0x14, 0x15..0x18],
        |p| DisconnectUser::decode(p).unwrap().encode().unwrap(),
    );
    gaps(
        "151",
        DisconnectMessage::default().encode().unwrap(),
        &[4..0x0c],
        |p| DisconnectMessage::decode(p).unwrap().encode().unwrap(),
    );
    gaps("155", RcvBroadcast::default().encode(), &[4..0x0e], |p| {
        RcvBroadcast::decode(p).unwrap().encode()
    });
    gaps(
        "174",
        ShutdownServer::default().encode().unwrap(),
        &[0..1, 2..4],
        |p| ShutdownServer::decode(p).unwrap().encode().unwrap(),
    );
    let record = HistoryRecord {
        time: 0,
        kind: MessageType(184),
        user_id: 0,
        unknown_10: [0; 0x32],
        unknown_44: [0; 2],
        unknown_4a: [0; 2],
        address: a(),
        nick: Vec::new(),
        login: Vec::new(),
        detail: Vec::new(),
        path: FilePath::default(),
        padding: Vec::new(),
    };
    let record_gaps = [0x10..0x42, 0x44..0x46, 0x4a..0x4c, 0x4d..0x50];
    gaps("183", record.encode_added().unwrap(), &record_gaps, |p| {
        HistoryRecord::decode_added(p)
            .unwrap()
            .encode_added()
            .unwrap()
    });
    let list = HistoryList {
        records: vec![record],
        ..HistoryList::default()
    };
    gaps(
        "181",
        list.encode().unwrap(),
        &[[0..4].as_slice(), &at(8, &record_gaps)].concat(),
        |p| HistoryList::decode(p).unwrap().encode().unwrap(),
    );
    let wire = MonitoredConnection::encode_list(&[MonitoredConnection {
        nick: a(),
        ..Default::default()
    }])
    .unwrap();
    gaps(
        "143",
        wire,
        &[0x40..0x42, 0x43..0x60, 0x64..0x66, 0x69..0x6c],
        |p| {
            MonitoredConnection::encode_list(&MonitoredConnection::decode_list(p).unwrap()).unwrap()
        },
    );
    let class = AccountClass {
        name: a(),
        ..AccountClass::default()
    };
    gaps(
        "503",
        class.encode_info().unwrap(),
        &[0x18..0x25, 0x26..0x30, 0x31..0x34, 0x37..0x38, 0x39..0x3c],
        |p| AccountClass::decode_info(p).unwrap().encode_info().unwrap(),
    );
    let wire = AccountClass::encode_list(&[class]).unwrap();
    gaps(
        "506",
        wire,
        &at(
            4,
            &[0x18..0x25, 0x26..0x28, 0x29..0x2c, 0x2f..0x30, 0x31..0x34],
        ),
        |p| AccountClass::encode_list(&AccountClass::decode_list(p).unwrap()).unwrap(),
    );
    let wire = Account {
        login: a(),
        ..Account::default()
    }
    .encode()
    .unwrap();
    gaps(
        "510",
        wire,
        &[0x58..0x76, 0x77..0x7a, 0x7e..0x80, 0x81..0x84],
        |p| Account::decode(p).unwrap().encode().unwrap(),
    );
    let wire = AccountListEntry::encode_list(&[AccountListEntry {
        login: a(),
        ..Default::default()
    }])
    .unwrap();
    gaps(
        "513",
        wire,
        &at(
            4,
            &[0x10..0x1b, 0x1c..0x20, 0x22..0x24, 0x26..0x28, 0x29..0x2c],
        ),
        |p| AccountListEntry::encode_list(&AccountListEntry::decode_list(p).unwrap()).unwrap(),
    );
    let processes = ProcessList {
        processes: vec![Process {
            user: a(),
            ..Process::default()
        }],
        ..ProcessList::default()
    };
    let process_gaps = at(0x20, &[4..8, 0x10..0x39, 0x3b..0x3c, 0x3d..0x40]);
    gaps(
        "803",
        processes.encode().unwrap(),
        &[[0x10..0x1c].as_slice(), &process_gaps].concat(),
        |p| ProcessList::decode(p).unwrap().encode().unwrap(),
    );
    let wire = FolderAccessItem::encode_list(&[FolderAccessItem {
        name: a(),
        ..Default::default()
    }])
    .unwrap();
    gaps("1107", wire, &at(4, &[8..0x10, 0x15..0x18]), |p| {
        FolderAccessItem::encode_list(&FolderAccessItem::decode_list(p).unwrap()).unwrap()
    });
    let wire = GetFolderAccessList {
        flags: Some(0),
        rest: vec![0; 4],
    }
    .encode();
    gaps("1106", wire, &[0..8], |p| {
        GetFolderAccessList::decode(p).unwrap().encode()
    });
    for n in 0..4 {
        gaps("1106 short", vec![0; n], &[0..n], |p| {
            GetFolderAccessList::decode(p).unwrap().encode()
        });
    }
    gaps("1101", vec![0; 4], &[0..4], |p| {
        FolderAccessName::decode(p).unwrap().encode()
    });
    let wire = FolderAccess {
        name: a(),
        ..FolderAccess::default()
    }
    .encode()
    .unwrap();
    gaps(
        "1104",
        wire,
        &[0x90..0xad, 0xb0..0xb1, 0xb2..0xb8, 0xb9..0xbc],
        |p| FolderAccess::decode(p).unwrap().encode().unwrap(),
    );

    // Chat.
    gaps(
        "203",
        ChatWindowInfo::default().encode().unwrap(),
        &[0x10..0x14, 0x18..0x1e, 0x23..0x24],
        |p| ChatWindowInfo::decode(p).unwrap().encode().unwrap(),
    );
    gaps(
        "205",
        ChatUserChange::default().encode().to_vec(),
        &[9..0x0c],
        |p| ChatUserChange::decode(p).unwrap().encode().to_vec(),
    );
    gaps(
        "207",
        ChatText::default().encode().unwrap(),
        &[8..0x0c],
        |p| ChatText::decode(p).unwrap().encode().unwrap(),
    );
    let wire = with(
        SetChatTopic {
            topic: a(),
            ..SetChatTopic::default()
        }
        .encode()
        .unwrap(),
        2,
    );
    gaps("209", wire, &[4..0x0a, 0x0b..0x0c, 0x0d..0x0f], |p| {
        SetChatTopic::decode(p).unwrap().encode().unwrap()
    });
    gaps(
        "210",
        ChatTopicChanged::default().encode().unwrap(),
        &[0x10..0x18, 0x1a..0x1c],
        |p| ChatTopicChanged::decode(p).unwrap().encode().unwrap(),
    );
    gaps(
        "211",
        ChatInvite::default().encode().unwrap(),
        &[4..8, 0x0a..0x0f],
        |p| ChatInvite::decode(p).unwrap().encode().unwrap(),
    );
    gaps(
        "212",
        ChatInvitation::default().encode().unwrap(),
        &[0x12..0x18, 0x1a..0x1c],
        |p| ChatInvitation::decode(p).unwrap().encode().unwrap(),
    );
    let wire = ChatListEntry::encode_list(&[ChatListEntry {
        name: a(),
        ..Default::default()
    }])
    .unwrap();
    gaps("215", wire, &at(4, &[8..0x0e, 0x15..0x18]), |p| {
        ChatListEntry::encode_list(&ChatListEntry::decode_list(p).unwrap()).unwrap()
    });
    gaps(
        "219",
        ChatRecord::default().encode().unwrap(),
        &[8..0x14, 0x20..0x28, 0x2e..0x30],
        |p| ChatRecord::decode(p).unwrap().encode().unwrap(),
    );

    // Files.
    let wire = FileListEntry::encode_list(&[FileListEntry {
        name: a(),
        ..Default::default()
    }])
    .unwrap();
    gaps(
        "401",
        wire,
        &at(
            4,
            &[0x18..0x20, 0x24..0x2e, 0x32..0x34, 0x37..0x38, 0x39..0x3c],
        ),
        |p| FileListEntry::encode_list(&FileListEntry::decode_list(p).unwrap()).unwrap(),
    );
    gaps(
        "402",
        CreateFolder::default().encode().unwrap(),
        &[0..0x13],
        |p| CreateFolder::decode(p).unwrap().encode().unwrap(),
    );
    let wire = with(FileInfo::default().encode().unwrap(), 2);
    gaps("410", wire, &[0x20..0x28, 0x2c..0x3d, 0x3f..0x42], |p| {
        FileInfo::decode(p).unwrap().encode().unwrap()
    });
    gaps(
        "411",
        SetFileInfo::default().encode().unwrap(),
        &[0..0x28, 0x2c..0x2e, 0x30..0x31, 0x32..0x34],
        |p| SetFileInfo::decode(p).unwrap().encode().unwrap(),
    );
    let wire = LaunchFile {
        path: file(),
        ..LaunchFile::default()
    }
    .encode()
    .unwrap();
    gaps("420", wire, &[0..8], |p| {
        LaunchFile::decode(p).unwrap().encode().unwrap()
    });
    let volume_gaps = [
        0x24..0x26,
        0x27..0x2c,
        0x34..0x44,
        0x4c..0x4e,
        0x4f..0x50,
        0x52..0x54,
    ];
    gaps(
        "422",
        VolumeInfo::default().encode().unwrap(),
        &volume_gaps,
        |p| VolumeInfo::decode(p).unwrap().encode().unwrap(),
    );
    gaps(
        "423",
        SetVolumeInfo::default().encode().unwrap(),
        &[0..0x44, 0x48..0x4e, 0x52..0x54],
        |p| SetVolumeInfo::decode(p).unwrap().encode().unwrap(),
    );
    gaps(
        "452",
        FileSearch::default().encode().unwrap(),
        &[0..0x54, 0x55..0x58],
        |p| FileSearch::decode(p).unwrap().encode().unwrap(),
    );
    let wire = FileSearchEntry::encode_list(&[FileSearchEntry {
        path: file(),
        ..Default::default()
    }])
    .unwrap();
    let hit_gaps = at(
        4,
        &[
            0x1c..0x20,
            0x24..0x2e,
            0x30..0x34,
            0x35..0x36,
            0x37..0x38,
            0x3e..0x40,
        ],
    );
    gaps("453", wire, &hit_gaps, |p| {
        FileSearchEntry::encode_list(&FileSearchEntry::decode_list(p).unwrap()).unwrap()
    });

    // News.
    let wire = NewsgroupEntry::encode_list(&[NewsgroupEntry {
        name: a(),
        ..Default::default()
    }])
    .unwrap();
    gaps(
        "606",
        wire,
        &at(4, &[0x0c..0x10, 0x12..0x16, 0x19..0x1c]),
        |p| NewsgroupEntry::encode_list(&NewsgroupEntry::decode_list(p).unwrap()).unwrap(),
    );
    gaps(
        "603",
        NewsgroupInfo::default().encode().unwrap(),
        &[4..0x10, 0x24..0x30, 0x34..0x3c, 0x3e..0x40],
        |p| NewsgroupInfo::decode(p).unwrap().encode().unwrap(),
    );
    let message = NewsMessage {
        poster: a(),
        ..NewsMessage::default()
    };
    let message_gaps = [0x10..0x18, 0x1a..0x1c, 0x1f..0x20, 0x21..0x24];
    gaps("610", message.encode().unwrap(), &message_gaps, |p| {
        NewsMessage::decode(p).unwrap().0.encode().unwrap()
    });
    let messages = NewsMessages {
        messages: vec![message],
        ..NewsMessages::default()
    };
    gaps(
        "608",
        messages.encode().unwrap(),
        &[[8..0x0c].as_slice(), &at(0x10, &message_gaps)].concat(),
        |p| NewsMessages::decode(p).unwrap().encode().unwrap(),
    );
    let wire = with(
        PostNewsMessage {
            subject: a(),
            ..PostNewsMessage::default()
        }
        .encode()
        .unwrap(),
        3,
    );
    gaps("611", wire, &[4..0x0c, 0x0f..0x10, 0x11..0x14], |p| {
        PostNewsMessage::decode(p).unwrap().encode().unwrap()
    });

    // Tracker.
    let wire = GroupListEntry::encode_list(&[GroupListEntry {
        name: a(),
        ..Default::default()
    }])
    .unwrap();
    gaps("701", wire, &at(4, &[9..0x0c]), |p| {
        GroupListEntry::encode_list(&GroupListEntry::decode_list(p).unwrap()).unwrap()
    });
    gaps(
        "702",
        GetServerList::default().encode().unwrap(),
        &[0..2, 3..4],
        |p| GetServerList::decode(p).unwrap().encode().unwrap(),
    );
    let server = ServerListEntry {
        first_seen: 0,
        unknown_08: [0; 6],
        flags: 0,
        users: 0,
        unknown_12: [0; 6],
        address: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 10700),
        name: a(),
        description: Vec::new(),
        extra: Vec::new(),
        padding: Vec::new(),
    };
    let wire = ServerListEntry::encode_list(&[server]).unwrap();
    gaps(
        "703",
        wire,
        &at(4, &[8..0x0e, 0x12..0x18, 0x23..0x24]),
        |p| ServerListEntry::encode_list(&ServerListEntry::decode_list(p).unwrap()).unwrap(),
    );
    gaps(
        "711",
        TrackerSettings::default().encode(),
        &[0x0a..0x0c],
        |p| TrackerSettings::decode(p).unwrap().encode(),
    );
    let wire = GeneralSettings {
        name: a(),
        ..GeneralSettings::default()
    }
    .encode()
    .unwrap();
    gaps(
        "711 general",
        wire,
        &[8..0x20, 0x28..0x2c, 0x38..0x46, 0x4d..0x50],
        |p| GeneralSettings::decode(p).unwrap().encode().unwrap(),
    );
    let wire = SettingsGroup::encode_list(&[SettingsGroup {
        name: a(),
        ..SettingsGroup::default()
    }])
    .unwrap();
    gaps(
        "711 groups",
        wire,
        &[0..0x0c, 0x14..0x18, 0x1a..0x1c, 0x1d..0x20],
        |p| SettingsGroup::encode_list(&SettingsGroup::decode_list(p).unwrap()).unwrap(),
    );
    let wire = AdminSettings {
        login: a(),
        ..AdminSettings::default()
    }
    .encode()
    .unwrap();
    gaps(
        "711 admin",
        wire,
        &[0..8, 0x28..0x90, 0x91..0x94, 0x95..0x98],
        |p| AdminSettings::decode(p).unwrap().encode().unwrap(),
    );
    gaps(
        "721",
        TrackerStats::default().encode(),
        &[0x18..0x20, 0x2c..0x38],
        |p| TrackerStats::decode(p).unwrap().encode(),
    );

    // GConsole: the padding after a record, and a record longer than its kind.
    let draw = Draw {
        records: vec![
            DrawRecord::Delete { id: 1 },
            DrawRecord::Other {
                kind: 3,
                body: vec![0; 3],
            },
        ],
        ..Draw::default()
    };
    gaps(
        "4001",
        draw.encode().unwrap(),
        &[6..8, 0x0c..0x0f, 0x0f..0x10],
        |p| Draw::decode(p).unwrap().encode().unwrap(),
    );
}
