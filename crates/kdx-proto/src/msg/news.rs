//! News: 603 Newsgroup Info, 606 Newsgroup List, 608 News Messages and 610 News Message.

use super::{len_u8, need, pad4, strings, u16_at, u32_at, u64_at, DecodeError, EncodeError};

/// An entry of 606 Newsgroup List.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NewsgroupEntry {
    /// Time in milliseconds.
    pub time: u64,
    /// Newsgroup ID.
    pub group_id: u32,
    /// Probably the message count.
    pub message_count: u16,
    /// Newsgroup name.
    pub name: Vec<u8>,
    /// Probably the description; empty in the lab.
    pub description: Vec<u8>,
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
            list.push(Self {
                time: u64_at(entry, 0),
                group_id: u32_at(entry, 8),
                message_count: u16_at(entry, 0x10),
                name,
                description,
            });
            at += pad4(end).min(entry.len());
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
            fixed[0x10..0x12].copy_from_slice(&entry.message_count.to_be_bytes());
            fixed[0x16] = len_u8(&entry.name, "newsgroup name")?;
            fixed[0x17] = len_u8(&entry.description, "newsgroup description")?;
            out.extend_from_slice(&fixed);
            out.extend_from_slice(&entry.name);
            out.extend_from_slice(&entry.description);
            out.resize(pad4(out.len()), 0);
        }
        Ok(out)
    }
}

/// 603 Newsgroup Info.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NewsgroupInfo {
    /// Newsgroup ID.
    pub group_id: u32,
    /// Time in milliseconds.
    pub time: u64,
    /// All ones in the lab; probably class bitmaps.
    pub classes: u64,
    /// Meaning unknown; 1 in the lab.
    pub unknown_20: u8,
    /// Probably the message count.
    pub message_count: u16,
    /// Probably the message limit; 0x32 in the lab.
    pub message_limit: u16,
    /// Newsgroup name.
    pub name: Vec<u8>,
}

impl NewsgroupInfo {
    /// Size of the fixed part before the name.
    pub const HEADER_LEN: usize = 0x40;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let ([name], _) = strings(buf, Self::HEADER_LEN, [usize::from(buf[0x3c])])?;
        Ok(Self {
            group_id: u32_at(buf, 0),
            time: u64_at(buf, 0x10),
            classes: u64_at(buf, 0x18),
            unknown_20: buf[0x20],
            message_count: u16_at(buf, 0x30),
            message_limit: u16_at(buf, 0x32),
            name,
        })
    }

    /// Encodes the payload, padded with zeros to 4.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..4].copy_from_slice(&self.group_id.to_be_bytes());
        out[0x10..0x18].copy_from_slice(&self.time.to_be_bytes());
        out[0x18..0x20].copy_from_slice(&self.classes.to_be_bytes());
        out[0x20] = self.unknown_20;
        out[0x30..0x32].copy_from_slice(&self.message_count.to_be_bytes());
        out[0x32..0x34].copy_from_slice(&self.message_limit.to_be_bytes());
        out[0x3c] = len_u8(&self.name, "newsgroup name")?;
        out.extend_from_slice(&self.name);
        out.resize(pad4(out.len()), 0);
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
    /// Poster's name.
    pub poster: Vec<u8>,
    /// Probably the poster's login.
    pub login: Vec<u8>,
    /// Subject.
    pub subject: Vec<u8>,
    /// Body text.
    pub body: Vec<u8>,
}

impl NewsMessage {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x20;

    /// Decodes one message and returns it with its unpadded length.
    pub fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [
            usize::from(buf[0x1c]),
            usize::from(buf[0x1d]),
            usize::from(buf[0x1e]),
            usize::from(u16_at(buf, 0x18)),
        ];
        let ([poster, login, subject, body], end) = strings(buf, Self::HEADER_LEN, lens)?;
        let message = Self {
            time: u64_at(buf, 0),
            group_id: u32_at(buf, 8),
            message_id: u32_at(buf, 0x0c),
            poster,
            login,
            subject,
            body,
        };
        Ok((message, end))
    }

    /// Encodes the message, padded with zeros to 4 bytes.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let body_len = u16::try_from(self.body.len()).map_err(|_| EncodeError::TooLong("body"))?;
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..8].copy_from_slice(&self.time.to_be_bytes());
        out[8..0x0c].copy_from_slice(&self.group_id.to_be_bytes());
        out[0x0c..0x10].copy_from_slice(&self.message_id.to_be_bytes());
        out[0x18..0x1a].copy_from_slice(&body_len.to_be_bytes());
        out[0x1c] = len_u8(&self.poster, "poster")?;
        out[0x1d] = len_u8(&self.login, "login")?;
        out[0x1e] = len_u8(&self.subject, "subject")?;
        for part in [&self.poster, &self.login, &self.subject, &self.body] {
            out.extend_from_slice(part);
        }
        out.resize(pad4(out.len()), 0);
        Ok(out)
    }
}

/// The payload of 608 News Messages.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NewsMessages {
    /// Newsgroup ID.
    pub group_id: u32,
    /// Unknown; 7 in the lab.
    pub unknown_04: u32,
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
            unknown_04: u32_at(buf, 4),
            messages,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = self.group_id.to_be_bytes().to_vec();
        out.extend_from_slice(&self.unknown_04.to_be_bytes());
        out.extend_from_slice(&[0; 4]);
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
