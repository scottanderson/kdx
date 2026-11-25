//! Messages the server sends on its own, decoded.

use kdx_proto::msg::admin::{
    AccountLogin, DisconnectMessage, FolderAccessName, HistoryRecord, RcvBroadcast,
};
use kdx_proto::msg::chat::{
    ChatDeleted, ChatInvitation, ChatText, ChatTopicChanged, ChatUserChange,
};
use kdx_proto::msg::news::NewsMessageChange;
use kdx_proto::msg::server_info::{ServerGreeting, ServerIcon};
use kdx_proto::msg::transfer::NotifySpeedLimit;
use kdx_proto::msg::user::UserDeparted;
use kdx_proto::msg::{DecodeError, PrivateMessage, UserRecord};
use kdx_proto::session::{Header, MessageType};

/// A message the server sent on its own, such as 208 Receive Chat Text.
///
/// [`Client::next_notice`](crate::Client::next_notice) decodes each event
/// into one of these; a type without a variant, or a payload that does not
/// decode, comes through as [`Notice::Other`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Notice {
    /// 103 User Joined.
    UserJoined(UserRecord),
    /// 102 User Changed.
    UserChanged(UserRecord),
    /// 104 User Departed.
    UserDeparted(UserDeparted),
    /// 208 Receive Chat Text.
    ChatText(ChatText),
    /// 205 Chat User Joined.
    ChatUserJoined(ChatUserChange),
    /// 206 Chat User Departed.
    ChatUserDeparted(ChatUserChange),
    /// 210 Chat Topic Changed.
    ChatTopicChanged(ChatTopicChanged),
    /// 212 Chat Invitation.
    ChatInvitation(ChatInvitation),
    /// 221 Chat Deleted.
    ChatDeleted(ChatDeleted),
    /// 301 Incoming Private Message.
    PrivateMessage(PrivateMessage),
    /// 155 Rcv Broadcast Message.
    Broadcast(RcvBroadcast),
    /// 151 Disconnect Message; the server closes the connection after it.
    Disconnect(DisconnectMessage),
    /// 613 News Message Posted.
    NewsPosted(NewsMessageChange),
    /// 614 News Message Deleted.
    NewsDeleted(NewsMessageChange),
    /// 515 Account Deleted.
    AccountDeleted(AccountLogin),
    /// 1102 Folder Access Deleted.
    FolderAccessDeleted(FolderAccessName),
    /// 183 Server History Added.
    HistoryAdded(HistoryRecord),
    /// 106 Server Icon.
    ServerIcon(ServerIcon),
    /// 107 Server Greeting.
    ServerGreeting(ServerGreeting),
    /// 213 Must Join Chat: the client is to join chat 1.
    MustJoinChat,
    /// 419 Notify Speed Limit.
    SpeedLimit(NotifySpeedLimit),
    /// Any other message, or one whose payload did not decode.
    Other(Header, Vec<u8>),
}

impl Notice {
    /// Decodes a server message by its type.
    ///
    /// A type with no variant decodes to [`Notice::Other`]; a payload that
    /// fails to decode is an error, which [`Client::next_notice`](crate::Client::next_notice)
    /// turns into [`Notice::Other`] as well.
    pub fn decode(header: &Header, payload: &[u8]) -> Result<Self, DecodeError> {
        Ok(match header.kind {
            MessageType::USER_JOINED => Self::UserJoined(UserRecord::decode(payload)?.0),
            MessageType::USER_CHANGED => Self::UserChanged(UserRecord::decode(payload)?.0),
            MessageType::USER_DEPARTED => Self::UserDeparted(UserDeparted::decode(payload)?),
            MessageType::RECEIVE_CHAT_TEXT => Self::ChatText(ChatText::decode(payload)?),
            MessageType::CHAT_USER_JOINED => Self::ChatUserJoined(ChatUserChange::decode(payload)?),
            MessageType::CHAT_USER_DEPARTED => {
                Self::ChatUserDeparted(ChatUserChange::decode(payload)?)
            }
            MessageType::CHAT_TOPIC_CHANGED => {
                Self::ChatTopicChanged(ChatTopicChanged::decode(payload)?)
            }
            MessageType::CHAT_INVITATION => Self::ChatInvitation(ChatInvitation::decode(payload)?),
            MessageType::CHAT_DELETED => Self::ChatDeleted(ChatDeleted::decode(payload)?),
            MessageType::INCOMING_PRIVATE_MESSAGE => {
                Self::PrivateMessage(PrivateMessage::decode(payload)?)
            }
            MessageType::RCV_BROADCAST_MESSAGE => Self::Broadcast(RcvBroadcast::decode(payload)?),
            MessageType::DISCONNECT_MESSAGE => {
                Self::Disconnect(DisconnectMessage::decode(payload)?)
            }
            MessageType::NEWS_MESSAGE_POSTED => {
                Self::NewsPosted(NewsMessageChange::decode(payload)?)
            }
            MessageType::NEWS_MESSAGE_DELETED => {
                Self::NewsDeleted(NewsMessageChange::decode(payload)?)
            }
            MessageType::ACCOUNT_DELETED => Self::AccountDeleted(AccountLogin::decode(payload)?),
            MessageType::FOLDER_ACCESS_DELETED => {
                Self::FolderAccessDeleted(FolderAccessName::decode(payload)?)
            }
            MessageType::SERVER_HISTORY_ADDED => {
                Self::HistoryAdded(HistoryRecord::decode_added(payload)?)
            }
            MessageType::SERVER_ICON => Self::ServerIcon(ServerIcon::decode(payload)?),
            MessageType::SERVER_GREETING => Self::ServerGreeting(ServerGreeting::decode(payload)?),
            MessageType::MUST_JOIN_CHAT => Self::MustJoinChat,
            MessageType::NOTIFY_SPEED_LIMIT => Self::SpeedLimit(NotifySpeedLimit::decode(payload)?),
            _ => Self::Other(*header, payload.to_vec()),
        })
    }
}
