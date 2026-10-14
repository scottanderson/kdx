//! Checks against `captures/kdx-accounts.pcapng`: the `kdx` CLI managing accounts on Server 1.620.

use kdx_proto::msg::admin::Account;
use kdx_proto::msg::Status;
use kdx_proto::session::{Header, MessageType};
use kdx_proto::AccountKey;

mod common;

macro_rules! connection {
    ($port:literal) => {
        connection!($port, common::PASSWORD)
    };
    ($port:literal, $password:expr) => {
        common::session_with_key(
            include_bytes!(concat!("fixtures/kdx-accounts-", $port, ".c2s.bin")),
            include_bytes!(concat!("fixtures/kdx-accounts-", $port, ".s2c.bin")),
            &AccountKey::from_password($password),
        )
    };
}

fn one(messages: &[(Header, Vec<u8>)], kind: MessageType) -> (Header, &[u8]) {
    let found: Vec<_> = messages.iter().filter(|(h, _)| h.kind == kind).collect();
    assert_eq!(found.len(), 1, "{kind}");
    (found[0].0, &found[0].1)
}

fn success(messages: &[(Header, Vec<u8>)]) -> u32 {
    match Status::decode(one(messages, MessageType::STATUS).1).unwrap() {
        Status::Success { value } => value,
        error => panic!("{error:?}"),
    }
}

#[test]
fn create() {
    let session = connection!("58284");
    let (_, wire) = one(&session, MessageType::CREATE_ACCOUNT);
    let account = Account::decode(wire).unwrap();
    assert_eq!(account.login, b"rs-test");
    assert_eq!(account.name, b"RS Test");
    assert_eq!(account.class_index, 1);
    assert_eq!(account.key, AccountKey::from_password(b"rs-test-pw").0);
    assert_eq!(account.encode().unwrap(), wire);
    // The success value is the request's type.
    assert_eq!(success(&session), 507);
}

#[test]
fn account_info() {
    let session = connection!("58290");
    let (_, wire) = one(&session, MessageType::ACCOUNT_INFO);
    let account = Account::decode(wire).unwrap();
    assert_eq!(account.login, b"rs-test");
    assert_eq!(account.editor, b"admin");
    assert!(account.has_password());
    assert_eq!(account.key[1..], [0; 31]);
    assert_eq!(account.created, account.modified);
    assert_eq!(account.last_login, 0);
    assert_eq!(account.encode().unwrap(), wire);

    let session = connection!("50184");
    let (_, wire) = one(&session, MessageType::ACCOUNT_INFO);
    let account = Account::decode(wire).unwrap();
    assert_eq!(account.login, b"rs-nopw");
    assert!(!account.has_password());
    // The server leaves the padding uninitialised; the codec keeps it.
    assert!(account.padding.iter().any(|&b| b != 0));
    assert_eq!(account.encode().unwrap(), wire);
}

#[test]
fn rename() {
    let session = connection!("47988");
    let (_, wire) = one(&session, MessageType::SET_ACCOUNT);
    let account = Account::decode(wire).unwrap();
    assert_eq!(account.editor, b"rs-test");
    assert_eq!(account.login, b"rs-test2");
    assert_eq!(account.key, [0; 32]);
    assert_eq!(account.encode().unwrap(), wire);
    assert_eq!(success(&session), 511);
}

#[test]
fn change_own_password() {
    let session = connection!("58336", b"rs-test-pw");
    let (_, wire) = one(&session, MessageType::CHANGE_OWN_PASSWORD);
    assert_eq!(wire, AccountKey::from_password(b"rs-test-pw2").0);
    let (header, wire) = one(&session, MessageType::CHANGE_OWN_PASSWORD_DONE);
    assert!(header.is_reply());
    assert!(wire.is_empty());
}

#[test]
fn delete() {
    let session = connection!("50188");
    let (_, wire) = one(&session, MessageType::DELETE_ACCOUNT);
    assert_eq!(wire, b"rs-test2");
    // The deleter gets 515 as a notification, not a reply.
    let (header, wire) = one(&session, MessageType::ACCOUNT_DELETED);
    assert!(!header.is_reply());
    assert_eq!(wire, b"rs-test2");
    assert!(session.iter().all(|(h, _)| h.kind != MessageType::STATUS));

    let session = connection!("33090");
    assert!(matches!(
        Status::decode(one(&session, MessageType::STATUS).1).unwrap(),
        Status::Error { code: 0x1000c, .. }
    ));
}
