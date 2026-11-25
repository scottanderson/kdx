//! Account and class requests against the mock.

mod common;

use kdx_client::mock::Step;
use kdx_client::{format_privileges, new_account, parse_privileges, privilege_slug, ServerError};
use kdx_proto::msg::admin::{Account, AccountClass, AccountListEntry};
use kdx_proto::msg::{Privilege, Privileges, Status};
use kdx_proto::session::MessageType;
use kdx_proto::AccountKey;

fn barrier() -> [Step; 2] {
    [
        Step::Expect(MessageType::GET_MY_ADDRESS),
        Step::Reply(MessageType::YOUR_ADDRESS, vec![127, 0, 0, 1]),
    ]
}

#[test]
fn parse_privileges_round_trips() {
    let p = parse_privileges("can-login,download,upload").unwrap();
    assert!(p.contains(Privilege::CanLogin));
    assert!(p.contains(Privilege::Download));
    assert!(!p.contains(Privilege::Broadcast));
    assert_eq!(format_privileges(&p), "can-login,download,upload");
    assert_eq!(format_privileges(&Privileges::NONE), "none");
    assert_eq!(parse_privileges("none").unwrap(), Privileges::NONE);
    assert_eq!(parse_privileges("all").unwrap(), !Privileges::NONE);
    assert!(parse_privileges("fly").is_err());
    assert_eq!(
        parse_privileges(" Can-Login , UPLOAD ").unwrap(),
        parse_privileges("can-login,upload").unwrap()
    );
    assert_eq!(
        privilege_slug(Privilege::ColorOverridesClass),
        "color-overrides-class"
    );
    assert_eq!(
        parse_privileges("bit4").unwrap(),
        Privileges::of(&[Privilege::CanLogin])
    );
}

#[test]
fn new_account_sets_key_from_password() {
    let a = new_account(
        b"tester",
        b"Tester",
        Some(b"t"),
        1,
        Privileges::NONE,
        Privileges::NONE,
    );
    assert_eq!(a.key, AccountKey::from_password(b"t").0);
    assert_eq!(a.class_index, 1);
    assert_eq!(a.login, b"tester");
    let keep = new_account(
        b"tester",
        b"Tester",
        None,
        1,
        Privileges::NONE,
        Privileges::NONE,
    );
    assert_eq!(keep.key, [0; 32]);
    // Milliseconds since 1900: well past the Unix epoch offset.
    assert!(a.created > 2_208_988_800_000);
    assert_eq!(a.created, a.modified);
}

#[tokio::test]
async fn account_list_show_set_and_delete() {
    let shown = new_account(
        b"tester",
        b"Tester",
        Some(b"t"),
        1,
        Privileges::NONE,
        Privileges::NONE,
    );
    let entry = AccountListEntry {
        class_index: 1,
        login: b"tester".to_vec(),
        name: b"Tester".to_vec(),
        padding: vec![0; 2],
        ..AccountListEntry::default()
    };
    let mut steps = vec![
        Step::Expect(MessageType::GET_ACCOUNT_LIST),
        Step::Reply(
            MessageType::ACCOUNT_LIST,
            AccountListEntry::encode_list(&[entry]).unwrap(),
        ),
        Step::Expect(MessageType::GET_ACCOUNT),
        Step::Reply(MessageType::ACCOUNT_INFO, shown.encode().unwrap()),
        Step::Expect(MessageType::SET_ACCOUNT),
        Step::Reply(
            MessageType::STATUS,
            Status::Success { value: 0x1ff }.encode(),
        ),
        Step::Expect(MessageType::SET_ACCOUNT),
        Step::Reply(
            MessageType::STATUS,
            Status::Success { value: 0x1ff }.encode(),
        ),
        Step::Expect(MessageType::DELETE_ACCOUNT),
        Step::Send(MessageType::ACCOUNT_DELETED, b"tester".to_vec()),
    ];
    steps.extend(barrier());
    let (server, mut client) = common::client_with(steps).await;
    let list = client.accounts().await.unwrap();
    assert_eq!(list[0].login, b"tester");
    let acct = client.account(b"tester").await.unwrap();
    assert_eq!(acct.name, b"Tester");
    let mut changed = acct.clone();
    changed.name = b"Tester 2".to_vec();
    changed.editor = b"admin".to_vec(); // as 510 returns it: the last editor
    client.set_account(&changed).await.unwrap();
    client
        .rename_account(
            b"tester",
            &Account {
                login: b"tester2".to_vec(),
                ..changed
            },
        )
        .await
        .unwrap();
    client.delete_account(b"tester").await.unwrap();
    drop(client);
    server.finish().await;

    let raw = &server.payloads(MessageType::SET_ACCOUNT);
    let sent = Account::decode(&raw[0]).unwrap();
    assert_eq!(sent.name, b"Tester 2");
    assert_eq!(
        sent.editor, b"tester",
        "511 names the current login in editor"
    );
    assert_eq!(raw[0].len() % 4, 0, "records go out padded to 4");
    let renamed = Account::decode(&raw[1]).unwrap();
    assert_eq!(
        (&renamed.login[..], &renamed.editor[..]),
        (&b"tester2"[..], &b"tester"[..])
    );
    assert_eq!(server.payloads(MessageType::DELETE_ACCOUNT)[0], b"tester");
}

