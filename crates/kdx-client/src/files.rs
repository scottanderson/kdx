//! File and folder requests on the main connection: 400 to 453.
//!
//! Transfers open their own connection; see [`Client::download`] and
//! [`Client::upload`].

use anyhow::Result;
use kdx_proto::msg::file::{
    CatalogStatus, CreateFolder, FileInfo, FileListEntry, FileSearch, FileSearchEntry, MoveToTrash,
};
use kdx_proto::msg::FilePath;
use kdx_proto::session::MessageType;

use crate::Client;

impl Client {
    /// 400 Get File List; an empty folder is an empty reply.
    pub async fn file_list(&mut self, path: &FilePath) -> Result<Vec<FileListEntry>> {
        let reply = self
            .request(MessageType::GET_FILE_LIST, &path.encode()?)
            .await?;
        Ok(FileListEntry::decode_list(&reply)?)
    }

    /// 409 Get File/Folder Info.
    pub async fn file_info(&mut self, path: &FilePath) -> Result<FileInfo> {
        let reply = self
            .request(MessageType::GET_FILE_FOLDER_INFO, &path.encode()?)
            .await?;
        Ok(FileInfo::decode(&reply)?)
    }

    /// 402 Create Folder; the server answers with a success Status.
    ///
    /// `access` is a folder access tag such as `DB`, and `owner` the drop
    /// box owner's login; both may be empty.
    pub async fn create_folder(
        &mut self,
        parent: &FilePath,
        name: &[u8],
        access: &[u8],
        owner: &[u8],
    ) -> Result<()> {
        let request = CreateFolder {
            path: parent.clone(),
            name: name.to_vec(),
            access: access.to_vec(),
            owner: owner.to_vec(),
            ..CreateFolder::default()
        };
        self.request(MessageType::CREATE_FOLDER, &request.encode()?)
            .await?;
        Ok(())
    }

    /// 405 Move To Trash; the server answers with a success Status.
    pub async fn move_to_trash(&mut self, folder: &FilePath, names: &[&[u8]]) -> Result<()> {
        let request = MoveToTrash {
            folder: folder.clone(),
            names: names.iter().map(|name| name.to_vec()).collect(),
        };
        self.request(MessageType::MOVE_TO_TRASH, &request.encode()?)
            .await?;
        Ok(())
    }

    /// 421 Empty Trash; the server answers with a success Status.
    pub async fn empty_trash(&mut self) -> Result<()> {
        self.request(MessageType::EMPTY_TRASH, &[]).await?;
        Ok(())
    }

    /// 450 Generate Catalog, then waits through the 451 Generate Catalog
    /// (S) messages for the one flagged done; returns its item count.
    pub async fn generate_catalog(&mut self) -> Result<u32> {
        self.send(MessageType::GENERATE_CATALOG, &[]).await?;
        let (_, payload) = self
            .wait_event(|header, payload| {
                header.kind == MessageType::GENERATE_CATALOG_STATUS
                    && CatalogStatus::decode(payload).is_ok_and(|s| s.is_done())
            })
            .await?;
        Ok(CatalogStatus::decode(&payload)?.items)
    }

    /// 452 File Search.
    pub async fn search(&mut self, query: &[u8]) -> Result<Vec<FileSearchEntry>> {
        let request = FileSearch {
            query: query.to_vec(),
            ..FileSearch::default()
        };
        let reply = self
            .request(MessageType::FILE_SEARCH, &request.encode()?)
            .await?;
        Ok(FileSearchEntry::decode_list(&reply)?)
    }
}
