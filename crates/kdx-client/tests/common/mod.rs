//! A logged-in client against the mock, for the per-area tests.
#![allow(dead_code)]

use kdx_client::mock::{MockServer, Step};
use kdx_client::{Client, Login};
use kdx_proto::msg::{ServerInfo, UserRecord};
use kdx_proto::session::MessageType;
use kdx_proto::{AccountKey, ChallengeTable};

/// The session key the mock's 105 Server Info carries; transfer
/// connections log in with it.
pub const SESSION_KEY: [u8; 32] = [0xab; 32];

pub fn user(id: u32, nick: &str) -> UserRecord {
    UserRecord {
        user_id: id,
        client_port: 10600,
        nick: nick.as_bytes().to_vec(),
        ..UserRecord::default()
    }
}

pub fn server_info(user_id: u32) -> ServerInfo {
    ServerInfo {
        session_key: SESSION_KEY,
        user_id,
        version: 1620,
        can_modify_settings: 1,
        name: b"New Server".to_vec(),
        ..ServerInfo::default()
    }
}

pub fn login_steps() -> Vec<Step> {
    vec![
        Step::Expect(MessageType::SET_IDENTITY),
        Step::Send(
            MessageType::USER_LIST,
            UserRecord::encode_list(&[user(1, "lab")]).unwrap(),
        ),
        Step::Send(MessageType::SERVER_INFO, server_info(1).encode().unwrap()),
    ]
}

pub fn login() -> Login {
    Login {
        name: b"admin".to_vec(),
        password: b"pw".to_vec(),
    }
}

/// A mock that runs login then `steps` on the first connection, and `extra`
/// scripts on later ones (transfer connections log in with the session key).
pub async fn client_with_extra(steps: Vec<Step>, extra: Vec<Vec<Step>>) -> (MockServer, Client) {
    let mut script = login_steps();
    script.extend(steps);
    let mut scripts = vec![script];
    scripts.extend(extra);
    let mut transfer_login = vec![0u8];
    transfer_login.extend_from_slice(&1u32.to_be_bytes());
    let server = MockServer::start(
        vec![
            (b"admin".to_vec(), AccountKey::from_password(b"pw")),
            (transfer_login, AccountKey(SESSION_KEY)),
        ],
        scripts,
    )
    .await;
    let client = Client::login(
        server.addr,
        &login(),
        &user(0, "me"),
        ChallengeTable::default(),
    )
    .await
    .unwrap();
    (server, client)
}

pub async fn client_with(steps: Vec<Step>) -> (MockServer, Client) {
    client_with_extra(steps, Vec::new()).await
}
