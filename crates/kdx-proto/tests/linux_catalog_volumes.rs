//! Checks against `captures/linux-catalog-volumes.pcapng`: Client 1.620 for Linux as `admin`.

use kdx_proto::msg::admin::{AccountClass, MonitoredConnection};
use kdx_proto::msg::file::{
    CatalogStatus, FileSearch, FileSearchEntry, GenerateCatalog, LaunchFile, VolumeInfo, ATTR_FILE,
};
use kdx_proto::msg::{FilePath, Status};
use kdx_proto::session::{Header, MessageType};

mod common;

fn session() -> Vec<(Header, Vec<u8>)> {
    common::session(
        include_bytes!("fixtures/linux-catalog-volumes-58160.c2s.bin"),
        include_bytes!("fixtures/linux-catalog-volumes-58160.s2c.bin"),
    )
}

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
    let session = session();
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

#[test]
fn set_account_class() {
    let session = session();
    let sets: Vec<_> = session
        .iter()
        .filter(|(h, _)| h.kind == MessageType::SET_ACCOUNT_CLASS)
        .collect();
    let lists: Vec<_> = session
        .iter()
        .filter(|(h, _)| h.kind == MessageType::ACCOUNT_CLASS_LIST)
        .collect();
    // The first 506 answers the Account Classes window; one follows each save.
    assert_eq!((sets.len(), lists.len()), (2, 3));
    // Frame 40 ticks See All Volumes (bit 22, e8 to ea in byte 2); frame 59 unticks it.
    for ((header, wire), (_, list), byte_2) in
        [(sets[0], lists[1], 0xea), (sets[1], lists[2], 0xe8)]
    {
        // 69 bytes, with no padding.
        assert_eq!(wire.len(), 69);
        let class = AccountClass::decode_set(wire).unwrap();
        let mut privileges = *b"\x28\x31\xe8\x5b\xf4\xd0\x3c\x0d\xc3\xe0\x80\0\0\0\0\0";
        privileges[2] = byte_2;
        assert_eq!(
            class,
            AccountClass {
                privileges,
                time: 0,
                unknown_25: 0x50,
                index: 0,
                unknown_after_index: [0xff, 0, 0],
                name: b"Administrator".to_vec(),
            }
        );
        assert_eq!(class.encode_set().unwrap(), *wire);

        // The success value is 0x1f8, which is 504.
        let (_, status) = session
            .iter()
            .find(|(h, _)| h.kind == MessageType::STATUS && h.txid == header.txid)
            .unwrap();
        assert_eq!(
            Status::decode(status).unwrap(),
            Status::Success { value: 504 }
        );

        // The 506 after it holds the saved class, with the time set by the server.
        let saved = &AccountClass::decode_list(list).unwrap()[0];
        assert_ne!(saved.time, 0);
        assert_eq!(saved.privileges, class.privileges);
        assert_eq!(saved.name, class.name);
    }
}

fn all(messages: &[(Header, Vec<u8>)], kind: MessageType) -> Vec<(Header, &[u8])> {
    messages
        .iter()
        .filter(|(h, _)| h.kind == kind)
        .map(|(h, p)| (*h, p.as_slice()))
        .collect()
}

fn one(messages: &[(Header, Vec<u8>)], kind: MessageType) -> (Header, &[u8]) {
    let found = all(messages, kind);
    assert_eq!(found.len(), 1, "{kind}");
    found[0]
}

#[test]
fn generate_catalog() {
    let session = session();
    // Frame 23.
    let (request, wire) = one(&session, MessageType::GENERATE_CATALOG);
    let catalog = GenerateCatalog::decode(wire).unwrap();
    assert_eq!(catalog, GenerateCatalog::default());
    assert_eq!(catalog.encode(), wire);

    // Frames 24 and 26, neither with the request's txid.
    let statuses = all(&session, MessageType::GENERATE_CATALOG_STATUS);
    let decoded: Vec<_> = statuses
        .iter()
        .map(|(header, wire)| {
            assert_ne!(header.txid, request.txid);
            let status = CatalogStatus::decode(wire).unwrap();
            assert_eq!(status.encode(), *wire);
            status
        })
        .collect();
    assert_eq!(
        decoded,
        [
            CatalogStatus { flags: 0, items: 2 },
            CatalogStatus {
                flags: CatalogStatus::DONE,
                items: 11
            },
        ]
    );
    assert!(!decoded[0].is_done());
    assert!(decoded[1].is_done());
}

#[test]
fn file_search() {
    let session = session();
    // Frame 28.
    let (_, wire) = one(&session, MessageType::FILE_SEARCH);
    let search = FileSearch::decode(wire).unwrap();
    assert_eq!(search.query, b"a.txt");
    assert_eq!(search.encode().unwrap(), wire);

    // Frame 29.
    let (_, wire) = one(&session, MessageType::FILE_SEARCH_REPLY);
    let hits = FileSearchEntry::decode_list(wire).unwrap();
    assert_eq!(hits.len(), 1);
    let hit = &hits[0];
    assert_eq!(hit.size, 6);
    assert_eq!(hit.attributes, ATTR_FILE);
    assert_eq!(hit.created, hit.modified);
    assert_eq!(hit.unknown_34, 0x10);
    assert!(hit.name.is_empty());
    assert_eq!(hit.path, FilePath::new(["lab-files", "a.txt"]));
    assert_eq!(FileSearchEntry::encode_list(&hits).unwrap(), wire);
}

#[test]
fn launch_file() {
    let session = session();
    // Frame 31.
    let (request, wire) = one(&session, MessageType::LAUNCH_FILE);
    let launch = LaunchFile::decode(wire).unwrap();
    assert_eq!(launch.path, FilePath::new(["lab-files", "hello.sh"]));
    assert_eq!(launch.encode().unwrap(), wire);

    // Frame 32: the success value is the request's type.
    let reply = all(&session, MessageType::STATUS)
        .into_iter()
        .find(|(h, _)| h.txid == request.txid)
        .unwrap();
    assert_eq!(
        Status::decode(reply.1).unwrap(),
        Status::Success { value: 420 }
    );
}

#[test]
fn volume_info() {
    let session = session();
    // Frame 51.
    let (_, wire) = one(&session, MessageType::VOLUME_INFO);
    let volume = VolumeInfo::decode(wire).unwrap();
    assert_eq!(volume.name, b"/tmp");
    assert_eq!(volume.format, b"tmpfs");
    assert!(volume.comments.is_empty());
    assert_eq!(volume.total, u64::MAX);
    assert_eq!(volume.free, u64::MAX);
    assert_eq!(volume.item_count, u32::MAX);
    assert_eq!(volume.disk_type, 0);
    assert_eq!(volume.icon, 0x3d);
    assert_eq!(volume.flags, 1);
    assert_eq!(volume.flags_mask, 1);
    assert_eq!(volume.encode().unwrap(), wire);
}
