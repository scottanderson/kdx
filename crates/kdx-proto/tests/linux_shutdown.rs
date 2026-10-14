//! Checks against `captures/linux-shutdown.pcapng`: Client 1.620 for Linux as `admin`.

use kdx_proto::msg::admin::{DisconnectMessage, ExitTerminateProcess, ProcessList, ShutdownServer};
use kdx_proto::msg::Status;
use kdx_proto::session::{Header, MessageType};

mod common;

/// The server's PID in the sandbox.
const SERVER_PID: u32 = 2;

/// The first connection: 175, the process monitor, and 804.
fn upgrade_exit() -> Vec<(Header, Vec<u8>)> {
    common::session(
        include_bytes!("fixtures/linux-shutdown-49124.c2s.bin"),
        include_bytes!("fixtures/linux-shutdown-49124.s2c.bin"),
    )
}

/// The second connection, after the restart: 174 and its 151.
fn shutdown() -> Vec<(Header, Vec<u8>)> {
    common::session(
        include_bytes!("fixtures/linux-shutdown-38848.c2s.bin"),
        include_bytes!("fixtures/linux-shutdown-38848.s2c.bin"),
    )
}

fn one(messages: &[(Header, Vec<u8>)], kind: MessageType) -> (Header, &[u8]) {
    let found: Vec<_> = messages.iter().filter(|(h, _)| h.kind == kind).collect();
    assert_eq!(found.len(), 1, "{kind}");
    (found[0].0, &found[0].1)
}

#[test]
fn upgrade_server_software() {
    let session = upgrade_exit();
    // Frame 19 is empty with both restart options unticked.
    let (request, wire) = one(&session, MessageType::UPGRADE_SERVER_SOFTWARE);
    assert!(wire.is_empty());
    // Frame 20: the lab has no upgrade helper.
    let (reply, wire) = one(&session, MessageType::STATUS);
    assert_eq!(reply.txid, request.txid);
    assert_eq!(
        Status::decode(wire).unwrap(),
        Status::Error {
            code: 0x30067,
            request: MessageType::UPGRADE_SERVER_SOFTWARE,
            module: 0x46,
            line: 0x198,
            extra: 0,
            value_14: 0,
        }
    );
}

#[test]
fn exit_process() {
    let session = upgrade_exit();
    // Frame 23 lists the server as PID 2.
    let (_, wire) = one(&session, MessageType::PROCESS_LIST);
    let list = ProcessList::decode(wire).unwrap();
    assert_eq!(list.processes[1].pid, SERVER_PID);
    assert_eq!(list.processes[1].command, b"./KDXServer.lexe");
    assert_eq!(list.encode().unwrap(), wire);

    // Frame 25: Exit Process sends SIGTERM, and the server sends no reply.
    let (header, wire) = one(&session, MessageType::EXIT_TERMINATE_PROCESS);
    let request = ExitTerminateProcess::decode(wire).unwrap();
    assert_eq!(request.flags, ExitTerminateProcess::EXIT);
    assert_eq!(request.pid, SERVER_PID);
    assert_eq!(request.value_08, 0);
    assert_eq!(request.encode(), wire);
    assert!(session
        .iter()
        .all(|(h, _)| h.txid != header.txid || !h.is_reply()));
}

#[test]
fn shutdown_server() {
    let session = shutdown();
    // Frame 47: exit the program only, with the message to the end of the body.
    let (_, wire) = one(&session, MessageType::SHUTDOWN_SERVER);
    let request = ShutdownServer::decode(wire).unwrap();
    assert_eq!(request.mode, ShutdownServer::EXIT);
    assert_eq!(request.message, b"lab shutdown");
    assert_eq!(request.encode().unwrap(), wire);

    // Frame 48 carries the message to every user.
    let (header, wire) = one(&session, MessageType::DISCONNECT_MESSAGE);
    assert!(!header.is_reply());
    let notice = DisconnectMessage::decode(wire).unwrap();
    assert_eq!(notice.flags, DisconnectMessage::SHUTDOWN);
    assert_eq!(notice.user_id, 1);
    assert_eq!(notice.message, request.message);
    assert_eq!(notice.name, "Zoë-日本".as_bytes());
    assert_eq!(notice.encode().unwrap(), wire);
}

#[test]
fn shutdown_message_limit() {
    let mut request = ShutdownServer {
        mode: ShutdownServer::EXIT,
        message: vec![b'x'; ShutdownServer::MAX_MESSAGE_LEN],
        ..ShutdownServer::default()
    };
    assert!(request.encode().is_ok());
    request.message.push(b'x');
    assert!(request.encode().is_err());
}
