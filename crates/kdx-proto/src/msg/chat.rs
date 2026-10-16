//! Chat: 201 Delete Chat through 222 Get Chat DCC Addresses.

use super::{
    len_u8, need, pad4, padding, strings, u16_at, u32_at, u64_at, DecodeError, EncodeError,
};

/// The ID of Public Chat, which always exists.
pub const PUBLIC_CHAT: u32 = 1;

/// A bare chat ID: 201 Delete Chat, 204 Leave Chat, 218 Get Chat, and 222 Get
/// Chat DCC Addresses.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChatId(pub u32);

impl ChatId {
    /// Payload length.
    pub const LEN: usize = 4;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::LEN)?;
        Ok(Self(u32_at(buf, 0)))
    }

    /// Encodes the payload.
    pub const fn encode(&self) -> [u8; Self::LEN] {
        self.0.to_be_bytes()
    }
}

/// 202 Join Chat.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct JoinChat {
    /// Chat ID.
    pub chat_id: u32,
    /// Key, needed for private chats.
    pub key: u32,
}

impl JoinChat {
    /// Payload length.
    pub const LEN: usize = 8;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::LEN)?;
        Ok(Self {
            chat_id: u32_at(buf, 0),
            key: u32_at(buf, 4),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> [u8; Self::LEN] {
        let mut out = [0; Self::LEN];
        out[..4].copy_from_slice(&self.chat_id.to_be_bytes());
        out[4..].copy_from_slice(&self.key.to_be_bytes());
        out
    }
}

/// 203 Chat Window Info, the reply to 202 Join Chat, and 216 Create Private
/// Chat (R), the reply to 200 Create Private Chat.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChatWindowInfo {
    /// Time in milliseconds.
    pub time: u64,
    /// Chat ID.
    pub chat_id: u32,
    /// Key.
    pub key: u32,
    /// Bytes 0x10 to 0x13; meaning unknown.
    pub ignored_10: [u8; 4],
    /// Flags: bit 0 private, bit 1 voice.
    pub flags: u16,
    /// Bytes 0x18 to 0x1d; meaning unknown.
    pub ignored_18: [u8; 6],
    /// Byte 0x23; meaning unknown.
    pub ignored_23: u8,
    /// User IDs of the members, the joiner included.
    pub members: Vec<u32>,
    /// The chat's name.
    pub name: Vec<u8>,
    /// The topic.
    pub topic: Vec<u8>,
    /// The nick that set the topic.
    pub topic_setter: Vec<u8>,
    /// The IRC server.
    pub irc_server: Vec<u8>,
    /// The IRC channel.
    pub irc_channel: Vec<u8>,
}

impl ChatWindowInfo {
    /// Size of the fixed part before the member IDs.
    pub const HEADER_LEN: usize = 0x24;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let count = usize::from(u16_at(buf, 0x16));
        let ids_end = Self::HEADER_LEN + 4 * count;
        need(buf, ids_end)?;
        let members = (0..count)
            .map(|i| u32_at(buf, Self::HEADER_LEN + 4 * i))
            .collect();
        let [irc_server_len, irc_channel_len, name_len, topic_len, topic_setter_len] =
            [0x1e, 0x1f, 0x20, 0x21, 0x22].map(|at| usize::from(buf[at]));
        let ([name, topic, topic_setter, irc_server, irc_channel], _) = strings(
            buf,
            ids_end,
            [
                name_len,
                topic_len,
                topic_setter_len,
                irc_server_len,
                irc_channel_len,
            ],
        )?;
        Ok(Self {
            time: u64_at(buf, 0),
            chat_id: u32_at(buf, 8),
            key: u32_at(buf, 0x0c),
            ignored_10: buf[0x10..0x14].try_into().expect("4 bytes"),
            flags: u16_at(buf, 0x14),
            ignored_18: buf[0x18..0x1e].try_into().expect("6 bytes"),
            ignored_23: buf[0x23],
            members,
            name,
            topic,
            topic_setter,
            irc_server,
            irc_channel,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let count =
            u16::try_from(self.members.len()).map_err(|_| EncodeError::TooLong("members"))?;
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..8].copy_from_slice(&self.time.to_be_bytes());
        out[8..0x0c].copy_from_slice(&self.chat_id.to_be_bytes());
        out[0x0c..0x10].copy_from_slice(&self.key.to_be_bytes());
        out[0x10..0x14].copy_from_slice(&self.ignored_10);
        out[0x14..0x16].copy_from_slice(&self.flags.to_be_bytes());
        out[0x16..0x18].copy_from_slice(&count.to_be_bytes());
        out[0x18..0x1e].copy_from_slice(&self.ignored_18);
        out[0x1e] = len_u8(&self.irc_server, "IRC server")?;
        out[0x1f] = len_u8(&self.irc_channel, "IRC channel")?;
        out[0x20] = len_u8(&self.name, "chat name")?;
        out[0x21] = len_u8(&self.topic, "chat topic")?;
        out[0x22] = len_u8(&self.topic_setter, "topic setter")?;
        out[0x23] = self.ignored_23;
        for id in &self.members {
            out.extend_from_slice(&id.to_be_bytes());
        }
        for s in [
            &self.name,
            &self.topic,
            &self.topic_setter,
            &self.irc_server,
            &self.irc_channel,
        ] {
            out.extend_from_slice(s);
        }
        Ok(out)
    }
}

