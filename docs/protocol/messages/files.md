# File messages

Listings, folder changes, deletion, volumes, launching, the catalog, and search. Downloads and uploads are on [their own page](transfers.md).

File requests address items with a file path. Each user is confined to a
base folder.

**400 Get File List** → **401 File List**
Initiator: Client. **[S `FUN_0804d938`; O]**

The request is a file path (empty for the root). A folder with no entries
gets an empty reply (len 0). Otherwise the reply is a u32 count, then one
entry per item, each padded to 4:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | size | 0x1000 for folders |
| 0x08 | 8 | created | time |
| 0x10 | 8 | modified | time |
| 0x18 | 8 | | |
| 0x20 | 4 | attributes | 1 for a file, 0x01000002 for a folder **[O]**; the server clears bit 24 and tests 0x42 **[S]** |
| 0x24 | 0xa | | |
| 0x2e | 2 | | 0xffff for folders, 0 for a file |
| 0x30 | 2 | folder icon | the folder access entry's icon: 0x2c Dropbox, 0x2b Uploads; 0 for a file (see 1100 Create Folder Access) **[S]** |
| 0x32 | 2 | | |
| 0x34 | 1 | | 4 for `Admin Dropbox [DB]`, 5 for `Uploads [UL]`, 0 for a file |
| 0x35 | 1 | name length | |
| 0x36 | 1 | kind length | 0 for folders; `exttype/bin` for a `.bin` file, as in 413 Download File (R) |
| 0x37 | 1 | | |
| 0x38 | | name, kind | |

**[O]** `captures/kdx-cli.pcapng`: a 600000-byte `cli-big.bin` is listed with
size 0x927c0 and kind `exttype/bin`. `kdx_proto::msg::file::FileListEntry`
decodes both kinds of entry.

When the folder access check (`FUN_08057e5c`) fails or the folder can't be
read, the server also replies with an empty 401 File List, and logs a
history record of kind 400 Get File List.

**409 Get File/Folder Info** → **410 File/Folder Info**
Initiator: Client. **[S `FUN_0804ef00`; O]**

The request is a file path. Reply (82 bytes for `Uploads [UL]`):

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | | 0x39 |
| 0x04 | 4 | | 0xbf |
| 0x08 | 8 | size | |
| 0x10 | 8 | created | |
| 0x18 | 8 | modified | |
| 0x20 | 8 | | |
| 0x28 | 4 | | 2 (frame 138 of `captures/probe.pcapng`) |
| 0x2c | 0x11 | | |
| 0x3d | 1 | name length | |
| 0x3e | 1 | kind length | |
| 0x40 | | name, kind | kind = `folder` |

If the path is a single name that is a volume (the user can see all
volumes), the reply is **422 Volume Info** instead **[S `FUN_0804ef00`,
client `FUN_080758dc`; O `captures/linux-catalog-volumes.pcapng` frames
50-51]**. A reply is not padded (93 bytes at 51):

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | total size | all ones if unknown |
| 0x08 | 8 | free space | the client shows used = total − free |
| 0x10 | 8 | modified | time |
| 0x18 | 8 | created | time |
| 0x20 | 4 | item count | 0xffffffff if unknown |
| 0x26 | 1 | disk type | 1 RAM, 2 Network, 3 Cartridge, 4 Hard Disk, 5 CD, 6 DVD, 7 Floppy; 0 on Linux **[O]** |
| 0x2c | 4 | | 0x88 **[O]** |
| 0x30 | 4 | icon | 0x3d **[O]** |
| 0x44 | 4 | flags | bit 0 name editable, bit 1 comments editable |
| 0x48 | 4 | flags mask | the client ANDs it with 0x44 |
| 0x4e | 1 | name length | |
| 0x50 | 1 | format length | e.g. the file system name |
| 0x51 | 1 | comments length | |
| 0x54 | | name, format, comments | `/tmp` and `tmpfs` at 51 **[O]** |

On Linux the sizes and item count are all ones and the times are 0 **[O]**.
`kdx_proto::msg::file::VolumeInfo` re-encodes the reply byte for byte. The
client labels are Free Size, Used Size, Total Size, Item Count, Disk Type,
Format, Created, Modified, and Comments. It makes the name or comments field
read-only when its bit is clear (`FUN_080a0410`).

