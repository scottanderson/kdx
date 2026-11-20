//! `Accounts.dat`: classes, accounts and folder access items, scrambled with
//! [`Stream::SERVER_ACCOUNTS`]. See `docs/formats/accounts-dat.md`.
//!
//! Plain layout: `%KDXAKNT`, version 2, a table of up to seven `(offset,
//! length)` sections at 8-aligned offsets, then the sections. Sections 0–2
//! are lists (`47 fd db 01`, count, zero, one cumulative end offset per
//! record, the records) of the wire records `AccountClass` (in its 503
//! layout), `Account` and `FolderAccess`. The file ends with four bytes that
//! the server leaves as zero or as the keystream; they are kept as read.

use crate::wire::{Reader, Writer};
use kdx_proto::msg::admin::FolderAccess;
use kdx_proto::msg::admin::{Account, AccountClass};
use kdx_proto::msg::{DecodeError, EncodeError};

use crate::error::{LegacyError, Result};
use crate::scramble::{scramble, unscramble, Stream};
use crate::server_stg::check_header;
use crate::store::{LegacyDir, ServerFile};

/// The file's first eight bytes.
pub const MAGIC: [u8; 8] = *b"%KDXAKNT";
const LIST_MAGIC: u32 = 0x47fd_db01;
const TABLE_LEN: usize = 7;
const HEADER_LEN: usize = 0x50;

/// The decoded `Accounts.dat`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountsFile {
    /// File version; 2.
    pub version: u16,
    /// Bytes 0x0a and 0x0b; zero.
    pub ignored_0a: [u8; 2],
    /// Bytes 0x0c to 0x0f; zero.
    pub ignored_0c: [u8; 4],
    /// `00 00 01 01 02 00 00 00` in every file seen.
    pub ignored_48: [u8; 8],
    /// Section 0: the account classes, in the 503 layout.
    pub classes: Vec<AccountClass>,
    /// Section 1: the accounts.
    pub accounts: Vec<Account>,
    /// Section 2: the folder access items.
    pub folder_access: Vec<FolderAccess>,
    /// Sections 3–6, never seen non-empty.
    pub other: [Vec<u8>; TABLE_LEN],
    /// Each list section's second header word, zero so far.
    pub list_ignored: [u32; 3],
    /// Bytes between the end of a section and the next 8-aligned offset:
    /// uninitialized in the originals, so kept as read.
    pub gaps: [Vec<u8>; TABLE_LEN],
    /// Everything after the last section.
    pub trailer: Vec<u8>,
}

impl Default for AccountsFile {
    fn default() -> Self {
        AccountsFile {
            version: 2,
            ignored_0a: [0; 2],
            ignored_0c: [0; 4],
            ignored_48: [0, 0, 1, 1, 2, 0, 0, 0],
            classes: Vec::new(),
            accounts: Vec::new(),
            folder_access: Vec::new(),
            other: Default::default(),
            list_ignored: [0; 3],
            gaps: Default::default(),
            trailer: vec![0; 4],
        }
    }
}

impl AccountsFile {
    /// The account with this login.
    pub fn find(&self, login: &[u8]) -> Option<&Account> {
        self.accounts.iter().find(|a| a.login == login)
    }

    /// Replaces the account with the same login in place, or appends.
    pub fn upsert(&mut self, account: Account) {
        match self.accounts.iter_mut().find(|a| a.login == account.login) {
            Some(slot) => *slot = account,
            None => self.accounts.push(account),
        }
    }

