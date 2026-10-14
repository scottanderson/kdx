//! News: 600 Create Newsgroup, 603 Newsgroup Info, 604 Set Newsgroup, 606 Newsgroup List,
//! 608 News Messages, 610 News Message, 611 Post News Message, and 613 and 614 News Message
//! Posted and Deleted.

use super::{
    len_u8, need, pad4, padding, strings, u16_at, u32_at, u64_at, DecodeError, EncodeError,
};

/// An entry of 606 Newsgroup List.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NewsgroupEntry {
    /// Time in milliseconds.
    pub time: u64,
    /// Newsgroup ID.
    pub group_id: u32,
    /// Bytes 0x0c to 0x0f; meaning unknown.
    pub unknown_0c: [u8; 4],
    /// Probably the message count.
    pub message_count: u16,
    /// Bytes 0x12 to 0x15; meaning unknown.
    pub unknown_12: [u8; 4],
    /// Newsgroup name.
    pub name: Vec<u8>,
    /// Probably the description; empty in the lab.
    pub description: Vec<u8>,
    /// Bytes after the description up to the next multiple of 4, as received; empty in a new entry.
    pub padding: Vec<u8>,
}

impl NewsgroupEntry {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x18;

    /// Decodes the payload of 606 Newsgroup List.
    pub fn decode_list(buf: &[u8]) -> Result<Vec<Self>, DecodeError> {
        let count = list_count(buf)?;
        let mut at = 4;
        let mut list = Vec::with_capacity(count.min(buf.len()));
        for _ in 0..count {
            let entry = &buf[at..];
            need(entry, Self::HEADER_LEN)?;
            let lens = [usize::from(entry[0x16]), usize::from(entry[0x17])];
            let ([name, description], end) = strings(entry, Self::HEADER_LEN, lens)?;
            let (padding, padded) = padding(entry, end);
            list.push(Self {
                time: u64_at(entry, 0),
                group_id: u32_at(entry, 8),
                unknown_0c: entry[0x0c..0x10].try_into().expect("4 bytes"),
                message_count: u16_at(entry, 0x10),
                unknown_12: entry[0x12..0x16].try_into().expect("4 bytes"),
                name,
                description,
                padding,
            });
            at += padded;
        }
        Ok(list)
    }

    /// Encodes the payload of 606 Newsgroup List.
    pub fn encode_list(list: &[Self]) -> Result<Vec<u8>, EncodeError> {
        let mut out = encode_count(list.len())?;
        for entry in list {
            let mut fixed = [0; Self::HEADER_LEN];
            fixed[0..8].copy_from_slice(&entry.time.to_be_bytes());
            fixed[8..0x0c].copy_from_slice(&entry.group_id.to_be_bytes());
            fixed[0x0c..0x10].copy_from_slice(&entry.unknown_0c);
            fixed[0x10..0x12].copy_from_slice(&entry.message_count.to_be_bytes());
            fixed[0x12..0x16].copy_from_slice(&entry.unknown_12);
            fixed[0x16] = len_u8(&entry.name, "newsgroup name")?;
            fixed[0x17] = len_u8(&entry.description, "newsgroup description")?;
            out.extend_from_slice(&fixed);
            out.extend_from_slice(&entry.name);
            out.extend_from_slice(&entry.description);
            out.extend_from_slice(&entry.padding);
            out.resize(pad4(out.len()), 0);
        }
        Ok(out)
    }
}

/// 603 Newsgroup Info, and the request of 600 Create Newsgroup and 604 Set Newsgroup.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NewsgroupInfo {
    /// Newsgroup ID.
    pub group_id: u32,
    /// Bytes 0x04 to 0x0f; meaning unknown.
    pub unknown_04: [u8; 0x0c],
    /// Time of the last post in milliseconds.
    pub time: u64,
    /// Classes that may read, class 0 in the top bit.
    pub read_classes: u32,
    /// Classes that may post, class 0 in the top bit.
    pub post_classes: u32,
    /// Meaning unknown; `01 00 00 00` in the lab.
    pub unknown_20: [u8; 4],
    /// Bytes 0x24 to 0x2f; meaning unknown.
    pub unknown_24: [u8; 0x0c],
    /// Message count.
    pub message_count: u16,
    /// Probably the size limit in KiB; 50 in the lab.
    pub size_limit_kib: u16,
    /// Bytes 0x34 to 0x3b; meaning unknown.
    pub unknown_34: [u8; 8],
    /// Bytes 0x3e and 0x3f; meaning unknown.
    pub unknown_3e: [u8; 2],
    /// Newsgroup name.
    pub name: Vec<u8>,
    /// Description.
    pub description: Vec<u8>,
}

