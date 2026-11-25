//! Differential tests: the client against KDX Server 1.620 and Tracker 1.110
//! in the OpenKDX lab.
//!
//! They run only with `KDX_LAB_SERVER` set (`HOST:PORT`; see
//! `lab/differential.sh` in spitfire915/OpenKDX), which the lab runner does
//! with `--test-threads=1`: the tests share one server and one address, and
//! Server 1.620 bans an address after 30 connections in 300 seconds, so the
//! suite opens at most 25. `KDX_LAB_TRACKER` names the tracker,
//! `KDX_LAB_PASSWORD` the admin password (default `openkdx-lab`), and
//! `KDX_LAB_TRACKER_HAS_SERVER` asserts that the server is registered.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use kdx_client::{new_account, set_server_name, Client, Login, Notice, Tracker};
use kdx_proto::msg::admin::{DisconnectMessage, FolderAccess};
use kdx_proto::msg::{FilePath, Privilege, Privileges, UserRecord};
use kdx_proto::session::MessageType;
use kdx_proto::{ChallengeTable, Story};

struct Lab {
    server: SocketAddr,
    tracker: Option<SocketAddr>,
    password: Vec<u8>,
}

fn lab() -> Option<Lab> {
    let Ok(server) = std::env::var("KDX_LAB_SERVER") else {
        eprintln!("skipped: KDX_LAB_SERVER not set");
        return None;
    };
    Some(Lab {
        server: server.parse().expect("KDX_LAB_SERVER is HOST:PORT"),
        tracker: std::env::var("KDX_LAB_TRACKER")
            .ok()
            .map(|t| t.parse().expect("KDX_LAB_TRACKER is HOST:PORT")),
        password: std::env::var("KDX_LAB_PASSWORD")
            .unwrap_or_else(|_| "openkdx-lab".into())
            .into_bytes(),
    })
}

fn table() -> ChallengeTable {
    kdx_test_tables::table(Story::Kdx)
}

fn me(nick: &str) -> UserRecord {
    UserRecord {
        nick: nick.as_bytes().to_vec(),
        ..UserRecord::default()
    }
}

impl Lab {
    fn admin(&self) -> Login {
        Login {
            name: b"admin".to_vec(),
            password: self.password.clone(),
        }
    }

    async fn login_admin(&self, nick: &str) -> Client {
        Client::login(self.server, &self.admin(), &me(nick), table())
            .await
            .expect("admin login")
    }

    async fn login_guest(&self, nick: &str) -> Client {
        Client::login(self.server, &Login::default(), &me(nick), table())
            .await
            .expect("guest login")
    }
}

/// Waits up to 10 seconds for the notice `wanted` picks out.
async fn wait_for<T>(client: &mut Client, mut wanted: impl FnMut(Notice) -> Option<T>) -> T {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        let notice = tokio::time::timeout_at(deadline, client.next_notice())
            .await
            .expect("the notice in time")
            .expect("the connection stays up");
        if let Some(found) = wanted(notice) {
            return found;
        }
    }
}

fn tempdir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kdx-client-lab-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

macro_rules! lab_test {
    ($name:ident, |$lab:ident| $body:block) => {
        #[tokio::test]
        async fn $name() {
            let Some($lab) = lab() else { return };
            $body
        }
    };
}

lab_test!(lab_login_admin, |lab| {
    let mut c = lab.login_admin("admin-login").await;
    assert_eq!(c.info.version, 1620, "Server 1.620");
    assert!(c.info.user_id >= 1);
    assert_eq!(c.info.can_modify_settings, 1);
    assert!(c.users.iter().any(|u| u.user_id == c.info.user_id));
    assert!(!c.my_address().await.unwrap().is_empty());
});

lab_test!(lab_login_guest, |lab| {
    let c = lab.login_guest("guest-login").await;
    assert_eq!(c.info.can_modify_settings, 0);
});

