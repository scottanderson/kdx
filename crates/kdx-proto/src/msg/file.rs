//! File listings, folder changes, volumes, launching, the catalog, and search.

use super::{
    len_u8, need, pad4, padding, strings, u16_at, u32_at, u64_at, DecodeError, EncodeError,
    FilePath,
};

/// The attributes value of a file.
pub const ATTR_FILE: u32 = 1;
/// The attributes value of a folder.
pub const ATTR_FOLDER: u32 = 0x0100_0002;

/// The payload of 414 Download File (D) and 417 Upload File (D): the bytes of one fork, unframed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileData(pub Vec<u8>);

impl FileData {
    /// Takes the payload as it is; any bytes are valid.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(Self(buf.to_vec()))
    }

    /// The payload.
    pub fn encode(&self) -> Vec<u8> {
        self.0.clone()
    }
}
/// An entry of 401 File List.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileListEntry {
    /// Size in bytes; 0x1000 for folders.
    pub size: u64,
    /// Creation time in milliseconds.
    pub created: u64,
    /// Modification time in milliseconds.
    pub modified: u64,
    /// Bytes 0x18 to 0x1f; meaning unknown.
    pub unknown_18: [u8; 8],
    /// [`ATTR_FILE`] or [`ATTR_FOLDER`].
    pub attributes: u32,
    /// Bytes 0x24 to 0x2d; meaning unknown.
    pub unknown_24: [u8; 10],
    /// Meaning unknown; 0xffff for folders and 0 for files in the lab.
    pub unknown_2e: u16,
    /// The folder access item that applies, as in 1107 Folder Access List.
    pub access_item: u16,
    /// Bytes 0x32 and 0x33; meaning unknown.
    pub unknown_32: [u8; 2],
    /// Meaning unknown; 4 or 5 in the lab.
    pub unknown_34: u8,
    /// Byte 0x37; meaning unknown.
    pub unknown_37: u8,
    /// Item name.
    pub name: Vec<u8>,
    /// For files, probably a type derived from the extension, such as `exttype/bin`.
    pub kind: Vec<u8>,
    /// Bytes after the kind up to the next multiple of 4, uninitialised on the wire; empty in a new entry.
    pub padding: Vec<u8>,
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
            let (padding, padded) = padding(entry, end);
            list.push(Self {
                size: u64_at(entry, 0),
                created: u64_at(entry, 8),
                modified: u64_at(entry, 0x10),
                unknown_18: entry[0x18..0x20].try_into().expect("8 bytes"),
                attributes: u32_at(entry, 0x20),
                unknown_24: entry[0x24..0x2e].try_into().expect("10 bytes"),
                unknown_2e: u16::from_be_bytes([entry[0x2e], entry[0x2f]]),
                access_item: u16::from_be_bytes([entry[0x30], entry[0x31]]),
                unknown_32: entry[0x32..0x34].try_into().expect("2 bytes"),
                unknown_34: entry[0x34],
                unknown_37: entry[0x37],
                name,
                kind,
                padding,
            });
            at += padded;
        }
        Ok(list)
    }

    /// Encodes the payload of 401 File List, padding with zeros past each stored padding.
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
            fixed[0x18..0x20].copy_from_slice(&entry.unknown_18);
            fixed[0x20..0x24].copy_from_slice(&entry.attributes.to_be_bytes());
            fixed[0x24..0x2e].copy_from_slice(&entry.unknown_24);
            fixed[0x2e..0x30].copy_from_slice(&entry.unknown_2e.to_be_bytes());
            fixed[0x30..0x32].copy_from_slice(&entry.access_item.to_be_bytes());
            fixed[0x32..0x34].copy_from_slice(&entry.unknown_32);
            fixed[0x34] = entry.unknown_34;
            fixed[0x35] = len_u8(&entry.name, "file name")?;
            fixed[0x36] = len_u8(&entry.kind, "kind")?;
            fixed[0x37] = entry.unknown_37;
            out.extend_from_slice(&fixed);
            out.extend_from_slice(&entry.name);
            out.extend_from_slice(&entry.kind);
            out.extend_from_slice(&entry.padding);
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
    /// Bytes 0x20 to 0x27; meaning unknown.
    pub unknown_20: [u8; 8],
    /// Meaning unknown; 2 for a folder in the lab.
    pub unknown_28: u32,
    /// Bytes 0x2c to 0x3c; meaning unknown.
    pub unknown_2c: [u8; 0x11],
    /// Byte 0x3f; meaning unknown.
    pub unknown_3f: u8,
    /// Item name.
    pub name: Vec<u8>,
    /// Kind, such as `folder`.
    pub kind: Vec<u8>,
    /// Bytes after the kind, as received; empty in a new message.
    pub rest: Vec<u8>,
}

