//! Checks against `captures/cli-connection-monitor.pcapng`: 143 while another session downloads 4 MiB.

use kdx_proto::msg::admin::MonitoredConnection;
use kdx_proto::msg::FilePath;
use kdx_proto::session::MessageType;
use kdx_proto::AccountKey;

mod common;

/// Frame 70: beside the session records, one transfer record with its ID, total and path.
#[test]
fn transfer_row_has_id_total_and_path() {
    let session = common::session_with_key(
        include_bytes!("fixtures/cli-connection-monitor-44682.c2s.bin"),
        include_bytes!("fixtures/cli-connection-monitor-44682.s2c.bin"),
        &AccountKey::from_password(common::LAB_PASSWORD),
    );
    let (_, wire) = session
        .iter()
        .find(|(h, _)| h.kind == MessageType::CONNECTION_MONITOR_INFO)
        .expect("a 143");
    let records = MonitoredConnection::decode_list(wire).unwrap();
    let transfers: Vec<_> = records.iter().filter(|r| r.transfer_id != 0).collect();
    assert_eq!(transfers.len(), 1, "{records:#?}");
    let t = transfers[0];
    assert_eq!(t.bytes_total, 4 * 1024 * 1024);
    assert_eq!(t.path, FilePath::new(["four.bin"]));
    assert!(
        records.iter().any(|r| r.transfer_id == 0),
        "session records too"
    );
}
