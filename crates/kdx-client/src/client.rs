//! A logged-in session with typed requests.
//!
//! The requests by area are in the sibling modules: `chat`, `news`, `files`,
//! `accounts` and `folder_access`.

use std::collections::VecDeque;
use std::net::SocketAddr;

use anyhow::{bail, Context, Result};
use kdx_proto::msg::status::error_name;
use kdx_proto::msg::transfer::{DownloadFile, DownloadInfo, FileStrings, UploadFile, UploadReply};
use kdx_proto::msg::{FilePath, PrivateMessage, ServerInfo, Status, UserInfo, UserRecord};
use kdx_proto::session::{Header, MessageType};
use kdx_proto::{AccountKey, ChallengeTable};
use tokio::io::{AsyncRead, AsyncSeek, AsyncSeekExt, AsyncWrite};
use tokio::net::{lookup_host, ToSocketAddrs};

use crate::{Connection, Login, Notice, Responses};

/// A request the server refused with 1 Status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("server refused {request}: error {code:#x}")]
pub struct ServerError {
    /// The refused request's type.
    pub request: MessageType,
    /// Error code, such as 0x10014 Access Denied.
    pub code: u32,
}

impl ServerError {
    /// The code's meaning in the server's table of error names, if it has one.
    pub const fn name(&self) -> Option<&'static str> {
        error_name(self.code)
    }
}

/// A message the server sent on its own, such as 208 Receive Chat Text.
pub type Event = (Header, Vec<u8>);

/// A logged-in user session.
#[derive(Debug)]
pub struct Client {
    conn: Connection,
    addr: SocketAddr,
    responses: Responses,
    pub(crate) events: VecDeque<Event>,
    /// The 105 Server Info received at login.
    pub info: ServerInfo,
    /// The 101 User List received at login.
    pub users: Vec<UserRecord>,
}

impl Client {
    /// Connects, logs in, and sends `me` as 100 Set Identity.
    pub async fn login(
        addr: impl ToSocketAddrs,
        login: &Login,
        me: &UserRecord,
        table: ChallengeTable,
    ) -> Result<Self> {
        Self::login_with(addr, login, me, Responses::Table(table)).await
    }

