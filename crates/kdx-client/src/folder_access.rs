//! Folder access requests: 1100 to 1107.

use anyhow::Result;
use kdx_proto::msg::admin::{FolderAccess, FolderAccessItem, GetFolderAccessList};
use kdx_proto::session::MessageType;

use crate::Client;

impl Client {
    /// 1106 Get Folder Access List, with flags 0 as Client 1.620 sends.
    pub async fn folder_access_list(&mut self) -> Result<Vec<FolderAccessItem>> {
        let request = GetFolderAccessList {
            flags: Some(0),
            rest: Vec::new(),
        };
        let reply = self
            .request(MessageType::GET_FOLDER_ACCESS_LIST, &request.encode())
            .await?;
        Ok(FolderAccessItem::decode_list(&reply)?)
    }

    /// 1103 Get Folder Access.
    pub async fn folder_access(&mut self, name: &[u8]) -> Result<FolderAccess> {
        let reply = self.request(MessageType::GET_FOLDER_ACCESS, name).await?;
        Ok(FolderAccess::decode(&reply)?)
    }

    /// 1100 Create Folder Access; the server answers with a success Status.
    pub async fn create_folder_access(&mut self, item: &FolderAccess) -> Result<()> {
        let record = FolderAccess {
            padding: Vec::new(),
            ..item.clone()
        };
        self.request(MessageType::CREATE_FOLDER_ACCESS, &record.encode()?)
            .await?;
        Ok(())
    }

    /// 1105 Set Folder Access; the server answers with a success Status.
    ///
    /// `current_name` names the item to change; when empty, `name` is used.
    pub async fn set_folder_access(&mut self, item: &FolderAccess) -> Result<()> {
        let record = FolderAccess {
            current_name: if item.current_name.is_empty() {
                item.name.clone()
            } else {
                item.current_name.clone()
            },
            padding: Vec::new(),
            ..item.clone()
        };
        self.request(MessageType::SET_FOLDER_ACCESS, &record.encode()?)
            .await?;
        Ok(())
    }

    /// 1101 Delete Folder Access; every user gets 1102 Folder Access Deleted.
    pub async fn delete_folder_access(&mut self, name: &[u8]) -> Result<()> {
        self.send_checked(MessageType::DELETE_FOLDER_ACCESS, name)
            .await
    }
}
