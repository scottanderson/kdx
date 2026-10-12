//! Checks against `captures/linux-catalog-volumes.pcapng`: Client 1.620 for Linux as `admin`.

use kdx_proto::msg::admin::MonitoredConnection;
use kdx_proto::msg::FilePath;
use kdx_proto::session::MessageType;

mod common;

/// The 143 payload with each record's padding zeroed; the server leaves it uninitialised.
fn zero_padding(wire: &[u8]) -> Vec<u8> {
    let mut out = wire.to_vec();
    let mut at = 0;
    while at < wire.len() {
        let record = &wire[at..];
        let strings: usize = record[0x60..0x64].iter().map(|&n| usize::from(n)).sum();
        let path = usize::from(u16::from_be_bytes([record[0x66], record[0x67]]));
        let end = at + MonitoredConnection::HEADER_LEN + strings + path;
        let padded = (end + 3) & !3;
        out[end..padded].fill(0);
        at = padded;
    }
    out
}

#[test]
fn connection_monitor_with_a_transfer() {
    let session = common::session(
        include_bytes!("fixtures/linux-catalog-volumes-58160.c2s.bin"),
        include_bytes!("fixtures/linux-catalog-volumes-58160.s2c.bin"),
    );
    let replies: Vec<_> = session
        .iter()
        .filter(|(h, _)| h.kind == MessageType::CONNECTION_MONITOR_INFO)
        .map(|(_, p)| p.as_slice())
        .collect();
    assert_eq!(replies.len(), 24);
    let mut transfers = 0;
    for wire in &replies {
        let list = MonitoredConnection::decode_list(wire).unwrap();
        assert_eq!(
            MonitoredConnection::encode_list(&list).unwrap(),
            zero_padding(wire)
        );
        for record in list.iter().filter(|r| r.transfer_id != 0) {
            assert_eq!((record.value_38, record.state), (0x5000, 412));
            transfers += 1;
        }
    }
    assert_eq!(transfers, 20);

    let wire = replies.iter().find(|w| w.len() == 452).unwrap();
    let list = MonitoredConnection::decode_list(wire).unwrap();
    let [me, peer, transfer] = &list[..] else {
        panic!("{list:?}")
    };
    assert_eq!((me.user_id, me.transfer_id), (1, 0));
    assert_eq!(me.nick, b"lab-linux");
    assert_eq!((peer.user_id, peer.transfer_id), (2, 0));
    assert_eq!(peer.address, b"127.0.0.1:53680");
    assert_eq!(peer.path, FilePath::default());

    assert_eq!((transfer.user_id, transfer.transfer_id), (2, 3));
    assert_eq!(transfer.nick, b"kdx-get");
    assert_eq!(transfer.address, b"127.0.0.1:53692");
    assert_eq!(transfer.path, FilePath::new(["speed.bin"]));
    assert_eq!(transfer.bytes_done, 0x14000);
    assert_eq!(transfer.bytes_total, 0x20_0000);
    assert_eq!(transfer.rate, 0x6aaa);
    assert_eq!(
        u64::from(transfer.seconds_left),
        (transfer.bytes_total - transfer.bytes_done) / u64::from(transfer.rate)
    );
    assert_eq!(transfer.value_38, 0x5000);
    assert_eq!(transfer.state, 412);
}
