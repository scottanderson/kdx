# Monitoring and administration messages

The connection monitor, disconnects, broadcasts, server history, settings, and upgrades.

**140 Open Connection Monitor** / **141 Close Connection Monitor**
Initiator: Client. **[S]**

These need Get User Info (bit 40) and turn a per-user flag on or off. There
is no reply.

**142 Get Connection Monitor Info** → **143 Connection Monitor Info**
Initiator: Client. **[S `FUN_0804b9b0`; O]**

The request is empty. It needs Get User Info (bit 40); without a 140 Open
Connection Monitor first, the server replies with 1 Status 0x10003
(protocol breach) **[O]**. The reply is one record per connection, each
followed by one record per file transfer of that user, with no count in
front, each padded to 4 **[S `FUN_0804b9b0`]**:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | user ID | |
| 0x04 | 4 | transfer ID | 0 in a connection record; the transfer's ID in a transfer record **[S]** |
| 0x08 | 8 | login time | the transfer's start time in a transfer record **[S]** |
| 0x10 | 8 | time | 0 in a transfer record **[S]** |
| 0x18 | 8 | current server time | **[S]** |
| 0x20 | 8 | bytes done | transfer record only: bytes moved minus bytes still queued to send **[S]** |
| 0x28 | 8 | total bytes | transfer record only **[S]** |
| 0x30 | 4 | rate | transfer record only: bytes per second since the start, not counting a resumed offset **[S]**; 0x51a1 under a 0x5000 limit **[O]** |
| 0x34 | 4 | seconds left | transfer record only: (total − done) / rate **[S]** |
| 0x38 | 4 | | transfer record only; a per-transfer value **[S]**; 0x5000, the server's speed limit, in every record **[O]** |
| 0x3c | 4 | state | transfer record only; 0x90 while data is queued and the state is 0 **[S]**; 0x19c (412) for a download **[O]** |
| 0x40 | 2 | 0 | |
| 0x42 | 1 | account class index | **[S]** |
| 0x60 | 1 | nick length | |
| 0x61 | 1 | login length | |
| 0x62 | 1 | account class name length | `?` if the class is gone **[S]** |
| 0x63 | 1 | address length | the connection's peer address; the transfer connection's in a transfer record **[S]** |
| 0x66 | 2 | path length | transfer record only: the file path, as in the file messages **[S, O]** |
| 0x68 | | nick, login, account class name, address, then the path | `probe`, `admin`, `Administrator`, `127.0.0.1:50984` |

The padding after the strings is uninitialised **[O]**; `kdx-proto` keeps
it on a round trip. Fields marked transfer record only are 0 in a connection
record.

In `captures/linux-catalog-volumes.pcapng`, the replies from frame 104 to 161
hold three records: the operator's connection, the `kdx-get` connection
(user 2), and its download of `speed.bin` (transfer 3) **[O]**. In frame
104 the transfer has done 0x14000 of 0x200000 bytes at 0x6aaa per second,
with 0x49 seconds left; in frame 161, 0x1f4000 at 0x51a1, with 2 left.

**150 Disconnect User**
Initiator: Client. Privilege: Disconnect Users (41), and Ban Users (43) to
ban. **[S `FUN_0804c42c`; O `captures/linux-chat-admin.pcapng` frames 159,
161]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | user ID | |
| 0x08 | 4 | ban seconds | ≤ 0x80000; 0 = no ban |
| 0x11 | 1 | options | bit 0: disconnect without a notice |
| 0x14 | 1 | message length | |
| 0x18 | | message | |

A target with Cannot Be Disconnected is refused unless the server's address
check passes. The target receives **151 Disconnect Message**:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | sender user ID | |
| 0x04 | 8 | 0 | |
| 0x0c | 2 | flags | 2 = banned, 4 = server shutting down or upgrading |
| 0x0e | 2 | | |
| 0x10 | 2 | | |
| 0x12 | 1 | message length | |
| 0x13 | 1 | name length | |
| 0x14 | | message, sender name | |

In `captures/linux-chat-admin.pcapng` the GUI (user 2) disconnected itself
with the message `test` and no ban: the 150 at frame 159 is 28 bytes, and
the 151 at frame 161 has user ID 2, flags 0, and the name `kdx-lab-linux`
**[O]**. Since the sender was also the target, the capture doesn't tell the
two apart at 0x00. `kdx_proto::msg::admin::DisconnectUser` and
`DisconnectMessage` re-encode both byte for byte.