impl NewsgroupInfo {
    /// Size of the fixed part before the name.
    pub const HEADER_LEN: usize = 0x40;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [usize::from(buf[0x3c]), usize::from(buf[0x3d])];
        let ([name, description], _) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            group_id: u32_at(buf, 0),
            unknown_04: buf[4..0x10].try_into().expect("12 bytes"),
            time: u64_at(buf, 0x10),
            read_classes: u32_at(buf, 0x18),
            post_classes: u32_at(buf, 0x1c),
            unknown_20: buf[0x20..0x24].try_into().expect("4 bytes"),
            unknown_24: buf[0x24..0x30].try_into().expect("12 bytes"),
            message_count: u16_at(buf, 0x30),
            size_limit_kib: u16_at(buf, 0x32),
            unknown_34: buf[0x34..0x3c].try_into().expect("8 bytes"),
            unknown_3e: buf[0x3e..0x40].try_into().expect("2 bytes"),
            name,
            description,
        })
    }

    /// Encodes the payload, without padding.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..4].copy_from_slice(&self.group_id.to_be_bytes());
        out[4..0x10].copy_from_slice(&self.unknown_04);
        out[0x10..0x18].copy_from_slice(&self.time.to_be_bytes());
        out[0x18..0x1c].copy_from_slice(&self.read_classes.to_be_bytes());
        out[0x1c..0x20].copy_from_slice(&self.post_classes.to_be_bytes());
        out[0x20..0x24].copy_from_slice(&self.unknown_20);
        out[0x24..0x30].copy_from_slice(&self.unknown_24);
        out[0x30..0x32].copy_from_slice(&self.message_count.to_be_bytes());
        out[0x32..0x34].copy_from_slice(&self.size_limit_kib.to_be_bytes());
        out[0x34..0x3c].copy_from_slice(&self.unknown_34);
        out[0x3c] = len_u8(&self.name, "newsgroup name")?;
        out[0x3d] = len_u8(&self.description, "newsgroup description")?;
        out[0x3e..0x40].copy_from_slice(&self.unknown_3e);
        out.extend_from_slice(&self.name);
        out.extend_from_slice(&self.description);
        Ok(out)
    }
}

/// A news message, as in 608 News Messages and 610 News Message.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NewsMessage {
    /// Post time in milliseconds.
    pub time: u64,
    /// Newsgroup ID.
    pub group_id: u32,
    /// Message ID within the group.
    pub message_id: u32,
    /// Bytes 0x10 to 0x17; 0 in the lab.
    pub unknown_10: [u8; 8],
    /// Bytes 0x1a and 0x1b; 0 in the lab.
    pub unknown_1a: [u8; 2],
    /// Byte 0x1f; 0 in the lab.
    pub unknown_1f: u8,
    /// Poster's name.
    pub poster: Vec<u8>,
    /// Poster's login, which 612 Delete News Message checks.
    pub login: Vec<u8>,
    /// Subject.
    pub subject: Vec<u8>,
    /// Body text.
    pub body: Vec<u8>,
    /// Bytes after the body up to the next multiple of 4, as received; empty in a new message.
    pub padding: Vec<u8>,
}

impl NewsMessage {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x20;

    /// Decodes one message with its padding to 4 and returns it with its length.
    pub fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [
            usize::from(buf[0x1c]),
            usize::from(buf[0x1d]),
            usize::from(buf[0x1e]),
            usize::from(u16_at(buf, 0x18)),
        ];
        let ([poster, login, subject, body], end) = strings(buf, Self::HEADER_LEN, lens)?;
        let (padding, padded) = padding(buf, end);
        let message = Self {
            time: u64_at(buf, 0),
            group_id: u32_at(buf, 8),
            message_id: u32_at(buf, 0x0c),
            unknown_10: buf[0x10..0x18].try_into().expect("8 bytes"),
            unknown_1a: buf[0x1a..0x1c].try_into().expect("2 bytes"),
            unknown_1f: buf[0x1f],
            poster,
            login,
            subject,
            body,
            padding,
        };
        Ok((message, padded))
    }

    /// Encodes the message, padded to 4 with zeros past the stored padding.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let body_len = u16::try_from(self.body.len()).map_err(|_| EncodeError::TooLong("body"))?;
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..8].copy_from_slice(&self.time.to_be_bytes());
        out[8..0x0c].copy_from_slice(&self.group_id.to_be_bytes());
        out[0x0c..0x10].copy_from_slice(&self.message_id.to_be_bytes());
        out[0x10..0x18].copy_from_slice(&self.unknown_10);
        out[0x18..0x1a].copy_from_slice(&body_len.to_be_bytes());
        out[0x1a..0x1c].copy_from_slice(&self.unknown_1a);
        out[0x1c] = len_u8(&self.poster, "poster")?;
        out[0x1d] = len_u8(&self.login, "login")?;
        out[0x1e] = len_u8(&self.subject, "subject")?;
        out[0x1f] = self.unknown_1f;
        for part in [
            &self.poster,
            &self.login,
            &self.subject,
            &self.body,
            &self.padding,
        ] {
            out.extend_from_slice(part);
        }
        out.resize(pad4(out.len()), 0);
        Ok(out)
    }
}

