//! Decodes KDX connections in a pcap or pcapng capture.
//!
//! For each TCP connection to the server port, prints both handshakes, finds the
//! account key from the candidate passwords, then decrypts and dumps every session
//! message in capture order.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use etherparse::{NetSlice, SlicedPacket, TransportSlice};
use kdx_proto::handshake::{compute_auth, Hello, Proof};
use kdx_proto::msg::admin::{
    Account, AccountClass, AccountListEntry, DisconnectMessage, DisconnectUser,
    ExitTerminateProcess, ExtraServerInfo, FolderAccess, FolderAccessItem, HistoryRecord,
    MonitoredConnection, ProcessList, RcvBroadcast, SendBroadcast, ShutdownServer, YourAddress,
};
use kdx_proto::msg::chat::{
    ChatDeleted, ChatId, ChatInvitation, ChatInvite, ChatListEntry, ChatRecord, ChatText,
    ChatTopicChanged, ChatUserChange, ChatWindowInfo, JoinChat, SetChatTopic,
};
use kdx_proto::msg::file::{
    CatalogStatus, CreateFolder, FileInfo, FileListEntry, FileSearch, FileSearchEntry,
    GenerateCatalog, LaunchFile, MoveItems, MoveToTrash, SetFileInfo, SetVolumeInfo, VolumeInfo,
};
use kdx_proto::msg::news::{
    NewsMessage, NewsMessageChange, NewsMessages, NewsgroupEntry, NewsgroupInfo, PostNewsMessage,
};
use kdx_proto::msg::server_info::{ServerGreeting, ServerIcon, ServerSettings};
use kdx_proto::msg::transfer::{
    DownloadFile, DownloadInfo, NotifySpeedLimit, UploadFile, UploadReply,
};
use kdx_proto::msg::user::{MiniUserIcon, MiniUserIconChanged, RegistrationNotice, UserDeparted};
use kdx_proto::msg::user_info::GetUserInfo;
use kdx_proto::msg::{FilePath, PrivateMessage, ServerInfo, Status, UserInfo, UserRecord};
use kdx_proto::session::{Header, SessionCipher};
use kdx_proto::{AccountKey, MessageType};
use pcap_file::pcap::PcapReader;
use pcap_file::pcapng::{Block, PcapNgReader};
use pcap_file::DataLink;

/// Longest payload prefix to hex dump; file data can run to megabytes.
const MAX_DUMP: usize = 0x400;

#[derive(Parser)]
#[command(about = "Decode KDX traffic from a packet capture")]
struct Args {
    /// pcap or pcapng file.
    capture: PathBuf,
    /// Server TCP port.
    #[arg(long, default_value_t = 10700)]
    port: u16,
    /// Candidate account password; repeat for several. The empty password is always tried.
    #[arg(long = "password", short)]
    passwords: Vec<String>,
    /// Write each connection's reassembled streams to DIR as `<client port>.c2s.bin` and `.s2c.bin`.
    #[arg(long, value_name = "DIR")]
    export: Option<PathBuf>,
}

/// One captured frame, numbered from 1 as in Wireshark.
struct Frame {
    number: usize,
    link: DataLink,
    data: Vec<u8>,
}

/// A TCP segment with payload.
struct Segment {
    frame: usize,
    seq: u32,
    payload: Vec<u8>,
}

/// One direction of a TCP connection.
#[derive(Default)]
struct Flow {
    isn: Option<u32>,
    segments: Vec<Segment>,
}

/// A reassembled byte stream and the frame that carried each byte range.
struct Stream {
    data: Vec<u8>,
    /// `(start offset, frame number)`, sorted by offset.
    frames: Vec<(usize, usize)>,
}

