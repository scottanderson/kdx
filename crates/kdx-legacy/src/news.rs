//! `News.dat`: newsgroups and their messages. See `docs/formats/news-dat.md`.
//!
//! `%KDXNUUZ`, version 2, the group count, a small table, then the group
//! entries (a 0x18-byte prefix with the group's message-list offset, length
//! and next message id, followed by the wire `NewsgroupInfo`, padded to 4),
//! then one list section per group (`47 fd db 01`, count, zero, cumulative
//! end offsets, wire `NewsMessage` records) at 8-aligned offsets. A 0-byte
//! file (the first run) is an empty store.

use crate::wire::{pad4_len, Reader, Writer};
use kdx_proto::msg::news::{NewsMessage, NewsgroupInfo};

use crate::error::{LegacyError, Result};
use crate::server_stg::check_header;
use crate::store::{LegacyDir, ServerFile};

/// The file's first eight bytes.
pub const MAGIC: [u8; 8] = *b"%KDXNUUZ";
const LIST_MAGIC: u32 = 0x47fd_db01;
const HEADER_LEN: usize = 0x40;
const GROUPS_OFFSET: u32 = 0x40;

#[derive(Debug, Clone, PartialEq, Eq)]
/// The decoded `News.dat`: groups, each with its message list.
pub struct NewsStore {
    /// File version; 1.
    pub version: u16,
    /// Bytes 0x0c to 0x0f; zero.
    pub ignored_0c: [u8; 4],
    /// Entries 0 and 2–4 of the header table, never seen non-zero; entry 1
    /// is the groups section and is derived.
    pub table_other: [(u32, u32); 5],
    /// `00 00 00 00 01 00 00 00` in every file seen.
    pub ignored_38: [u8; 8],
    /// The newsgroups, in file order.
    pub groups: Vec<NewsGroupRecord>,
    /// Bytes between the groups section and the first message list.
    pub groups_gap: Vec<u8>,
    /// Bytes after the last list up to the 8-aligned end of the file.
    pub trailer: Vec<u8>,
    /// False for the 0-byte file of a fresh server; true once a header has
    /// been written, even with no groups left.
    pub has_header: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// One newsgroup: its entry in the groups section and its message list.
pub struct NewsGroupRecord {
    /// The last message id handed out; a new group starts at 1, so the
    /// first post is 2. Never reused after a delete.
    pub last_message_id: u32,
    /// Bytes 0x0c to 0x17 of the entry; zero.
    pub ignored_0c: [u8; 12],
    /// The group as 603 Newsgroup Info carries it.
    pub info: NewsgroupInfo,
    /// The entry's `pad4` when it was not zero, else empty.
    pub padding: Vec<u8>,
    /// The message list's third header word; zero so far.
    pub list_ignored: u32,
    /// The messages, in file order.
    pub messages: Vec<NewsMessage>,
    /// Bytes after this group's list up to the next 8-aligned offset.
    pub gap: Vec<u8>,
}

impl Default for NewsStore {
    fn default() -> Self {
        NewsStore {
            version: 2,
            ignored_0c: [0; 4],
            table_other: [(0, 0); 5],
            ignored_38: [0, 0, 0, 0, 1, 0, 0, 0],
            groups: Vec::new(),
            groups_gap: Vec::new(),
            trailer: Vec::new(),
            has_header: false,
        }
    }
}

impl NewsStore {
    /// The group with this id.
    pub fn group(&self, group_id: u32) -> Option<&NewsGroupRecord> {
        self.groups.iter().find(|g| g.info.group_id == group_id)
    }

    /// The group with this id, for editing.
    pub fn group_mut(&mut self, group_id: u32) -> Option<&mut NewsGroupRecord> {
        self.groups.iter_mut().find(|g| g.info.group_id == group_id)
    }

    /// The id the next post in this group gets.
    pub fn next_message_id(&self, group_id: u32) -> Option<u32> {
        self.group(group_id).map(|g| g.last_message_id + 1)
    }

