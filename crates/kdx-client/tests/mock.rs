//! The mock server runs a real handshake and reports script violations.

mod common;

use kdx_client::mock::{MockServer, Step};
use kdx_client::{Client, Login};
use kdx_proto::session::MessageType;
use kdx_proto::{AccountKey, ChallengeTable};

#[tokio::test]
async fn mock_logs_a_client_in() {
    let (server, client) = common::client_with(Vec::new()).await;
    assert_eq!(client.info.name, b"New Server");
    assert_eq!(client.users[0].nick, b"lab");
    assert_eq!(server.payloads(MessageType::SET_IDENTITY).len(), 1);
    drop(client);
    server.finish().await;
}

#[tokio::test]
#[should_panic(
    expected = "mock script violated: [\"expected 214 Get Chat List, got 100 Set Identity\"]"
)]
async fn mock_runs_the_handshake_and_records_violations() {
    let server = MockServer::start(
        vec![(b"admin".to_vec(), AccountKey::from_password(b"pw"))],
        vec![vec![Step::Expect(MessageType::GET_CHAT_LIST)]],
    )
    .await;
    // The script closes on the violation, so the login fails.
    let result = Client::login(
        server.addr,
        &common::login(),
        &common::user(0, "me"),
        ChallengeTable::default(),
    )
    .await;
    assert!(result.is_err());
    server.finish().await;
}

#[tokio::test]
async fn mock_rejects_an_unknown_login() {
    let server = MockServer::start(vec![], vec![vec![]]).await;
    let result = Client::login(
        server.addr,
        &Login {
            name: b"nobody".to_vec(),
            password: b"x".to_vec(),
        },
        &common::user(0, "me"),
        ChallengeTable::default(),
    )
    .await;
    assert!(result.is_err());
    server.finish().await;
}