impl Stream {
    fn frame_at(&self, offset: usize) -> usize {
        let i = self.frames.partition_point(|&(start, _)| start <= offset);
        self.frames[i.saturating_sub(1)].1
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Dir {
    ClientToServer,
    ServerToClient,
}

impl std::fmt::Display for Dir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ClientToServer => "C->S",
            Self::ServerToClient => "S->C",
        })
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let frames = read_frames(&args.capture)?;

    let mut connections: BTreeMap<(SocketAddr, SocketAddr), [Flow; 2]> = BTreeMap::new();
    for frame in &frames {
        let Some((src, dst, tcp)) = parse_tcp(frame) else {
            continue;
        };
        let (key, dir) = if dst.port() == args.port {
            ((src, dst), Dir::ClientToServer)
        } else if src.port() == args.port {
            ((dst, src), Dir::ServerToClient)
        } else {
            continue;
        };
        let flow = &mut connections.entry(key).or_default()[dir as usize];
        if tcp.syn {
            flow.isn = Some(tcp.seq);
        }
        if !tcp.payload.is_empty() {
            flow.segments.push(Segment {
                frame: frame.number,
                seq: tcp.seq,
                payload: tcp.payload,
            });
        }
    }

    let mut keys = vec![("empty password".to_string(), AccountKey::EMPTY_PASSWORD)];
    for (i, password) in args.passwords.iter().enumerate() {
        keys.push((
            format!("--password #{}", i + 1),
            AccountKey::from_password(password.as_bytes()),
        ));
    }

    if connections.is_empty() {
        println!("no TCP connections on port {}", args.port);
    }
    let mut pending = Vec::new();
    for ((client, server), [c2s, s2c]) in &connections {
        let streams = [reassemble(c2s), reassemble(s2c)];
        if let Some(dir) = &args.export {
            for (stream, suffix) in streams.iter().zip(["c2s", "s2c"]) {
                let path = dir.join(format!("{}.{suffix}.bin", client.port()));
                std::fs::write(&path, &stream.data)
                    .with_context(|| format!("writing {}", path.display()))?;
            }
        }
        pending.push((format!("connection {client} -> {server}\n"), streams, None));
    }

    // Transfer connections use a session key from another connection's 105
    // Server Info, so retry until no new key turns up.
    loop {
        let mut found = Vec::new();
        for (_, streams, output) in pending.iter_mut().filter(|(_, _, o)| o.is_none()) {
            let mut out = String::new();
            match dissect(streams, &keys, &mut out) {
                Ok(Some(new_keys)) => {
                    found.extend(new_keys);
                    *output = Some(out);
                }
                Ok(None) => {}
                Err(err) => {
                    writeln!(out, "  stopped: {err:#}").unwrap();
                    *output = Some(out);
                }
            }
        }
        if found.is_empty() {
            break;
        }
        keys.extend(found);
    }

    for (title, streams, output) in pending {
        let output = output.unwrap_or_else(|| {
            let mut out = String::new();
            dissect(&streams, &keys, &mut out).ok();
            out.push_str(
                "  stopped: no candidate key matches the client's proof; try --password\n",
            );
            out
        });
        println!("{title}{output}");
    }
    Ok(())
}

/// Writes one connection to `out`, and returns the session keys its 105 Server
/// Info messages carry, or `None` if no key matches its proof.
fn dissect(
    streams: &[Stream; 2],
    keys: &[(String, AccountKey)],
    out: &mut String,
) -> Result<Option<Vec<(String, AccountKey)>>> {
    let dirs = [Dir::ClientToServer, Dir::ServerToClient];
    let mut hellos = Vec::new();
    for (stream, dir) in streams.iter().zip(dirs) {
        let (hello, len) = Hello::decode(&stream.data).with_context(|| format!("{dir} hello"))?;
        writeln!(
            out,
            "  [{}] {dir} hello: service {} format {} version {:#04x} challenge {} nonce {} name {}",
            stream.frame_at(0),
            fourcc(hello.service),
            hello.format,
            hello.version,
            hello.challenge,
            hex::encode(hello.nonce),
            quoted(&hello.name)
        )?;
        hellos.push((hello, len));
    }

    let mut proofs = Vec::new();
    for ((stream, dir), (_, hello_len)) in streams.iter().zip(dirs).zip(&hellos) {
        let proof =
            Proof::decode(&stream.data[*hello_len..]).with_context(|| format!("{dir} proof"))?;
        writeln!(
            out,
            "  [{}] {dir} proof: status {:?} auth {} response {}",
            stream.frame_at(*hello_len),
            proof.status,
            hex::encode(proof.auth),
            quoted(&proof.response)
        )?;
        proofs.push(proof);
    }

    let server_nonce = &hellos[1].0.nonce;
    let Some((label, key)) = keys
        .iter()
        .find(|(_, key)| proofs[0].auth == compute_auth(key, server_nonce))
    else {
        return Ok(None);
    };
    let server_ok = proofs[1].auth == compute_auth(key, &hellos[0].0.nonce);
    writeln!(
        out,
        "  key: {label} (server proof {})",
        if server_ok {
            "matches"
        } else {
            "DOES NOT match"
        }
    )?;

    let cipher = SessionCipher::new(key);
    let mut messages = Vec::new();
    for ((stream, dir), (_, hello_len)) in streams.iter().zip(dirs).zip(&hellos) {
        let mut at = hello_len + Proof::LEN;
        while at < stream.data.len() {
            let frame = stream.frame_at(at);
            let Some(block) = stream.data.get(at..at + Header::LEN) else {
                messages.push((
                    frame,
                    dir,
                    Err(format!("{} trailing bytes", stream.data.len() - at)),
                ));
                break;
            };
            let header = match cipher.open_data_header(block.try_into().unwrap()) {
                Ok(header) => header,
                Err(err) => {
                    messages.push((
                        frame,
                        dir,
                        Err(format!("bad header at stream offset {at}: {err}")),
                    ));
                    break;
                }
            };
            let payload = cipher.open_payload(&header, &stream.data[at + Header::LEN..]);
            match payload {
                Ok(payload) => messages.push((frame, dir, Ok((header, payload)))),
                Err(err) => {
                    messages.push((frame, dir, Err(format!("txid {:08x}: {err}", header.txid))));
                    break;
                }
            }
            at += Header::LEN + header.padded_len();
        }
    }

    messages.sort_by_key(|(frame, dir, _)| (*frame, *dir));
    let mut new_keys = Vec::new();
    for (frame, dir, message) in messages {
        match message {
            Ok((header, payload)) => {
                writeln!(
                    out,
                    "  [{frame}] {dir} txid {:08x} type {}{} len {}",
                    header.txid,
                    header.kind,
                    if header.is_reply() { " reply" } else { "" },
                    header.len
                )?;
                if header.kind == MessageType::SERVER_INFO {
                    if let Ok(info) = ServerInfo::decode(&payload) {
                        new_keys.push((
                            format!("session key of user {}", info.user_id),
                            AccountKey(info.session_key),
                        ));
                    }
                }
                match summary(header.kind, &payload) {
                    Ok(Some(text)) => writeln!(out, "      = {text}")?,
                    Ok(None) => {}
                    Err(err) => writeln!(out, "      ! {err}")?,
                }
                out.push_str(&hexdump(&payload[..payload.len().min(MAX_DUMP)], "      "));
                if payload.len() > MAX_DUMP {
                    writeln!(out, "      … {} more bytes", payload.len() - MAX_DUMP)?;
                }
            }
            Err(err) => writeln!(out, "  [{frame}] {dir} {err}")?,
        }
    }
    Ok(Some(new_keys))
}