    /// Every string must fit its length field (one byte; the body two).
    pub fn validate(&self) -> Result<()> {
        for g in &self.groups {
            if g.info.name.len() > 0xff || g.info.description.len() > 0xff {
                return Err(LegacyError::Invalid(
                    "News.dat: group string longer than 255 bytes",
                ));
            }
            if g.messages.len() > 0xffff {
                return Err(LegacyError::Invalid(
                    "News.dat: more than 65535 messages in a group",
                ));
            }
            for m in &g.messages {
                validate_message(m)?;
            }
        }
        if self.groups.len() > 0xffff {
            return Err(LegacyError::Invalid("News.dat: more than 65535 groups"));
        }
        Ok(())
    }

    /// Appends a message; its `group_id` and `message_id` are set here.
    pub fn post(&mut self, group_id: u32, mut message: NewsMessage) -> Result<u32> {
        validate_message(&message)?;
        let g = self
            .group_mut(group_id)
            .ok_or(LegacyError::Invalid("News.dat: no such newsgroup"))?;
        let id = g.last_message_id + 1;
        message.group_id = group_id;
        message.message_id = id;
        g.messages.push(message);
        g.last_message_id = id;
        g.info.message_count = g.messages.len().min(0xffff) as u16;
        Ok(id)
    }

    /// Removes a message; whether there was one.
    pub fn delete_message(&mut self, group_id: u32, message_id: u32) -> bool {
        let Some(g) = self.group_mut(group_id) else {
            return false;
        };
        let before = g.messages.len();
        g.messages.retain(|m| m.message_id != message_id);
        g.info.message_count = g.messages.len().min(0xffff) as u16;
        g.messages.len() != before
    }

    /// Adds a group with the next free id (ids start at 1) and returns it.
    pub fn create_group(&mut self, mut info: NewsgroupInfo) -> u32 {
        let id = self
            .groups
            .iter()
            .map(|g| g.info.group_id)
            .max()
            .unwrap_or(0)
            + 1;
        info.group_id = id;
        info.message_count = 0;
        self.groups.push(NewsGroupRecord {
            last_message_id: 1,
            info,
            ..Default::default()
        });
        id
    }

