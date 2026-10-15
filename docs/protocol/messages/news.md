# News messages

Message types 600 to 614: newsgroups and their messages.

## <a id="m600"></a>600 Create Newsgroup
Initiator: Client. Privilege: Create/Modify Newsgroups (73). **[S
`FUN_0805c8f8`, client `FUN_08089034`; O
`captures/linux-accounts-news.pcapng` frames 58-63]**

The request is a [603 Newsgroup Info](#m603) record. The server ignores the ID
at 0x00 and picks one. The name must be 1..0x3f bytes and the description at
most 0x7f. Success is [1 Status](status.md#m1) 0x258, and a history record. The
client's editor has Name, Description, Archive Size, and the class access list;
it sends 600 or [604 Set Newsgroup](#m604), then
[605 Get Newsgroup List](#m605).

Frame 58 creates `test newsgroup` (description `desc`, archive size 0x1a4,
read and post classes `80000000`, so Administrator only). It is 82 bytes,
unpadded, with the ID, time, message count, and the bitmap at 0x20 all 0.
The server gives it ID 2 (frame 63) **[O]**. `NewsgroupInfo` re-encodes 600,
603, and 604 byte for byte (test `linux_accounts_news`).

When a group's stored messages reach 1.5 times the archive size, the server
moves the oldest ones into a file under `News Archives` named after the
group **[S `FUN_0805ed88`]**.

## <a id="m601"></a>601 Delete Newsgroup
Initiator: Client. Privilege: Create/Modify Newsgroups (73). **[S
`FUN_0805caf4`; O `captures/linux-accounts-news.pcapng` frames 81-84]**

The request is a u32 group ID, 4 bytes, as `NewsgroupId` encodes it (frame 81)
**[O]**. An unknown group gets 0x1000c. Success is [1 Status](status.md#m1)
0x259 (frame 84). [602 Get Newsgroup](#m602) also writes history record 0x25a.

## <a id="m602"></a>602 Get Newsgroup
Initiator: Client. Privilege: Create/Modify Newsgroups (73). **[S
`FUN_0805cc24`; O 76 bytes; O `captures/linux-accounts-news.pcapng` frames
68-69]**

The request is a u32 group ID, 4 bytes and unpadded, which `NewsgroupId` encodes
(`captures/linux-accounts-news.pcapng` frame 68,
`captures/linux-first-connect.pcapng` frame 130) **[O]**. An unknown group gets
[1 Status](status.md#m1) 0x1000c.

## <a id="m603"></a>603 Newsgroup Info
Initiator: Server. Reply to [602 Get Newsgroup](#m602).

The reply is not padded: frame 69 of `captures/linux-accounts-news.pcapng` is
82 bytes, for the 14-byte name `test newsgroup` and the description `desc`
**[O]**. The record is:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | group ID | |
| 0x04 | 0xc | | 0 |
| 0x10 | 8 | time of the last post | [611 Post News Message](#m611) sets it |
| 0x18 | 4 | read classes | a class bitmap, class 0 in the top bit; `ffffffff` |
| 0x1c | 4 | post classes | the same; `ffffffff` |
| 0x20 | 4 | | a third class bitmap; `01 00 00 00`; the client sends 0 **[S]** |
| 0x24 | 0xc | | 0 |
| 0x30 | 2 | message count | 1 |
| 0x32 | 2 | archive size | KiB, clamped to 4..1024; 0x32 (50 KiB) by default **[S `FUN_0805c8f8`]** |
| 0x34 | 8 | | 0 |
| 0x3c | 1 | name length | at most 0x3f |
| 0x3d | 1 | description length | at most 0x7f; 0 in the lab |
| 0x3e | 2 | | 0 |
| 0x40 | | name, description | `General News` |

The read and post class bitmaps are the ones [607 Get News Messages](#m607) and
611 Post News Message check **[S]**.

## <a id="m604"></a>604 Set Newsgroup
Initiator: Client. Privilege: Create/Modify Newsgroups (73). **[S
`FUN_0805ce60`; O `captures/linux-accounts-news.pcapng` frames 71-74]**

The request is a [603 Newsgroup Info](#m603) record with the group ID at 0x00.
Success is [1 Status](status.md#m1) 0x25c (frame 74). Frame 71 renames group 2
to `Lab Group`, with the description `edited description` and archive size 0x32.
It is 91 bytes, unpadded **[O]**.

## <a id="m605"></a>605 Get Newsgroup List
Initiator: Client. Privilege: View News (70). **[S `FUN_0805d038`; O]**

## <a id="m606"></a>606 Newsgroup List
Initiator: Server. Reply to [605 Get Newsgroup List](#m605).

A u32 count, then one entry per group:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | time | |
| 0x08 | 4 | group ID | |
| 0x0c | 4 | | |
| 0x10 | 2 | message count | 1 **[I]** |
| 0x12 | 4 | | |
| 0x16 | 1 | name length | |
| 0x17 | 1 | | 0, probably the description length **[I]** |
| 0x18 | | name | `General News` |

## <a id="m607"></a>607 Get News Messages
Initiator: Client. **[S `FUN_0805d2e8`; O]**

The request is a u32 group ID, 4 bytes, as `NewsgroupId` encodes it
(`captures/kdx-news.pcapng` frame 76, `captures/linux-accounts-news.pcapng`
frames 65, 78, and 107) **[O]**.

## <a id="m608"></a>608 News Messages
Initiator: Server. Reply to [607 Get News Messages](#m607).

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | group ID | |
| 0x04 | 4 | permissions | 1 post, 2 delete any message, 4 read; 7 for admin, 4 for guest |
| 0x08 | 4 | | |
| 0x0c | 4 | count | |
| 0x10 | | news message records | same as [610 News Message](#m610) |

The server computes the permissions from the user's privileges and the
group's class bitmaps **[S `FUN_0805d2e8`; O `captures/kdx-news.pcapng`
frames 77 and 204]**. Without read access but with post access, the reply is
just the 16-byte header with permissions 1 and count 0 **[S]**.

## <a id="m609"></a>609 Get News Message
Initiator: Client. **[S `FUN_0805d54c`; O]**

Request: {u32 group ID, u32 message ID}, 8 bytes, as `NewsMessageId` encodes
it (`captures/kdx-news.pcapng` frame 137, `captures/linux-first-connect.pcapng`
frames 99 and 112) **[O]**. 609 Get News Message does not reset the idle
timer.

## <a id="m610"></a>610 News Message
Initiator: Server. Reply to [609 Get News Message](#m609).

One news message record, padded to 4:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | time | |
| 0x08 | 4 | group ID | |
| 0x0c | 4 | message ID | |
| 0x10 | 8 | | 0 |
| 0x18 | 2 | body length | 0x82c |
| 0x1a | 2 | | 0 |
| 0x1c | 1 | poster name length | the poster's nick; `KDX` |
| 0x1d | 1 | poster login length | `KDX`; `admin` in `captures/kdx-news.pcapng` frame 138 **[S `FUN_0805d758`; O]** |
| 0x1e | 1 | subject length | `Welcome` |
| 0x1f | 1 | | 0 |
| 0x20 | | name, login, subject, body | |

## <a id="m611"></a>611 Post News Message
Initiator: Client. Privilege: Post News (71), and the user's class must be in
the group's post classes unless the user has privilege 0. **[S
`FUN_0805d758`; O `captures/kdx-news.pcapng`]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | group ID | |
| 0x04 | 8 | | ignored by the server **[S]** |
| 0x0c | 2 | body length | the server keeps at most 0x4000 bytes |
| 0x0e | 1 | subject length | the server keeps at most 0x80 bytes |
| 0x0f | 1 | | ignored by the server **[S]** |
| 0x10 | | subject, body | lines end in CR |

Client 1.620 beta 5 for Linux sends the request unpadded: 39, 18 and 23
bytes in `captures/linux-first-connect.pcapng` frames 94, 107 and 115, and
66 bytes in `captures/linux-text-encoding.pcapng` frame 319 **[O]**.
`kdx-proto` encodes it the same way. The `kdx` CLI's earlier posts in
`captures/kdx-news.pcapng` frames 32 and 112 were padded to 4 with zeros, and
the server accepted them with [613 News Message Posted](#m613) at frames 35 and
115 **[O]**.

There is no reply on success (frames 356-361). The server stores a
[610 News Message](#m610) record, with the user's nick and login, under the
group's next message ID, then sends 613 News Message Posted to every
user whose class may read the group, the poster included. Errors **[O]**:
0x1000c for an unknown group (frame 160) and 0x10014 for a guest (frame 182).

## <a id="m612"></a>612 Delete News Message
Initiator: Client. **[S `FUN_0805db44`; O]**

Request: {u32 group ID, u32 message ID}, 8 bytes, as `NewsMessageId` encodes it
(`captures/linux-first-connect.pcapng` frame 102, `captures/kdx-news.pcapng`
frames 225 and 406) **[O]**. The user's class must be in the group's read
classes unless the user has privilege 0. Without Delete News Messages (72), the
user may delete only the group's last message, and only if they posted it (the
login at 0x1d matches); otherwise [1 Status](status.md#m1) 0x10014 (`kdx-news`
frame 226, a guest).

There is no reply on success. The server broadcasts
[614 News Message Deleted](#m614) to every user (frames 409-411). An unknown
group or message gets no reply and no 614 (frame 435).

## <a id="m613"></a>613 News Message Posted
Initiator: Server. **[S; O]** Sent after a successful
[611 Post News Message](#m611).

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | group ID |
| 0x04 | 4 | message ID |
| 0x08 | 4 | messages in the group afterwards |

## <a id="m614"></a>614 News Message Deleted
Initiator: Server. **[S; O]** Broadcast to every user after a
[612 Delete News Message](#m612). Same layout as
[613 News Message Posted](#m613).