/// Describes a payload whose layout `kdx_proto::msg` knows.
fn summary(kind: MessageType, payload: &[u8]) -> Result<Option<String>> {
    let user = |u: &UserRecord| {
        format!(
            "user {} {} status {} flags {:#04x} icon {} bytes info {}",
            u.user_id,
            quoted(&u.nick),
            u.status,
            u.flags,
            u.icon.len(),
            quoted(&u.info)
        )
    };
    let class = |c: &AccountClass| {
        format!(
            "class {} {} privileges {}",
            c.index,
            quoted(&c.name),
            hex(&c.privileges)
        )
    };
    let history = |r: &HistoryRecord| {
        let mut text = format!(
            "time {} kind {} user {} {} login {} from {}",
            r.time,
            r.kind,
            r.user_id,
            quoted(&r.nick),
            quoted(&r.login),
            quoted(&r.address)
        );
        if !r.detail.is_empty() {
            text += &format!(" detail {}", quoted(&r.detail));
        }
        if !r.path.0.is_empty() {
            text += &format!(" path {}", path(&r.path));
        }
        text
    };
    let text = match kind {
        MessageType::STATUS => match Status::decode(payload)? {
            Status::Success { value } => format!("success {value:#x}"),
            Status::Error {
                code,
                request,
                module,
                line,
                extra,
            } => format!(
                "error {code:#x} for {request} at module {module:#x} line {line} extra {extra:#x}"
            ),
            Status::TrackerError { code, request } => format!("error {code:#x} for {request}"),
        },
        MessageType::SET_IDENTITY | MessageType::USER_CHANGED | MessageType::USER_JOINED => {
            user(&UserRecord::decode(payload)?.0)
        }
        MessageType::USER_LIST => UserRecord::decode_list(payload)?
            .iter()
            .map(user)
            .collect::<Vec<_>>()
            .join("; "),
        MessageType::USER_DEPARTED => {
            let departed = UserDeparted::decode(payload)?;
            format!("user {} value {}", departed.user_id, departed.value_4)
        }
        MessageType::SERVER_INFO => {
            let info = ServerInfo::decode(payload)?;
            format!(
                "you are user {}, server {} version {} name {} description {}",
                info.user_id,
                info.server_value,
                info.version,
                quoted(&info.name),
                quoted(&info.description)
            )
        }
        MessageType::SERVER_ICON => format!("{} bytes", ServerIcon::decode(payload)?.0.len()),
        MessageType::SERVER_GREETING => quoted(&ServerGreeting::decode(payload)?.text),
        MessageType::SET_MINI_USER_ICON => {
            format!("status {}", MiniUserIcon::decode(payload)?.status)
        }
        MessageType::MINI_USER_ICON_CHANGED => {
            let changed = MiniUserIconChanged::decode(payload)?;
            format!("user {} status {}", changed.user_id, changed.icon.status)
        }
        MessageType::UNNAMED_118 => {
            let notice = RegistrationNotice::decode(payload)?;
            format!(
                "value {} ({})",
                notice.value,
                if notice.is_registered() {
                    "registered"
                } else {
                    "unregistered"
                }
            )
        }
        MessageType::SEND_PRIVATE_MESSAGE | MessageType::INCOMING_PRIVATE_MESSAGE => {
            let message = PrivateMessage::decode(payload)?;
            format!("user {} text {}", message.user_id, quoted(&message.text))
        }
        MessageType::GET_FILE_LIST | MessageType::GET_FILE_FOLDER_INFO => {
            path(&FilePath::decode(payload)?)
        }
        MessageType::DISCONNECT_USER => {
            let request = DisconnectUser::decode(payload)?;
            format!(
                "user {} ban {} s options {:#x} message {}",
                request.user_id,
                request.ban_seconds,
                request.options,
                quoted(&request.message)
            )
        }
        MessageType::DISCONNECT_MESSAGE => {
            let notice = DisconnectMessage::decode(payload)?;
            format!(
                "from user {} {} flags {:#x} message {}",
                notice.user_id,
                quoted(&notice.name),
                notice.flags,
                quoted(&notice.message)
            )
        }
        MessageType::GET_USER_INFO => format!("user {}", GetUserInfo::decode(payload)?.user_id),
        MessageType::USER_INFO => {
            let info = UserInfo::decode(payload)?;
            format!(
                "user {} {} login {} from {}",
                info.user_id,
                quoted(&info.nick),
                quoted(&info.login),
                quoted(&info.address)
            )
        }
        MessageType::DELETE_CHAT
        | MessageType::LEAVE_CHAT
        | MessageType::GET_CHAT
        | MessageType::GET_CHAT_DCC_ADDRESSES => format!("chat {}", ChatId::decode(payload)?.0),
        MessageType::SEND_BROADCAST_MESSAGE => {
            let broadcast = SendBroadcast::decode(payload)?;
            format!(
                "value {:#x} text {}",
                broadcast.value_2,
                quoted(&broadcast.text)
            )
        }
        MessageType::RCV_BROADCAST_MESSAGE => {
            let broadcast = RcvBroadcast::decode(payload)?;
            format!(
                "from user {} value {:#x} text {}",
                broadcast.user_id,
                broadcast.value_0e,
                quoted(&broadcast.text)
            )
        }
        MessageType::SHUTDOWN_SERVER => {
            let request = ShutdownServer::decode(payload)?;
            format!(
                "mode {:#x} message {}",
                request.mode,
                quoted(&request.message)
            )
        }
        MessageType::JOIN_CHAT => {
            let join = JoinChat::decode(payload)?;
            format!("chat {} key {:#x}", join.chat_id, join.key)
        }
        MessageType::CHAT_WINDOW_INFO | MessageType::CREATE_PRIVATE_CHAT_REPLY => {
            let window = ChatWindowInfo::decode(payload)?;
            format!(
                "chat {} {} members {:?}",
                window.chat_id,
                quoted(window.name()),
                window.members
            )
        }
        MessageType::CHAT_USER_JOINED | MessageType::CHAT_USER_DEPARTED => {
            let change = ChatUserChange::decode(payload)?;
            format!("chat {} user {}", change.chat_id, change.user_id)
        }
        MessageType::SEND_CHAT_TEXT | MessageType::RECEIVE_CHAT_TEXT => {
            let chat = ChatText::decode(payload)?;
            format!(
                "chat {} user {} text {}",
                chat.chat_id,
                chat.user_id,
                quoted(&chat.text)
            )
        }
        MessageType::SET_CHAT_TOPIC => {
            let set = SetChatTopic::decode(payload)?;
            format!("chat {} topic {}", set.chat_id, quoted(&set.topic))
        }
        MessageType::CHAT_TOPIC_CHANGED => {
            let topic = ChatTopicChanged::decode(payload)?;
            format!(
                "chat {} topic {} by {} {}",
                topic.chat_id,
                quoted(&topic.topic),
                topic.user_id,
                quoted(&topic.nick)
            )
        }
        MessageType::CHAT_INVITE => {
            let invite = ChatInvite::decode(payload)?;
            format!(
                "chat {} users {:?} message {}",
                invite.chat_id,
                invite.user_ids,
                quoted(&invite.message)
            )
        }
        MessageType::CHAT_INVITATION => {
            let invitation = ChatInvitation::decode(payload)?;
            format!(
                "chat {} {} key {:#x} from user {} message {}",
                invitation.chat_id,
                quoted(&invitation.name),
                invitation.key,
                invitation.user_id,
                quoted(&invitation.message)
            )
        }
        MessageType::CHAT_LIST => ChatListEntry::decode_list(payload)?
            .iter()
            .map(|c| {
                format!(
                    "chat {} {} ({} members)",
                    c.chat_id,
                    quoted(&c.name),
                    c.member_count
                )
            })
            .collect::<Vec<_>>()
            .join("; "),
        MessageType::CREATE_CHAT | MessageType::CHAT_INFO | MessageType::SET_CHAT => {
            let chat = ChatRecord::decode(payload)?;
            format!(
                "chat {} {} {} flags {:#x}",
                chat.chat_id,
                quoted(&chat.name),
                quoted(&chat.description),
                chat.flags
            )
        }
        MessageType::CHAT_DELETED => {
            let deleted = ChatDeleted::decode(payload)?;
            format!("chat {} by user {}", deleted.chat_id, deleted.user_id)
        }
        MessageType::FILE_LIST => FileListEntry::decode_list(payload)?
            .iter()
            .map(|e| {
                if e.is_folder() {
                    format!("{}/", quoted(&e.name))
                } else {
                    format!("{} {} bytes", quoted(&e.name), e.size)
                }
            })
            .collect::<Vec<_>>()
            .join("; "),
        MessageType::FILE_FOLDER_INFO => {
            let info = FileInfo::decode(payload)?;
            format!(
                "{} kind {} size {}",
                quoted(&info.name),
                quoted(&info.kind),
                info.size
            )
        }
        MessageType::CREATE_FOLDER => {
            let create = CreateFolder::decode(payload)?;
            format!(
                "{} in {} access {} owner {}",
                quoted(&create.name),
                path(&create.path),
                quoted(&create.access),
                quoted(&create.owner)
            )
        }
        MessageType::CREATE_ALIAS | MessageType::MOVE_FILE_FOLDER => {
            let request = MoveItems::decode(payload)?;
            format!(
                "{} from {} to {}",
                names(&request.names),
                path(&request.source),
                path(&request.destination)
            )
        }
        MessageType::MOVE_TO_TRASH => {
            let request = MoveToTrash::decode(payload)?;
            format!("{} in {}", names(&request.names), path(&request.folder))
        }
        MessageType::SET_FILE_FOLDER_INFO => {
            let set = SetFileInfo::decode(payload)?;
            format!(
                "{} flags {:#x} name {}",
                path(&set.path),
                set.flags,
                quoted(&set.name)
            )
        }
        MessageType::VOLUME_INFO => {
            let volume = VolumeInfo::decode(payload)?;
            format!(
                "{} format {} comments {} disk type {} icon {:#x} flags {:#x} of {:#x}",
                quoted(&volume.name),
                quoted(&volume.format),
                quoted(&volume.comments),
                volume.disk_type,
                volume.icon,
                volume.flags,
                volume.flags_mask
            )
        }
        MessageType::SET_VOLUME_INFO => {
            let set = SetVolumeInfo::decode(payload)?;
            format!(
                "{} flags {:#x} new name {} comments {}",
                quoted(&set.name),
                set.flags,
                quoted(&set.new_name),
                quoted(&set.comments)
            )
        }
        MessageType::DOWNLOAD_FILE => {
            let request = DownloadFile::decode(payload)?;
            format!(
                "{} from {} options {:#x}",
                path(&request.path),
                request.data_offset,
                request.options
            )
        }
        MessageType::DOWNLOAD_FILE_REPLY => {
            let info = DownloadInfo::decode(payload)?;
            format!(
                "{} data {} bytes rsrc {} bytes",
                quoted(&info.strings.name),
                info.data_size,
                info.rsrc_size
            )
        }
        MessageType::UPLOAD_FILE => {
            let request = UploadFile::decode(payload)?;
            format!(
                "{} into {} data {} bytes options {:#x}",
                quoted(&request.strings.name),
                path(&request.folder),
                request.data_size,
                request.options
            )
        }
        MessageType::UPLOAD_FILE_REPLY => {
            let reply = UploadReply::decode(payload)?;
            format!("send data from {}", reply.data_offset)
        }
        MessageType::LAUNCH_FILE => path(&LaunchFile::decode(payload)?.path),
        MessageType::GENERATE_CATALOG => {
            let catalog = GenerateCatalog::decode(payload)?;
            if catalog.flags & GenerateCatalog::CANCEL != 0 {
                "cancel".to_owned()
            } else {
                format!("start, flags {:#x}", catalog.flags)
            }
        }
        MessageType::GENERATE_CATALOG_STATUS => {
            let status = CatalogStatus::decode(payload)?;
            let done = if status.is_done() { ", done" } else { "" };
            format!("{} items{done}", status.items)
        }
        MessageType::FILE_SEARCH => {
            format!("query {}", quoted(&FileSearch::decode(payload)?.query))
        }
        MessageType::FILE_SEARCH_REPLY => FileSearchEntry::decode_list(payload)?
            .iter()
            .map(|e| format!("{} {} bytes", path(&e.path), e.size))
            .collect::<Vec<_>>()
            .join("; "),
        MessageType::NOTIFY_SPEED_LIMIT => format!(
            "{} B/s",
            NotifySpeedLimit::decode(payload)?.bytes_per_second
        ),
        MessageType::NEWSGROUP_LIST => NewsgroupEntry::decode_list(payload)?
            .iter()
            .map(|g| {
                format!(
                    "group {} {} ({} messages)",
                    g.group_id,
                    quoted(&g.name),
                    g.message_count
                )
            })
            .collect::<Vec<_>>()
            .join("; "),
        MessageType::DELETE_NEWSGROUP
        | MessageType::GET_NEWSGROUP
        | MessageType::GET_NEWS_MESSAGES => {
            need(payload, 4)?;
            format!("group {}", u32::from_be_bytes(payload[..4].try_into()?))
        }
        MessageType::GET_NEWS_MESSAGE | MessageType::DELETE_NEWS_MESSAGE => {
            need(payload, 8)?;
            format!(
                "group {} message {}",
                u32::from_be_bytes(payload[..4].try_into()?),
                u32::from_be_bytes(payload[4..8].try_into()?)
            )
        }
        MessageType::NEWS_MESSAGE => {
            let (m, _) = NewsMessage::decode(payload)?;
            format!(
                "group {} message {} {} by {} login {}, {} bytes",
                m.group_id,
                m.message_id,
                quoted(&m.subject),
                quoted(&m.poster),
                quoted(&m.login),
                m.body.len()
            )
        }
        MessageType::POST_NEWS_MESSAGE => {
            let post = PostNewsMessage::decode(payload)?;
            format!(
                "group {} subject {} body {}",
                post.group_id,
                quoted(&post.subject),
                quoted(&post.body)
            )
        }
        MessageType::NEWS_MESSAGE_POSTED | MessageType::NEWS_MESSAGE_DELETED => {
            let change = NewsMessageChange::decode(payload)?;
            format!(
                "group {} message {}, {} messages now",
                change.group_id, change.message_id, change.message_count
            )
        }
        MessageType::NEWS_MESSAGES => {
            let news = NewsMessages::decode(payload)?;
            let subjects: Vec<String> = news
                .messages
                .iter()
                .map(|m| {
                    format!(
                        "{} {} by {}",
                        m.message_id,
                        quoted(&m.subject),
                        quoted(&m.poster)
                    )
                })
                .collect();
            format!(
                "group {} permissions {:#x}: {}",
                news.group_id,
                news.permissions,
                subjects.join("; ")
            )
        }
        MessageType::CREATE_NEWSGROUP
        | MessageType::NEWSGROUP_INFO
        | MessageType::SET_NEWSGROUP => {
            let group = NewsgroupInfo::decode(payload)?;
            format!(
                "group {} {} ({} messages, limit {} KiB) read {:#010x} post {:#010x}",
                group.group_id,
                quoted(&group.name),
                group.message_count,
                group.size_limit_kib,
                group.read_classes,
                group.post_classes
            )
        }
        MessageType::EXTRA_SERVER_INFO => {
            let info = ExtraServerInfo::decode(payload)?;
            format!("uptime {} s, os {}", info.uptime, quoted(&info.os))
        }
        MessageType::SERVER_SETTINGS_INFO | MessageType::SET_SERVER_SETTINGS => {
            let settings = ServerSettings::decode(payload)?;
            format!(
                "version {} name {} greeting {} ({}) speed limit {} B/s icon {} bytes",
                settings.version,
                quoted(settings.name()),
                settings
                    .greeting()
                    .map_or_else(|| "none".to_owned(), |g| quoted(&g.text)),
                if settings.greeting_enabled() {
                    "enabled"
                } else {
                    "disabled"
                },
                settings.speed_limit(),
                settings.icon().len()
            )
        }
        MessageType::YOUR_ADDRESS => quoted(&YourAddress::decode(payload)?.0),
        MessageType::CONNECTION_MONITOR_INFO => MonitoredConnection::decode_list(payload)?
            .iter()
            .map(|c| match c.transfer_id {
                0 => format!(
                    "user {} {} login {} from {}",
                    c.user_id,
                    quoted(&c.nick),
                    quoted(&c.login),
                    quoted(&c.address)
                ),
                id => format!(
                    "transfer {id} of user {} {}: {}/{} bytes at {} B/s, {} s left, state {:#x}",
                    c.user_id,
                    path(&c.path),
                    c.bytes_done,
                    c.bytes_total,
                    c.rate,
                    c.seconds_left,
                    c.state
                ),
            })
            .collect::<Vec<_>>()
            .join("; "),
        MessageType::GET_ACCOUNT_CLASS => {
            need(payload, 1)?;
            format!("class {}", payload[0])
        }
        MessageType::SERVER_HISTORY_JOIN_REPLY => HistoryRecord::decode_list(payload)?
            .iter()
            .map(history)
            .collect::<Vec<_>>()
            .join("; "),
        MessageType::SERVER_HISTORY_ADDED => history(&HistoryRecord::decode_added(payload)?),
        MessageType::ACCOUNT_CLASS_INFO => class(&AccountClass::decode_info(payload)?),
        MessageType::SET_ACCOUNT_CLASS => class(&AccountClass::decode_set(payload)?),
        MessageType::ACCOUNT_CLASS_LIST => AccountClass::decode_list(payload)?
            .iter()
            .map(class)
            .collect::<Vec<_>>()
            .join("; "),
        MessageType::ACCOUNT_INFO | MessageType::CREATE_ACCOUNT | MessageType::SET_ACCOUNT => {
            let account = Account::decode(payload)?;
            // The key is the account's secret; show only whether one is set.
            let key = if account.key == [0; 32] {
                "none"
            } else if kind == MessageType::ACCOUNT_INFO {
                "flag set"
            } else {
                "given"
            };
            format!(
                "login {} name {} class {} key {key} editor {} privileges {} of {}",
                quoted(&account.login),
                quoted(&account.name),
                account.class_index,
                quoted(&account.editor),
                hex(&account.privileges),
                hex(&account.privilege_mask)
            )
        }
        MessageType::GET_ACCOUNT | MessageType::DELETE_ACCOUNT | MessageType::ACCOUNT_DELETED => {
            format!("login {}", quoted(payload))
        }
        MessageType::CHANGE_OWN_PASSWORD => format!("new key, {} bytes", payload.len()),
        MessageType::ACCOUNT_LIST => AccountListEntry::decode_list(payload)?
            .iter()
            .map(|a| {
                format!(
                    "login {} name {} class {}",
                    quoted(&a.login),
                    quoted(&a.name),
                    a.class_index
                )
            })
            .collect::<Vec<_>>()
            .join("; "),
        MessageType::PROCESS_LIST => ProcessList::decode(payload)?
            .processes
            .iter()
            .map(|p| {
                format!(
                    "pid {} user {} {}",
                    p.pid,
                    quoted(&p.user),
                    quoted(&p.command)
                )
            })
            .collect::<Vec<_>>()
            .join("; "),
        MessageType::EXIT_TERMINATE_PROCESS => {
            let request = ExitTerminateProcess::decode(payload)?;
            format!("pid {} flags {:#x}", request.pid, request.flags)
        }
        MessageType::FOLDER_ACCESS_LIST => FolderAccessItem::decode_list(payload)?
            .iter()
            .map(|i| {
                format!(
                    "item {:#x} {} {}",
                    i.item_id,
                    quoted(&i.name),
                    quoted(&i.description)
                )
            })
            .collect::<Vec<_>>()
            .join("; "),
        MessageType::GET_FOLDER_ACCESS_LIST if payload.is_empty() => String::new(),
        MessageType::GET_FOLDER_ACCESS_LIST => {
            need(payload, 4)?;
            format!(
                "flags {:#010x}",
                u32::from_be_bytes(payload[..4].try_into()?)
            )
        }
        MessageType::DELETE_FOLDER_ACCESS
        | MessageType::FOLDER_ACCESS_DELETED
        | MessageType::GET_FOLDER_ACCESS => format!("item {}", quoted(payload)),
        MessageType::CREATE_FOLDER_ACCESS
        | MessageType::FOLDER_ACCESS_INFO
        | MessageType::SET_FOLDER_ACCESS => {
            let item = FolderAccess::decode(payload)?;
            let mut text = format!(
                "item {} owner {} icon {:#x} flags {:#x} {}",
                quoted(&item.name),
                quoted(&item.owner),
                item.icon,
                item.flags,
                quoted(&item.description)
            );
            if !item.current_name.is_empty() {
                write!(text, " was {}", quoted(&item.current_name))?;
            }
            text
        }
        _ => return Ok(None),
    };
    if text.is_empty() {
        return Ok(Some("(empty)".to_owned()));
    }
    Ok(Some(text))
}