**152 Get User Info** → **153 User Info**
Initiator: Client. Privilege: Get User Info (40). **[S `FUN_0804c8a8`; O]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | user ID |

If the user doesn't exist, the server replies with 1 Status 0x1000c **[O]**.
`kdx_proto::msg::user_info::GetUserInfo` encodes the request.

Reply (120 bytes for the prober itself), padded to 4:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | user ID | |
| 0x04 | 4 | | |
| 0x08 | 8 | login time | |
| 0x10 | 8 | time | |
| 0x18 | 8 | time | |
| 0x20 | 0x1b | | |
| 0x3b | 1 | address length | |
| 0x3d | 1 | nick length | |
| 0x3f | 1 | name length | `Administrator`; the class or the account name **[I]** |
| 0x40 | 1 | login length | |
| 0x41 | 1 | name length | `Administrator`; the class or the account name **[I]** |
| 0x44 | | address, nick, name, login, name | |

**154 Send Broadcast Message**
Initiator: Client. Privilege: Broadcast (33). **[S `FUN_0804d1a8`; O
`captures/linux-chat-admin.pcapng` frames 129, 130, 132]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 2 | |
| 0x02 | 2 | (copied through) |
| 0x04 | | text, ≤ 0x800 |

Everyone receives **155 Rcv Broadcast Message**: {u32 sender ID, 8 zero
bytes, u16 0, u16 (copied from the request), text}. The GUI (user 2) sent
`00 00 00 00` and `test broadcast` (frame 129), and both the peer (130) and
the sender (132) got user ID 2, 12 zero bytes, and the text **[O]**.
`kdx_proto::msg::admin::SendBroadcast` and `RcvBroadcast` re-encode them.

**170 Get Server Settings** → **171 Server Settings Info**
Initiator: Client. Privilege: Modify Server Settings (61). **[S
`FUN_0805f598`; O 216 and 764 bytes]**

The reply is the server's settings file. It starts with the magic `%KDXSSTG`
and a version, followed by a table of section offsets and the sections.
**172 Set Server Settings** sends the same blob back, and the server answers
with 173 Set Server Settings Done **[O]**.

What the captures show (`captures/linux-server-settings.pcapng` frames 36,
40, 56, 221, 225) **[O]**:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | magic | `%KDXSSTG` |
| 0x08 | 2 | version | 1 |
| 0x0c | 0x58 | section table | 11 entries of u32 offset, u32 length **[I]** |
| 0x0c | 8 | main section | entry 0: 0x70, 0x68 |
| 0x24 | 8 | greeting section | entry 3: 0xd8, 0x17; length 0 when no greeting is set |
| 0x34 | 8 | icon section | entry 5: 0xf0, 0x20c; 0xd8, 0 when none is set; 0xf0, 0 in a 172 without it |
| 0x44 | 8 | | entry 7: offset the end of the blob and length 0 in a 171; 0, 0 in a 172 |
| 0x64 | 12 | | `01 01 01 01 00 01 00 02 00 00 00 00` in a 171; 0x6b is 0 in a 172, and 0x69 too in a 172 without the icon |
| 0x93 | 1 | flags | bit 1: greeting enabled |
| 0xa8 | 4 | total outgoing speed limit | bytes per second; 0x5000, or 0 when off |
| 0xc2 | 1 | | 0x0a, the length of both names seen **[I]** |
| 0xcc | | server name | NUL-terminated, at the end of the main section |

Entries 1 and 2 are (0xd8, 0) and entries 4, 6, and 8 to 10 are all zero in
every capture. Each section starts on a multiple of 4. The greeting section
has the layout of 107 Server Greeting, and the icon section holds the bytes
of 106 Server Icon. A 172 may give the icon section length 0, and the server
keeps its icon: frames 56 and 225 sent none, and the 171 at frame 221 still
holds it.

`kdx_proto::msg::server_info::ServerSettings` decodes the table and reads the
named fields from the main section, keeping the rest of the bytes as sent.
It re-encodes all six blobs of `captures/linux-server-settings.pcapng` and
the 171 at frame 116 of `captures/probe.pcapng` byte for byte.

