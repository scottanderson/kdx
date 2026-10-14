//! Re-encodes each decoded payload to find message types the codecs don't reproduce.

use kdx_proto::msg::admin::{
    Account, AccountClass, AccountListEntry, DisconnectMessage, DisconnectUser,
    ExitTerminateProcess, ExtraServerInfo, FolderAccess, FolderAccessItem, FolderAccessName,
    GetFolderAccessList, HistoryList, HistoryRecord, MonitoredConnection, ProcessList,
    RcvBroadcast, SendBroadcast, ShutdownServer, YourAddress,
};
use kdx_proto::msg::chat::{
    ChatDeleted, ChatId, ChatInvitation, ChatInvite, ChatListEntry, ChatRecord, ChatText,
    ChatTopicChanged, ChatUserChange, ChatWindowInfo, JoinChat, SetChatTopic,
};
use kdx_proto::msg::dcc::Skpk;
use kdx_proto::msg::file::{
    CatalogStatus, CreateFolder, FileInfo, FileListEntry, FileSearch, FileSearchEntry,
    GenerateCatalog, LaunchFile, MoveItems, MoveToTrash, SetFileInfo, SetVolumeInfo, VolumeInfo,
};
use kdx_proto::msg::gconsole::{Draw, Key, Mouse, OpenWindow};
use kdx_proto::msg::news::{
    NewsMessage, NewsMessageChange, NewsMessageId, NewsMessages, NewsgroupEntry, NewsgroupId,
    NewsgroupInfo, PostNewsMessage,
};
use kdx_proto::msg::server_info::{ServerGreeting, ServerIcon, ServerSettings};
use kdx_proto::msg::tracker::{
    GetServerList, GroupListEntry, ServerListEntry, TrackerSettings, TrackerStats,
};
use kdx_proto::msg::transfer::{
    DownloadFile, DownloadInfo, NotifySpeedLimit, UploadFile, UploadReply,
};
use kdx_proto::msg::user::{MiniUserIcon, MiniUserIconChanged, RegistrationNotice, UserDeparted};
use kdx_proto::msg::user_info::GetUserInfo;
use kdx_proto::msg::{DecodeError, EncodeError};
use kdx_proto::msg::{FilePath, PrivateMessage, ServerInfo, Status, UserInfo, UserRecord};
use kdx_proto::MessageType;

/// How a payload fared when decoded and encoded again.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The encoder gave back the payload byte for byte.
    Same,
    /// No `kdx-proto` type decodes this message type.
    NoCodec,
    /// The decoder failed; `summary` reports why.
    Undecodable,
    /// The bytes that came back differ, which is a codec bug.
    Mismatch(String),
}

/// Encoder results of any shape, as bytes.
trait Bytes {
    fn bytes(self) -> Result<Vec<u8>, EncodeError>;
}

impl Bytes for Vec<u8> {
    fn bytes(self) -> Result<Vec<u8>, EncodeError> {
        Ok(self)
    }
}

impl<const N: usize> Bytes for [u8; N] {
    fn bytes(self) -> Result<Vec<u8>, EncodeError> {
        Ok(self.to_vec())
    }
}

impl<T: Bytes> Bytes for Result<T, EncodeError> {
    fn bytes(self) -> Result<Vec<u8>, EncodeError> {
        self?.bytes()
    }
}

/// Decodes `payload` as `kind`, encodes it again, and compares.
pub fn roundtrip(kind: MessageType, payload: &[u8]) -> Outcome {
    match reencode(kind, payload) {
        None => Outcome::NoCodec,
        Some(Err(Fault::Decode)) => Outcome::Undecodable,
        Some(Err(Fault::Encode(err))) => Outcome::Mismatch(format!("encode failed: {err}")),
        Some(Ok(out)) => {
            if out == payload {
                Outcome::Same
            } else {
                Outcome::Mismatch(difference(payload, &out))
            }
        }
    }
}

/// Where `out` first departs from `wire`, with a note when the lengths differ.
fn difference(wire: &[u8], out: &[u8]) -> String {
    let at = wire.iter().zip(out).take_while(|(a, b)| a == b).count();
    let show = |b: &[u8]| hex::encode(&b[at.min(b.len())..(at + 8).min(b.len())]);
    let mut how = format!("first at {at:#x}: {} became {}", show(wire), show(out));
    if wire.len() != out.len() {
        how += &format!(" ({} bytes in, {} out)", wire.len(), out.len());
    }
    how
}

