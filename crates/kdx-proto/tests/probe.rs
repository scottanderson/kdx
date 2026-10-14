//! Checks against `captures/probe.pcapng`: kdx-probe sessions as `admin` on Server 1.620.

use kdx_proto::msg::admin::{
    Account, AccountClass, AccountListEntry, ExtraServerInfo, FolderAccessItem,
    MonitoredConnection, ProcessList, YourAddress,
};
use kdx_proto::msg::chat::{
    ChatListEntry, ChatRecord, ChatText, ChatTopicChanged, ChatWindowInfo, JoinChat, SetChatTopic,
    PUBLIC_CHAT,
};
use kdx_proto::msg::file::{FileInfo, FileListEntry, ATTR_FOLDER};
use kdx_proto::msg::news::NewsgroupInfo;
use kdx_proto::msg::server_info::{ServerGreeting, ServerSettings};
use kdx_proto::msg::transfer::{DownloadFile, DownloadInfo, UploadFile, UploadReply};
use kdx_proto::msg::user::RegistrationNotice;
use kdx_proto::msg::user_info::GetUserInfo;
use kdx_proto::msg::{FilePath, ServerInfo, UserInfo};
use kdx_proto::session::{Header, MessageType};
use kdx_proto::AccountKey;

mod common;

macro_rules! connection {
    ($port:literal) => {
        common::session(
            include_bytes!(concat!("fixtures/probe-", $port, ".c2s.bin")),
            include_bytes!(concat!("fixtures/probe-", $port, ".s2c.bin")),
        )
    };
}

/// The payloads of every message of one type.
fn payloads(messages: &[(Header, Vec<u8>)], kind: MessageType) -> Vec<&[u8]> {
    let found: Vec<&[u8]> = messages
        .iter()
        .filter(|(h, _)| h.kind == kind)
        .map(|(_, p)| p.as_slice())
        .collect();
    assert!(!found.is_empty(), "no {kind}");
    found
}

fn one(messages: &[(Header, Vec<u8>)], kind: MessageType) -> &[u8] {
    let found = payloads(messages, kind);
    assert_eq!(found.len(), 1, "{kind}");
    found[0]
}

#[test]
fn chat() {
    let session = connection!("35844");

    let join = one(&session, MessageType::JOIN_CHAT);
    assert_eq!(JoinChat::decode(join).unwrap().chat_id, PUBLIC_CHAT);
    assert_eq!(JoinChat::decode(join).unwrap().encode(), join);

    let window = one(&session, MessageType::CHAT_WINDOW_INFO);
    let info = ChatWindowInfo::decode(window).unwrap();
    assert_eq!(info.name(), b"Public Chat");
    assert_eq!(info.members, [3]);
    assert_eq!(info.encode().unwrap(), window);

    for kind in [MessageType::SEND_CHAT_TEXT, MessageType::RECEIVE_CHAT_TEXT] {
        let wire = one(&session, kind);
        let text = ChatText::decode(wire).unwrap();
        assert_eq!(text.text, b"Hello");
        assert_eq!(text.encode().unwrap(), wire);
    }
    let received = ChatText::decode(one(&session, MessageType::RECEIVE_CHAT_TEXT)).unwrap();
    assert_eq!(received.user_id, 3);

    // The probe put the topic at 0x10, so the server read four zeros and "H".
    let set = one(&session, MessageType::SET_CHAT_TOPIC);
    assert_eq!(SetChatTopic::decode(set).unwrap().topic, b"\0\0\0\0H");
    let changed = one(&session, MessageType::CHAT_TOPIC_CHANGED);
    let topic = ChatTopicChanged::decode(changed).unwrap();
    assert_eq!(topic.topic, b"\0\0\0\0H");
    assert_eq!(topic.nick, b"probe");
    assert_eq!(topic.user_id, 3);
    assert_eq!(topic.encode().unwrap(), changed);

    let record = one(&session, MessageType::CHAT_INFO);
    let chat = ChatRecord::decode(record).unwrap();
    assert_eq!(chat.name, b"Public Chat");
    assert_eq!(chat.description, b"For all users");
    assert_eq!(chat.join_classes, u32::MAX);
    assert_eq!(chat.encode().unwrap(), record);
}

#[test]
fn chat_list() {
    let session = connection!("50770");
    let wire = one(&session, MessageType::CHAT_LIST);
    let list = ChatListEntry::decode_list(wire).unwrap();
    let names: Vec<&[u8]> = list.records.iter().map(|c| c.name.as_slice()).collect();
    assert_eq!(names, [&b"Public Chat"[..], b"Voice Chat", b"Admin Lounge"]);
    assert_eq!(list.records[1].flags, 2);
    assert_eq!(ChatListEntry::encode_list(&list).unwrap(), wire);
}