/// 205 Chat User Joined and 206 Chat User Departed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChatUserChange {
    /// Chat ID.
    pub chat_id: u32,
    /// The user who joined or left.
    pub user_id: u32,
    /// Meaning unknown; 0 in 205 and 1 in 206 after 204 Leave Chat.
    pub ignored_08: u8,
    /// Bytes 0x09 to 0x0b; meaning unknown.
    pub ignored_09: [u8; 3],
}

impl ChatUserChange {
    /// Payload length.
    pub const LEN: usize = 0x0c;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::LEN)?;
        Ok(Self {
            chat_id: u32_at(buf, 0),
            user_id: u32_at(buf, 4),
            ignored_08: buf[8],
            ignored_09: buf[9..0x0c].try_into().expect("3 bytes"),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> [u8; Self::LEN] {
        let mut out = [0; Self::LEN];
        out[..4].copy_from_slice(&self.chat_id.to_be_bytes());
        out[4..8].copy_from_slice(&self.user_id.to_be_bytes());
        out[8] = self.ignored_08;
        out[9..].copy_from_slice(&self.ignored_09);
        out
    }
}

/// 207 Send Chat Text and 208 Receive Chat Text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChatText {
    /// Chat ID.
    pub chat_id: u32,
    /// The sender in 208 Receive Chat Text; the server ignores it in 207.
    pub user_id: u32,
    /// Bytes 0x08 to 0x0b; meaning unknown.
    pub ignored: [u8; 4],
    /// Message text.
    pub text: Vec<u8>,
}

impl ChatText {
    /// Size of the fixed part before the text.
    pub const HEADER_LEN: usize = 0x0c;
    /// Longest text the server accepts.
    pub const MAX_TEXT_LEN: usize = 0x1000;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        Ok(Self {
            chat_id: u32_at(buf, 0),
            user_id: u32_at(buf, 4),
            ignored: buf[8..0x0c].try_into().expect("4 bytes"),
            text: buf[Self::HEADER_LEN..].to_vec(),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        if self.text.len() > Self::MAX_TEXT_LEN {
            return Err(EncodeError::TooLong("chat text"));
        }
        let mut out = vec![0; Self::HEADER_LEN];
        out[..4].copy_from_slice(&self.chat_id.to_be_bytes());
        out[4..8].copy_from_slice(&self.user_id.to_be_bytes());
        out[8..0x0c].copy_from_slice(&self.ignored);
        out.extend_from_slice(&self.text);
        Ok(out)
    }
}

/// 209 Set Chat Topic.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SetChatTopic {
    /// Chat ID.
    pub chat_id: u32,
    /// Bytes 0x04 to 0x09; meaning unknown, 0 from the client.
    pub ignored_04: [u8; 6],
    /// Byte 0x0b; meaning unknown, 0 from the client.
    pub ignored_0b: u8,
    /// New topic.
    pub topic: Vec<u8>,
    /// Bytes after the topic, which the server ignores; empty from the client.
    pub rest: Vec<u8>,
}

