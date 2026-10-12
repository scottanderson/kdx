//! File listings, folder changes, volumes, launching, the catalog, and search.

use super::{
    len_u8, need, pad4, strings, u16_at, u32_at, u64_at, DecodeError, EncodeError, FilePath,
};

/// The attributes value of a file.
pub const ATTR_FILE: u32 = 1;
/// The attributes value of a folder.
pub const ATTR_FOLDER: u32 = 0x0100_0002;

/// An entry of 401 File List.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileListEntry {
    /// Size in bytes; 0x1000 for folders.
    pub size: u64,
    /// Creation time in milliseconds.
    pub created: u64,
    /// Modification time in milliseconds.
    pub modified: u64,
    /// [`ATTR_FILE`] or [`ATTR_FOLDER`].
    pub attributes: u32,
    /// Meaning unknown; 0xffff for folders and 0 for files in the lab.
    pub unknown_2e: u16,
    /// The folder access item that applies, as in 1107 Folder Access List.
    pub access_item: u16,
    /// Meaning unknown; 4 or 5 in the lab.
    pub unknown_34: u8,
    /// Item name.
    pub name: Vec<u8>,
    /// For files, probably a type derived from the extension, such as `exttype/bin`.
    pub kind: Vec<u8>,
}

impl FileListEntry {
    /// Size of the fixed part before the name.
    pub const HEADER_LEN: usize = 0x38;

    /// Whether the entry is a folder.
    pub const fn is_folder(&self) -> bool {
        self.attributes & 2 != 0
    }

    /// Decodes the payload of 401 File List; an empty payload is an empty folder.
    pub fn decode_list(buf: &[u8]) -> Result<Vec<Self>, DecodeError> {
        if buf.is_empty() {
            return Ok(Vec::new());
        }
        need(buf, 4)?;
        let count = u32_at(buf, 0) as usize;
        let mut at = 4;
        let mut list = Vec::with_capacity(count.min(buf.len()));
        for _ in 0..count {
            let entry = &buf[at..];
            need(entry, Self::HEADER_LEN)?;
            let lens = [usize::from(entry[0x35]), usize::from(entry[0x36])];
            let ([name, kind], end) = strings(entry, Self::HEADER_LEN, lens)?;
            list.push(Self {
                size: u64_at(entry, 0),
                created: u64_at(entry, 8),
                modified: u64_at(entry, 0x10),
                attributes: u32_at(entry, 0x20),
                unknown_2e: u16::from_be_bytes([entry[0x2e], entry[0x2f]]),
                access_item: u16::from_be_bytes([entry[0x30], entry[0x31]]),
                unknown_34: entry[0x34],
                name,
                kind,
            });
            at += pad4(end).min(entry.len());
        }
        Ok(list)
    }

    /// Encodes the payload of 401 File List, padding with zeros.
    pub fn encode_list(list: &[Self]) -> Result<Vec<u8>, EncodeError> {
        if list.is_empty() {
            return Ok(Vec::new());
        }
        let count = u32::try_from(list.len()).map_err(|_| EncodeError::TooLong("file list"))?;
        let mut out = count.to_be_bytes().to_vec();
        for entry in list {
            let mut fixed = [0; Self::HEADER_LEN];
            fixed[0..8].copy_from_slice(&entry.size.to_be_bytes());
            fixed[8..0x10].copy_from_slice(&entry.created.to_be_bytes());
            fixed[0x10..0x18].copy_from_slice(&entry.modified.to_be_bytes());
            fixed[0x20..0x24].copy_from_slice(&entry.attributes.to_be_bytes());
            fixed[0x2e..0x30].copy_from_slice(&entry.unknown_2e.to_be_bytes());
            fixed[0x30..0x32].copy_from_slice(&entry.access_item.to_be_bytes());
            fixed[0x34] = entry.unknown_34;
            fixed[0x35] = len_u8(&entry.name, "file name")?;
            fixed[0x36] = len_u8(&entry.kind, "kind")?;
            out.extend_from_slice(&fixed);
            out.extend_from_slice(&entry.name);
            out.extend_from_slice(&entry.kind);
            out.resize(pad4(out.len()), 0);
        }
        Ok(out)
    }
}