    /// Every string must fit its length byte, as the wire encoders assume.
    pub fn validate(&self) -> Result<()> {
        let long = |v: &Vec<u8>| v.len() > 0xff;
        for c in &self.classes {
            if long(&c.name) || long(&c.second) || long(&c.editor) {
                return Err(LegacyError::Invalid(
                    "Accounts.dat: class string longer than 255 bytes",
                ));
            }
        }
        for a in &self.accounts {
            if long(&a.login) || long(&a.name) || long(&a.third) || long(&a.editor) {
                return Err(LegacyError::Invalid(
                    "Accounts.dat: account string longer than 255 bytes",
                ));
            }
        }
        for f in &self.folder_access {
            if long(&f.name) || long(&f.owner) || long(&f.description) || long(&f.current_name) {
                return Err(LegacyError::Invalid(
                    "Accounts.dat: folder access string longer than 255 bytes",
                ));
            }
        }
        Ok(())
    }

    /// Removes the account with this login; whether there was one.
    pub fn remove(&mut self, login: &[u8]) -> bool {
        let before = self.accounts.len();
        self.accounts.retain(|a| a.login != login);
        self.accounts.len() != before
    }

    fn section_bytes(&self, i: usize) -> Result<Vec<u8>> {
        Ok(match i {
            0 => encode_list(
                self.list_ignored[0],
                &self.classes,
                AccountClass::encode_info,
            )?,
            1 => encode_list(self.list_ignored[1], &self.accounts, Account::encode)?,
            2 => encode_list(
                self.list_ignored[2],
                &self.folder_access,
                FolderAccess::encode,
            )?,
            _ => self.other[i].clone(),
        })
    }
}

/// A list section: magic, count, a word, cumulative end offsets, records.
fn encode_list<T>(
    ignored: u32,
    items: &[T],
    one: impl Fn(&T) -> std::result::Result<Vec<u8>, EncodeError>,
) -> Result<Vec<u8>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }
    let mut records = Writer::new();
    let mut ends = Vec::with_capacity(items.len());
    for item in items {
        let start = records.pos();
        records.bytes(&one(item)?);
        // The wire encoders pad a record to 4 past its stored padding; so
        // does the file, so this is a no-op kept for the invariant.
        records.pad4(start, &[]);
        ends.push(records.pos() as u32);
    }
    let mut w = Writer::new();
    w.u32(LIST_MAGIC);
    w.u32(items.len() as u32);
    w.u32(ignored);
    for e in ends {
        w.u32(e);
    }
    w.bytes(&records.into_inner());
    Ok(w.into_inner())
}