    pub(crate) async fn login_with(
        addr: impl ToSocketAddrs,
        login: &Login,
        me: &UserRecord,
        responses: Responses,
    ) -> Result<Self> {
        let addr = lookup_host(addr)
            .await?
            .next()
            .context("no address to connect to")?;
        let key = AccountKey::from_password(&login.password);
        let mut conn =
            Connection::connect_with_key(addr, login.name.clone(), key, &responses).await?;
        conn.send(MessageType::SET_IDENTITY, &me.encode()?).await?;

        let mut events = VecDeque::new();
        let (mut users, mut info) = (None, None);
        while users.is_none() || info.is_none() {
            let (header, payload) = conn.recv().await.context("waiting for the login replies")?;
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
            responses,
            events,
            info: info.expect("set above"),
            users: users.expect("set above"),
        })
    }

    /// Sends a request that has no reply.
    pub async fn send(&mut self, kind: MessageType, payload: &[u8]) -> Result<()> {
        self.conn.send(kind, payload).await?;
        Ok(())
    }

    /// Sends a request and waits for its reply, queueing other messages as events.
    ///
    /// A 1 Status error reply becomes a [`ServerError`]; a success Status
    /// returns an empty payload.
    pub async fn request(&mut self, kind: MessageType, payload: &[u8]) -> Result<Vec<u8>> {
        let txid = self.conn.send(kind, payload).await?;
        wait_reply(&mut self.conn, kind, txid, &mut self.events).await
    }

    /// The next message the server sent on its own, waiting if there is none.
    pub async fn next_event(&mut self) -> Result<Event> {
        match self.events.pop_front() {
            Some(event) => Ok(event),
            None => self.conn.recv().await,
        }
    }

    /// The next message the server sent on its own, decoded as a [`Notice`].
    ///
    /// A payload that does not decode comes through as [`Notice::Other`].
    pub async fn next_notice(&mut self) -> Result<Notice> {
        let (header, payload) = self.next_event().await?;
        Ok(Notice::decode(&header, &payload).unwrap_or(Notice::Other(header, payload)))
    }

    /// The first event that satisfies `wanted`, waiting for it if none is
    /// queued; the others stay queued in order.
    pub(crate) async fn wait_event(
        &mut self,
        mut wanted: impl FnMut(&Header, &[u8]) -> bool,
    ) -> Result<Event> {
        if let Some(at) = self.events.iter().position(|(h, p)| wanted(h, p)) {
            return Ok(self.events.remove(at).expect("found above"));
        }
        loop {
            let (header, payload) = self.conn.recv().await?;
            if wanted(&header, &payload) {
                return Ok((header, payload));
            }
            self.events.push_back((header, payload));
        }
    }

    /// 300 Send Private Message.
    pub async fn send_private_message(&mut self, user_id: u32, text: &[u8]) -> Result<()> {
        if text.is_empty() {
            bail!("empty private message");
        }
        let message = PrivateMessage {
            user_id,
            text: text.to_vec(),
            ..PrivateMessage::default()
        };
        self.send(MessageType::SEND_PRIVATE_MESSAGE, &message.encode())
            .await
    }

    /// 152 Get User Info.
    pub async fn user_info(&mut self, user_id: u32) -> Result<UserInfo> {
        let reply = self
            .request(MessageType::GET_USER_INFO, &user_id.to_be_bytes())
            .await?;
        Ok(UserInfo::decode(&reply)?)
    }

    /// Sends a request that has no reply on success and checks that no error came back.
    ///
    /// The server handles requests in order, so an error reply arrives before
    /// the reply to a 115 Get My Address sent after the request. A success
    /// Status for the request is accepted too; a Status for another
    /// transaction stays queued as an event.
    pub async fn send_checked(&mut self, kind: MessageType, payload: &[u8]) -> Result<()> {
        let txid = self.conn.send(kind, payload).await?;
        self.barrier(kind, txid).await
    }

    /// Waits for the server to reach the request `txid` of `kind`, and takes
    /// its 1 Status reply out of the queue if there is one.
    pub(crate) async fn barrier(&mut self, kind: MessageType, txid: u32) -> Result<()> {
        self.request(MessageType::GET_MY_ADDRESS, &[]).await?;
        let Some(at) = self
            .events
            .iter()
            .position(|(h, _)| h.txid == txid && h.is_reply())
        else {
            return Ok(());
        };
        let (header, payload) = self.events.remove(at).expect("found above");
        match header.kind {
            MessageType::STATUS => match Status::decode(&payload)? {
                Status::Success { .. } => Ok(()),
                Status::Error { code, .. } | Status::TrackerError { code, .. } => {
                    Err(ServerError {
                        request: kind,
                        code,
                    }
                    .into())
                }
            },
            other => bail!("unexpected {other} reply to {kind}"),
        }
    }

    /// Downloads a file's data fork from `offset` into `sink` on a new transfer connection.
    ///
    /// The server resumes at `offset` rounded down to 0x1000, or so it
    /// should. The data is written at the start the server's length implies,
    /// which is also returned.
    pub async fn download(
        &mut self,
        path: &FilePath,
        offset: u64,
        sink: &mut (impl AsyncWrite + AsyncSeek + Unpin),
    ) -> Result<(DownloadInfo, u64)> {
        let mut conn = self.attach().await?;
        let request = DownloadFile {
            data_offset: offset,
            rsrc_offset: 0,
            options: 0,
            path: path.clone(),
            ..DownloadFile::default()
        };
        let txid = conn
            .send(MessageType::DOWNLOAD_FILE, &request.encode()?)
            .await?;
        let reply = wait_reply(
            &mut conn,
            MessageType::DOWNLOAD_FILE,
            txid,
            &mut VecDeque::new(),
        )
        .await?;
        let info = DownloadInfo::decode(&reply)?;

        let data = conn.recv_data_header().await?;
        if data.kind != MessageType::DOWNLOAD_FILE_DATA || data.txid != txid {
            bail!("expected the data fork, got {}", data.kind);
        }
        let start = info
            .data_size
            .checked_sub(data.len)
            .context("more data than the file holds")?;
        sink.seek(std::io::SeekFrom::Start(start)).await?;
        conn.recv_data_body(&data, sink).await?;
        conn.recv_data(&mut tokio::io::sink()).await?;
        Ok((info, start))
    }

    /// Uploads `len` bytes from `source` as `name` into `folder` on a new transfer connection.
    ///
    /// If an earlier upload of the same name was cut short, the server resumes
    /// it; `source` is then read from the returned offset onward, so it must
    /// support skipping there.
    pub async fn upload(
        &mut self,
        folder: &FilePath,
        name: &[u8],
        len: u64,
        source: &mut (impl AsyncRead + AsyncSeek + Unpin),
    ) -> Result<u64> {
        let mut conn = self.attach().await?;
        let request = UploadFile {
            data_size: len,
            rsrc_size: 0,
            options: 0,
            strings: FileStrings {
                name: name.to_vec(),
                ..FileStrings::default()
            },
            folder: folder.clone(),
            ..UploadFile::default()
        };
        let txid = conn
            .send(MessageType::UPLOAD_FILE, &request.encode()?)
            .await?;
        let reply = wait_reply(
            &mut conn,
            MessageType::UPLOAD_FILE,
            txid,
            &mut VecDeque::new(),
        )
        .await?;
        let start = UploadReply::decode(&reply)?.data_offset.min(len);

        source.seek(std::io::SeekFrom::Start(start)).await?;
        let left = len - start;
        if left > Header::MAX_LEN {
            bail!("file too large for one transfer");
        }
        conn.send_data(MessageType::UPLOAD_FILE_DATA, left, source)
            .await?;
        conn.send_data(MessageType::UPLOAD_FILE_DATA, 0, &mut tokio::io::empty())
            .await?;
        loop {
            let (header, payload) = conn.recv().await?;
            match header.kind {
                MessageType::UPLOAD_FILE_COMPLETE => return Ok(start),
                MessageType::STATUS => bail!(status_error(MessageType::UPLOAD_FILE, &payload)),
                _ => {}
            }
        }
    }

    pub(crate) async fn attach(&self) -> Result<Connection> {
        Connection::attach_with(
            self.addr,
            self.info.user_id,
            self.info.session_key,
            &self.responses,
        )
        .await
        .context("opening a transfer connection")
    }

    /// Sends `kind` and returns its transaction ID, for a reply awaited later.
    pub(crate) async fn send_txid(&mut self, kind: MessageType, payload: &[u8]) -> Result<u32> {
        self.conn.send(kind, payload).await
    }
}