/// Why a payload has no re-encoded form.
enum Fault {
    Decode,
    Encode(EncodeError),
}

impl From<DecodeError> for Fault {
    fn from(_: DecodeError) -> Self {
        Self::Decode
    }
}

impl From<EncodeError> for Fault {
    fn from(err: EncodeError) -> Self {
        Self::Encode(err)
    }
}

/// The re-encoded payload, or `None` without a codec for `kind`.
fn reencode(kind: MessageType, p: &[u8]) -> Option<Result<Vec<u8>, Fault>> {
    macro_rules! one {
        ($t:ty) => {
            (|| Ok(<$t>::decode(p)?.encode().bytes()?))()
        };
    }
    macro_rules! list {
        ($t:ty) => {
            (|| Ok(<$t>::encode_list(&<$t>::decode_list(p)?).bytes()?))()
        };
    }
    type M = MessageType;
    let out: Result<Vec<u8>, Fault> = match kind {
        M::STATUS => one!(Status),
        M::SET_IDENTITY | M::USER_CHANGED | M::USER_JOINED => {
            (|| Ok(UserRecord::decode(p)?.0.encode()?))()
        }
        M::USER_LIST => (|| Ok(UserRecord::encode_list(&UserRecord::decode_list(p)?)?))(),
        M::USER_DEPARTED => one!(UserDeparted),
        M::SERVER_INFO => one!(ServerInfo),
        M::SERVER_ICON => one!(ServerIcon),
        M::SERVER_GREETING => one!(ServerGreeting),
        M::SET_MINI_USER_ICON => one!(MiniUserIcon),
        M::MINI_USER_ICON_CHANGED => one!(MiniUserIconChanged),
        M::UNNAMED_118 => one!(RegistrationNotice),
        M::SKPK => one!(Skpk),
        M::SEND_PRIVATE_MESSAGE | M::INCOMING_PRIVATE_MESSAGE => one!(PrivateMessage),
        M::GET_FILE_LIST | M::GET_FILE_FOLDER_INFO => one!(FilePath),
        M::DISCONNECT_USER => one!(DisconnectUser),
        M::DISCONNECT_MESSAGE => one!(DisconnectMessage),
        M::GET_USER_INFO => one!(GetUserInfo),
        M::USER_INFO => one!(UserInfo),
        M::DELETE_CHAT | M::LEAVE_CHAT | M::GET_CHAT | M::GET_CHAT_DCC_ADDRESSES => one!(ChatId),
        M::SEND_BROADCAST_MESSAGE => one!(SendBroadcast),
        M::RCV_BROADCAST_MESSAGE => one!(RcvBroadcast),
        M::SHUTDOWN_SERVER => one!(ShutdownServer),
        M::JOIN_CHAT => one!(JoinChat),
        M::CHAT_WINDOW_INFO | M::CREATE_PRIVATE_CHAT_REPLY => one!(ChatWindowInfo),
        M::CHAT_USER_JOINED | M::CHAT_USER_DEPARTED => one!(ChatUserChange),
        M::SEND_CHAT_TEXT | M::RECEIVE_CHAT_TEXT => one!(ChatText),
        M::SET_CHAT_TOPIC => one!(SetChatTopic),
        M::CHAT_TOPIC_CHANGED => one!(ChatTopicChanged),
        M::CHAT_INVITE => one!(ChatInvite),
        M::CHAT_INVITATION => one!(ChatInvitation),
        M::CHAT_LIST => list!(ChatListEntry),
        M::CREATE_CHAT | M::CHAT_INFO | M::SET_CHAT => one!(ChatRecord),
        M::CHAT_DELETED => one!(ChatDeleted),
        M::FILE_LIST => list!(FileListEntry),
        M::FILE_FOLDER_INFO => one!(FileInfo),
        M::CREATE_FOLDER => one!(CreateFolder),
        M::CREATE_ALIAS | M::MOVE_FILE_FOLDER => one!(MoveItems),
        M::MOVE_TO_TRASH => one!(MoveToTrash),
        M::SET_FILE_FOLDER_INFO => one!(SetFileInfo),
        M::VOLUME_INFO => one!(VolumeInfo),
        M::SET_VOLUME_INFO => one!(SetVolumeInfo),
        M::DOWNLOAD_FILE => one!(DownloadFile),
        M::DOWNLOAD_FILE_REPLY => one!(DownloadInfo),
        M::UPLOAD_FILE => one!(UploadFile),
        M::UPLOAD_FILE_REPLY => one!(UploadReply),
        M::LAUNCH_FILE => one!(LaunchFile),
        M::GENERATE_CATALOG => one!(GenerateCatalog),
        M::GENERATE_CATALOG_STATUS => one!(CatalogStatus),
        M::FILE_SEARCH => one!(FileSearch),
        M::FILE_SEARCH_REPLY => list!(FileSearchEntry),
        M::NOTIFY_SPEED_LIMIT => one!(NotifySpeedLimit),
        M::DELETE_NEWSGROUP | M::GET_NEWSGROUP | M::GET_NEWS_MESSAGES => one!(NewsgroupId),
        M::GET_NEWS_MESSAGE | M::DELETE_NEWS_MESSAGE => one!(NewsMessageId),
        M::NEWSGROUP_LIST => list!(NewsgroupEntry),
        M::NEWS_MESSAGE => (|| Ok(NewsMessage::decode(p)?.0.encode()?))(),
        M::POST_NEWS_MESSAGE => one!(PostNewsMessage),
        M::NEWS_MESSAGE_POSTED | M::NEWS_MESSAGE_DELETED => one!(NewsMessageChange),
        M::NEWS_MESSAGES => one!(NewsMessages),
        M::CREATE_NEWSGROUP | M::NEWSGROUP_INFO | M::SET_NEWSGROUP => one!(NewsgroupInfo),
        M::EXTRA_SERVER_INFO => one!(ExtraServerInfo),
        M::SERVER_SETTINGS_INFO | M::SET_SERVER_SETTINGS => one!(ServerSettings),
        M::YOUR_ADDRESS => one!(YourAddress),
        M::CONNECTION_MONITOR_INFO => list!(MonitoredConnection),
        M::SERVER_HISTORY_JOIN_REPLY => one!(HistoryList),
        M::SERVER_HISTORY_ADDED => (|| Ok(HistoryRecord::decode_added(p)?.encode_added()?))(),
        M::ACCOUNT_CLASS_INFO => (|| Ok(AccountClass::decode_info(p)?.encode_info()?))(),
        M::SET_ACCOUNT_CLASS => (|| Ok(AccountClass::decode_set(p)?.encode_set()?))(),
        M::ACCOUNT_CLASS_LIST => list!(AccountClass),
        M::ACCOUNT_INFO | M::CREATE_ACCOUNT | M::SET_ACCOUNT => one!(Account),
        M::ACCOUNT_LIST => list!(AccountListEntry),
        M::PROCESS_LIST => one!(ProcessList),
        M::EXIT_TERMINATE_PROCESS => one!(ExitTerminateProcess),
        M::DELETE_FOLDER_ACCESS | M::FOLDER_ACCESS_DELETED | M::GET_FOLDER_ACCESS => {
            one!(FolderAccessName)
        }
        M::GET_FOLDER_ACCESS_LIST => one!(GetFolderAccessList),
        M::FOLDER_ACCESS_LIST => list!(FolderAccessItem),
        M::CREATE_FOLDER_ACCESS | M::FOLDER_ACCESS_INFO | M::SET_FOLDER_ACCESS => {
            one!(FolderAccess)
        }
        M::TRACKER_GROUP_LIST => list!(GroupListEntry),
        M::GET_TRACKER_SERVER_LIST => one!(GetServerList),
        M::TRACKER_SERVER_LIST => list!(ServerListEntry),
        M::TRACKERSERV_SETTINGS_INFO | M::SET_TRACKERSERV_SETTINGS => one!(TrackerSettings),
        M::TRACKERSERV_STATISTICS => one!(TrackerStats),
        M::GCONSOLE_OPEN_WINDOW => one!(OpenWindow),
        M::GCONSOLE_DRAW => one!(Draw),
        M::GCONSOLE_KEY => one!(Key),
        M::GCONSOLE_MOUSE => one!(Mouse),
        _ => return None,
    };
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_size_payload_with_extra_bytes_is_a_mismatch() {
        let outcome = roundtrip(MessageType::GET_NEWSGROUP, &[0, 0, 0, 2, 0xee]);
        assert_eq!(
            outcome,
            Outcome::Mismatch("first at 0x4: ee became  (5 bytes in, 4 out)".into())
        );
        let outcome = roundtrip(MessageType::DELETE_NEWS_MESSAGE, &[0, 0, 0, 1, 0, 0, 0]);
        assert_eq!(outcome, Outcome::Undecodable);
    }
}
