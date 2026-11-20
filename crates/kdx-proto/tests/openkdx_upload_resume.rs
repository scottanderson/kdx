//! Checks against `captures/openkdx-upload-resume.pcapng`: the OpenKDX client cuts and resumes a 256 KiB upload.

use kdx_proto::msg::file::FileListEntry;
use kdx_proto::msg::transfer::UploadReply;
use kdx_proto::msg::ServerInfo;
use kdx_proto::session::{Header, MessageType};
use kdx_proto::AccountKey;

mod common;

const PASSWORD: &[u8] = b"openkdx-lab";

fn session(c2s: &[u8], s2c: &[u8]) -> Vec<(Header, Vec<u8>)> {
    common::session_with_key(c2s, s2c, &AccountKey::from_password(PASSWORD))
}

/// Frame 61: after a cut at 150000 bytes the folder lists `big.bin.prt`.
#[test]
fn prt_partial_is_listed() {
    let session = session(
        include_bytes!("fixtures/openkdx-upload-resume-36672.c2s.bin"),
        include_bytes!("fixtures/openkdx-upload-resume-36672.s2c.bin"),
    );
    let (_, wire) = session
        .iter()
        .find(|(h, _)| h.kind == MessageType::FILE_LIST)
        .expect("a 401");
    let entries = FileListEntry::decode_list(wire).unwrap();
    let names: Vec<&[u8]> = entries.iter().map(|e| e.name.as_slice()).collect();
    assert!(names.contains(&&b"big.bin.prt"[..]), "{names:?}");
}

/// Frame 88: the server kept 131072 bytes of the 150000 and offers 0x20000.
///
/// The upload runs on a transfer connection keyed by the parent session's 105.
#[test]
fn server_offers_0x20000_after_a_150000_byte_cut() {
    let parent = session(
        include_bytes!("fixtures/openkdx-upload-resume-36678.c2s.bin"),
        include_bytes!("fixtures/openkdx-upload-resume-36678.s2c.bin"),
    );
    let (_, info) = parent
        .iter()
        .find(|(h, _)| h.kind == MessageType::SERVER_INFO)
        .expect("a 105");
    let key = AccountKey(ServerInfo::decode(info).unwrap().session_key);
    let session = common::session_with_key(
        include_bytes!("fixtures/openkdx-upload-resume-36680.c2s.bin"),
        include_bytes!("fixtures/openkdx-upload-resume-36680.s2c.bin"),
        &key,
    );
    let (_, wire) = session
        .iter()
        .find(|(h, _)| h.kind == MessageType::UPLOAD_FILE_REPLY)
        .expect("a 416");
    let reply = UploadReply::decode(wire).unwrap();
    assert_eq!(reply.data_offset, 0x20000);
    assert_eq!(reply.data_offset % 0x1000, 0);
}
