//! Checks against `captures/linux-files.pcapng`: Client 1.620 for Linux as `admin`.

use kdx_proto::msg::admin::{FolderAccess, FolderAccessName, GetFolderAccessList};
use kdx_proto::msg::file::{CreateFolder, MoveItems, MoveToTrash, SetFileInfo};
use kdx_proto::msg::{FilePath, Status};
use kdx_proto::session::{Header, MessageType};

mod common;

fn session() -> Vec<(Header, Vec<u8>)> {
    common::session(
        include_bytes!("fixtures/linux-files-59246.c2s.bin"),
        include_bytes!("fixtures/linux-files-59246.s2c.bin"),
    )
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

/// The 1 Status reply to `request`, if any.
fn status(messages: &[(Header, Vec<u8>)], request: &Header) -> Option<Status> {
    all(messages, MessageType::STATUS)
        .into_iter()
        .find(|(h, _)| h.txid == request.txid)
        .map(|(_, wire)| Status::decode(wire).unwrap())
}

#[test]
fn get_file_list() {
    let session = session();
    let requests = all(&session, MessageType::GET_FILE_LIST);
    assert_eq!(requests.len(), 25);
    for (_, wire) in requests {
        assert_eq!(FilePath::decode(wire).unwrap().encode().unwrap(), wire);
    }
}

#[test]
fn create_folder() {
    let session = session();
    // Frame 26.
    let (request, wire) = one(&session, MessageType::CREATE_FOLDER);
    let create = CreateFolder::decode(wire).unwrap();
    assert_eq!(create.path, FilePath::new(["Uploads [UL]"]));
    assert_eq!(create.name, b"lab-folder");
    assert!(create.access.is_empty() && create.owner.is_empty());
    assert_eq!(create.encode().unwrap(), wire);
    // Frame 29.
    assert_eq!(
        status(&session, &request),
        Some(Status::Success { value: 0x192 })
    );
}

#[test]
fn create_alias() {
    let session = session();
    // Frame 95.
    let (request, wire) = one(&session, MessageType::CREATE_ALIAS);
    let alias = MoveItems::decode(wire).unwrap();
    assert_eq!(alias.destination, FilePath::new(["Uploads [UL]"]));
    assert_eq!(alias.source, FilePath::new(["Uploads [UL]"]));
    assert_eq!(alias.names, [b"upload2.txt"]);
    assert_eq!(alias.encode().unwrap(), wire);
    // Frame 98.
    assert_eq!(
        status(&session, &request),
        Some(Status::Success { value: 0x193 })
    );
}

#[test]
fn move_to_trash() {
    let session = session();
    // Frame 241: the root is a count of 0.
    let (request, wire) = one(&session, MessageType::MOVE_TO_TRASH);
    let trash = MoveToTrash::decode(wire).unwrap();
    assert_eq!(trash.folder, FilePath::default());
    assert_eq!(trash.names, [b"lab-folder"]);
    assert_eq!(trash.encode().unwrap(), wire);
    // Frame 244.
    assert_eq!(
        status(&session, &request),
        Some(Status::Success { value: 0x195 })
    );
}

#[test]
fn move_file_folder() {
    let session = session();
    // Frames 76, 127, and 206.
    let requests = all(&session, MessageType::MOVE_FILE_FOLDER);
    let moves: Vec<_> = requests
        .iter()
        .map(|(request, wire)| {
            let items = MoveItems::decode(wire).unwrap();
            assert_eq!(items.encode().unwrap(), *wire);
            // Frames 80, 131, and 210.
            assert_eq!(
                status(&session, request),
                Some(Status::Success { value: 0x197 })
            );
            items
        })
        .collect();
    let uploads = FilePath::new(["Uploads [UL]"]);
    let folder = FilePath::new(["Uploads [UL]", "lab-folder"]);
    assert_eq!(
        moves,
        [
            MoveItems {
                destination: uploads.clone(),
                source: folder.clone(),
                names: vec![b"upload.txt".to_vec()],
            },
            MoveItems {
                destination: folder,
                source: uploads.clone(),
                names: vec![b"upload2.txt".to_vec()],
            },
            MoveItems {
                destination: FilePath::default(),
                source: uploads,
                names: vec![b"lab-folder".to_vec()],
            },
        ]
    );
}

#[test]
fn set_file_folder_info() {
    let session = session();
    // Frame 89.
    let (request, wire) = one(&session, MessageType::SET_FILE_FOLDER_INFO);
    let set = SetFileInfo::decode(wire).unwrap();
    assert_eq!(set.flags, SetFileInfo::RENAME);
    assert_eq!(set.path, FilePath::new(["Uploads [UL]", "upload.txt"]));
    assert_eq!(set.name, b"upload2.txt");
    assert_eq!(set.encode().unwrap(), wire);
    // No reply.
    assert_eq!(status(&session, &request), None);
}

#[test]
fn folder_access() {
    let session = session();
    // Frames 23 and 232.
    let lists: Vec<_> = all(&session, MessageType::GET_FOLDER_ACCESS_LIST)
        .into_iter()
        .map(|(_, wire)| {
            let list = GetFolderAccessList::decode(wire).unwrap();
            assert_eq!(list.encode(), wire);
            list.flags
        })
        .collect();
    assert_eq!(lists, [Some(1), None]);

    // Frames 225 and 228.
    let gets: Vec<_> = all(&session, MessageType::GET_FOLDER_ACCESS)
        .into_iter()
        .map(|(_, wire)| {
            let name = FolderAccessName::decode(wire).unwrap();
            assert_eq!(name.encode(), wire);
            name.0
        })
        .collect();
    assert_eq!(gets, [&b"DB"[..], b"[Default]"]);

    // Frames 226 and 229.
    let infos: Vec<_> = all(&session, MessageType::FOLDER_ACCESS_INFO)
        .into_iter()
        .map(|(_, wire)| {
            let item = FolderAccess::decode(wire).unwrap();
            assert_eq!(item.encode().unwrap(), wire);
            item
        })
        .collect();
    let [dropbox, default] = &infos[..] else {
        panic!("{infos:?}");
    };
    assert_eq!(dropbox.name, b"DB");
    assert_eq!(dropbox.description, b"Dropbox.");
    assert_eq!(
        (dropbox.flags, dropbox.icon, dropbox.rule),
        (FolderAccess::OWNER_IN_NAME, 0x2c, 0xfffe_0000)
    );
    assert!(dropbox.class_rules[..31].iter().all(|&r| r == 0x9d40_0000));
    assert_eq!(dropbox.class_rules[31], 0xfffe_0000);
    assert_eq!(default.name, b"[Default]");
    assert_eq!(
        (default.flags, default.icon, default.rule),
        (FolderAccess::BUILT_IN, 0, 0xfffe_0000)
    );
    assert!(default.class_rules[..31].iter().all(|&r| r == 0xe0a0_0000));
    assert_eq!(default.class_rules[31], 0);
    assert!(infos.iter().all(|i| i.owner.is_empty()));

    // Frame 231.
    let (request, wire) = one(&session, MessageType::CREATE_FOLDER_ACCESS);
    let create = FolderAccess::decode(wire).unwrap();
    assert_eq!(
        create,
        FolderAccess {
            name: b"test".to_vec(),
            ..FolderAccess::default()
        }
    );
    assert_eq!(create.encode().unwrap(), wire);
    // Frame 234.
    assert_eq!(
        status(&session, &request),
        Some(Status::Success { value: 0x44c })
    );
}