lab_test!(lab_login_wrong_password, |lab| {
    let login = Login {
        name: b"admin".to_vec(),
        password: b"not-the-password".to_vec(),
    };
    let err = Client::login(lab.server, &login, &me("x"), table())
        .await
        .expect_err("a wrong password is refused");
    eprintln!("refused as: {err:#}");
});

lab_test!(lab_chat_say_and_echo, |lab| {
    let mut c = lab.login_admin("chatter").await;
    let list = c.chat_list().await.unwrap();
    let public = list.iter().find(|r| r.chat_id == 1).expect("Public Chat");
    let info = c.join_chat(public.chat_id, public.key).await.unwrap();
    assert!(info.members.contains(&c.info.user_id));
    c.say(1, b"hello from kdx", false).await.unwrap();
    let echo = wait_for(&mut c, |n| match n {
        Notice::ChatText(t) if t.text == b"hello from kdx" => Some(t),
        _ => None,
    })
    .await;
    assert_eq!(echo.user_id, c.info.user_id);
    assert!(!echo.is_action());
    c.say(1, b"waves", true).await.unwrap();
    let action = wait_for(&mut c, |n| match n {
        Notice::ChatText(t) if t.text == b"waves" => Some(t),
        _ => None,
    })
    .await;
    assert!(action.is_action(), "the /me flag survives the round trip");
    c.set_topic(1, b"kdx was here").await.unwrap();
    wait_for(&mut c, |n| match n {
        Notice::ChatTopicChanged(t) if t.topic == b"kdx was here" => Some(()),
        _ => None,
    })
    .await;
});

lab_test!(lab_pm_between_two_sessions, |lab| {
    let mut a = lab.login_admin("pm-a").await;
    let mut b = lab.login_guest("pm-b").await;
    a.send_private_message(b.info.user_id, b"psst")
        .await
        .unwrap();
    let pm = wait_for(&mut b, |n| match n {
        Notice::PrivateMessage(p) => Some(p),
        _ => None,
    })
    .await;
    assert_eq!(pm.user_id, a.info.user_id);
    assert_eq!(pm.text, b"psst");
});

lab_test!(lab_news_post_and_read, |lab| {
    let mut c = lab.login_admin("newsie").await;
    let groups = c.newsgroups().await.unwrap();
    let group = groups.first().expect("a newsgroup").group_id;
    let id = c
        .post_news_id(group, b"from kdx", b"line one\nline two")
        .await
        .unwrap();
    assert_ne!(id, 0, "the 613 arrives before the barrier");
    let msg = c.news_message(group, id).await.unwrap();
    assert_eq!(msg.subject, b"from kdx");
    assert_eq!(msg.body, b"line one\rline two");
    let list = c.news(group).await.unwrap();
    assert!(list.messages.iter().any(|m| m.message_id == id));
});

fn body(n: usize) -> Vec<u8> {
    let mut x: u32 = 0x9e37_79b9;
    (0..n)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            x as u8
        })
        .collect()
}