**174 Shutdown Server**
Initiator: Client. Privilege: Shutdown Server (64). **[S `FUN_08061598`]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 1 | 0 | |
| 0x01 | 1 | mode | low 3 bits nonzero, else 0x10002 |
| 0x02 | 2 | 0 | |
| 0x04 | | message | the rest of the body, no length prefix; ≤ 0xff |

The server broadcasts 151 Disconnect Message with flag 4, writes history
record 0xae, then exits after 5000 ms (`FUN_0806155c`). Mode 1 exits the
program only **[O `captures/linux-shutdown.pcapng` frames 47, 48]**. Before
exiting, mode bit 2 runs `/sbin/shutdown -h now` and bit 1 `/sbin/shutdown
-r now` (`FUN_080916e4`) **[S]**. `kdx_proto::msg::admin::ShutdownServer`
re-encodes frame 47.

**175 Upgrade Server Software**
Initiator: Client. Privileges: Shutdown Server (64) and Launch Programs
(82). **[S `FUN_0806172c`]**

The request has no fields; the handler reads no body **[S]**. The dialog's
Restart KDX option doesn't reach the server: the body is empty with it
unticked and ticked **[O `captures/linux-shutdown.pcapng` frame 19,
`captures/linux-upgrade.pcapng` frame 19]**.

Without `KDXServer.lexe` in the server's folder the reply is 1 Status
0x1000c. The server then opens `KDXServer.lexe` in the folder it resolves
for `Bases` (`FUN_08092db8`) and `KDXUpgradeHelper.lexe`; a missing file
gets 0x30067 **[S]**. Without the new program in `Bases` the first open
fails its `stat`: 0x30067, module 0x46, line 0x198 **[O
`captures/linux-shutdown.pcapng` frame 20]**.

On success there is no 1 Status. The server broadcasts 151 Disconnect
Message with flag 4 and "Upgrading server software, be back in 5 seconds!"
**[O `captures/linux-upgrade.pcapng` frame 20]**, launches the helper,
writes history record 0xaf, and exits after 0xd48 ms with mode 1 of 174
**[S]**. The helper moves `Bases/KDXServer.lexe` over the server's program
and starts it, whatever its version. In the lab, 175 downgraded Server 1.620
to 1.600, which was back about 5 s after the 151; the next login's 105
reports version 1600 **[O `captures/linux-upgrade.pcapng` frame 37]**.

**180 Server History Join** / **182 Server History Leave**
Initiator: Client. Privilege: View Server History (63). **[S `FUN_08062674`,
`FUN_080627f8`, `FUN_08061d80`]**

The reply (181 Server History Join (R)) is {u32 0, u32 count, buffered
records}, each padded to 4. While joined, the client receives **183 Server
History Added** for each new record, one record padded to 4. A record's
kind is one of 184 to 191 (Login, Disconnected, Abort File Download, …), or
the type of the logged request (e.g. 400 Get File List for a failed file list).

A record **[O `captures/linux-server-settings.pcapng` frames 33, 89, and the
183s from 38 to 228; `captures/linux-chat-admin.pcapng` frame 135]**:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | time | ms |
| 0x08 | 4 | kind | 0xbe (190 Server Started) first in each join reply |
| 0x0c | 4 | user ID | 0 for Server Started |
| 0x10 | 0x32 | | 0 in every capture |
| 0x42 | 2 | path length | |
| 0x46 | 1 | address length | |
| 0x47 | 1 | nick length | the nick before the event |
| 0x48 | 1 | login length | |
| 0x49 | 1 | detail length | the new nick for 100 Set Identity (frame 197), the chat name for 202 Join Chat (frame 208) |
| 0x4c | | address, nick, login, detail, then the path | |

The path is a file path, as in the file messages: `speed.bin` for 412
Download File and 188 Finished File Download (frames 108, 190). Saving the
settings logs 170 Get Server Settings and 172 Set Server Settings (frames
38, 43). The bytes at 0x44, 0x45, 0x4a, and 0x4b are 0 in every capture.
`kdx_proto::msg::admin::HistoryList` (181) and `HistoryRecord` (183)
re-encode every payload byte for byte, the unnamed bytes and padding
included.