impl FileInfo {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x40;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let lens = [usize::from(buf[0x3d]), usize::from(buf[0x3e])];
        let ([name, kind], end) = strings(buf, Self::HEADER_LEN, lens)?;
        Ok(Self {
            unknown_00: u32_at(buf, 0),
            unknown_04: u32_at(buf, 4),
            size: u64_at(buf, 8),
            created: u64_at(buf, 0x10),
            modified: u64_at(buf, 0x18),
            unknown_20: buf[0x20..0x28].try_into().expect("8 bytes"),
            unknown_28: u32_at(buf, 0x28),
            unknown_2c: buf[0x2c..0x3d].try_into().expect("17 bytes"),
            unknown_3f: buf[0x3f],
            name,
            kind,
            rest: buf[end..].to_vec(),
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
        out[0x20..0x28].copy_from_slice(&self.unknown_20);
        out[0x28..0x2c].copy_from_slice(&self.unknown_28.to_be_bytes());
        out[0x2c..0x3d].copy_from_slice(&self.unknown_2c);
        out[0x3d] = len_u8(&self.name, "file name")?;
        out[0x3e] = len_u8(&self.kind, "kind")?;
        out[0x3f] = self.unknown_3f;
        out.extend_from_slice(&self.name);
        out.extend_from_slice(&self.kind);
        out.extend_from_slice(&self.rest);
        Ok(out)
    }
}

/// 402 Create Folder.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CreateFolder {
    /// Bytes 0x00 to 0x12; meaning unknown.
    pub unknown_00: [u8; 0x13],
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
            unknown_00: buf[..0x13].try_into().expect("19 bytes"),
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
        out[..0x13].copy_from_slice(&self.unknown_00);
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetFileInfo {
    /// Bytes 0x00 to 0x27; meaning unknown.
    pub unknown_00: [u8; 0x28],
    /// [`Self::RENAME`], and bits 0x86 to set attributes.
    pub flags: u32,
    /// Bytes 0x2c and 0x2d; meaning unknown.
    pub unknown_2c: [u8; 2],
    /// Byte 0x30; meaning unknown.
    pub unknown_30: u8,
    /// Bytes 0x32 and 0x33; meaning unknown.
    pub unknown_32: [u8; 2],
    /// The item to change.
    pub path: FilePath,
    /// The new name.
    pub name: Vec<u8>,
}

impl Default for SetFileInfo {
    fn default() -> Self {
        Self {
            unknown_00: [0; 0x28],
            flags: 0,
            unknown_2c: [0; 2],
            unknown_30: 0,
            unknown_32: [0; 2],
            path: FilePath::default(),
            name: Vec::new(),
        }
    }
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
            unknown_00: buf[..0x28].try_into().expect("40 bytes"),
            flags: u32_at(buf, 0x28),
            unknown_2c: buf[0x2c..0x2e].try_into().expect("2 bytes"),
            unknown_30: buf[0x30],
            unknown_32: buf[0x32..0x34].try_into().expect("2 bytes"),
            path: FilePath::decode(&path)?,
            name,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let path = self.path.encode()?;
        let path_len = u16::try_from(path.len()).map_err(|_| EncodeError::TooLong("path"))?;
        let mut out = vec![0; Self::HEADER_LEN];
        out[..0x28].copy_from_slice(&self.unknown_00);
        out[0x28..0x2c].copy_from_slice(&self.flags.to_be_bytes());
        out[0x2c..0x2e].copy_from_slice(&self.unknown_2c);
        out[0x2e..0x30].copy_from_slice(&path_len.to_be_bytes());
        out[0x30] = self.unknown_30;
        out[0x31] = len_u8(&self.name, "new name")?;
        out[0x32..0x34].copy_from_slice(&self.unknown_32);
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
    /// Bytes 0x24 and 0x25; meaning unknown.
    pub unknown_24: [u8; 2],
    /// Disk type: 1 RAM, 2 Network, 3 Cartridge, 4 Hard Disk, 5 CD, 6 DVD, 7 Floppy.
    pub disk_type: u8,
    /// Bytes 0x27 to 0x2b; meaning unknown.
    pub unknown_27: [u8; 5],
    /// Meaning unknown; 0x88 in the lab.
    pub unknown_2c: u32,
    /// Icon.
    pub icon: u32,
    /// Bytes 0x34 to 0x43; meaning unknown.
    pub unknown_34: [u8; 0x10],
    /// Bit 0: name editable; bit 1: comments editable.
    pub flags: u32,
    /// The bits of `flags` that apply.
    pub flags_mask: u32,
    /// Bytes 0x4c and 0x4d; meaning unknown.
    pub unknown_4c: [u8; 2],
    /// Byte 0x4f; meaning unknown.
    pub unknown_4f: u8,
    /// Bytes 0x52 and 0x53; meaning unknown.
    pub unknown_52: [u8; 2],
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
            unknown_24: buf[0x24..0x26].try_into().expect("2 bytes"),
            disk_type: buf[0x26],
            unknown_27: buf[0x27..0x2c].try_into().expect("5 bytes"),
            unknown_2c: u32_at(buf, 0x2c),
            icon: u32_at(buf, 0x30),
            unknown_34: buf[0x34..0x44].try_into().expect("16 bytes"),
            flags: u32_at(buf, 0x44),
            flags_mask: u32_at(buf, 0x48),
            unknown_4c: buf[0x4c..0x4e].try_into().expect("2 bytes"),
            unknown_4f: buf[0x4f],
            unknown_52: buf[0x52..0x54].try_into().expect("2 bytes"),
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
        out[0x24..0x26].copy_from_slice(&self.unknown_24);
        out[0x26] = self.disk_type;
        out[0x27..0x2c].copy_from_slice(&self.unknown_27);
        out[0x2c..0x30].copy_from_slice(&self.unknown_2c.to_be_bytes());
        out[0x30..0x34].copy_from_slice(&self.icon.to_be_bytes());
        out[0x34..0x44].copy_from_slice(&self.unknown_34);
        out[0x44..0x48].copy_from_slice(&self.flags.to_be_bytes());
        out[0x48..0x4c].copy_from_slice(&self.flags_mask.to_be_bytes());
        out[0x4c..0x4e].copy_from_slice(&self.unknown_4c);
        out[0x4e] = len_u8(&self.name, "volume name")?;
        out[0x4f] = self.unknown_4f;
        out[0x50] = len_u8(&self.format, "volume format")?;
        out[0x51] = len_u8(&self.comments, "volume comments")?;
        out[0x52..0x54].copy_from_slice(&self.unknown_52);
        out.extend_from_slice(&self.name);
        out.extend_from_slice(&self.format);
        out.extend_from_slice(&self.comments);
        Ok(out)
    }
}

