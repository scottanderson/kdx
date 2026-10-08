//! A logged-in session with typed requests.

use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};

use anyhow::{bail, Context, Result};
use kdx_proto::msg::chat::{ChatListEntry, ChatText, ChatWindowInfo, JoinChat};
use kdx_proto::msg::file::FileListEntry;
use kdx_proto::msg::news::{NewsMessages, NewsgroupEntry};
use kdx_proto::msg::transfer::{DownloadFile, DownloadInfo, FileStrings, UploadFile, UploadReply};
use kdx_proto::msg::{FilePath, PrivateMessage, ServerInfo, Status, UserInfo, UserRecord};
use kdx_proto::session::{Header, MessageType};
use kdx_proto::ChallengeTable;

use crate::{Connection, Login};

/// A request the server refused with 1 Status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("server refused {request}: error {code:#x}")]
pub struct ServerError {
    /// The refused request's type.
    pub request: MessageType,
    /// Error code, such as 0x10014 Access Denied.
    pub code: u32,
}

/// A message the server sent on its own, such as 208 Receive Chat Text.
pub type Event = (Header, Vec<u8>);

/// A logged-in user session.
#[derive(Debug)]
pub struct Client {
    conn: Connection,
    addr: SocketAddr,
    table: ChallengeTable,
    events: VecDeque<Event>,
    /// The 105 Server Info received at login.
    pub info: ServerInfo,
    /// The 101 User List received at login.
    pub users: Vec<UserRecord>,
}

impl Client {
    /// Connects, logs in, and sends `me` as 100 Set Identity.
    pub fn login(
        addr: impl ToSocketAddrs,
        login: &Login,
        me: &UserRecord,
        table: ChallengeTable,
    ) -> Result<Self> {
        let addr = addr
            .to_socket_addrs()?
            .next()
            .context("no address to connect to")?;
        let mut conn = Connection::connect(addr, login, &table)?;
        conn.send(MessageType::SET_IDENTITY, &me.encode()?)?;

        let mut events = VecDeque::new();
        let (mut users, mut info) = (None, None);
        while users.is_none() || info.is_none() {
            let (header, payload) = conn.recv().context("waiting for the login replies")?;
            match header.kind {
                MessageType::USER_LIST => users = Some(UserRecord::decode_list(&payload)?),
                MessageType::SERVER_INFO => info = Some(ServerInfo::decode(&payload)?),
                MessageType::STATUS => bail!(status_error(MessageType::SET_IDENTITY, &payload)),
                _ => events.push_back((header, payload)),
            }
        }
        Ok(Self {
            conn,
            addr,
            table,
            events,
            info: info.expect("set above"),
            users: users.expect("set above"),
        })
    }

    /// Sends a request that has no reply.
    pub fn send(&mut self, kind: MessageType, payload: &[u8]) -> Result<()> {
        self.conn.send(kind, payload)?;
        Ok(())
    }

    /// Sends a request and waits for its reply, queueing other messages as events.
    ///
    /// A 1 Status error reply becomes a [`ServerError`].
    pub fn request(&mut self, kind: MessageType, payload: &[u8]) -> Result<Vec<u8>> {
        let txid = self.conn.send(kind, payload)?;
        wait_reply(&mut self.conn, kind, txid, &mut self.events)
    }

    /// The next message the server sent on its own, waiting if there is none.
    pub fn next_event(&mut self) -> Result<Event> {
        match self.events.pop_front() {
            Some(event) => Ok(event),
            None => self.conn.recv(),
        }
    }

    /// The underlying socket, for timeouts.
    pub const fn stream(&self) -> &TcpStream {
        self.conn.stream()
    }

    /// 214 Get Chat List.
    pub fn chat_list(&mut self) -> Result<Vec<ChatListEntry>> {
        let reply = self.request(MessageType::GET_CHAT_LIST, &[])?;
        Ok(ChatListEntry::decode_list(&reply)?)
    }

    /// 202 Join Chat.
    pub fn join_chat(&mut self, chat_id: u32, key: u32) -> Result<ChatWindowInfo> {
        let request = JoinChat { chat_id, key }.encode();
        let reply = self.request(MessageType::JOIN_CHAT, &request)?;
        Ok(ChatWindowInfo::decode(&reply)?)
    }

    /// 204 Leave Chat.
    pub fn leave_chat(&mut self, chat_id: u32) -> Result<()> {
        self.send(MessageType::LEAVE_CHAT, &chat_id.to_be_bytes())
    }

    /// 207 Send Chat Text.
    pub fn send_chat(&mut self, chat_id: u32, text: &[u8]) -> Result<()> {
        let message = ChatText {
            chat_id,
            user_id: 0,
            text: text.to_vec(),
        };
        self.send(MessageType::SEND_CHAT_TEXT, &message.encode()?)
    }

    /// 300 Send Private Message.
    pub fn send_private_message(&mut self, user_id: u32, text: &[u8]) -> Result<()> {
        let message = PrivateMessage {
            user_id,
            text: text.to_vec(),
        };
        self.send(MessageType::SEND_PRIVATE_MESSAGE, &message.encode())
    }