/// 410 File/Folder Info.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileInfo {
    /// Meaning unknown; 0x39 in the lab.
    pub unknown_00: u32,
    /// Meaning unknown; 0xbf in the lab.
    pub unknown_04: u32,
    /// Size in bytes.
    pub size: u64,
    /// Creation time in milliseconds.
    pub created: u64,
    /// Modification time in milliseconds.
    pub modified: u64,
    /// Meaning unknown; 2 for a folder in the lab.
    pub unknown_28: u32,
    /// Item name.
    pub name: Vec<u8>,
    /// Kind, such as `folder`.
    pub kind: Vec<u8>,
}

impl FileInfo {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x40;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [usize::from(buf[0x3d]), usize::from(buf[0x3e])];
        let ([name, kind], _) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            unknown_00: u32_at(buf, 0),
            unknown_04: u32_at(buf, 4),
            size: u64_at(buf, 8),
            created: u64_at(buf, 0x10),
            modified: u64_at(buf, 0x18),
            unknown_28: u32_at(buf, 0x28),
            name,
            kind,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..4].copy_from_slice(&self.unknown_00.to_be_bytes());
        out[4..8].copy_from_slice(&self.unknown_04.to_be_bytes());
        out[8..0x10].copy_from_slice(&self.size.to_be_bytes());
        out[0x10..0x18].copy_from_slice(&self.created.to_be_bytes());
        out[0x18..0x20].copy_from_slice(&self.modified.to_be_bytes());
        out[0x28..0x2c].copy_from_slice(&self.unknown_28.to_be_bytes());
        out[0x3d] = len_u8(&self.name, "file name")?;
        out[0x3e] = len_u8(&self.kind, "kind")?;
        out.extend_from_slice(&self.name);
        out.extend_from_slice(&self.kind);
        Ok(out)
    }
}

/// 402 Create Folder.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CreateFolder {
    /// The parent folder, or the new folder itself if `name` is empty.
    pub path: FilePath,
    /// The new folder's name.
    pub name: Vec<u8>,
    /// A folder access tag, such as `DB`, that the server appends as ` [tag]`.
    pub access: Vec<u8>,
    /// An owner login that the server appends to the tag.
    pub owner: Vec<u8>,
}

impl CreateFolder {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x18;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [
            usize::from(buf[0x16]),
            usize::from(buf[0x13]),
            usize::from(buf[0x17]),
            usize::from(u16_at(buf, 0x14)),
        ];
        let ([name, access, owner, path], _) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            path: FilePath::decode(&path)?,
            name,
            access,
            owner,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let path = self.path.encode()?;
        let path_len = u16::try_from(path.len()).map_err(|_| EncodeError::TooLong("path"))?;
        let mut out = vec![0; Self::HEADER_LEN];
        out[0x13] = len_u8(&self.access, "folder access tag")?;
        out[0x14..0x16].copy_from_slice(&path_len.to_be_bytes());
        out[0x16] = len_u8(&self.name, "folder name")?;
        out[0x17] = len_u8(&self.owner, "owner login")?;
        for part in [&self.name, &self.access, &self.owner, &path] {
            out.extend_from_slice(part);
        }
        Ok(out)
    }
}

/// Decodes a name list that fills `buf`.
fn decode_names(buf: &[u8]) -> Result<Vec<Vec<u8>>, DecodeError> {
    let (names, len) = FilePath::decode_delimited(buf)?;
    if len != buf.len() {
        return Err(DecodeError::Invalid("bytes after name list"));
    }
    Ok(names.0)
}

/// 403 Create Alias and 407 Move File/Folder: items of one folder, and the folder they go to.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MoveItems {
    /// The folder the items, or their aliases, go to.
    pub destination: FilePath,
    /// The folder that holds the items.
    pub source: FilePath,
    /// The items' names.
    pub names: Vec<Vec<u8>>,
}

impl MoveItems {
    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        let (destination, at) = FilePath::decode_delimited(buf)?;
        let (source, len) = FilePath::decode_delimited(&buf[at..])?;
        Ok(Self {
            destination,
            source,
            names: decode_names(&buf[at + len..])?,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = self.destination.encode_delimited()?;
        out.extend(self.source.encode_delimited()?);
        out.extend(FilePath(self.names.clone()).encode_delimited()?);
        Ok(out)
    }
}

/// 405 Move To Trash.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MoveToTrash {
    /// The folder that holds the items.
    pub folder: FilePath,
    /// The items' names.
    pub names: Vec<Vec<u8>>,
}

