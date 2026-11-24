//! Checks against `captures/cli-upload-resume.pcapng`: the command-line client cuts and resumes a 256 KiB upload.

use kdx_proto::msg::file::FileListEntry;
use kdx_proto::msg::transfer::UploadReply;
use kdx_proto::msg::ServerInfo;
use kdx_proto::session::{Header, MessageType};
use kdx_proto::AccountKey;

mod common;

fn session(c2s: &[u8], s2c: &[u8]) -> Vec<(Header, Vec<u8>)> {
    common::session_with_key(c2s, s2c, &AccountKey::from_password(common::LAB_PASSWORD))
}

/// The size that frame 61 gives the `big.bin.prt` partial.
fn prt_size() -> u64 {
    let session = session(
        include_bytes!("fixtures/cli-upload-resume-36672.c2s.bin"),
        include_bytes!("fixtures/cli-upload-resume-36672.s2c.bin"),
    );
    let (_, wire) = session
        .iter()
        .find(|(h, _)| h.kind == MessageType::FILE_LIST)
        .expect("a 401");
    let entries = FileListEntry::decode_list(wire).unwrap();
    let prt = entries
        .iter()
        .find(|e| e.name == b"big.bin.prt")
        .unwrap_or_else(|| panic!("no partial in {entries:#?}"));
    prt.size
}

/// The offset that frame 88, on the transfer connection of the resumed upload, offers.
///
/// The transfer connection is keyed by the parent session's 105.
fn offered_offset() -> u64 {
    let parent = session(
        include_bytes!("fixtures/cli-upload-resume-36678.c2s.bin"),
        include_bytes!("fixtures/cli-upload-resume-36678.s2c.bin"),
    );
    let (_, info) = parent
        .iter()
        .find(|(h, _)| h.kind == MessageType::SERVER_INFO)
        .expect("a 105");
    let key = AccountKey(ServerInfo::decode(info).unwrap().session_key);
    let session = common::session_with_key(
        include_bytes!("fixtures/cli-upload-resume-36680.c2s.bin"),
        include_bytes!("fixtures/cli-upload-resume-36680.s2c.bin"),
        &key,
    );
    let (_, wire) = session
        .iter()
        .find(|(h, _)| h.kind == MessageType::UPLOAD_FILE_REPLY)
        .expect("a 416");
    UploadReply::decode(wire).unwrap().data_offset
}

/// Frames 61 and 88: the 401 lists the partial at 0x20000 bytes, and the 416 offers 0x20000.
///
/// The client was cut after 150000 bytes, and rounding that down to a multiple of 0x1000
/// would give 0x24000, so the offset is the size of the `.prt` on disk.
#[test]
fn prt_size_is_the_resume_offset() {
    assert_eq!(prt_size(), 0x20000);
    assert_eq!(offered_offset(), prt_size());
}