impl SetChatTopic {
    /// Size of the fixed part before the topic.
    pub const HEADER_LEN: usize = 0x0c;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let ([topic], end) = strings(buf, Self::HEADER_LEN, [usize::from(buf[0x0a])])?;
        Ok(Self {
            chat_id: u32_at(buf, 0),
            ignored_04: buf[4..0x0a].try_into().expect("6 bytes"),
            ignored_0b: buf[0x0b],
            topic,
            rest: buf[end..].to_vec(),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[..4].copy_from_slice(&self.chat_id.to_be_bytes());
        out[4..0x0a].copy_from_slice(&self.ignored_04);
        out[0x0a] = len_u8(&self.topic, "topic")?;
        out[0x0b] = self.ignored_0b;
        out.extend_from_slice(&self.topic);
        out.extend_from_slice(&self.rest);
        Ok(out)
    }
}

/// 210 Chat Topic Changed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChatTopicChanged {
    /// Time in milliseconds.
    pub time: u64,
    /// Chat ID.
    pub chat_id: u32,
    /// The user who set the topic.
    pub user_id: u32,
    /// Bytes 0x10 to 0x17; meaning unknown.
    pub ignored_10: [u8; 8],
    /// Bytes 0x1a and 0x1b; meaning unknown.
    pub ignored_1a: [u8; 2],
    /// New topic.
    pub topic: Vec<u8>,
    /// The setter's nick.
    pub nick: Vec<u8>,
}

impl ChatTopicChanged {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x1c;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [usize::from(buf[0x18]), usize::from(buf[0x19])];
        let ([topic, nick], _) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            time: u64_at(buf, 0),
            chat_id: u32_at(buf, 8),
            user_id: u32_at(buf, 0x0c),
            ignored_10: buf[0x10..0x18].try_into().expect("8 bytes"),
            ignored_1a: buf[0x1a..0x1c].try_into().expect("2 bytes"),
            topic,
            nick,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..8].copy_from_slice(&self.time.to_be_bytes());
        out[8..0x0c].copy_from_slice(&self.chat_id.to_be_bytes());
        out[0x0c..0x10].copy_from_slice(&self.user_id.to_be_bytes());
        out[0x10..0x18].copy_from_slice(&self.ignored_10);
        out[0x18] = len_u8(&self.topic, "topic")?;
        out[0x19] = len_u8(&self.nick, "nick")?;
        out[0x1a..0x1c].copy_from_slice(&self.ignored_1a);
        out.extend_from_slice(&self.topic);
        out.extend_from_slice(&self.nick);
        Ok(out)
    }
}

/// 211 Chat Invite.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChatInvite {
    /// Chat ID.
    pub chat_id: u32,
    /// Bytes 0x04 to 0x07; meaning unknown.
    pub ignored_04: [u8; 4],
    /// Bytes 0x0a to 0x0e; meaning unknown.
    pub ignored_0a: [u8; 5],
    /// User IDs of the invitees.
    pub user_ids: Vec<u32>,
    /// Message to the invitees.
    pub message: Vec<u8>,
}