impl MoveToTrash {
    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        let (folder, at) = FilePath::decode_delimited(buf)?;
        Ok(Self {
            folder,
            names: decode_names(&buf[at..])?,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = self.folder.encode_delimited()?;
        out.extend(FilePath(self.names.clone()).encode_delimited()?);
        Ok(out)
    }
}

/// 411 Set File/Folder Info.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SetFileInfo {
    /// [`Self::RENAME`], and bits 0x86 to set attributes.
    pub flags: u32,
    /// The item to change.
    pub path: FilePath,
    /// The new name.
    pub name: Vec<u8>,
}

impl SetFileInfo {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x34;
    /// Renames the item to `name`.
    pub const RENAME: u32 = 1;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [usize::from(u16_at(buf, 0x2e)), usize::from(buf[0x31])];
        let ([path, name], _) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            flags: u32_at(buf, 0x28),
            path: FilePath::decode(&path)?,
            name,
        })
    }

    /// Encodes the payload, with zeros for the fields the client leaves empty.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let path = self.path.encode()?;
        let path_len = u16::try_from(path.len()).map_err(|_| EncodeError::TooLong("path"))?;
        let mut out = vec![0; Self::HEADER_LEN];
        out[0x28..0x2c].copy_from_slice(&self.flags.to_be_bytes());
        out[0x2e..0x30].copy_from_slice(&path_len.to_be_bytes());
        out[0x31] = len_u8(&self.name, "new name")?;
        out.extend_from_slice(&path);
        out.extend_from_slice(&self.name);
        Ok(out)
    }
}

/// 422 Volume Info, the reply to 409 Get File/Folder Info for a volume.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VolumeInfo {
    /// Total size in bytes; all ones if unknown.
    pub total: u64,
    /// Free space in bytes; all ones if unknown.
    pub free: u64,
    /// Modification time in milliseconds.
    pub modified: u64,
    /// Creation time in milliseconds.
    pub created: u64,
    /// Item count; 0xffffffff if unknown.
    pub item_count: u32,
    /// Disk type: 1 RAM, 2 Network, 3 Cartridge, 4 Hard Disk, 5 CD, 6 DVD, 7 Floppy.
    pub disk_type: u8,
    /// Meaning unknown; 0x88 in the lab.
    pub unknown_2c: u32,
    /// Icon.
    pub icon: u32,
    /// Bit 0: name editable; bit 1: comments editable.
    pub flags: u32,
    /// The bits of `flags` that apply.
    pub flags_mask: u32,
    /// Volume name, such as `/kdx`.
    pub name: Vec<u8>,
    /// Format, such as the file system name.
    pub format: Vec<u8>,
    /// Comments.
    pub comments: Vec<u8>,
}

impl VolumeInfo {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x54;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [buf[0x4e], buf[0x50], buf[0x51]].map(usize::from);
        let ([name, format, comments], _) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            total: u64_at(buf, 0),
            free: u64_at(buf, 8),
            modified: u64_at(buf, 0x10),
            created: u64_at(buf, 0x18),
            item_count: u32_at(buf, 0x20),
            disk_type: buf[0x26],
            unknown_2c: u32_at(buf, 0x2c),
            icon: u32_at(buf, 0x30),
            flags: u32_at(buf, 0x44),
            flags_mask: u32_at(buf, 0x48),
            name,
            format,
            comments,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0..8].copy_from_slice(&self.total.to_be_bytes());
        out[8..0x10].copy_from_slice(&self.free.to_be_bytes());
        out[0x10..0x18].copy_from_slice(&self.modified.to_be_bytes());
        out[0x18..0x20].copy_from_slice(&self.created.to_be_bytes());
        out[0x20..0x24].copy_from_slice(&self.item_count.to_be_bytes());
        out[0x26] = self.disk_type;
        out[0x2c..0x30].copy_from_slice(&self.unknown_2c.to_be_bytes());
        out[0x30..0x34].copy_from_slice(&self.icon.to_be_bytes());
        out[0x44..0x48].copy_from_slice(&self.flags.to_be_bytes());
        out[0x48..0x4c].copy_from_slice(&self.flags_mask.to_be_bytes());
        out[0x4e] = len_u8(&self.name, "volume name")?;
        out[0x50] = len_u8(&self.format, "volume format")?;
        out[0x51] = len_u8(&self.comments, "volume comments")?;
        out.extend_from_slice(&self.name);
        out.extend_from_slice(&self.format);
        out.extend_from_slice(&self.comments);
        Ok(out)
    }
}