fn decode_list<T>(
    bytes: &[u8],
    one: impl Fn(&[u8]) -> std::result::Result<T, DecodeError>,
) -> Result<(u32, Vec<T>)> {
    if bytes.is_empty() {
        return Ok((0, Vec::new()));
    }
    let mut r = Reader::new(bytes);
    if r.u32()? != LIST_MAGIC {
        return Err(LegacyError::Invalid("Accounts.dat: list section magic"));
    }
    let count = r.u32()? as usize;
    let ignored = r.u32()?;
    if count > r.remaining() / 4 {
        return Err(LegacyError::Invalid(
            "Accounts.dat: list count past the section",
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
        let slice = records.get(start..end).ok_or(LegacyError::Invalid(
            "Accounts.dat: record past its section",
        ))?;
        // The wire decoders keep the bytes after the strings as padding, so
        // the whole slot comes back on encode.
        out.push(one(slice)?);
        start = end;
    }
    if start != records.len() {
        return Err(LegacyError::Invalid(
            "Accounts.dat: bytes after the last record",
        ));
    }
    Ok((ignored, out))
}

/// Decodes a scrambled `Accounts.dat`.
pub fn decode_accounts(bytes: &[u8]) -> Result<AccountsFile> {
    let plain = unscramble(bytes, Stream::SERVER_ACCOUNTS);
    check_header(&plain, MAGIC, 2, ServerFile::Accounts.name())?;
    let mut r = Reader::new(&plain);
    r.bytes(8)?;
    let version = r.u16()?;
    let ignored_0a: [u8; 2] = r.array()?;
    let ignored_0c: [u8; 4] = r.array()?;
    let mut table = [(0usize, 0usize); TABLE_LEN];
    for entry in &mut table {
        *entry = (r.u32()? as usize, r.u32()? as usize);
    }
    let ignored_48: [u8; 8] = r.array()?;
    debug_assert_eq!(r.pos(), HEADER_LEN);
    let mut sections: [Vec<u8>; TABLE_LEN] = Default::default();
    let mut gaps: [Vec<u8>; TABLE_LEN] = Default::default();
    let mut seen_one = false;
    for (i, &(offset, len)) in table.iter().enumerate() {
        if len == 0 {
            continue;
        }
        let here = r.pos();
        if offset < here || offset - here > 7 {
            return Err(LegacyError::Invalid("Accounts.dat: section offset"));
        }
        if seen_one {
            gaps[i] = r.bytes(offset - here)?;
        } else if offset != here {
            return Err(LegacyError::Invalid(
                "Accounts.dat: gap before the first section",
            ));
        }
        sections[i] = r.bytes(len)?;
        seen_one = true;
    }
    let trailer = r.rest();
    let [s0, s1, s2, s3, s4, s5, s6] = sections;
    let (l0, classes) = decode_list(&s0, AccountClass::decode_info)?;
    let (l1, accounts) = decode_list(&s1, Account::decode)?;
    let (l2, folder_access) = decode_list(&s2, FolderAccess::decode)?;
    let mut other: [Vec<u8>; TABLE_LEN] = Default::default();
    other[3] = s3;
    other[4] = s4;
    other[5] = s5;
    other[6] = s6;
    Ok(AccountsFile {
        version,
        ignored_0a,
        ignored_0c,
        ignored_48,
        classes,
        accounts,
        folder_access,
        other,
        list_ignored: [l0, l1, l2],
        gaps,
        trailer,
    })
}

/// Encodes and scrambles an `Accounts.dat`; fails when a string does not fit
/// its length byte (see [`AccountsFile::validate`]).
pub fn encode_accounts(file: &AccountsFile) -> Result<Vec<u8>> {
    let mut body = Writer::new();
    let mut table = [(0u32, 0u32); TABLE_LEN];
    let mut offset = HEADER_LEN;
    let mut first = true;
    for (i, entry) in table.iter_mut().enumerate() {
        let bytes = file.section_bytes(i)?;
        if bytes.is_empty() {
            continue;
        }
        if !first {
            // Sections start 8-aligned; the gap keeps what the file had if
            // it still fits, else zeros.
            let want = (8 - offset % 8) % 8;
            let kept = &file.gaps[i];
            let gap: Vec<u8> = (0..want)
                .map(|k| kept.get(k).copied().unwrap_or(0))
                .collect();
            body.bytes(&gap);
            offset += want;
        }
        first = false;
        *entry = (offset as u32, bytes.len() as u32);
        body.bytes(&bytes);
        offset += bytes.len();
    }
    let mut w = Writer::new();
    w.bytes(&MAGIC);
    w.u16(file.version);
    w.bytes(&file.ignored_0a);
    w.bytes(&file.ignored_0c);
    for (o, l) in table {
        w.u32(o);
        w.u32(l);
    }
    w.bytes(&file.ignored_48);
    w.bytes(&body.into_inner());
    w.bytes(&file.trailer);
    Ok(scramble(&w.into_inner(), Stream::SERVER_ACCOUNTS))
}

impl LegacyDir {
    /// Reads `Accounts.dat` if the directory has one.
    pub fn load_accounts(&self) -> Result<Option<AccountsFile>> {
        self.read(ServerFile::Accounts)?
            .map(|b| decode_accounts(&b))
            .transpose()
    }

    /// Writes `Accounts.dat`, keeping the first original as `.orig`.
    pub fn save_accounts(&self, file: &AccountsFile) -> Result<()> {
        file.validate()?;
        self.save(ServerFile::Accounts, &encode_accounts(file)?)
    }
}