#[test]
fn files() {
    for session in [connection!("32980"), connection!("54328")] {
        for wire in payloads(&session, MessageType::FILE_LIST) {
            let list = FileListEntry::decode_list(wire).unwrap();
            if let [first, second] = &list.records[..] {
                assert_eq!(first.name, b"Admin Dropbox [DB]");
                assert_eq!(second.name, b"Uploads [UL]");
                assert!(list
                    .records
                    .iter()
                    .all(|e| e.attributes == ATTR_FOLDER && e.is_folder()));
                assert_eq!((first.access_item, second.access_item), (0x2c, 0x2b));
                // The server leaves the padding after the first name uninitialised.
                assert_eq!(first.padding.len(), 2);
                assert_ne!(first.padding, [0, 0]);
            } else {
                assert!(list.records.is_empty());
            }
            assert_eq!(FileListEntry::encode_list(&list).unwrap(), wire);
        }
    }

    let session = connection!("54328");
    let request = one(&session, MessageType::GET_FILE_FOLDER_INFO);
    assert_eq!(
        FilePath::decode(request).unwrap(),
        FilePath::new(["Uploads [UL]"])
    );
    let wire = one(&session, MessageType::FILE_FOLDER_INFO);
    let info = FileInfo::decode(wire).unwrap();
    assert_eq!(info.name, b"Uploads [UL]");
    assert_eq!(info.kind, b"folder");
    assert_eq!(info.encode().unwrap(), wire);
}

#[test]
fn user_info() {
    let session = connection!("50984");
    let wire = one(&session, MessageType::USER_INFO);
    let info = UserInfo::decode(wire).unwrap();
    assert_eq!(info.address, b"127.0.0.1:50984");
    assert_eq!(info.nick, b"probe");
    assert_eq!(info.login, b"admin");
    assert_eq!(info.name_3f, b"Administrator");
    assert_eq!(info.encode().unwrap(), wire);
}

/// The session of a transfer connection, opened with the key from its parent's 105 Server Info.
macro_rules! transfer {
    ($parent:literal, $port:literal) => {{
        let parent = connection!($parent);
        let info = ServerInfo::decode(one(&parent, MessageType::SERVER_INFO)).unwrap();
        let session = common::session_with_key(
            include_bytes!(concat!("fixtures/probe-", $port, ".c2s.bin")),
            include_bytes!(concat!("fixtures/probe-", $port, ".s2c.bin")),
            &AccountKey(info.session_key),
        );
        (info.user_id, session)
    }};
}

/// The data of each 414 Download File (D), in order.
fn download_data(session: &[(Header, Vec<u8>)]) -> Vec<&[u8]> {
    payloads(session, MessageType::DOWNLOAD_FILE_DATA)
}

#[test]
fn downloads() {
    let probe_dl = FilePath::new(["probe-dl.bin"]);
    let (_, full) = transfer!("33068", "33082");
    let (_, resumed) = transfer!("53514", "53518");
    let (_, past_end) = transfer!("56346", "56352");
    let mut whole = Vec::new();
    for (session, offset, data_len) in [
        (&full, 0, 70000),
        (&resumed, 5000, 70000 - 0x1000),
        (&past_end, 70000, 0),
    ] {
        let wire = one(session, MessageType::DOWNLOAD_FILE);
        let request = DownloadFile::decode(wire).unwrap();
        assert_eq!(request.data_offset, offset);
        assert_eq!(request.path, probe_dl);
        assert_eq!(request.encode().unwrap(), wire);

        let wire = one(session, MessageType::DOWNLOAD_FILE_REPLY);
        let info = DownloadInfo::decode(wire).unwrap();
        assert_eq!(info.data_size, 70000);
        assert_eq!(info.strings.name, b"probe-dl.bin");
        assert_eq!(info.strings.second, b"exttype/bin");
        assert_eq!(info.encode().unwrap(), wire);

        // The data fork comes as a reply to the request, then the resource fork on its own.
        let request_txid = session
            .iter()
            .find(|(h, _)| h.kind == MessageType::DOWNLOAD_FILE)
            .unwrap()
            .0
            .txid;
        let data: Vec<_> = session
            .iter()
            .filter(|(h, _)| h.kind == MessageType::DOWNLOAD_FILE_DATA)
            .collect();
        assert_eq!(data.len(), 2);
        assert!(data[0].0.is_reply() && data[0].0.txid == request_txid);
        assert!(!data[1].0.is_reply() && data[1].1.is_empty());
        assert_eq!(data[0].1.len(), data_len);
        if offset == 0 {
            whole = data[0].1.clone();
        }
    }
    assert_eq!(download_data(&resumed)[0], &whole[0x1000..]);
}