    /// Removes a group and its messages; whether there was one.
    pub fn delete_group(&mut self, group_id: u32) -> bool {
        let before = self.groups.len();
        self.groups.retain(|g| g.info.group_id != group_id);
        self.groups.len() != before
    }
}

const fn validate_message(m: &NewsMessage) -> Result<()> {
    if m.poster.len() > 0xff || m.login.len() > 0xff || m.subject.len() > 0xff {
        return Err(LegacyError::Invalid(
            "News.dat: message string longer than 255 bytes",
        ));
    }
    if m.body.len() > 0xffff {
        return Err(LegacyError::Invalid(
            "News.dat: message body longer than 65535 bytes",
        ));
    }
    Ok(())
}

const fn pad8(len: usize) -> usize {
    (8 - len % 8) % 8
}

/// Padding bytes as read when they are still the right length, else zeros.
fn gap(kept: &[u8], want: usize) -> Vec<u8> {
    (0..want)
        .map(|i| kept.get(i).copied().unwrap_or(0))
        .collect()
}

/// Decodes a `News.dat`; the empty file of a fresh server decodes to no groups.
pub fn decode_news(bytes: &[u8]) -> Result<NewsStore> {
    if bytes.is_empty() {
        return Ok(NewsStore::default());
    }
    check_header(bytes, MAGIC, 2, ServerFile::News.name())?;
    let mut r = Reader::new(bytes);
    r.bytes(8)?;
    let version = r.u16()?;
    let count = r.u16()? as usize;
    let ignored_0c: [u8; 4] = r.array()?;
    let mut table = [(0u32, 0u32); 5];
    for entry in &mut table {
        *entry = (r.u32()?, r.u32()?);
    }
    let ignored_38: [u8; 8] = r.array()?;
    debug_assert_eq!(r.pos(), HEADER_LEN);
    let (groups_offset, groups_len) = table[1];
    if groups_offset != GROUPS_OFFSET {
        return Err(LegacyError::Invalid("News.dat: groups section offset"));
    }
    let mut table_other = table;
    table_other[1] = (0, 0);

    let section_end = HEADER_LEN + groups_len as usize;
    if count > groups_len as usize / 0x70 + 1 || section_end > bytes.len() {
        return Err(LegacyError::Invalid(
            "News.dat: group count past the groups section",
        ));
    }
    let mut groups = Vec::with_capacity(count);
    for _ in 0..count {
        let start = r.pos();
        let list_offset = r.u32()? as usize;
        let list_len = r.u32()? as usize;
        let last_message_id = r.u32()?;
        let ignored_0c: [u8; 12] = r.array()?;
        let info = NewsgroupInfo::decode(r.peek())?;
        r.skip(NewsgroupInfo::HEADER_LEN + info.name.len() + info.description.len())?;
        let mut padding = r.pad4(start)?;
        if padding.iter().all(|&b| b == 0) {
            padding.clear();
        }
        groups.push((
            list_offset,
            list_len,
            NewsGroupRecord {
                last_message_id,
                ignored_0c,
                info,
                padding,
                ..Default::default()
            },
        ));
    }
    if r.pos() != section_end {
        return Err(LegacyError::Invalid("News.dat: groups section length"));
    }
    let mut cursor = section_end;
    let mut groups_gap = Vec::new();
    let mut out = Vec::with_capacity(count);
    for (i, (list_offset, list_len, mut g)) in groups.into_iter().enumerate() {
        if list_offset < cursor || list_offset - cursor > 7 {
            return Err(LegacyError::Invalid("News.dat: message list offset"));
        }
        let gap_bytes = bytes
            .get(cursor..list_offset)
            .ok_or(LegacyError::Invalid("News.dat: truncated"))?
            .to_vec();
        if i == 0 {
            groups_gap = gap_bytes;
        } else if let Some(prev) = out.last_mut() {
            let prev: &mut NewsGroupRecord = prev;
            prev.gap = gap_bytes;
        }
        let list = bytes
            .get(list_offset..list_offset + list_len)
            .ok_or(LegacyError::Invalid("News.dat: message list past the end"))?;
        let (list_ignored, messages) = decode_list(list, g.info.group_id)?;
        g.list_ignored = list_ignored;
        g.messages = messages;
        out.push(g);
        cursor = list_offset + list_len;
    }
    let trailer = bytes.get(cursor..).unwrap_or_default().to_vec();
    if trailer.len() > 7 {
        return Err(LegacyError::Invalid("News.dat: bytes after the last list"));
    }
    Ok(NewsStore {
        version,
        ignored_0c,
        table_other,
        ignored_38,
        groups: out,
        groups_gap,
        trailer,
        has_header: true,
    })
}

fn decode_list(bytes: &[u8], group_id: u32) -> Result<(u32, Vec<NewsMessage>)> {
    let mut r = Reader::new(bytes);
    if r.u32()? != LIST_MAGIC {
        return Err(LegacyError::Invalid("News.dat: list section magic"));
    }
    let count = r.u32()? as usize;
    let ignored = r.u32()?;
    if count > r.remaining() / 4 {
        return Err(LegacyError::Invalid(
            "News.dat: message count past its list",
        ));
    }
    let mut ends = Vec::with_capacity(count);
    for _ in 0..count {
        ends.push(r.u32()? as usize);
    }
    let records = r.rest();
    let mut out = Vec::with_capacity(count);
    let mut start = 0;
    for end in ends {
        let slice = records
            .get(start..end)
            .ok_or(LegacyError::Invalid("News.dat: message past its list"))?;
        let (m, used) = NewsMessage::decode(slice)?;
        if used != slice.len() || m.group_id != group_id {
            return Err(LegacyError::Invalid("News.dat: message record"));
        }
        out.push(m);
        start = end;
    }
    if start != records.len() {
        return Err(LegacyError::Invalid(
            "News.dat: bytes after the last message",
        ));
    }
    Ok((ignored, out))
}

fn encode_list(g: &NewsGroupRecord) -> Result<Vec<u8>> {
    let mut records = Writer::new();
    let mut ends = Vec::with_capacity(g.messages.len());
    for m in &g.messages {
        // `NewsMessage::encode` pads to 4 past the stored padding, as the
        // file does.
        records.bytes(&m.encode()?);
        ends.push(records.pos() as u32);
    }
    let mut w = Writer::new();
    w.u32(LIST_MAGIC);
    w.u32(g.messages.len() as u32);
    w.u32(g.list_ignored);
    for e in ends {
        w.u32(e);
    }
    w.bytes(&records.into_inner());
    Ok(w.into_inner())
}

/// Encodes a `News.dat`; fails when a string does not fit its length byte (see [`NewsStore::validate`]).
pub fn encode_news(store: &NewsStore) -> Result<Vec<u8>> {
    if store.groups.is_empty() && !store.has_header {
        return Ok(Vec::new());
    }
    // Lists first, to know their offsets and lengths for the entries.
    let lists = store
        .groups
        .iter()
        .map(encode_list)
        .collect::<Result<Vec<_>>>()?;
    let mut entries = Writer::new();
    let mut entry_lens = Vec::new();
    for g in &store.groups {
        let start = entries.pos();
        entries.u32(0);
        entries.u32(0);
        entries.u32(g.last_message_id);
        entries.bytes(&g.ignored_0c);
        entries.bytes(&g.info.encode()?);
        let want = pad4_len(entries.pos() - start);
        entries.bytes(&gap(&g.padding, want));
        entry_lens.push(entries.pos() - start);
    }
    let mut entries = entries.into_inner();
    let groups_len = entries.len();
    let mut offset = HEADER_LEN + groups_len;
    let groups_gap = gap(&store.groups_gap, pad8(offset));
    offset += groups_gap.len();
    let mut body = Writer::new();
    body.bytes(&groups_gap);
    let mut entry_start = 0;
    for (i, (g, list)) in store.groups.iter().zip(&lists).enumerate() {
        if i > 0 {
            let fill = gap(&store.groups[i - 1].gap, pad8(offset));
            body.bytes(&fill);
            offset += fill.len();
        }
        entries[entry_start..entry_start + 4].copy_from_slice(&(offset as u32).to_be_bytes());
        entries[entry_start + 4..entry_start + 8]
            .copy_from_slice(&(list.len() as u32).to_be_bytes());
        entry_start += entry_lens[i];
        body.bytes(list);
        offset += list.len();
        let _ = g;
    }
    let trailer = gap(&store.trailer, pad8(offset));
    let mut w = Writer::new();
    w.bytes(&MAGIC);
    w.u16(store.version);
    w.u16(store.groups.len().min(0xffff) as u16);
    w.bytes(&store.ignored_0c);
    let mut table = store.table_other;
    table[1] = (GROUPS_OFFSET, groups_len as u32);
    for (o, l) in table {
        w.u32(o);
        w.u32(l);
    }
    w.bytes(&store.ignored_38);
    w.bytes(&entries);
    w.bytes(&body.into_inner());
    w.bytes(&trailer);
    Ok(w.into_inner())
}

impl LegacyDir {
    /// Reads `News.dat` if the directory has one.
    pub fn load_news(&self) -> Result<Option<NewsStore>> {
        self.read(ServerFile::News)?
            .map(|b| decode_news(&b))
            .transpose()
    }

    /// Writes `News.dat` atomically, after [`NewsStore::validate`].
    pub fn save_news(&self, store: &NewsStore) -> Result<()> {
        store.validate()?;
        self.save(ServerFile::News, &encode_news(store)?)
    }
}
