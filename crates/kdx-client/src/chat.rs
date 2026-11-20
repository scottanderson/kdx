//! Chat requests: 200 to 223.

use anyhow::Result;
use kdx_proto::msg::chat::{
    ChatId, ChatInvite, ChatListEntry, ChatRecord, ChatText, ChatWindowInfo, JoinChat, SetChatTopic,
};
use kdx_proto::session::MessageType;

use crate::Client;

/// The chat every server has.
pub const PUBLIC_CHAT: u32 = 1;

impl Client {
    /// 214 Get Chat List.
    pub async fn chat_list(&mut self) -> Result<Vec<ChatListEntry>> {
        let reply = self.request(MessageType::GET_CHAT_LIST, &[]).await?;
        Ok(ChatListEntry::decode_list(&reply)?)
    }

    /// 202 Join Chat; `key` comes from the chat list entry, 0 for public chats.
    pub async fn join_chat(&mut self, chat_id: u32, key: u32) -> Result<ChatWindowInfo> {
        let request = JoinChat { chat_id, key }.encode();
        let reply = self.request(MessageType::JOIN_CHAT, &request).await?;
        Ok(ChatWindowInfo::decode(&reply)?)
    }

    /// 204 Leave Chat.
    pub async fn leave_chat(&mut self, chat_id: u32) -> Result<()> {
        self.send(MessageType::LEAVE_CHAT, &ChatId(chat_id).encode())
            .await
    }

    /// 207 Send Chat Text; the server echoes it to every member as 208.
    ///
    /// `action` sets [`ChatText::FLAG_ACTION`], a `/me` line; the text goes
    /// out without the `/me ` prefix.
    pub async fn say(&mut self, chat_id: u32, text: &[u8], action: bool) -> Result<()> {
        let message = ChatText {
            chat_id,
            user_id: 0,
            flags: if action { ChatText::FLAG_ACTION } else { 0 },
            text: text.to_vec(),
            ..ChatText::default()
        };
        self.send(MessageType::SEND_CHAT_TEXT, &message.encode()?)
            .await
    }

    /// 207 Send Chat Text, as plain text; see [`Self::say`].
    pub async fn send_chat(&mut self, chat_id: u32, text: &[u8]) -> Result<()> {
        self.say(chat_id, text, false).await
    }

    /// 209 Set Chat Topic; the server sends 210 to every member.
    pub async fn set_topic(&mut self, chat_id: u32, topic: &[u8]) -> Result<()> {
        let request = SetChatTopic {
            chat_id,
            topic: topic.to_vec(),
            ..SetChatTopic::default()
        };
        self.send(MessageType::SET_CHAT_TOPIC, &request.encode()?)
            .await
    }

    /// 217 Create Chat; the reply is the 219 Chat Info of the new room.
    pub async fn create_chat(&mut self, chat: &ChatRecord) -> Result<ChatRecord> {
        let reply = self
            .request(MessageType::CREATE_CHAT, &chat.encode()?)
            .await?;
        Ok(ChatRecord::decode(&reply)?)
    }

    /// 211 Chat Invite, to at most [`ChatInvite::MAX_INVITEES`] users.
    pub async fn invite(&mut self, chat_id: u32, user_ids: &[u32], message: &[u8]) -> Result<()> {
        let request = ChatInvite {
            chat_id,
            user_ids: user_ids.to_vec(),
            message: message.to_vec(),
            ..ChatInvite::default()
        };
        self.send(MessageType::CHAT_INVITE, &request.encode()?)
            .await
    }

    /// 201 Delete Chat; the server answers with a success Status.
    pub async fn delete_chat(&mut self, chat_id: u32) -> Result<()> {
        self.request(MessageType::DELETE_CHAT, &ChatId(chat_id).encode())
            .await?;
        Ok(())
    }
}