/// 423 Set Volume Info, in the 422 layout with a second name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetVolumeInfo {
    /// Bytes 0x00 to 0x43, which the server ignores.
    pub unknown_00: [u8; 0x44],
    /// [`Self::RENAME`] and [`Self::SET_COMMENTS`]: the fields to change.
    pub flags: u32,
    /// Bytes 0x48 to 0x4d, which the server ignores.
    pub unknown_48: [u8; 6],
    /// Bytes 0x52 and 0x53, which the server ignores.
    pub unknown_52: [u8; 2],
    /// The volume's current name.
    pub name: Vec<u8>,
    /// The new name.
    pub new_name: Vec<u8>,
    /// The format; the server ignores it.
    pub format: Vec<u8>,
    /// The new comments.
    pub comments: Vec<u8>,
}

impl Default for SetVolumeInfo {
    fn default() -> Self {
        Self {
            unknown_00: [0; 0x44],
            flags: 0,
            unknown_48: [0; 6],
            unknown_52: [0; 2],
            name: Vec::new(),
            new_name: Vec::new(),
            format: Vec::new(),
            comments: Vec::new(),
        }
    }
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
            unknown_00: buf[..0x44].try_into().expect("68 bytes"),
            flags: u32_at(buf, 0x44),
            unknown_48: buf[0x48..0x4e].try_into().expect("6 bytes"),
            unknown_52: buf[0x52..0x54].try_into().expect("2 bytes"),
            name,
            new_name,
            format,
            comments,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[..0x44].copy_from_slice(&self.unknown_00);
        out[0x44..0x48].copy_from_slice(&self.flags.to_be_bytes());
        out[0x48..0x4e].copy_from_slice(&self.unknown_48);
        out[0x4e] = len_u8(&self.name, "volume name")?;
        out[0x4f] = len_u8(&self.new_name, "new volume name")?;
        out[0x50] = len_u8(&self.format, "volume format")?;
        out[0x51] = len_u8(&self.comments, "volume comments")?;
        out[0x52..0x54].copy_from_slice(&self.unknown_52);
        for part in [&self.name, &self.new_name, &self.format, &self.comments] {
            out.extend_from_slice(part);
        }
        Ok(out)
    }
}