fn read_frames(path: &PathBuf) -> Result<Vec<Frame>> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let mut frames = Vec::new();
    if bytes.starts_with(&[0x0a, 0x0d, 0x0d, 0x0a]) {
        let mut reader = PcapNgReader::new(&bytes[..])?;
        let mut links = Vec::new();
        while let Some(block) = reader.next_block() {
            let (interface, data) = match block? {
                Block::InterfaceDescription(idb) => {
                    links.push(idb.linktype);
                    continue;
                }
                Block::EnhancedPacket(epb) => (epb.interface_id as usize, epb.data.into_owned()),
                Block::SimplePacket(spb) => (0, spb.data.into_owned()),
                Block::Packet(pb) => (usize::from(pb.interface_id), pb.data.into_owned()),
                _ => continue,
            };
            let link = *links
                .get(interface)
                .context("packet before its interface description")?;
            frames.push(Frame {
                number: frames.len() + 1,
                link,
                data,
            });
        }
    } else {
        let mut reader = PcapReader::new(&bytes[..])?;
        let link = reader.header().datalink;
        while let Some(packet) = reader.next_packet() {
            frames.push(Frame {
                number: frames.len() + 1,
                link,
                data: packet?.data.into_owned(),
            });
        }
    }
    Ok(frames)
}

