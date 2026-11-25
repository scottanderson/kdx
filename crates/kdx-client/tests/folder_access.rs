//! Folder access requests against the mock.

mod common;

use kdx_client::mock::Step;
use kdx_proto::msg::admin::{FolderAccess, FolderAccessItem, GetFolderAccessList};
use kdx_proto::msg::Status;
use kdx_proto::session::MessageType;

fn barrier() -> [Step; 2] {
    [
        Step::Expect(MessageType::GET_MY_ADDRESS),
        Step::Reply(MessageType::YOUR_ADDRESS, vec![127, 0, 0, 1]),
    ]
}

#[tokio::test]
async fn folder_access_create_is_padded_to_four() {
    let (server, mut client) = common::client_with(vec![
        Step::Expect(MessageType::CREATE_FOLDER_ACCESS),
        Step::Reply(
            MessageType::STATUS,
            Status::Success { value: 0x44c }.encode(),
        ),
    ])
    .await;
    client
        .create_folder_access(&FolderAccess {
            icon: FolderAccess::ICON_PICTURES,
            name: b"Pictures1".to_vec(),
            description: b"pics".to_vec(),
            ..FolderAccess::default()
        })
        .await
        .unwrap();
    drop(client);
    server.finish().await;
    let raw = &server.payloads(MessageType::CREATE_FOLDER_ACCESS)[0];
    assert_eq!(raw.len(), (0xb8 + 9 + 4_usize).div_ceil(4) * 4);
    assert_eq!(FolderAccess::ICON_PICTURES, 0x34);
    assert_eq!(FolderAccess::ICON_UPLOADS, 0x2b);
    assert_eq!(FolderAccess::ICON_DROPBOX, 0x2c);
    assert_eq!(FolderAccess::ICON_NORMAL, 0x28);
    assert_eq!(FolderAccess::ICON_AUDIO, 0x35);
}

#[tokio::test]
async fn folder_access_list_sends_flags_zero() {
    let item = FolderAccessItem {
        icon: FolderAccess::ICON_UPLOADS,
        name: b"Uploads".to_vec(),
        padding: vec![0; 2],
        ..FolderAccessItem::default()
    };
    let (server, mut client) = common::client_with(vec![
        Step::Expect(MessageType::GET_FOLDER_ACCESS_LIST),
        Step::Reply(
            MessageType::FOLDER_ACCESS_LIST,
            FolderAccessItem::encode_list(&[item]).unwrap(),
        ),
    ])
    .await;
    assert_eq!(
        client.folder_access_list().await.unwrap()[0].name,
        b"Uploads"
    );
    drop(client);
    server.finish().await;
    let sent =
        GetFolderAccessList::decode(&server.payloads(MessageType::GET_FOLDER_ACCESS_LIST)[0])
            .unwrap();
    assert_eq!(sent.flags, Some(0));
}

#[tokio::test]
async fn folder_access_get_set_and_delete() {
    let item = FolderAccess {
        icon: FolderAccess::ICON_DROPBOX,
        name: b"Drop".to_vec(),
        ..FolderAccess::default()
    };
    let mut steps = vec![
        Step::Expect(MessageType::GET_FOLDER_ACCESS),
        Step::Reply(MessageType::FOLDER_ACCESS_INFO, item.encode().unwrap()),
        Step::Expect(MessageType::SET_FOLDER_ACCESS),
        Step::Reply(
            MessageType::STATUS,
            Status::Success { value: 0x451 }.encode(),
        ),
        Step::Expect(MessageType::DELETE_FOLDER_ACCESS),
        Step::Send(MessageType::FOLDER_ACCESS_DELETED, b"Drop".to_vec()),
    ];
    steps.extend(barrier());
    let (server, mut client) = common::client_with(steps).await;
    let got = client.folder_access(b"Drop").await.unwrap();
    assert_eq!(got.icon, FolderAccess::ICON_DROPBOX);
    client.set_folder_access(&got).await.unwrap();
    client.delete_folder_access(b"Drop").await.unwrap();
    drop(client);
    server.finish().await;
    assert_eq!(server.payloads(MessageType::GET_FOLDER_ACCESS)[0], b"Drop");
    let set = FolderAccess::decode(&server.payloads(MessageType::SET_FOLDER_ACCESS)[0]).unwrap();
    assert_eq!(set.current_name, b"Drop", "1105 names the item to change");
    assert_eq!(
        server.payloads(MessageType::DELETE_FOLDER_ACCESS)[0],
        b"Drop"
    );
}