/// 420 Launch File.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LaunchFile {
    /// Bytes 0x00 to 0x07, 0 from the client.
    pub unknown_00: [u8; 8],
    /// The file to run.
    pub path: FilePath,
}

impl LaunchFile {
    /// Size of the fixed part before the path.
    pub const HEADER_LEN: usize = 8;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN + 1)?;
        Ok(Self {
            unknown_00: buf[..Self::HEADER_LEN].try_into().expect("8 bytes"),
            path: FilePath::decode(&buf[Self::HEADER_LEN..])?,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = self.unknown_00.to_vec();
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSearch {
    /// Bytes 0x00 to 0x53, which the server ignores.
    pub unknown_00: [u8; 0x54],
    /// Bytes 0x55 to 0x57, which the server ignores.
    pub unknown_55: [u8; 3],
    /// Text to find in item names; at least 3 bytes.
    pub query: Vec<u8>,
}

impl Default for FileSearch {
    fn default() -> Self {
        Self {
            unknown_00: [0; 0x54],
            unknown_55: [0; 3],
            query: Vec::new(),
        }
    }
}

impl FileSearch {
    /// Size of the fixed part before the query.
    pub const HEADER_LEN: usize = 0x58;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::HEADER_LEN)?;
        let ([query], _) = strings(buf, Self::HEADER_LEN, [usize::from(buf[0x54])])?;
        Ok(Self {
            unknown_00: buf[..0x54].try_into().expect("84 bytes"),
            unknown_55: buf[0x55..0x58].try_into().expect("3 bytes"),
            query,
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = vec![0; Self::HEADER_LEN];
        out[..0x54].copy_from_slice(&self.unknown_00);
        out[0x54] = len_u8(&self.query, "search query")?;
        out[0x55..0x58].copy_from_slice(&self.unknown_55);
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
    /// Bytes 0x1c to 0x1f; meaning unknown.
    pub unknown_1c: [u8; 4],
    /// [`ATTR_FILE`] or [`ATTR_FOLDER`].
    pub attributes: u32,
    /// Bytes 0x24 to 0x2d; meaning unknown.
    pub unknown_24: [u8; 10],
    /// As in [`FileListEntry::unknown_2e`].
    pub unknown_2e: u16,
    /// Bytes 0x30 to 0x33; meaning unknown.
    pub unknown_30: [u8; 4],
    /// Meaning unknown; 0x10 for a file in the lab.
    pub unknown_34: u8,
    /// Byte 0x35; meaning unknown.
    pub unknown_35: u8,
    /// Byte 0x37; meaning unknown.
    pub unknown_37: u8,
    /// Item name; empty in the lab, where the path holds it.
    pub name: Vec<u8>,
    /// The item's full path, root first.
    pub path: FilePath,
    /// Bytes after the path up to the next multiple of 4, as received; empty in a new entry.
    pub padding: Vec<u8>,
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
            let (padding, padded) = padding(entry, end);
            list.push(Self {
                size: u64_at(entry, 0),
                created: u64_at(entry, 8),
                modified: u64_at(entry, 0x10),
                unknown_1c: entry[0x1c..0x20].try_into().expect("4 bytes"),
                attributes: u32_at(entry, 0x20),
                unknown_24: entry[0x24..0x2e].try_into().expect("10 bytes"),
                unknown_2e: u16_at(entry, 0x2e),
                unknown_30: entry[0x30..0x34].try_into().expect("4 bytes"),
                unknown_34: entry[0x34],
                unknown_35: entry[0x35],
                unknown_37: entry[0x37],
                name,
                path: FilePath::decode(&path)?,
                padding,
            });
            at += padded;
        }
        Ok(list)
    }

    /// Encodes the payload of 453 File Search (R), padding with zeros past each stored padding.
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
            fixed[0x1c..0x20].copy_from_slice(&entry.unknown_1c);
            fixed[0x20..0x24].copy_from_slice(&entry.attributes.to_be_bytes());
            fixed[0x24..0x2e].copy_from_slice(&entry.unknown_24);
            fixed[0x2e..0x30].copy_from_slice(&entry.unknown_2e.to_be_bytes());
            fixed[0x30..0x34].copy_from_slice(&entry.unknown_30);
            fixed[0x34] = entry.unknown_34;
            fixed[0x35] = entry.unknown_35;
            fixed[0x36] = len_u8(&entry.name, "file name")?;
            fixed[0x37] = entry.unknown_37;
            out.extend_from_slice(&fixed);
            out.extend_from_slice(&entry.name);
            out.extend_from_slice(&path);
            out.extend_from_slice(&entry.padding);
            out.resize(pad4(out.len()), 0);
        }
        Ok(out)
    }
}