struct Tcp {
    seq: u32,
    syn: bool,
    payload: Vec<u8>,
}

fn parse_tcp(frame: &Frame) -> Option<(SocketAddr, SocketAddr, Tcp)> {
    let data = &frame.data;
    let packet = match frame.link {
        DataLink::ETHERNET => SlicedPacket::from_ethernet(data),
        DataLink::LINUX_SLL => SlicedPacket::from_linux_sll(data),
        DataLink::RAW | DataLink::IPV4 | DataLink::IPV6 => SlicedPacket::from_ip(data),
        DataLink::NULL | DataLink::LOOP => SlicedPacket::from_ip(data.get(4..)?),
        _ => return None,
    }
    .ok()?;
    let (src, dst): (IpAddr, IpAddr) = match packet.net? {
        NetSlice::Ipv4(ip) => (
            ip.header().source_addr().into(),
            ip.header().destination_addr().into(),
        ),
        NetSlice::Ipv6(ip) => (
            ip.header().source_addr().into(),
            ip.header().destination_addr().into(),
        ),
        _ => return None,
    };
    let TransportSlice::Tcp(tcp) = packet.transport? else {
        return None;
    };
    Some((
        SocketAddr::new(src, tcp.source_port()),
        SocketAddr::new(dst, tcp.destination_port()),
        Tcp {
            seq: tcp.sequence_number(),
            syn: tcp.syn(),
            payload: tcp.payload().to_vec(),
        },
    ))
}

