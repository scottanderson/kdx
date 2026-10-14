//! Checks against `captures/linux-accounts-folder-access.pcapng`: Client 1.620 for Linux as `admin`.

use kdx_proto::msg::admin::{FolderAccessName, GetFolderAccessList};
use kdx_proto::session::MessageType;

mod common;

#[test]
fn delete_and_get_folder_access() {
    let session = common::session(
        include_bytes!("fixtures/linux-accounts-folder-access-55746.c2s.bin"),
        include_bytes!("fixtures/linux-accounts-folder-access-55746.s2c.bin"),
    );
    let names = |kind| -> Vec<Vec<u8>> {
        session
            .iter()
            .filter(|(h, _)| h.kind == kind)
            .map(|(_, wire)| {
                let name = FolderAccessName::decode(wire).unwrap();
                assert_eq!(name.encode(), *wire);
                name.0
            })
            .collect()
    };
    // Frames 60, 70 and 71.
    assert_eq!(names(MessageType::GET_FOLDER_ACCESS), [b"lab-item"]);
    assert_eq!(names(MessageType::DELETE_FOLDER_ACCESS), [b"lab-item"]);
    assert_eq!(names(MessageType::FOLDER_ACCESS_DELETED), [b"lab-item"]);

    // Frames 50, 54 and 64.
    let lists: Vec<_> = session
        .iter()
        .filter(|(h, _)| h.kind == MessageType::GET_FOLDER_ACCESS_LIST)
        .map(|(_, wire)| {
            let list = GetFolderAccessList::decode(wire).unwrap();
            assert_eq!(list.encode(), *wire);
            list
        })
        .collect();
    assert_eq!(lists, vec![GetFolderAccessList::default(); 3]);
}