/// 423 Set Volume Info, in the 422 layout with a second name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SetVolumeInfo {
    /// [`Self::RENAME`] and [`Self::SET_COMMENTS`]: the fields to change.
    pub flags: u32,
    /// The volume's current name.
    pub name: Vec<u8>,
    /// The new name.
    pub new_name: Vec<u8>,
    /// The format; the server ignores it.
    pub format: Vec<u8>,
    /// The new comments.
    pub comments: Vec<u8>,
}

impl SetVolumeInfo {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = VolumeInfo::HEADER_LEN;
    /// Renames the volume to `new_name`.
    pub const RENAME: u32 = 1;
    /// Sets the comments.
    pub const SET_COMMENTS: u32 = 2;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [buf[0x4e], buf[0x4f], buf[0x50], buf[0x51]].map(usize::from);
        let ([name, new_name, format, comments], _) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            flags: u32_at(buf, 0x44),
            name,
            new_name,
            format,
            comments,
        })
    }

    /// Encodes the payload, with zeros for the fields the server ignores.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0x44..0x48].copy_from_slice(&self.flags.to_be_bytes());
        out[0x4e] = len_u8(&self.name, "volume name")?;
        out[0x4f] = len_u8(&self.new_name, "new volume name")?;
        out[0x50] = len_u8(&self.format, "volume format")?;
        out[0x51] = len_u8(&self.comments, "volume comments")?;
        for part in [&self.name, &self.new_name, &self.format, &self.comments] {
            out.extend_from_slice(part);
        }
        Ok(out)
    }
}

/// 420 Launch File.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LaunchFile {
    /// The file to run.
    pub path: FilePath,
}

impl LaunchFile {
    /// Size of the zeros before the path.
    pub const HEADER_LEN: usize = 8;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN + 1)?;
        Ok(Self {
            path: FilePath::decode(&buf[Self::HEADER_LEN..])?,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out.extend_from_slice(&self.path.encode()?);
        Ok(out)
    }
}

/// 450 Generate Catalog.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GenerateCatalog {
    /// [`Self::CANCEL`] or 0.
    pub flags: u32,
}

impl GenerateCatalog {
    /// Cancels a running catalog; only its starter may.
    pub const CANCEL: u32 = 1;

    /// Decodes the payload; an empty payload has no flags.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        if buf.is_empty() {
            return Ok(Self::default());
        }
        need(buf, 4)?;
        Ok(Self {
            flags: u32_at(buf, 0),
        })
    }

    /// Encodes the payload; no flags encode as nothing, as the client sends it.
    pub fn encode(&self) -> Vec<u8> {
        if self.flags == 0 {
            Vec::new()
        } else {
            self.flags.to_be_bytes().to_vec()
        }
    }
}

/// 451 Generate Catalog (S), the progress of a catalog.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CatalogStatus {
    /// [`Self::DONE`] or 0.
    pub flags: u32,
    /// Items indexed so far.
    pub items: u32,
}

impl CatalogStatus {
    /// Size of the payload.
    pub const LEN: usize = 8;
    /// Set on the last status, when the catalog is built.
    pub const DONE: u32 = 1;

    /// Whether the catalog is built.
    pub const fn is_done(&self) -> bool {
        self.flags & Self::DONE != 0
    }

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::LEN)?;
        Ok(Self {
            flags: u32_at(buf, 0),
            items: u32_at(buf, 4),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> [u8; Self::LEN] {
        let mut out = [0; Self::LEN];
        out[0..4].copy_from_slice(&self.flags.to_be_bytes());
        out[4..8].copy_from_slice(&self.items.to_be_bytes());
        out
    }
}

/// 452 File Search.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileSearch {
    /// Text to find in item names; at least 3 bytes.
    pub query: Vec<u8>,
}