#[test]
fn upload() {
    let (_, session) = transfer!("48784", "48796");
    let wire = one(&session, MessageType::UPLOAD_FILE);
    let request = UploadFile::decode(wire).unwrap();
    assert_eq!(request.data_size, 100);
    assert_eq!(request.strings.name, b"probe-ul.bin");
    assert_eq!(request.folder, FilePath::new(["Uploads [UL]"]));
    assert_eq!(request.encode().unwrap(), wire);

    let wire = one(&session, MessageType::UPLOAD_FILE_REPLY);
    let reply = UploadReply::decode(wire).unwrap();
    assert_eq!((reply.data_offset, reply.rsrc_offset), (0, u64::MAX));
    assert_eq!(reply.encode(), wire);

    let data = payloads(&session, MessageType::UPLOAD_FILE_DATA);
    assert_eq!(data.iter().map(|d| d.len()).collect::<Vec<_>>(), [100, 0]);
    assert!(data[0].iter().enumerate().all(|(i, &b)| b == i as u8));
    assert!(one(&session, MessageType::UPLOAD_FILE_COMPLETE).is_empty());

    let listing = payloads(&session, MessageType::FILE_LIST);
    let names: Vec<_> = FileListEntry::decode_list(listing[0])
        .unwrap()
        .records
        .into_iter()
        .map(|e| (e.name, e.size))
        .collect();
    assert_eq!(names, [(b"probe-ul.bin".to_vec(), 100)]);
}

#[test]
fn server_info() {
    let session = connection!("50770");
    let wire = one(&session, MessageType::EXTRA_SERVER_INFO);
    let info = ExtraServerInfo::decode(wire).unwrap();
    assert_eq!(info.uptime, 301);
    assert_eq!(info.now - info.start_time, 301_740);
    assert_eq!((info.os_value_24, info.os_value_28), (0x1000, 1000));
    assert!(info.os.starts_with(b"Linux "));
    assert_eq!(info.encode().unwrap(), wire);

    let wire = one(&session, MessageType::YOUR_ADDRESS);
    let address = YourAddress::decode(wire).unwrap();
    assert_eq!(address.address, b"127.0.0.1:50770");
    assert_eq!(address.encode().unwrap(), wire);
}

#[test]
fn connection_monitor() {
    let session = connection!("50984");
    let wire = one(&session, MessageType::CONNECTION_MONITOR_INFO);
    let list = MonitoredConnection::decode_list(wire).unwrap();
    let [me] = &list[..] else { panic!("{list:?}") };
    assert_eq!(me.user_id, 8);
    assert_eq!(me.nick, b"probe");
    assert_eq!(me.login, b"admin");
    assert_eq!(me.account, b"Administrator");
    assert_eq!(me.address, b"127.0.0.1:50984");
    // The server leaves the padding uninitialised; the codec keeps it.
    assert_eq!(me.padding, wire[wire.len() - 2..]);
    assert_eq!(MonitoredConnection::encode_list(&list).unwrap(), wire);
}

#[test]
fn account_classes() {
    let session = connection!("50770");
    let wire = one(&session, MessageType::ACCOUNT_CLASS_LIST);
    let list = AccountClass::decode_list(wire).unwrap();
    assert_eq!(list.records.len(), 12);
    let names: Vec<&[u8]> = list.records[..3]
        .iter()
        .map(|c| c.name.as_slice())
        .collect();
    assert_eq!(names, [&b"Administrator"[..], b"User", b"Guest"]);
    assert!(list
        .records
        .iter()
        .enumerate()
        .all(|(i, c)| usize::from(c.index) == i));
    assert_eq!(list.records[0].unknown_after_index, [0xff, 0, 0]);
    assert_eq!(list.records[1].unknown_after_index, [0, 0, 0xff]);
    assert_eq!(AccountClass::encode_list(&list).unwrap(), wire);

    let session = connection!("59006");
    let replies = payloads(&session, MessageType::ACCOUNT_CLASS_INFO);
    for (wire, index, unknown_25) in [(replies[0], 0, 0x50), (replies[1], 2, 0x28)] {
        let class = AccountClass::decode_info(wire).unwrap();
        assert_eq!(
            class,
            AccountClass {
                unknown_25,
                unknown_28: class.unknown_28,
                padding: class.padding.clone(),
                ..list.records[index].clone()
            }
        );
        // The server leaves the padding uninitialised; the codec keeps it.
        assert_eq!(class.padding, wire[0x38 + class.name.len()..]);
        assert_eq!(class.encode_info().unwrap(), wire);
    }
}