pub(crate) async fn wait_reply(
    conn: &mut Connection,
    kind: MessageType,
    txid: u32,
    events: &mut VecDeque<Event>,
) -> Result<Vec<u8>> {
    loop {
        let (header, payload) = conn.recv().await?;
        if header.txid != txid || !header.is_reply() {
            events.push_back((header, payload));
        } else if header.kind == MessageType::STATUS {
            match Status::decode(&payload)? {
                Status::Success { .. } => return Ok(Vec::new()),
                Status::Error { code, .. } | Status::TrackerError { code, .. } => {
                    bail!(ServerError {
                        request: kind,
                        code
                    })
                }
            }
        } else {
            return Ok(payload);
        }
    }
}

pub(crate) fn status_error(request: MessageType, payload: &[u8]) -> anyhow::Error {
    match Status::decode(payload) {
        Ok(Status::Error { code, .. } | Status::TrackerError { code, .. }) => {
            ServerError { request, code }.into()
        }
        Ok(Status::Success { .. }) => anyhow::anyhow!("unexpected 1 Status success"),
        Err(err) => err.into(),
    }
}

/// Pads `bytes` to a multiple of 4 with zeros, as the server requires of
/// records that end in padding.
pub(crate) fn pad4(mut bytes: Vec<u8>) -> Vec<u8> {
    bytes.resize(bytes.len().next_multiple_of(4), 0);
    bytes
}