impl ChatInvite {
    /// Size of the fixed part before the user IDs.
    pub const HEADER_LEN: usize = 0x10;
    /// Most invitees the server accepts.
    pub const MAX_INVITEES: usize = 20;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let count = usize::from(u16_at(buf, 8));
        let ids_end = Self::HEADER_LEN + 4 * count;
        need(buf, ids_end)?;
        let user_ids = (0..count)
            .map(|i| u32_at(buf, Self::HEADER_LEN + 4 * i))
            .collect();
        let ([message], _) = strings(buf, ids_end, [usize::from(buf[0x0f])])?;
        Ok(Self {
            chat_id: u32_at(buf, 0),
            ignored_04: buf[4..8].try_into().expect("4 bytes"),
            ignored_0a: buf[0x0a..0x0f].try_into().expect("5 bytes"),
            user_ids,
            message,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        if self.user_ids.len() > Self::MAX_INVITEES {
            return Err(EncodeError::TooLong("invitees"));
        }
        let mut out = vec![0; Self::HEADER_LEN];
        out[..4].copy_from_slice(&self.chat_id.to_be_bytes());
        out[4..8].copy_from_slice(&self.ignored_04);
        out[8..0x0a].copy_from_slice(&(self.user_ids.len() as u16).to_be_bytes());
        out[0x0a..0x0f].copy_from_slice(&self.ignored_0a);
        out[0x0f] = len_u8(&self.message, "invite message")?;
        for id in &self.user_ids {
            out.extend_from_slice(&id.to_be_bytes());
        }
        out.extend_from_slice(&self.message);
        Ok(out)
    }
}

/// 212 Chat Invitation, sent to each invitee of 211 Chat Invite.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChatInvitation {
    /// Chat ID.
    pub chat_id: u32,
    /// Key, for 202 Join Chat.
    pub key: u32,
    /// The inviter.
    pub user_id: u32,
    /// Meaning unknown; 1 then 2 in the lab.
    pub ignored_0c: u32,
    /// Meaning unknown; 1 in the lab.
    pub ignored_10: u16,
    /// Bytes 0x12 to 0x17; meaning unknown.
    pub ignored_12: [u8; 6],
    /// Bytes 0x1a and 0x1b; meaning unknown.
    pub ignored_1a: [u8; 2],
    /// Probably the invite message; empty in the lab.
    pub message: Vec<u8>,
    /// Chat name.
    pub name: Vec<u8>,
}

impl ChatInvitation {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x1c;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [usize::from(buf[0x18]), usize::from(buf[0x19])];
        let ([message, name], _) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            chat_id: u32_at(buf, 0),
            key: u32_at(buf, 4),
            user_id: u32_at(buf, 8),
            ignored_0c: u32_at(buf, 0x0c),
            ignored_10: u16_at(buf, 0x10),
            ignored_12: buf[0x12..0x18].try_into().expect("6 bytes"),
            ignored_1a: buf[0x1a..0x1c].try_into().expect("2 bytes"),
            message,
            name,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..4].copy_from_slice(&self.chat_id.to_be_bytes());
        out[4..8].copy_from_slice(&self.key.to_be_bytes());
        out[8..0x0c].copy_from_slice(&self.user_id.to_be_bytes());
        out[0x0c..0x10].copy_from_slice(&self.ignored_0c.to_be_bytes());
        out[0x10..0x12].copy_from_slice(&self.ignored_10.to_be_bytes());
        out[0x12..0x18].copy_from_slice(&self.ignored_12);
        out[0x18] = len_u8(&self.message, "invite message")?;
        out[0x19] = len_u8(&self.name, "chat name")?;
        out[0x1a..0x1c].copy_from_slice(&self.ignored_1a);
        out.extend_from_slice(&self.message);
        out.extend_from_slice(&self.name);
        Ok(out)
    }
}

/// An entry of 215 Chat List.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChatListEntry {
    /// Chat ID.
    pub chat_id: u32,
    /// Key.
    pub key: u32,
    /// Bytes 0x08 to 0x0d; meaning unknown.
    pub ignored: [u8; 6],
    /// Flags: bit 1 voice.
    pub flags: u16,
    /// Number of members.
    pub member_count: u16,
    /// Chat name.
    pub name: Vec<u8>,
    /// Description.
    pub description: Vec<u8>,
    /// Bytes after the description up to the next multiple of 4, as received; empty in a new entry.
    pub padding: Vec<u8>,
}

impl ChatListEntry {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x14;