#[test]
fn accounts() {
    for session in [connection!("50770"), connection!("59006")] {
        let wire = one(&session, MessageType::ACCOUNT_LIST);
        let list = AccountListEntry::decode_list(wire).unwrap();
        let logins: Vec<&[u8]> = list.records.iter().map(|a| a.login.as_slice()).collect();
        assert_eq!(logins, [&b"admin"[..], b"guest"]);
        assert_eq!(
            (list.records[0].class_index, list.records[1].class_index),
            (0, 2)
        );
        assert_eq!(list.records[1].last_login, 0);
        assert_eq!(AccountListEntry::encode_list(&list).unwrap(), wire);
    }

    let session = connection!("59006");
    let wire = one(&session, MessageType::ACCOUNT_INFO);
    let account = Account::decode(wire).unwrap();
    assert_eq!(account.login, b"admin");
    assert_eq!(account.name, b"Administrator");
    assert_eq!(account.privilege_mask, account.privileges);
    assert!(account.has_password());
    assert_eq!(account.class_index, 0);
    assert!(account.editor.is_empty());
    assert_eq!(account.encode().unwrap(), wire);
}

#[test]
fn processes() {
    let session = connection!("59006");
    let wire = one(&session, MessageType::PROCESS_LIST);
    let list = ProcessList::decode(wire).unwrap();
    let pids: Vec<u32> = list.processes.iter().map(|p| p.pid).collect();
    assert_eq!(pids, [1, 2, 3]);
    assert_eq!(list.processes[1].command, b"./KDXServer.lexe");
    assert_eq!(list.processes[2].command, b"/bin/ps auxwww");
    assert_eq!(list.processes[0].command.len(), 0xff);
    assert_eq!(list.encode().unwrap(), wire);
}

#[test]
fn folder_access() {
    let session = connection!("59006");
    let wire = one(&session, MessageType::FOLDER_ACCESS_LIST);
    let list = FolderAccessItem::decode_list(wire).unwrap();
    let items: Vec<(u32, &[u8])> = list
        .records
        .iter()
        .map(|i| (i.icon, i.name.as_slice()))
        .collect();
    assert_eq!(
        items,
        [(0, &b"[Default]"[..]), (0x2c, b"DB"), (0x2b, b"UL")]
    );
    assert!(list.records.iter().all(|i| i.second.is_empty()));
    assert_eq!(list.records[1].description, b"Dropbox.");
    assert_eq!(FolderAccessItem::encode_list(&list).unwrap(), wire);
}

#[test]
fn newsgroup_info() {
    let session = connection!("32980");
    let wire = one(&session, MessageType::NEWSGROUP_INFO);
    let group = NewsgroupInfo::decode(wire).unwrap();
    assert_eq!(group.group_id, 1);
    assert_eq!(group.name, b"General News");
    assert_eq!(
        (group.read_classes, group.post_classes),
        (u32::MAX, u32::MAX)
    );
    assert_eq!((group.message_count, group.size_limit_kib), (1, 50));
    assert_eq!(group.encode().unwrap(), wire);
}

#[test]
fn greeting_and_registration() {
    let session = connection!("45802");
    // Frame 163, the answer to 112.
    let wire = one(&session, MessageType::SERVER_GREETING);
    let greeting = ServerGreeting::decode(wire).unwrap();
    assert_eq!(greeting.text, b"(No greeting.)");
    assert_eq!(greeting.encode(), wire);

    // Frame 165.
    let wire = one(&session, MessageType::UNNAMED_118);
    let notice = RegistrationNotice::decode(wire).unwrap();
    assert_eq!(notice.value, 1);
    assert!(notice.is_registered());
    assert_eq!(notice.encode(), wire);
}

#[test]
fn get_user_info() {
    // Frames 229 and 232; user 9 doesn't exist.
    let session = connection!("50984");
    let ids: Vec<_> = payloads(&session, MessageType::GET_USER_INFO)
        .into_iter()
        .map(|wire| {
            let request = GetUserInfo::decode(wire).unwrap();
            assert_eq!(request.encode(), wire);
            request.user_id
        })
        .collect();
    assert_eq!(ids, [8, 9]);
}

#[test]
fn default_server_settings() {
    // Frame 116: the settings of a new server.
    let session = connection!("32980");
    let wire = one(&session, MessageType::SERVER_SETTINGS_INFO);
    let settings = ServerSettings::decode(wire).unwrap();
    assert_eq!(settings.name(), b"New Server");
    assert_eq!(settings.greeting(), None);
    assert!(settings.icon().is_empty());
    assert_eq!(settings.speed_limit(), 0);
    assert_eq!(settings.encode(), wire);
}