/// Orders segments by sequence number, drops retransmitted bytes, and stops at the first gap.
fn reassemble(flow: &Flow) -> Stream {
    let mut stream = Stream {
        data: Vec::new(),
        frames: Vec::new(),
    };
    let Some(first) = flow.segments.first() else {
        return stream;
    };
    let base = flow.isn.map_or(first.seq, |isn| isn.wrapping_add(1));
    let mut segments: Vec<(usize, &Segment)> = flow
        .segments
        .iter()
        .map(|s| (s.seq.wrapping_sub(base) as usize, s))
        .filter(|&(offset, _)| offset < 1 << 31)
        .collect();
    segments.sort_by_key(|&(offset, s)| (offset, s.frame));
    for (offset, segment) in segments {
        let end = offset + segment.payload.len();
        if offset > stream.data.len() {
            break;
        }
        if end > stream.data.len() {
            stream.frames.push((stream.data.len(), segment.frame));
            stream
                .data
                .extend_from_slice(&segment.payload[stream.data.len() - offset..]);
        }
    }
    stream
}

fn need(payload: &[u8], len: usize) -> Result<()> {
    anyhow::ensure!(payload.len() >= len, "payload too short: need {len} bytes");
    Ok(())
}

fn path(path: &FilePath) -> String {
    let names: Vec<String> = path.0.iter().map(|n| quoted(n)).collect();
    format!("/{}", names.join("/"))
}

fn names(names: &[Vec<u8>]) -> String {
    names
        .iter()
        .map(|n| quoted(n))
        .collect::<Vec<_>>()
        .join(", ")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn fourcc(value: u32) -> String {
    quoted(&value.to_be_bytes())
}

fn quoted(bytes: &[u8]) -> String {
    format!("\"{}\"", bytes.escape_ascii())
}

fn hexdump(data: &[u8], indent: &str) -> String {
    let mut out = String::new();
    for (i, chunk) in data.chunks(16).enumerate() {
        let hex: Vec<String> = chunk.iter().map(|b| format!("{b:02x}")).collect();
        let ascii: String = chunk
            .iter()
            .map(|&b| {
                if b.is_ascii_graphic() || b == b' ' {
                    b as char
                } else {
                    '.'
                }
            })
            .collect();
        writeln!(
            out,
            "{indent}{:04x}  {:<47}  {ascii}",
            i * 16,
            hex.join(" ")
        )
        .unwrap();
    }
    out
}