    /// Decodes the payload of 215 Chat List.
    pub fn decode_list(buf: &[u8]) -> Result<Vec<Self>, DecodeError> {
        need(buf, 4)?;
        let count = u32_at(buf, 0) as usize;
        let mut at = 4;
        let mut list = Vec::with_capacity(count.min(buf.len()));
        for _ in 0..count {
            let entry = &buf[at..];
            need(entry, Self::HEADER_LEN)?;
            let lens = [usize::from(entry[0x12]), usize::from(entry[0x13])];
            let ([name, description], end) = strings(entry, Self::HEADER_LEN, lens)?;
            let (padding, padded) = padding(entry, end);
            list.push(Self {
                chat_id: u32_at(entry, 0),
                key: u32_at(entry, 4),
                ignored: entry[8..0x0e].try_into().expect("6 bytes"),
                flags: u16_at(entry, 0x0e),
                member_count: u16_at(entry, 0x10),
                name,
                description,
                padding,
            });
            at += padded;
        }
        Ok(list)
    }

    /// Encodes the payload of 215 Chat List.
    pub fn encode_list(list: &[Self]) -> Result<Vec<u8>, EncodeError> {
        let count = u32::try_from(list.len()).map_err(|_| EncodeError::TooLong("chat list"))?;
        let mut out = count.to_be_bytes().to_vec();
        for entry in list {
            let mut fixed = [0; Self::HEADER_LEN];
            fixed[0..4].copy_from_slice(&entry.chat_id.to_be_bytes());
            fixed[4..8].copy_from_slice(&entry.key.to_be_bytes());
            fixed[8..0x0e].copy_from_slice(&entry.ignored);
            fixed[0x0e..0x10].copy_from_slice(&entry.flags.to_be_bytes());
            fixed[0x10..0x12].copy_from_slice(&entry.member_count.to_be_bytes());
            fixed[0x12] = len_u8(&entry.name, "chat name")?;
            fixed[0x13] = len_u8(&entry.description, "chat description")?;
            out.extend_from_slice(&fixed);
            out.extend_from_slice(&entry.name);
            out.extend_from_slice(&entry.description);
            out.extend_from_slice(&entry.padding);
            out.resize(pad4(out.len()), 0);
        }
        Ok(out)
    }
}

/// One member's entry in 223 Chat DCC Addresses.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DccAddress {
    /// User ID.
    pub user_id: u32,
    /// Bytes 0x04 to 0x0d; the server writes zeros.
    pub ignored_04: [u8; 10],
    /// Byte 0x0f; the server writes zero.
    pub ignored_0f: u8,
    /// `ip:port` text of the member's DCC listener.
    pub address: Vec<u8>,
    /// Bytes after the address up to the next multiple of 4, as received.
    pub padding: Vec<u8>,
}

impl DccAddress {
    /// Size of the fixed part before the address.
    pub const HEADER_LEN: usize = 0x10;

    /// Decodes the payload of 223 Chat DCC Addresses: entries until the end.
    pub fn decode_list(buf: &[u8]) -> Result<Vec<Self>, DecodeError> {
        let mut at = 0;
        let mut list = Vec::new();
        while at < buf.len() {
            let entry = &buf[at..];
            need(entry, Self::HEADER_LEN)?;
            let ([address], end) = strings(entry, Self::HEADER_LEN, [usize::from(entry[0x0e])])?;
            let (padding, padded) = padding(entry, end);
            list.push(Self {
                user_id: u32_at(entry, 0),
                ignored_04: entry[4..0x0e].try_into().expect("10 bytes"),
                ignored_0f: entry[0x0f],
                address,
                padding,
            });
            at += padded;
        }
        Ok(list)
    }

    /// Encodes the payload of 223 Chat DCC Addresses.
    pub fn encode_list(list: &[Self]) -> Result<Vec<u8>, EncodeError> {
        let mut out = Vec::new();
        for entry in list {
            let start = out.len();
            out.extend_from_slice(&entry.user_id.to_be_bytes());
            out.extend_from_slice(&entry.ignored_04);
            out.push(len_u8(&entry.address, "DCC address")?);
            out.push(entry.ignored_0f);
            out.extend_from_slice(&entry.address);
            out.extend_from_slice(&entry.padding);
            out.resize(start + pad4(out.len() - start), 0);
        }
        Ok(out)
    }
}