Server 1.620 for Linux writes `caps & 2 | 1` to both 0x44 and 0x48, where
`caps` is the constant 0x39 from `FUN_08096780`. So the name is always
editable and the comments never are **[S `FUN_0804ef00`]**. Both words are
`00000001` in frame 51 of `captures/linux-catalog-volumes.pcapng`, which
also ends with the format, `tmpfs` **[O]**.

A *name list* below has the same layout as a file path: a u16 count, then
for each name a u16 0, a u8 length, and the name. When a request carries
paths back to back, each one is self-delimiting, and the root is written as
a count of 0 (`00 00`) rather than left empty (`captures/linux-files.pcapng`
frames 206 and 241) **[O]**. `kdx_proto::msg::FilePath::decode_delimited`
reads such a path. Each request also passes the folder access check
(`FUN_08057e5c`) with an access kind; a failure gets 1 Status 0x10014.
`kdx_proto::msg::file` re-encodes each request below byte for byte (test
`linux_files`).

**402 Create Folder**
Initiator: Client. Privilege: Upload (11). **[S `FUN_0804e0f8`; O
`captures/linux-files.pcapng` frames 26-29]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 0x13 | 0 | |
| 0x13 | 1 | folder access length | the tag, such as `DB`; needs Modify Folder Access (28) |
| 0x14 | 2 | path length | the parent folder |
| 0x16 | 1 | name length | empty: the path's last name is the new folder |
| 0x17 | 1 | owner login length | |
| 0x18 | | name, folder access, owner login, path | |

With a tag the folder is named `name [tag]`, or `name [tag,owner]` with an
owner login. A name ending in a reserved suffix such as `lnk` gets 0x30076.
Success is 1 Status 0x192 (frame 29). The client's New Folder dialog has
Name, a Folder Access popup, and Owner Login. Frame 26 is 51 bytes: the
name `lab-folder` and the path `Uploads [UL]`, with no tag or owner and no
padding **[O]** (`CreateFolder`).

**403 Create Alias**
Initiator: Client. Privilege: Create Aliases (15). **[S `FUN_0804e4ec`; O
`captures/linux-files.pcapng` frames 95-98]**

The request is the destination folder path, the source folder path, then a
name list of items in the source folder (frame 95: `Uploads [UL]`,
`Uploads [UL]`, `upload2.txt`). Success is 1 Status 0x193 (frame 98). It
has the 407 layout (`MoveItems`).

**405 Move To Trash**
Initiator: Client. Privilege: Delete (18). **[S `FUN_0804e980`; O
`captures/linux-files.pcapng` frames 241-244]**

The request is a folder path, then a name list (frame 241: the root, then
`lab-folder`). Each item gets a history record. Success is 1 Status 0x195
(frame 244). `MoveToTrash` holds the request.

**407 Move File/Folder**
Initiator: Client. Privilege: Move (16). **[S `FUN_0804ebd8`; O
`captures/linux-files.pcapng` frames 76, 80, 127, 131, 206, 210]**

The request is the destination folder path, the source folder path, then a
name list. Frame 76 moves `upload.txt` from `Uploads [UL]/lab-folder` to
`Uploads [UL]`, 127 moves `upload2.txt` back into `lab-folder`, and 206
moves `lab-folder` to the root. Success is 1 Status 0x197 (frames 80, 131,
210). `MoveItems` holds the request.

**411 Set File/Folder Info**
Initiator: Client. **[S `FUN_0804f684`; O `captures/linux-files.pcapng`
frame 89]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 0x28 | 0 | |
| 0x28 | 4 | flags | bit 0 rename: needs Rename (17); bits 0x86 set attributes: need Set File/Folder Attributes (20) |
| 0x2c | 2 | | 0 |
| 0x2e | 2 | path length | 0x1e in frame 89 |
| 0x30 | 1 | | 0 |
| 0x31 | 1 | new name length | |
| 0x32 | 2 | other lengths | 0 from the client |
| 0x34 | | path, new name | frame 89 renames `Uploads [UL]/upload.txt` to `upload2.txt` |

Without Modify Folder Access (28), a new name must keep the folder's
` [tag]` suffix. There is no reply, success or not (frame 89: no Status for
its txid). Frame 89 is 93 bytes, unpadded **[O]** (`SetFileInfo`).

**420 Launch File**
Initiator: Client. Privilege: Launch Programs (82). **[S `FUN_0804fa90`,
client `FUN_0807455c`; O `captures/linux-catalog-volumes.pcapng` frames
31-32]**