lab_test!(lab_files_list_get_put_resume, |lab| {
    let mut c = lab.login_admin("filer").await;
    let root = c.file_list(&FilePath::default()).await.unwrap();
    let uploads = root
        .iter()
        .find(|e| e.is_folder() && u32::from(e.access_item) == FolderAccess::ICON_UPLOADS)
        .expect("an [UL] folder");
    let folder = FilePath::new([&uploads.name]);
    let data = body(256 * 1024);
    let dir = tempdir();
    let local = dir.join("big.bin");
    std::fs::write(&local, &data).unwrap();
    let name = format!("kdx-{}.bin", std::process::id());

    // A partial upload: stop after 150000 bytes. The server keeps the
    // partial as `<name>.prt` and flushes it in large chunks, so what it
    // holds is some multiple of 0x1000 below what was sent.
    let err = c
        .upload_from(&local, &folder, name.as_bytes(), |_| {}, Some(150_000))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("stopped"), "{err:#}");
    tokio::time::sleep(Duration::from_millis(500)).await;
    let listed = c.file_list(&folder).await.unwrap();
    assert!(
        listed
            .iter()
            .any(|e| e.name == format!("{name}.prt").as_bytes()),
        "the partial is listed as .prt: {:?}",
        listed
            .iter()
            .map(|e| String::from_utf8_lossy(&e.name).into_owned())
            .collect::<Vec<_>>()
    );

    // Resume from 416's offset.
    let mut first = None;
    c.upload_from(
        &local,
        &folder,
        name.as_bytes(),
        |p| {
            first.get_or_insert(p.done);
        },
        None,
    )
    .await
    .unwrap();
    let first = first.unwrap();
    assert!(
        first > 0 && first % 0x1000 == 0 && first <= 150_000,
        "416 data_offset {first:#x}"
    );

    let listed = c.file_list(&folder).await.unwrap();
    let entry = listed
        .iter()
        .find(|e| e.name == name.as_bytes())
        .expect("the upload is listed");
    assert_eq!(entry.size, data.len() as u64);

    // Download it whole, then resume a truncated copy.
    let mut remote = folder.clone();
    remote.0.push(name.clone().into_bytes());
    let dest = dir.join("down.bin");
    let info = c.download_to(&remote, &dest, false, |_| {}).await.unwrap();
    assert_eq!(info.data_size, data.len() as u64);
    assert_eq!(std::fs::read(&dest).unwrap(), data);
    std::fs::write(&dest, &data[..5000]).unwrap();
    c.download_to(&remote, &dest, true, |_| {}).await.unwrap();
    assert_eq!(std::fs::read(&dest).unwrap(), data);
});

lab_test!(lab_accounts_create_show_delete, |lab| {
    let mut c = lab.login_admin("accountant").await;
    let login = format!("tester{}", std::process::id() % 1000);
    let privs = Privileges::of(&[Privilege::CanLogin, Privilege::Download]);
    let account = new_account(login.as_bytes(), b"Tester", Some(b"t"), 1, privs, privs);
    c.create_account(&account).await.unwrap();
    let shown = c.account(login.as_bytes()).await.unwrap();
    assert_eq!(shown.name, b"Tester");
    assert!(shown.has_password());
    assert_eq!(shown.class_index, 1);
    assert!(c
        .accounts()
        .await
        .unwrap()
        .iter()
        .any(|a| a.login == login.as_bytes()));
    let mut changed = shown.clone();
    changed.key = [0; 32];
    changed.name = b"Tester 2".to_vec();
    c.set_account(&changed).await.unwrap();
    assert_eq!(c.account(login.as_bytes()).await.unwrap().name, b"Tester 2");
    c.delete_account(login.as_bytes()).await.unwrap();
    assert!(!c
        .accounts()
        .await
        .unwrap()
        .iter()
        .any(|a| a.login == login.as_bytes()));
});

lab_test!(lab_classes_list_and_show, |lab| {
    let mut c = lab.login_admin("classy").await;
    let classes = c.account_classes().await.unwrap();
    assert!(!classes.is_empty());
    let first = c.class(classes[0].index).await.unwrap();
    assert_eq!(first.index, classes[0].index);
});

lab_test!(lab_folder_access_list_create_delete, |lab| {
    let mut c = lab.login_admin("fa").await;
    let before = c.folder_access_list().await.unwrap();
    assert!(before.iter().any(|i| i.icon == FolderAccess::ICON_UPLOADS));
    let name = format!("Pictures{}", std::process::id() % 1000);
    c.create_folder_access(&FolderAccess {
        icon: FolderAccess::ICON_PICTURES,
        name: name.clone().into_bytes(),
        description: b"pics".to_vec(),
        ..FolderAccess::default()
    })
    .await
    .unwrap();
    let shown = c.folder_access(name.as_bytes()).await.unwrap();
    assert_eq!(shown.description, b"pics");
    c.delete_folder_access(name.as_bytes()).await.unwrap();
    let after = c.folder_access_list().await.unwrap();
    assert!(!after.iter().any(|i| i.name == name.as_bytes()));
});