    /// 152 Get User Info.
    pub fn user_info(&mut self, user_id: u32) -> Result<UserInfo> {
        let reply = self.request(MessageType::GET_USER_INFO, &user_id.to_be_bytes())?;
        Ok(UserInfo::decode(&reply)?)
    }

    /// 400 Get File List.
    pub fn file_list(&mut self, path: &FilePath) -> Result<Vec<FileListEntry>> {
        let reply = self.request(MessageType::GET_FILE_LIST, &path.encode()?)?;
        Ok(FileListEntry::decode_list(&reply)?)
    }

    /// 605 Get Newsgroup List.
    pub fn newsgroups(&mut self) -> Result<Vec<NewsgroupEntry>> {
        let reply = self.request(MessageType::GET_NEWSGROUP_LIST, &[])?;
        Ok(NewsgroupEntry::decode_list(&reply)?)
    }

    /// 607 Get News Messages.
    pub fn news(&mut self, group_id: u32) -> Result<NewsMessages> {
        let reply = self.request(MessageType::GET_NEWS_MESSAGES, &group_id.to_be_bytes())?;
        Ok(NewsMessages::decode(&reply)?)
    }

    /// Downloads a file's data fork from `offset` into `sink` on a new transfer connection.
    ///
    /// The server resumes at `offset` rounded down to 0x1000; the returned
    /// offset is where the written data starts.
    pub fn download(
        &mut self,
        path: &FilePath,
        offset: u64,
        sink: &mut impl Write,
    ) -> Result<(DownloadInfo, u64)> {
        let mut conn = self.attach()?;
        let request = DownloadFile {
            data_offset: offset,
            rsrc_offset: 0,
            options: 0,
            path: path.clone(),
        };
        let txid = conn.send(MessageType::DOWNLOAD_FILE, &request.encode()?)?;
        let reply = wait_reply(
            &mut conn,
            MessageType::DOWNLOAD_FILE,
            txid,
            &mut VecDeque::new(),
        )?;
        let info = DownloadInfo::decode(&reply)?;

        let data = conn.recv_data(sink)?;
        if data.kind != MessageType::DOWNLOAD_FILE_DATA || data.txid != txid {
            bail!("expected the data fork, got {}", data.kind);
        }
        let start = info
            .data_size
            .checked_sub(u64::from(data.len))
            .context("more data than the file holds")?;
        conn.recv_data(&mut std::io::sink())?;
        Ok((info, start))
    }

    /// Uploads `len` bytes from `source` as `name` into `folder` on a new transfer connection.
    ///
    /// If an earlier upload of the same name was cut short, the server resumes
    /// it; `source` is then read from the returned offset onward, so it must
    /// support skipping there.
    pub fn upload(
        &mut self,
        folder: &FilePath,
        name: &[u8],
        len: u64,
        source: &mut (impl Read + std::io::Seek),
    ) -> Result<u64> {
        let mut conn = self.attach()?;
        let request = UploadFile {
            data_size: len,
            rsrc_size: 0,
            options: 0,
            strings: FileStrings {
                name: name.to_vec(),
                ..FileStrings::default()
            },
            folder: folder.clone(),
        };
        let txid = conn.send(MessageType::UPLOAD_FILE, &request.encode()?)?;
        let reply = wait_reply(
            &mut conn,
            MessageType::UPLOAD_FILE,
            txid,
            &mut VecDeque::new(),
        )?;
        let start = UploadReply::decode(&reply)?.data_offset.min(len);

        source.seek(std::io::SeekFrom::Start(start))?;
        let left = u32::try_from(len - start).context("file too large for one transfer")?;
        conn.send_data(MessageType::UPLOAD_FILE_DATA, left, source)?;
        conn.send_data(MessageType::UPLOAD_FILE_DATA, 0, &mut std::io::empty())?;
        loop {
            let (header, payload) = conn.recv()?;
            match header.kind {
                MessageType::UPLOAD_FILE_COMPLETE => return Ok(start),
                MessageType::STATUS => bail!(status_error(MessageType::UPLOAD_FILE, &payload)),
                _ => {}
            }
        }
    }

    fn attach(&self) -> Result<Connection> {
        Connection::attach(
            self.addr,
            self.info.user_id,
            self.info.session_key,
            &self.table,
        )
        .context("opening a transfer connection")
    }
}

fn wait_reply(
    conn: &mut Connection,
    kind: MessageType,
    txid: u32,
    events: &mut VecDeque<Event>,
) -> Result<Vec<u8>> {
    loop {
        let (header, payload) = conn.recv()?;
        if header.txid != txid || !header.is_reply() {
            events.push_back((header, payload));
        } else if header.kind == MessageType::STATUS {
            match Status::decode(&payload)? {
                Status::Success { .. } => return Ok(Vec::new()),
                Status::Error { code, .. } => bail!(ServerError {
                    request: kind,
                    code
                }),
            }
        } else {
            return Ok(payload);
        }
    }
}

fn status_error(request: MessageType, payload: &[u8]) -> anyhow::Error {
    match Status::decode(payload) {
        Ok(Status::Error { code, .. }) => ServerError { request, code }.into(),
        Ok(Status::Success { .. }) => anyhow::anyhow!("unexpected 1 Status success"),
        Err(err) => err.into(),
    }
}