impl FileSearch {
    /// Size of the fixed part before the query.
    pub const HEADER_LEN: usize = 0x58;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let ([query], _) = strings(buf, Self::HEADER_LEN, [usize::from(buf[0x54])])?;
        Ok(Self { query })
    }

    /// Encodes the payload, with zeros for the fields the server ignores.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[0x54] = len_u8(&self.query, "search query")?;
        out.extend_from_slice(&self.query);
        Ok(out)
    }
}

/// An entry of 453 File Search (R).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileSearchEntry {
    /// Size in bytes.
    pub size: u64,
    /// Probably the creation time, as in 401.
    pub created: u64,
    /// Probably the modification time, as in 401.
    pub modified: u64,
    /// [`ATTR_FILE`] or [`ATTR_FOLDER`].
    pub attributes: u32,
    /// As in [`FileListEntry::unknown_2e`].
    pub unknown_2e: u16,
    /// Meaning unknown; 0x10 for a file in the lab.
    pub unknown_34: u8,
    /// Item name; empty in the lab, where the path holds it.
    pub name: Vec<u8>,
    /// The item's full path, root first.
    pub path: FilePath,
}

impl FileSearchEntry {
    /// Size of the fixed part before the name.
    pub const HEADER_LEN: usize = 0x38;

    /// Decodes the payload of 453 File Search (R); an empty payload has no hits.
    pub fn decode_list(buf: &[u8]) -> Result<Vec<Self>, DecodeError> {
        if buf.is_empty() {
            return Ok(Vec::new());
        }
        need(buf, 4)?;
        let count = u32_at(buf, 0) as usize;
        let mut at = 4;
        let mut list = Vec::with_capacity(count.min(buf.len()));
        for _ in 0..count {
            let entry = &buf[at..];
            need(entry, Self::HEADER_LEN)?;
            let path_len = u32_at(entry, 0x18) as usize;
            let ([name, path], end) = strings(
                entry,
                Self::HEADER_LEN,
                [usize::from(entry[0x36]), path_len],
            )?;
            list.push(Self {
                size: u64_at(entry, 0),
                created: u64_at(entry, 8),
                modified: u64_at(entry, 0x10),
                attributes: u32_at(entry, 0x20),
                unknown_2e: u16_at(entry, 0x2e),
                unknown_34: entry[0x34],
                name,
                path: FilePath::decode(&path)?,
            });
            at += pad4(end).min(entry.len());
        }
        Ok(list)
    }

    /// Encodes the payload of 453 File Search (R), padding with zeros.
    pub fn encode_list(list: &[Self]) -> Result<Vec<u8>, EncodeError> {
        if list.is_empty() {
            return Ok(Vec::new());
        }
        let count =
            u32::try_from(list.len()).map_err(|_| EncodeError::TooLong("search results"))?;
        let mut out = count.to_be_bytes().to_vec();
        for entry in list {
            let path = entry.path.encode()?;
            let path_len = u32::try_from(path.len()).map_err(|_| EncodeError::TooLong("path"))?;
            let mut fixed = [0; Self::HEADER_LEN];
            fixed[0..8].copy_from_slice(&entry.size.to_be_bytes());
            fixed[8..0x10].copy_from_slice(&entry.created.to_be_bytes());
            fixed[0x10..0x18].copy_from_slice(&entry.modified.to_be_bytes());
            fixed[0x18..0x1c].copy_from_slice(&path_len.to_be_bytes());
            fixed[0x20..0x24].copy_from_slice(&entry.attributes.to_be_bytes());
            fixed[0x2e..0x30].copy_from_slice(&entry.unknown_2e.to_be_bytes());
            fixed[0x34] = entry.unknown_34;
            fixed[0x36] = len_u8(&entry.name, "file name")?;
            out.extend_from_slice(&fixed);
            out.extend_from_slice(&entry.name);
            out.extend_from_slice(&path);
            out.resize(pad4(out.len()), 0);
        }
        Ok(out)
    }
}