/// The chat record of 217 Create Chat, 219 Chat Info and 220 Set Chat.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChatRecord {
    /// Chat ID.
    pub chat_id: u32,
    /// Key.
    pub key: u32,
    /// Bytes 0x08 to 0x13; meaning unknown.
    pub ignored_08: [u8; 0x0c],
    /// Account classes that may join, most significant bit = class 0.
    pub join_classes: u32,
    /// Account classes that may send text.
    pub send_classes: u32,
    /// Account classes that may set the topic.
    pub topic_classes: u32,
    /// Bytes 0x20 to 0x27; meaning unknown.
    pub ignored_20: [u8; 8],
    /// Flags: bit 0 private, bit 1 voice.
    pub flags: u16,
    /// Bytes 0x2e and 0x2f; meaning unknown.
    pub ignored_2e: [u8; 2],
    /// Chat name.
    pub name: Vec<u8>,
    /// Description.
    pub description: Vec<u8>,
    /// IRC server, `host:port`; the client adds `:6667` when the port is missing.
    pub irc_server: Vec<u8>,
    /// IRC channel; the client adds `#` if the name has none.
    pub irc_channel: Vec<u8>,
}

impl ChatRecord {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x30;

    /// Decodes the record.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [0x2a, 0x2b, 0x2c, 0x2d].map(|at| usize::from(buf[at]));
        let ([name, description, irc_server, irc_channel], _) =
            strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            chat_id: u32_at(buf, 0),
            key: u32_at(buf, 4),
            ignored_08: buf[8..0x14].try_into().expect("12 bytes"),
            join_classes: u32_at(buf, 0x14),
            send_classes: u32_at(buf, 0x18),
            topic_classes: u32_at(buf, 0x1c),
            ignored_20: buf[0x20..0x28].try_into().expect("8 bytes"),
            flags: u16_at(buf, 0x28),
            ignored_2e: buf[0x2e..0x30].try_into().expect("2 bytes"),
            name,
            description,
            irc_server,
            irc_channel,
        })
    }

    /// Encodes the record.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..4].copy_from_slice(&self.chat_id.to_be_bytes());
        out[4..8].copy_from_slice(&self.key.to_be_bytes());
        out[8..0x14].copy_from_slice(&self.ignored_08);
        out[0x14..0x18].copy_from_slice(&self.join_classes.to_be_bytes());
        out[0x18..0x1c].copy_from_slice(&self.send_classes.to_be_bytes());
        out[0x1c..0x20].copy_from_slice(&self.topic_classes.to_be_bytes());
        out[0x20..0x28].copy_from_slice(&self.ignored_20);
        out[0x28..0x2a].copy_from_slice(&self.flags.to_be_bytes());
        out[0x2a] = len_u8(&self.name, "chat name")?;
        out[0x2b] = len_u8(&self.description, "chat description")?;
        out[0x2c] = len_u8(&self.irc_server, "irc_server string")?;
        out[0x2d] = len_u8(&self.irc_channel, "irc_channel string")?;
        out[0x2e..0x30].copy_from_slice(&self.ignored_2e);
        for s in [
            &self.name,
            &self.description,
            &self.irc_server,
            &self.irc_channel,
        ] {
            out.extend_from_slice(s);
        }
        Ok(out)
    }
}

/// 221 Chat Deleted, sent to each member of a chat that 201 Delete Chat removes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChatDeleted {
    /// Chat ID.
    pub chat_id: u32,
    /// The user who deleted it.
    pub user_id: u32,
}

impl ChatDeleted {
    /// Payload length.
    pub const LEN: usize = 8;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::LEN)?;
        Ok(Self {
            chat_id: u32_at(buf, 0),
            user_id: u32_at(buf, 4),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> [u8; Self::LEN] {
        let mut out = [0; Self::LEN];
        out[..4].copy_from_slice(&self.chat_id.to_be_bytes());
        out[4..].copy_from_slice(&self.user_id.to_be_bytes());
        out
    }
}
