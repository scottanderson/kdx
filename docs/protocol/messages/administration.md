# Administration messages

Tags, hex, and field notation: see [Conventions](../README.md#conventions).

Message types 150 to 183: disconnects, user info, broadcasts, server settings,
shutdown, upgrades, and server history; and 184 to 191, the kinds of history
record.

## <a id="m150"></a>150 Disconnect User
Initiator: Client. Privilege: Disconnect Users (41), and Ban Users (43) to ban.
**[S `FUN_0804c42c`; O `captures/linux-chat-admin.pcapng` frames 159, 161]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | user ID | |
| 0x08 | 4 | ban seconds | ≤ 0x80000; 0 = no ban |
| 0x11 | 1 | options | bit 0: disconnect without a notice |
| 0x14 | 1 | message length | |
| 0x18 | | message | |

A target with Cannot Be Disconnected is refused unless the server's address
check passes. The target receives [151 Disconnect Message](#m151).

In `captures/linux-chat-admin.pcapng` the GUI (user 2) disconnected itself
with the message `test` and no ban: the [150 Disconnect User](#m150) at frame
159 is 28 bytes **[O]**. `kdx_proto::msg::admin::DisconnectUser` re-encodes it
byte for byte.

## <a id="m151"></a>151 Disconnect Message
Initiator: Server.

Sent to the target of [150 Disconnect User](#m150), and broadcast by
[174 Shutdown Server](#m174) and [175 Upgrade Server Software](#m175):

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

In that capture the [151 Disconnect Message](#m151) at frame 161 has user ID 2,
flags 0, and the name `kdx-lab-linux` **[O]**. Since the sender was also the
target, the capture doesn't tell the two apart at 0x00.
`kdx_proto::msg::admin::DisconnectMessage` re-encodes it byte for byte.

## <a id="m152"></a>152 Get User Info
Initiator: Client. Privilege: Get User Info (40). **[S `FUN_0804c8a8`; O]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | user ID |

If the user doesn't exist, the server replies with [1 Status](status.md#m1)
0x1000c **[O]**. `kdx_proto::msg::user_info::GetUserInfo` encodes the request.

## <a id="m153"></a>153 User Info
Initiator: Server. **[S `FUN_0804c8a8`; O]**

Reply to [152 Get User Info](#m152) (120 bytes for the prober itself), padded
to 4:

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

## <a id="m154"></a>154 Send Broadcast Message
Initiator: Client. Privilege: Broadcast (33). **[S `FUN_0804d1a8`; O
`captures/linux-chat-admin.pcapng` frames 129, 130, 132]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 2 | |
| 0x02 | 2 | (copied through) |
| 0x04 | | text, ≤ 0x800 |

Everyone receives [155 Rcv Broadcast Message](#m155). The GUI (user 2) sent
`00 00 00 00` and `test broadcast` (frame 129) **[O]**.
`kdx_proto::msg::admin::SendBroadcast` re-encodes it.

## <a id="m155"></a>155 Rcv Broadcast Message
Initiator: Server.

Sent to everyone for a [154 Send Broadcast Message](#m154): {u32 sender ID,
8 zero bytes, u16 0, u16 (copied from the request), text}. For the 154 at
frame 129, both the peer (130) and the sender (132) got user ID 2, 12 zero
bytes, and the text **[O]**. `kdx_proto::msg::admin::RcvBroadcast`
re-encodes them.

## <a id="m170"></a>170 Get Server Settings
Initiator: Client. Privilege: Modify Server Settings (61). Observed sizes: 216
and 764 bytes. **[S `FUN_0805f598`; O]**

The reply is [171 Server Settings Info](#m171).

## <a id="m171"></a>171 Server Settings Info
Initiator: Server. Observed sizes: 216 and 764 bytes. **[S `FUN_0805f598`; O]**

Reply to [170 Get Server Settings](#m170). The reply is the server's
settings file. It starts with the magic `%KDXSSTG` and a version, followed by
a table of section offsets and the sections.

What the captures show (`captures/linux-server-settings.pcapng` frames 36,
40, 56, 221, 225) **[O]**:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | magic | `%KDXSSTG` |
| 0x08 | 2 | version | 1 |
| 0x0c | 0x58 | section table | 11 entries of u32 offset, u32 length **[I]** |
| 0x0c | 8 | main section | entry 0: 0x70, 0x68 |
| 0x24 | 8 | greeting section | entry 3: 0xd8, 0x17; length 0 when no greeting is set |
| 0x34 | 8 | icon section | entry 5: 0xf0, 0x20c; 0xd8, 0 when none is set; 0xf0, 0 in a [172 Set Server Settings](#m172) without it |
| 0x44 | 8 | | entry 7: offset the end of the blob and length 0 in a [171 Server Settings Info](#m171); 0, 0 in a 172 |
| 0x64 | 12 | | `01 01 01 01 00 01 00 02 00 00 00 00` in a 171; 0x6b is 0 in a 172, and 0x69 too in a 172 without the icon |
| 0x93 | 1 | flags | bit 1: greeting enabled |
| 0xa8 | 4 | total outgoing speed limit | bytes per second; 0x5000, or 0 when off |
| 0xc2 | 1 | | 0x0a, the length of both names seen **[I]** |
| 0xcc | | server name | NUL-terminated, at the end of the main section |

Entries 1 and 2 are (0xd8, 0) and entries 4, 6, and 8 to 10 are all zero in
every capture. Each section starts on a multiple of 4. The greeting section has
the layout of [107 Server Greeting](session.md#m107), and the icon section holds
the bytes of [106 Server Icon](session.md#m106). A 172 may give the icon section
length 0, and the server keeps its icon: frames 56 and 225 sent none, and the
171 at frame 221 still holds it.

`kdx_proto::msg::server_info::ServerSettings` decodes the table and reads the
named fields from the main section, keeping the rest of the bytes as sent.
It re-encodes all six blobs of `captures/linux-server-settings.pcapng` and
the 171 at frame 116 of `captures/probe.pcapng` byte for byte.

## <a id="m172"></a>172 Set Server Settings
Initiator: Client.
Handler: **[S `FUN_0805f638`]**.

[172 Set Server Settings](#m172) sends the same blob as
[171 Server Settings Info](#m171) back, and the server answers with
[173 Set Server Settings Done](#m173) **[O]**.

## <a id="m173"></a>173 Set Server Settings Done
Initiator: Server.

Reply to [172 Set Server Settings](#m172) **[O]**.

The reply is empty **[O `captures/linux-server-settings.pcapng` frames 41,
57]**.

## <a id="m174"></a>174 Shutdown Server
Initiator: Client. Privilege: Shutdown Server (64). **[S `FUN_08061598`]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 1 | 0 | |
| 0x01 | 1 | mode | low 3 bits nonzero, else 0x10002 |
| 0x02 | 2 | 0 | |
| 0x04 | | message | the rest of the body, no length prefix; ≤ 0xff |

The server broadcasts [151 Disconnect Message](#m151) with flag 4, writes
history record 0xae, then exits after 5000 ms (`FUN_0806155c`). Mode 1 exits the
program only **[O `captures/linux-shutdown.pcapng` frames 47, 48]**. Before
exiting, mode bit 2 runs `/sbin/shutdown -h now` and bit 1
`/sbin/shutdown -r now` (`FUN_080916e4`) **[S]**.
`kdx_proto::msg::admin::ShutdownServer` re-encodes frame 47.

## <a id="m175"></a>175 Upgrade Server Software
Initiator: Client. Privileges: Shutdown Server (64) and Launch Programs
(82). **[S `FUN_0806172c`]**

The request has no fields; the handler reads no body **[S]**. The dialog's
Restart KDX option doesn't reach the server: the body is empty with it unticked
and ticked **[O `captures/linux-shutdown.pcapng` frame 19; O
`captures/linux-upgrade.pcapng` frame 19]**.

Without `KDXServer.lexe` in the server's folder the reply is
[1 Status](status.md#m1) 0x1000c. The server then opens `KDXServer.lexe` in the
folder it resolves for `Bases` (`FUN_08092db8`) and `KDXUpgradeHelper.lexe`; a
missing file gets 0x30067 **[S]**. Without the new program in `Bases` the first
open fails its `stat`: 0x30067, module 0x46, line 0x198 **[O
`captures/linux-shutdown.pcapng` frame 20]**.

On success there is no 1 Status. The server broadcasts
[151 Disconnect Message](#m151) with flag 4 and "Upgrading server software, be
back in 5 seconds!" **[O `captures/linux-upgrade.pcapng` frame 20]**, launches
the helper, writes history record 0xaf, and exits after 0xd48 ms with mode 1 of
[174 Shutdown Server](#m174) **[S]**. The helper moves `Bases/KDXServer.lexe`
over the server's program and starts it, whatever its version. In the lab,
[175 Upgrade Server Software](#m175) downgraded Server 1.620 to 1.600, which was
back about 5 s after the 151; the next login's
[105 Server Info](session.md#m105) reports version 1600 **[O
`captures/linux-upgrade.pcapng` frame 37]**.

## <a id="m180"></a>180 Server History Join
Initiator: Client. Privilege: View Server History (63). **[S `FUN_08062674`,
`FUN_080627f8`, `FUN_08061d80`]**

The request is empty **[O `captures/linux-server-settings.pcapng` frames 32,
88; O `captures/linux-chat-admin.pcapng` frame 134]**.

The reply is [181 Server History Join (R)](#m181). While joined, the client
receives [183 Server History Added](#m183) for each new record. A record's kind
is one of 184 to 191 (Login, Disconnected, Abort File Download, …), or the type
of the logged request (e.g. [400 Get File List](files.md#m400) for a failed file
list).

A record, which the 183s in frames 38 to 228 repeat **[O
`captures/linux-server-settings.pcapng` frames 33, 89; O
`captures/linux-chat-admin.pcapng` frame 135]**:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | time | ms |
| 0x08 | 4 | kind | 0xbe ([190 Server Started](#m190)) first in each join reply |
| 0x0c | 4 | user ID | 0 for Server Started |
| 0x10 | 0x32 | | 0 in every capture |
| 0x42 | 2 | path length | |
| 0x46 | 1 | address length | |
| 0x47 | 1 | nick length | the nick before the event |
| 0x48 | 1 | login length | |
| 0x49 | 1 | detail length | the new nick for [100 Set Identity](session.md#m100) (frame 197), the chat name for [202 Join Chat](chat.md#m202) (frame 208) |
| 0x4c | | address, nick, login, detail, then the path | |

The path is a file path, as in the file messages: `speed.bin` for
[412 Download File](files.md#m412) and [188 Finished File Download](#m188)
(frames 108, 190). Saving the settings logs [170 Get Server Settings](#m170) and
[172 Set Server Settings](#m172) (frames 38, 43). The bytes at 0x44, 0x45, 0x4a,
and 0x4b are 0 in every capture. `kdx_proto::msg::admin::HistoryList` (181) and
`HistoryRecord` (183) re-encode every payload byte for byte, the unnamed bytes
and padding included.

## <a id="m181"></a>181 Server History Join (R)
Initiator: Server.

Reply to [180 Server History Join](#m180): {u32 0, u32 count, buffered
records}, each padded to 4.

## <a id="m182"></a>182 Server History Leave
Initiator: Client. Privilege: View Server History (63).
Handler: **[S `FUN_080627f8`]**.

The request is empty **[O `captures/linux-chat-admin.pcapng` frame 137]**.

Leaves the history joined with [180 Server History Join](#m180).

## <a id="m183"></a>183 Server History Added
Initiator: Server.
Handler: **[S `FUN_08061d80`]**.

Sent while joined with [180 Server History Join](#m180), for each new
record: one record padded to 4.

## <a id="m184"></a><a id="m185"></a><a id="m186"></a><a id="m187"></a><a id="m188"></a><a id="m189"></a><a id="m190"></a><a id="m191"></a>184 to 191 History record kinds
Initiator: Server. **[S]**

The kind of a history record in [181 Server History Join (R)](#m181) and
[183 Server History Added](#m183): [184 Login](#m184),
[185 Disconnected](#m185), [186 Abort File Download](#m186),
[187 Abort File Upload](#m187), [188 Finished File Download](#m188),
[189 Finished File Upload](#m189), [190 Server Started](#m190), and
[191 Login Failed](#m191). The record layout is under
[180 Server History Join](#m180).