The request is 8 zero bytes, then the file path (at least 9 bytes in all).
Success is 1 Status 0x1a4, and the server runs the file. Frames 31 and 32 of
`captures/linux-catalog-volumes.pcapng` launch `lab-files/hello.sh` this way
**[O]**.

**421 Empty Trash**
Initiator: Client. Privilege: Empty Trash (13). Handled inline; success is
1 Status 0x1a5 **[S]**.

**423 Set Volume Info**
Initiator: Client. Privilege: See All Volumes (22). **[S `FUN_0804f8d0`,
client `FUN_08074420`; O `captures/linux-catalog-volumes.pcapng` frames
53-54]**

The request has the 422 Volume Info layout, but its strings differ. At 0x54
come the volume's current name (length at 0x4e), the new name (length at
0x4f), the format (length at 0x50, ignored), and the comments (length at
0x51). Flags at 0x44: bit 0 renames the volume, bit 1 sets the comments. The
client sets a bit only for a field the user edited, and sends nothing if
neither was **[S client `FUN_08074420`]**. On success there is no reply;
the server writes history record 0x1a7 **[S]**.

In Server 1.620 for Linux both setters are stubs **[S]**. Renaming
(`FUN_08096530`) throws 0x10005 (module 0x46, line 0x87f) before the history
record is written, and the client gets a 1 Status error with it. In frame 53
of `captures/linux-catalog-volumes.pcapng`, 423 has flags `00000001`, the
name `/tmp`, and the new name `/tmp-renamed`; frame 54 is the error, at
module 0x46, line 2175 **[O]**. Setting the comments (`FUN_08094e6c`) does
nothing. Frame 53 is 100 bytes, unpadded, with every field but the flags
and string lengths 0 **[O]**. `kdx_proto::msg::file::SetVolumeInfo`
re-encodes it byte for byte (test `linux_catalog_volumes`).

**404 Delete File/Folder**, **406 Rename File/Folder**, **408 Copy
File/Folder**
Server 1.620 has no handler for these. 405 Move To Trash, 407 Move
File/Folder, and 411 Set File/Folder Info cover the same ground **[S
`FUN_0804fdac`]**.

**450 Generate Catalog** → **451 Generate Catalog (S)**
Initiator: Client. Privilege: Modify Server Settings (61). **[S
`FUN_0805a728`, `FUN_0805a878`; O `captures/linux-catalog-volumes.pcapng`
frames 23-26]**

The request is empty or a u32 of flags; bit 0 cancels a running catalog
(only its starter may). A second start while one runs gets 1 Status
0x30075. The server writes history record 0x1c2 and builds an index of all
files. 450 itself gets no reply. The server sends 451 progress to the
starter at most every 2000 ms, and a final one when done:

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | flags: 0x00000001 = done |
| 0x04 | 4 | items indexed so far |

In `captures/linux-catalog-volumes.pcapng`, the empty 450 at frame 23 gets
`00000000 00000002` (24), then `00000001 0000000b` (26) **[O]**.

**452 File Search** → **453 File Search (R)**
Initiator: Client. **[S `FUN_0805abc0`; O
`captures/linux-catalog-volumes.pcapng` frames 28-29]**

The request is at least 0x58 bytes. Byte 0x54 is the query length (at
least 3; the client checks this too), and the query starts at 0x58. The
client sends the rest as zeros and the server ignores it. Without a
catalog the reply is 1 Status 0x640001, as in `captures/linux-files.pcapng`
**[O]**. Otherwise the server writes history
record 0x1c4 and replies with a u32 count and at most 100 entries, each
padded to 4:

| Off | Size | Field |
|---|---|---|
| 0x00 | 8 | size |
| 0x08 | 8 | time |
| 0x10 | 8 | time |
| 0x18 | 4 | path length |
| 0x20 | 4 | attributes, as in 401 |
| 0x2e | 2 | as in 401 |
| 0x34 | 1 | 0x10 for a file **[O]** |
| 0x36 | 1 | name length |
| 0x38 | | name, then the item's full file path (root first) |

Items the user can't see through folder access are left out. No hit gives
an empty 453.

Frame 29 of `captures/linux-catalog-volumes.pcapng` has one hit for `a.txt`:
size 6, attributes 1, name length 0, and a path length of 0x16 for the
path `lab-files`, `a.txt` **[O]**. So the name can be empty, with the item's
name only in its path. `kdx_proto::msg::file` re-encodes 420 and 450 to 453
byte for byte (test `linux_catalog_volumes`).