/// In [`NewsMessages::permissions`], the user may post.
pub const NEWS_POST: u32 = 1;
/// In [`NewsMessages::permissions`], the user may delete any message.
pub const NEWS_DELETE: u32 = 2;
/// In [`NewsMessages::permissions`], the user may read; without it the list is empty.
pub const NEWS_READ: u32 = 4;

/// The payload of 608 News Messages.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NewsMessages {
    /// Newsgroup ID.
    pub group_id: u32,
    /// What the user may do in the group: [`NEWS_POST`], [`NEWS_DELETE`], [`NEWS_READ`].
    pub permissions: u32,
    /// Bytes 0x08 to 0x0b; meaning unknown.
    pub unknown_08: [u8; 4],
    /// The messages.
    pub messages: Vec<NewsMessage>,
}

impl NewsMessages {
    /// Size of the fixed part before the messages.
    pub const HEADER_LEN: usize = 0x10;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let count = u32_at(buf, 0x0c) as usize;
        let mut at = Self::HEADER_LEN;
        let mut messages = Vec::with_capacity(count.min(buf.len()));
        for _ in 0..count {
            let (message, len) = NewsMessage::decode(&buf[at..])?;
            messages.push(message);
            at = (at + pad4(len)).min(buf.len());
        }
        Ok(Self {
            group_id: u32_at(buf, 0),
            permissions: u32_at(buf, 4),
            unknown_08: buf[8..0x0c].try_into().expect("4 bytes"),
            messages,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = self.group_id.to_be_bytes().to_vec();
        out.extend_from_slice(&self.permissions.to_be_bytes());
        out.extend_from_slice(&self.unknown_08);
        out.extend(encode_count(self.messages.len())?);
        for message in &self.messages {
            out.extend(message.encode()?);
        }
        Ok(out)
    }
}

fn list_count(buf: &[u8]) -> Result<usize, DecodeError> {
    need(buf, 4)?;
    Ok(u32_at(buf, 0) as usize)
}

fn encode_count(count: usize) -> Result<Vec<u8>, EncodeError> {
    let count = u32::try_from(count).map_err(|_| EncodeError::TooLong("list"))?;
    Ok(count.to_be_bytes().to_vec())
}

/// The request of 611 Post News Message.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PostNewsMessage {
    /// Newsgroup ID.
    pub group_id: u32,
    /// Bytes 0x04 to 0x0b, which the server ignores.
    pub unknown_04: [u8; 8],
    /// Byte 0x0f, which the server ignores.
    pub unknown_0f: u8,
    /// Subject; the server keeps at most 0x80 bytes.
    pub subject: Vec<u8>,
    /// Body text; the server keeps at most 0x4000 bytes.
    pub body: Vec<u8>,
    /// Bytes after the body, as received; empty in a new message, as Client 1.620 sends it.
    pub rest: Vec<u8>,
}

impl PostNewsMessage {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x10;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [usize::from(buf[0x0e]), usize::from(u16_at(buf, 0x0c))];
        let ([subject, body], end) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            group_id: u32_at(buf, 0),
            unknown_04: buf[4..0x0c].try_into().expect("8 bytes"),
            unknown_0f: buf[0x0f],
            subject,
            body,
            rest: buf[end..].to_vec(),
        })
    }

    /// Encodes the payload, unpadded unless [`Self::rest`] holds padding.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let body_len = u16::try_from(self.body.len()).map_err(|_| EncodeError::TooLong("body"))?;
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..4].copy_from_slice(&self.group_id.to_be_bytes());
        out[4..0x0c].copy_from_slice(&self.unknown_04);
        out[0x0c..0x0e].copy_from_slice(&body_len.to_be_bytes());
        out[0x0e] = len_u8(&self.subject, "subject")?;
        out[0x0f] = self.unknown_0f;
        out.extend_from_slice(&self.subject);
        out.extend_from_slice(&self.body);
        out.extend_from_slice(&self.rest);
        Ok(out)
    }
}

/// 613 News Message Posted and 614 News Message Deleted.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NewsMessageChange {
    /// Newsgroup ID.
    pub group_id: u32,
    /// The message posted or deleted.
    pub message_id: u32,
    /// Messages in the group afterwards.
    pub message_count: u32,
}

impl NewsMessageChange {
    /// Size of the payload.
    pub const LEN: usize = 0x0c;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::LEN)?;
        Ok(Self {
            group_id: u32_at(buf, 0),
            message_id: u32_at(buf, 4),
            message_count: u32_at(buf, 8),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> [u8; Self::LEN] {
        let mut out = [0; Self::LEN];
        out[0..4].copy_from_slice(&self.group_id.to_be_bytes());
        out[4..8].copy_from_slice(&self.message_id.to_be_bytes());
        out[8..0x0c].copy_from_slice(&self.message_count.to_be_bytes());
        out
    }
}