lab_test!(lab_server_settings_set_name_round_trip, |lab| {
    let mut c = lab.login_admin("settings").await;
    let original = c.server_settings().await.unwrap();
    let old_name = original.main.name.clone();
    let mut changed = original.clone();
    set_server_name(&mut changed, b"Renamed Lab").unwrap();
    c.set_server_settings(&changed).await.unwrap();
    let again = c.server_settings().await.unwrap();
    assert_eq!(again.main.name, b"Renamed Lab");
    let mut restore = again;
    set_server_name(&mut restore, &old_name).unwrap();
    c.set_server_settings(&restore).await.unwrap();
});

lab_test!(lab_broadcast_received_by_other_session, |lab| {
    let mut a = lab.login_admin("announcer").await;
    let mut b = lab.login_guest("listener").await;
    a.broadcast(b"hello all").await.unwrap();
    let got = wait_for(&mut b, |n| match n {
        Notice::Broadcast(r) => Some(r),
        _ => None,
    })
    .await;
    assert_eq!(got.text, b"hello all");
});

lab_test!(lab_disconnect_from_victim_side, |lab| {
    let mut admin = lab.login_admin("bouncer").await;
    let mut victim = lab.login_guest("victim").await;
    admin
        .disconnect_user(victim.info.user_id, 5, b"bye", false)
        .await
        .unwrap();
    let bye = wait_for(&mut victim, |n| match n {
        Notice::Disconnect(d) => Some(d),
        _ => None,
    })
    .await;
    assert_eq!(bye.message, b"bye");
    assert!(bye.flags & DisconnectMessage::BANNED != 0);
    // The server closes after the 151.
    let closed = tokio::time::timeout(Duration::from_secs(10), victim.next_event())
        .await
        .expect("the close in time");
    assert!(closed.is_err(), "the connection is closed after the 151");
    // The ban is by address, and the admin shares 127.0.0.1, so a reconnect
    // test would ban the whole suite; `lab/scenarios/disconnect-ban.sh`
    // shows the ban itself.
});

lab_test!(lab_history_join_sees_login_record, |lab| {
    let mut c = lab.login_admin("historian").await;
    let list = c.history_join().await.unwrap();
    assert!(
        list.records
            .iter()
            .any(|r| r.kind == MessageType::HISTORY_LOGIN),
        "a Login record"
    );
    let other = lab.login_guest("passer-by").await;
    let added = wait_for(&mut c, |n| match n {
        Notice::HistoryAdded(r) if r.kind == MessageType::HISTORY_LOGIN => Some(r),
        _ => None,
    })
    .await;
    assert_eq!(added.user_id, other.info.user_id);
    c.history_leave().await.unwrap();
});

lab_test!(lab_connection_monitor_lists_sessions, |lab| {
    let mut c = lab.login_admin("monitor").await;
    c.monitor_open().await.unwrap();
    let info = c.monitor_info().await.unwrap();
    assert!(info.iter().any(|r| r.user_id == c.info.user_id));
    c.monitor_close().await.unwrap();
});

lab_test!(lab_tracker_groups_and_servers, |lab| {
    let Some(addr) = lab.tracker else {
        eprintln!("skipped: KDX_LAB_TRACKER not set");
        return;
    };
    let mut t = Tracker::login(addr, &Login::default(), table())
        .await
        .unwrap();
    let groups = t.groups().await.unwrap();
    assert!(!groups.is_empty());
    let servers = t.servers(&groups[0].name).await.unwrap();
    // A fresh lab server has no tracker configured (that takes the GUI's
    // Server Settings; see lab/scenarios/tracker-basic.sh), so the list is
    // usually empty here; `captures/tracker-basic` shows a registered one.
    if std::env::var("KDX_LAB_TRACKER_HAS_SERVER").is_ok() {
        assert!(
            servers.iter().any(|s| s.port == lab.server.port()),
            "the lab server registered: {servers:?}"
        );
    }
});
