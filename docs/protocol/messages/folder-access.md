# Folder access messages

Tags, hex, and field notation: see [Conventions](../README.md#conventions).

Message types 1100 to 1107: per-folder access rules.

A folder access item is a named set of rules. A folder names its item with a
` [tag]` suffix, such as `Uploads [UL]`. Item `[Default]` applies when a folder
and its parents name no item. The u16 at offset 0x30 of a
[401 File List](files.md#m401) entry is the item's icon, not an ID **[S]**.

## <a id="m1100"></a>1100 Create Folder Access
Initiator: Client. Privilege: Modify Folder Access (28). **[S `FUN_080572f0`; O
`captures/linux-files.pcapng` frames 231 to 234]**

The request is a folder access record. An invalid name gets 0x1001e, an existing
one 0x1000d. Success is [1 Status](status.md#m1) 0x44c. Frame 231 creates `test`
with every other field 0, and frame 234 is the success.

## <a id="m1101"></a>1101 Delete Folder Access
Initiator: Client. Privilege: Modify Folder Access (28). **[S `FUN_08057460`; O
`captures/linux-accounts-folder-access.pcapng` frames 70 to 71]**

The request is the item name, with no length byte. An unknown name gets
0x1000c. There is no Status reply: the server sends
[1102 Folder Access Deleted](#m1102) to the requester and writes history record
0x44d. `kdx_proto::msg::admin::FolderAccessName` re-encodes frames 70 and 71
(`lab-item`, 8 bytes, unpadded) **[O]**.

## <a id="m1102"></a>1102 Folder Access Deleted
Initiator: Server. Sent on a successful
[1101 Delete Folder Access](#m1101).

The payload is the name. [1102 Folder Access Deleted](#m1102) carries the
server's own next txid, not the request's (`f2cfecc3` at frame 71) **[O]**.

## <a id="m1103"></a>1103 Get Folder Access
Initiator: Client. Privilege: View Folder Access (27). **[S `FUN_08057534`; O
`captures/linux-files.pcapng` frames 225 to 229]**

The request is the item name, with no length byte (`DB` at 225, `[Default]`
at 228). The request is unpadded: 2 bytes and 9 bytes in
`captures/linux-files.pcapng` frames 225 and 228 **[O]**.
`kdx_proto::msg::admin::FolderAccessName` re-encodes it (also
`captures/linux-accounts-folder-access.pcapng` frame 60) **[O]**.

## <a id="m1104"></a>1104 Folder Access Info
Initiator: Server. Reply to [1103 Get Folder Access](#m1103).

The stored folder access record. After a [1105 Set Folder Access](#m1105), that
is the 1105's record as sent, with its current name
(`captures/linux-folder-access-guest.pcapng` frames 30, 38) **[O]**.

## <a id="m1105"></a>1105 Set Folder Access
Initiator: Client. Privilege: Modify Folder Access (28). **[S `FUN_08057634`; O
`captures/linux-accounts-folder-access.pcapng` frames 63, 66]**

The request is a folder access record whose current name (0xb1) picks the item;
it is renamed if the name differs. Success is [1 Status](status.md#m1) 0x451.
The client sends the name and the current name even when they are equal: frame
63 is 200 bytes, with `lab-item` twice and length 8 at 0xad and 0xb1. It sets
byte 0x14 to 0x80, bit 0 (Can See Folder) of the Guest rule **[O]**. Ticking Can
See Folder for Guest alone and then for User alone sets 0x14 and then 0x10
(`captures/linux-folder-access-guest.pcapng` frames 30, 40) **[O]**.

## <a id="m1106"></a>1106 Get Folder Access List
Initiator: Client. Privilege: View Folder Access (27). **[S `FUN_080578d8`; O]**

The request is empty or a u32. If the u32's top byte has bit 0 set, a user
without the privilege gets no reply instead of [1 Status](status.md#m1) 0x10014.
Client 1.620 for Linux sends `00000001` when it opens the file window and an
empty request after [1100 Create Folder Access](#m1100)
(`captures/linux-files.pcapng` frames 23 and 232) **[O]**.
`kdx_proto::msg::admin::GetFolderAccessList` keeps the u32 and any bytes after
it, or all of a payload under 4 bytes. It re-encodes frames 23 and 232 and the
three empty requests of `captures/linux-accounts-folder-access.pcapng` (frames
50, 54, 64) **[O]**.

## <a id="m1107"></a>1107 Folder Access List
Initiator: Server. Reply to [1106 Get Folder Access List](#m1106).

A u32 count, then one entry per item, each padded to 4:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | flags | 1 built-in (`[Default]`), 2 Owner Login In Folder Name; 1, 2, 0 in the lab **[S]** |
| 0x04 | 4 | icon | 0 for `[Default]`, 0x2c Dropbox (`DB`), 0x2b Uploads (`UL`) **[S]** |
| 0x08 | 8 | | not written by the server |
| 0x10 | 1 | | |
| 0x11 | 1 | name length | |
| 0x12 | 1 | second string length | 0 in the lab |
| 0x13 | 1 | description length | |
| 0x14 | | name, second string, description | `[Default]`, empty, `If a folder and its parents…` |

**Folder access record** ([1105 Set Folder Access](#m1105) in the last two
captures), also read by the client's folder access editor **[S `FUN_080572f0`,
`FUN_08057634`; O `captures/linux-files.pcapng`
frames 226, 229, 231; O `captures/linux-accounts-folder-access.pcapng`; O
`captures/linux-folder-access-guest.pcapng`]**

[1100 Create Folder Access](#m1100), [1104 Folder Access Info](#m1104), and
[1105 Set Folder Access](#m1105) share this record. Its total length, padded to
4, must equal the payload length. `kdx_proto::msg::admin::FolderAccess`
re-encodes the three captured records byte for byte, padding included (test
`linux_files`).

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | | |
| 0x04 | 4 | flags | byte 7: bit 0 built-in, bit 1 Owner Login In Folder Name |
| 0x08 | 4 | icon | 0x28 Normal, 0x2b Uploads, 0x2c Dropbox, 0x34 Pictures, 0x35 Audio |
| 0x0c | 0x80 | class rules | 32 entries at 0x0c + 4 × class index: u16 permission bits, MSB = bit 0, then u16 0. User (class 1) is at 0x10 and Guest (class 2) at 0x14 **[O]**. The first, 0xfffe0000 (every permission) in both lab items, is probably Administrator's **[I]** |
| 0x8c | 4 | | 0xfffe0000 in `DB`, 0 in `[Default]`; probably the owner's rule **[I]** |
| 0x90 | 0x1d | 0 | |
| 0xad | 1 | name length | the tag, such as `DB` |
| 0xae | 1 | owner login length | |
| 0xaf | 1 | description length | |
| 0xb0 | 1 | | |
| 0xb1 | 1 | current name length | 1105 only: the item to change |
| 0xb2 | 6 | | |
| 0xb8 | | strings in the order of their lengths | |

Permission bits (client menu at file offset 0xf6448): 0 Can See Folder, 1 See
Folder Content, 2 Read/Download Files, 3 Create/Upload Files, 4 Create/Upload
Folders, 5 Move In, 6 Move Out, 7 Alias In, 8 Alias Out, 9 Duplicate In, 10
Duplicate Out, 11 Delete Items, 12 Rename, 13 Set Item Attributes, 14 Modify
File Contents. In the lab, `DB` grants 0x9d40 (bits 0, 3, 4, 5, 7, 9: a drop
box) and `[Default]` grants 0xe0a0 (bits 0, 1, 2, 8, 10) **[O]**, for classes
1..31 (frames 226, 229).
The client editor shows Name, Description, Owner Login, Owner Login In Folder
Name, the Access Privileges table, and Icon.