#[tokio::test]
async fn success_status_reply_is_not_an_error_and_is_correlated_by_txid() {
    let stray = Status::Error {
        code: 0x10014,
        request: MessageType::GET_USER_INFO,
        module: 0,
        line: 0,
        extra: 0,
        value_14: 0,
    };
    let (server, mut client) = common::client_with(vec![
        Step::Expect(MessageType::CREATE_ACCOUNT),
        // A Status for another transaction, pushed with the server's own txid.
        Step::Send(MessageType::STATUS, stray.encode()),
        Step::Reply(
            MessageType::STATUS,
            Status::Success { value: 0x1fb }.encode(),
        ),
    ])
    .await;
    let account = new_account(
        b"new",
        b"New",
        Some(b"n"),
        1,
        Privileges::NONE,
        Privileges::NONE,
    );
    client.create_account(&account).await.unwrap();
    let (header, payload) = client.next_event().await.unwrap();
    assert_eq!(header.kind, MessageType::STATUS);
    assert_eq!(Status::decode(&payload).unwrap(), stray);
    drop(client);
    server.finish().await;
}

#[tokio::test]
async fn send_checked_accepts_a_success_status_and_names_errors() {
    let denied = Status::Error {
        code: 0x10014,
        request: MessageType::DELETE_ACCOUNT,
        module: 0,
        line: 0,
        extra: 0,
        value_14: 0,
    };
    let mut steps = vec![
        Step::Expect(MessageType::DELETE_ACCOUNT),
        Step::Reply(
            MessageType::STATUS,
            Status::Success { value: 0x1fc }.encode(),
        ),
    ];
    steps.extend(barrier());
    steps.push(Step::Expect(MessageType::DELETE_ACCOUNT));
    steps.push(Step::Reply(MessageType::STATUS, denied.encode()));
    steps.extend(barrier());
    let (server, mut client) = common::client_with(steps).await;
    client.delete_account(b"gone").await.unwrap();
    let err = client.delete_account(b"admin").await.unwrap_err();
    let err = err.downcast_ref::<ServerError>().unwrap();
    assert_eq!(err.code, 0x10014);
    assert_eq!(err.name(), Some("Access Denied (a missing privilege)"));
    assert_eq!(
        kdx_proto::msg::status::error_name(0x40077),
        Some("Banned for flooding.")
    );
    assert_eq!(
        kdx_proto::msg::status::error_name(0x20065),
        Some("memory errors")
    );
    assert_eq!(kdx_proto::msg::status::error_name(0x99999), None);
    drop(client);
    server.finish().await;
}

#[tokio::test]
async fn class_get_and_set() {
    let class = AccountClass {
        index: 2,
        name: b"Members".to_vec(),
        privileges: Privileges::of(&[Privilege::CanLogin]),
        ..AccountClass::default()
    };
    let (server, mut client) = common::client_with(vec![
        Step::Expect(MessageType::GET_ACCOUNT_CLASS),
        Step::Reply(
            MessageType::ACCOUNT_CLASS_INFO,
            class.encode_info().unwrap(),
        ),
        Step::Expect(MessageType::SET_ACCOUNT_CLASS),
        Step::Reply(
            MessageType::STATUS,
            Status::Success { value: 0x1f8 }.encode(),
        ),
    ])
    .await;
    let got = client.class(2).await.unwrap();
    assert_eq!(got.name, b"Members");
    client.set_class(&got).await.unwrap();
    drop(client);
    server.finish().await;
    assert_eq!(server.payloads(MessageType::GET_ACCOUNT_CLASS)[0], [2]);
    let raw = &server.payloads(MessageType::SET_ACCOUNT_CLASS)[0];
    assert_eq!(raw.len() % 4, 0, "records go out padded to 4");
    assert_eq!(AccountClass::decode_set(raw).unwrap().name, b"Members");
}
