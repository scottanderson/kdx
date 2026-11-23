//! File requests on the main connection against the mock.

mod common;

use kdx_client::mock::Step;
use kdx_proto::msg::file::{
    CatalogStatus, CreateFolder, FileInfo, FileListEntry, FileSearch, FileSearchEntry, MoveToTrash,
    ATTR_FILE,
};
use kdx_proto::msg::{FilePath, Status};
use kdx_proto::session::MessageType;

#[tokio::test]
async fn list_root_sends_empty_path_and_decodes_entries() {
    let entry = FileListEntry {
        size: 3,
        attributes: ATTR_FILE,
        name: b"tiny.txt".to_vec(),
        padding: vec![0; 2],
        ..FileListEntry::default()
    };
    let (server, mut client) = common::client_with(vec![
        Step::Expect(MessageType::GET_FILE_LIST),
        Step::Reply(
            MessageType::FILE_LIST,
            FileListEntry::encode_list(&[entry]).unwrap(),
        ),
        Step::Expect(MessageType::GET_FILE_LIST),
        Step::Reply(MessageType::FILE_LIST, Vec::new()),
    ])
    .await;
    let entries = client.file_list(&FilePath::default()).await.unwrap();
    assert_eq!(entries[0].name, b"tiny.txt");
    assert!(!entries[0].is_folder());
    let empty = client.file_list(&FilePath::new(["Uploads"])).await.unwrap();
    assert!(empty.is_empty());
    drop(client);
    server.finish().await;
    let sent = server.payloads(MessageType::GET_FILE_LIST);
    assert_eq!(sent[0], FilePath::default().encode().unwrap());
    assert_eq!(
        FilePath::decode(&sent[1]).unwrap(),
        FilePath::new(["Uploads"])
    );
}

#[tokio::test]
async fn create_folder_returns_the_servers_answer() {
    let (server, mut client) = common::client_with(vec![
        Step::Expect(MessageType::CREATE_FOLDER),
        Step::Reply(
            MessageType::STATUS,
            Status::Success { value: 0x192 }.encode(),
        ),
    ])
    .await;
    client
        .create_folder(&FilePath::new(["Uploads"]), b"New", b"", b"")
        .await
        .unwrap();
    drop(client);
    server.finish().await;
    let sent = CreateFolder::decode(&server.payloads(MessageType::CREATE_FOLDER)[0]).unwrap();
    assert_eq!(sent.name, b"New");
    assert_eq!(sent.path, FilePath::new(["Uploads"]));
}

#[tokio::test]
async fn file_info_trash_and_search() {
    let info = FileInfo {
        size: 3,
        name: b"tiny.txt".to_vec(),
        kind: b"TEXT".to_vec(),
        ..FileInfo::default()
    };
    let found = FileSearchEntry {
        size: 3,
        attributes: ATTR_FILE,
        name: b"tiny.txt".to_vec(),
        path: FilePath::new(["Uploads"]),
        ..FileSearchEntry::default()
    };
    let (server, mut client) = common::client_with(vec![
        Step::Expect(MessageType::GET_FILE_FOLDER_INFO),
        Step::Reply(MessageType::FILE_FOLDER_INFO, info.encode().unwrap()),
        Step::Expect(MessageType::MOVE_TO_TRASH),
        Step::Reply(
            MessageType::STATUS,
            Status::Success { value: 0x195 }.encode(),
        ),
        Step::Expect(MessageType::EMPTY_TRASH),
        Step::Reply(
            MessageType::STATUS,
            Status::Success { value: 0x1a5 }.encode(),
        ),
        Step::Expect(MessageType::FILE_SEARCH),
        Step::Reply(
            MessageType::FILE_SEARCH_REPLY,
            FileSearchEntry::encode_list(std::slice::from_ref(&found)).unwrap(),
        ),
    ])
    .await;
    let path = FilePath::new(["Uploads", "tiny.txt"]);
    assert_eq!(client.file_info(&path).await.unwrap(), info);
    client
        .move_to_trash(&FilePath::new(["Uploads"]), &[b"tiny.txt", b"other"])
        .await
        .unwrap();
    client.empty_trash().await.unwrap();
    let hits = client.search(b"tiny").await.unwrap();
    assert_eq!(
        (&hits[0].name[..], &hits[0].path),
        (&b"tiny.txt"[..], &found.path)
    );
    drop(client);
    server.finish().await;
    let trash = MoveToTrash::decode(&server.payloads(MessageType::MOVE_TO_TRASH)[0]).unwrap();
    assert_eq!(trash.folder, FilePath::new(["Uploads"]));
    assert_eq!(trash.names, [b"tiny.txt".to_vec(), b"other".to_vec()]);
    let search = FileSearch::decode(&server.payloads(MessageType::FILE_SEARCH)[0]).unwrap();
    assert_eq!(search.query, b"tiny");
}

#[tokio::test]
async fn generate_catalog_waits_for_the_done_status() {
    let running = CatalogStatus {
        flags: 0,
        items: 10,
    };
    let done = CatalogStatus {
        flags: CatalogStatus::DONE,
        items: 25,
    };
    let (server, mut client) = common::client_with(vec![
        Step::Expect(MessageType::GENERATE_CATALOG),
        Step::Send(
            MessageType::GENERATE_CATALOG_STATUS,
            running.encode().to_vec(),
        ),
        Step::Send(MessageType::GENERATE_CATALOG_STATUS, done.encode().to_vec()),
    ])
    .await;
    assert_eq!(client.generate_catalog().await.unwrap(), 25);
    drop(client);
    server.finish().await;
    assert!(server.payloads(MessageType::GENERATE_CATALOG)[0].is_empty());
}
