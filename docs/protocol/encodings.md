# Common encodings

For hex and field notation, see [Conventions](README.md#conventions).

This page describes the strings, times, paths, and user and chat records that
many messages share.

## Strings

A string has no terminator, and its length is not stored beside it. Instead, a
fixed-size part of the record holds the lengths, usually as u8. The strings
follow that part in a fixed order. Most records of variable size are then padded
to a multiple of 4. The padding bytes are uninitialized memory, so ignore them,
such as the `86 ab` at the end of the first entry in the [401 File
List](messages/files.md#m401) example. <!-- [O] --> `kdx-proto` still keeps
them, with any other bytes that it does not model, and writes them back, so a
decoded payload encodes to the same bytes. <!-- [O] kdx-dissect --audit over
every capture -->

Text is UTF-8 in the 1.6 builds, but the Linux server passes file names through
as raw bytes:

- Server 1.600 for Windows lists files with `FindFirstFileW`, and encodes each
  UTF-16 name as UTF-8 (`FUN_00428640`, `FUN_00428380`). It decodes an incoming
  path as UTF-8 before `CreateFileW` (`FUN_0042a740`, `FUN_004286e0`), so an
  invalid byte becomes U+FFFD and the server does not find the file. <!-- [S]
  -->
- Client 1.600 for Windows keeps text as UTF-8, with its decoder at
  0x00457190. <!-- [S] -->
- All three Windows builds (client, server, and tracker) use only the wide Win32
  calls, and `CF_UNICODETEXT` (0xd) for the clipboard. Their only
  `MultiByteToWideChar` calls convert font and device names (client
  `FUN_0045ecd0`, server `FUN_0041b460`), so no Windows code page touches
  protocol text. <!-- [S] -->
- Server 1.600 for Mac OS X converts `readdir` names to NFC UTF-8 with
  `ConvertFromUnicodeToText` (`FUN_000348cc`, encoding 0x08000100). The Carbon
  client 1.600 converts HFS+ names in the same way. <!-- [S] -->
- The Carbon server 1.600 (PEF) reads names with `FSGetCatalogInfoBulk` and
  makes files with `FSCreateFileUnicode`, so it gets UTF-16 names from the File
  Manager, even on Mac OS 9. `FUN_10032a80` normalizes a name to NFC
  (`CreateTextEncoding(0x100,3,0)`) and encodes it as UTF-8. If the Text
  Encoding Converter is older than 0x1020, it encodes the name without
  normalizing it. <!-- [S] --> The Mac builds use the system script encoding
  (`UpgradeScriptInfoToTextEncoding`, `FUN_10060990`) only for local keyboard
  and clipboard text. <!-- [S] -->
- Client 1.620 beta 5 for Linux reads keys with `Xutf8LookupString`, reads the
  clipboard as `UTF8_STRING`, and has no other codec. <!-- [S] --> It keeps
  local file names as raw bytes (`readdir64` in `FUN_0810e6c0`). <!-- [S] -->
- Server 1.620 for Linux copies `readdir64` names without change
  (`FUN_08091b6c`), so a name is the bytes on disk. <!-- [S] --> Names copied
  from a classic Mac, or uploaded by an older client, can be Mac OS Roman, such
  as `Erik’s` with byte 0xd5. <!-- [O] user report --> The Mach-O and Linux
  trackers also read their folders with raw `readdir`. <!-- [S] -->

Chat, nicks, and news follow from this. Each client makes text from UTF-16 or
UTF-8 input and has no other codec, so it sends UTF-8. <!-- [I] --> Servers
relay chat bytes without decoding them. [207 Send Chat
Text](messages/chat.md#m207) copies the text into [208 Receive Chat
Text](messages/chat.md#m208) after it cuts the message to 0x1000 bytes, which
can split a multi-byte character. <!-- [S] FUN_08058d74 -->

The Linux client and server confirm this. Client 1.620 beta 5 sends UTF-8 in
[207 Send Chat Text](messages/chat.md#m207), [300 Send Private
Message](messages/private-messages.md#m300), [611 Post News
Message](messages/news.md#m611), [100 Set Identity](messages/session.md#m100),
[402 Create Folder](messages/files.md#m402), and [415 Upload
File](messages/files.md#m415). Server 1.620 stores the bytes on disk and relays
them without change in [208 Receive Chat Text](messages/chat.md#m208), [301
Incoming Private Message](messages/private-messages.md#m301), [610 News
Message](messages/news.md#m610), and [102 User
Changed](messages/session.md#m102). <!-- [O
`captures/linux-text-encoding.pcapng` frames 90, 103, 118 to 121, 249 to 250,
319 to 325, 327 to 330] --> String lengths count bytes. A name matches only its
own bytes: with `caf\xc3\xa9.txt` and `caf\x8e.txt` in one folder, [401 File
List](messages/files.md#m401) lists both, and [412 Download
File](messages/files.md#m412) gets each one by its own spelling. <!-- [O frames
41, 52, 71] -->

The decoders never fail. Each invalid sequence becomes one U+FFFD:

- A stray continuation byte (0x80 to 0xbf), or a lead byte of 0xf8 or more, uses
  1 byte.
- A lead byte with bad continuations, or an overlong form, uses the whole length
  that it claims.
- A sequence cut off at the end uses the rest.

So `Erik\xd5s` shows as `Erik�`, and the `s` is lost. <!-- [S] Windows client
0x00457190, Windows server FUN_00428490, Linux client FUN_080e75d8 --> The Linux
client shows `caf\x8e \xd5 \xd1` as `caf� ��`: the 0xd5 lead byte uses the space
after it, so the client draws three boxes. <!-- [O
`captures/linux-text-encoding.pcapng` frames 195, 302] --> In the other
direction, a lone UTF-16 surrogate becomes U+FFFD (`FUN_00428640`). <!-- [S] -->

KDX changed to Unicode in 1.2. Each client and server archive in
`vendor/MANIFEST.md` holds Haxial's release notes, as `UPGRADE WARNINGS.html`
(clients) or `NEW UPGRADE WARNINGS.html` (servers). Their "KDX 1.2 -- UNICODE"
entry says three things: earlier versions supported only Latin text; 1.2 can
still connect to the previous version, but the text can be "somewhat mutilated";
and text made by earlier versions has the same problem. <!-- [S] --> The Windows
client's `WINDOWS XP REQUIRED.html` gives Unicode as the reason that it needs
Windows NT. <!-- [S] --> The notes in the tracker archives are older than that
entry. The repo has no builds before 1.2, so their charset is unknown. It was
probably Mac OS Roman on a Mac and a Windows code page on a PC. <!-- [I] -->

Nothing negotiates the charset. The handshake's sub-version is 0x10 for all 1.6
clients, and Server 1.620 accepts only 0x10; see [handshake.md](handshake.md).
Because 1.2 could still connect to 1.1, the change probably kept that
byte. <!-- [I] --> No build changes how it handles text for the peer's version.
So a client of 1.2 or later decodes the bytes of an older peer as UTF-8, and
shows U+FFFD as above. A client older than 1.2 shows UTF-8 as two or three Latin
characters for each letter. <!-- [I] --> Old text stored on a server, such as
file names or news, can still hold bytes from before 1.2.

No 1.6 build has a Mac OS Roman table. No capture has non-ASCII text from an
older peer yet.

## Times

A time is a u64 count of milliseconds since 1900-01-01 00:00 UTC. The server
makes it as (tv_sec + 2208988800) × 1000 + ms, <!-- [S `FUN_08090eb0`] --> so
0x3a36c4895ba is 2026-10-08. <!-- [O] -->

## Colors

A color is three bytes: red, green, then blue. <!-- [S client `FUN_0804d654`] -->

Struct `Color` (3 bytes):<a id="sColor"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 1 | `u8` | `r` |  |
| 0x01 | 1 | `u8` | `g` |  |
| 0x02 | 1 | `u8` | `b` |  |

## File paths

A path is a list of names. Each name is one folder level below the user's base
folder. <!-- [S `FUN_08069880`, `FUN_08096b4c`; O] -->

Struct `FilePath` (dynamic):<a id="sFilePath"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 2 | `u16` | `names_count` |  |
| 0x02 | `names_count` | [`PathName`](#sPathName)`[]` | `names` | 1 to 0x400 |

Struct `PathName` (dynamic):<a id="sPathName"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 2 | `u16` | `ignored` | 0 as sent. |
| 0x02 | 1 | `u8` | `name_len` |  |
| 0x03 | `name_len` | `str` | `name` | nonzero |

The root is an empty payload, not a count of 0. A path with names must be at
least 6 bytes; if not, the server gives 0x1000a. A [400 Get File
List](messages/files.md#m400) of `00 00` gets this error, so `kdx-dissect` flags
that request as invalid. <!-- [O `captures/probe.pcapng` frames 106, 107] -->
The server joins the names with `/` and checks each one with
`FUN_08094084`. <!-- [S] --> For example, `00 01 00 00 0c "Uploads [UL]"` is the
folder `Uploads [UL]` at the root. <!-- [O] -->

Struct `DelimitedFilePath` (dynamic):<a id="sDelimitedFilePath"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 2 | `u16` | `names_count` |  |
| 0x02 | `names_count` | [`PathName`](#sPathName)`[]` | `names` | 0 for the root, otherwise 1 to 0x400 |

A `DelimitedFilePath` is a `FilePath` with more fields after it in the same
payload, so it must show where it ends. Its root is `00 00`, not an empty
payload. `kdx_proto::msg::FilePath::decode_delimited` reads it.

## User record

<!-- [S `FUN_0804af04`, `FUN_0804b304`; O `captures/first-connect.pcapng`] -->

[100 Set Identity](messages/session.md#m100) (from the client, with user ID 0),
[101 User List](messages/session.md#m101), [102 User
Changed](messages/session.md#m102), and [103 User
Joined](messages/session.md#m103) use this record.

Struct `UserRecord` (dynamic):<a id="sUserRecord"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `u32` | `user_id` | The server assigns it. 0 in [100 Set Identity](messages/session.md#m100). |
| 0x04 | 1 | `u8` | `status` | The value that [110 Set Mini User Icon](messages/session.md#m110) sets. |
| 0x05 | 5 | `u8[5]` | `ignored_05` | 0 as sent. |
| 0x0a | 2 | `u16` | `client_port` | The port of the client's DCC listener, 10600 by default, or 0 if it has none. <!-- [S client `FUN_0804d654`; O] 10600 in the capture --> |
| 0x0c | 2 | `u16` | `icon_len` |  |
| 0x0e | 3 | [`Color`](#sColor) | `text_color` | The [color](#colors) of the nick and info text in the user list. The client chooses it. <!-- [S client `FUN_0804d654`, `FUN_080542b4`] --> `00 00 00` in the capture. <!-- [O] --> |
| 0x11 | 3 | [`Color`](#sColor) | `background_color` | The background of the user's row in the user list. The client chooses it. <!-- [S client `FUN_0804d654`, `FUN_080542b4`] --> `ff cc 00` in the capture. <!-- [O] --> |
| 0x14 | 3 | [`Color`](#sColor) | `account_color` | The strip at the left of the user's row. The server sets it to the color of the user's class, or of the account under Color Overrides Class. <!-- [S `FUN_0804af04`, `FUN_08054900`, client `FUN_080542b4`] --> 0 from the client, and `ff 00 00` for admin. <!-- [O] --> |
| 0x17 | 1 | `u8` | `flags` | Bit 2 comes from the client, and makes the right part of the row a gradient from `background_color` to `text_color`. <!-- [S client `FUN_080542b4`] --> The server sets bit 1; see below. |
| 0x18 | 1 | `u8` | `extra_len` |  |
| 0x19 | 1 | `u8` | `nick_len` |  |
| 0x1a | 1 | `u8` | `info_len` |  |
| 0x1b | 1 | `u8` | `ignored_1b` | 0 as sent. |
| 0x1c | `icon_len` | `u8[]` | `icon` | Less than 0x709 bytes. |
|  | `extra_len` | `u8[]` | `extra` | The server skips it. |
|  | `nick_len` | `str` | `nick` |  |
|  | `info_len` | `str` | `info` | At most 0x7f bytes. |
|  | pad4 | `u8[]` | `padding` | The server adds it in [101 User List](messages/session.md#m101). The client sends none. |

- The client sends [100 Set Identity](messages/session.md#m100) without padding:
  565 bytes in the capture. The server pads each record of [101 User
  List](messages/session.md#m101) with zeros to a multiple of 4: 568 bytes. <!--
  [O] --> `kdx_proto::msg::UserRecord` encodes both again.
- The icon is a bitmap with 2 bits per pixel, about 0x200 bytes in the
  capture. <!-- [O] -->
- The server removes `\r`, `\n`, and `\t` from the info text.
- If the info text ends with 0xac, the server hashes its last 0x17 bytes (FNV,
  seed 0x496c). If the result is 0x9652a79d, the bytes are a hidden marker: the
  server removes them and sets flag bit 1. The marker probably identifies the
  client. <!-- [S `FUN_0804b304`; I] -->

## Chat record

<!-- [S `FUN_08059944`, `FUN_08059be8`, `FUN_08059e24`; O] -->

[217 Create Chat](messages/chat.md#m217), [219 Chat
Info](messages/chat.md#m219), and [220 Set Chat](messages/chat.md#m220) use this
record.

Struct `ChatRecord` (dynamic):<a id="sChatRecord"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `u32` | `chat_id` |  |
| 0x04 | 4 | `u32` | `key` |  |
| 0x08 | 12 | `u8[12]` | `ignored_08` |  |
| 0x14 | 4 | `u32` | `join_classes` | A bitmap of account classes. The MSB is class 0. |
| 0x18 | 4 | `u32` | `send_classes` |  |
| 0x1c | 4 | `u32` | `topic_classes` |  |
| 0x20 | 8 | `u8[8]` | `ignored_20` |  |
| 0x28 | 2 | `u16` | `flags` | Bit 0 is private, and bit 1 is voice. |
| 0x2a | 1 | `u8` | `name_len` |  |
| 0x2b | 1 | `u8` | `description_len` |  |
| 0x2c | 1 | `u8` | `irc_server_len` |  |
| 0x2d | 1 | `u8` | `irc_channel_len` |  |
| 0x2e | 2 | `u8[2]` | `ignored_2e` |  |
| 0x30 | `name_len` | `str` | `name` | At most 0x3f bytes. |
|  | `description_len` | `str` | `description` | At most 0x7f bytes. |
|  | `irc_server_len` | `str` | `irc_server` | At most 0x7f bytes. `host:port`; if there is no port, the client adds `:6667`. <!-- [S client `FUN_080823a0`] --> |
|  | `irc_channel_len` | `str` | `irc_channel` | At most 0x3f bytes. If the name has no channel prefix, the client adds `#`. <!-- [S] --> |

The two IRC strings make the chat an IRC gateway. The client's chat editor calls
them Make IRC Gateway: IRC Server, Channel. The server only stores them. Each
client that joins the chat opens its own connection to the IRC server and joins
the channel, with a nick made from the user's nick (at most 9 characters). The
client's IRC sender sends a line that starts with `/` as an IRC command, and a
line that starts with `\` as raw IRC. How a chat window divides text between KDX
and IRC is not traced. <!-- [I] --> The client answers CTCP VERSION with "Unholy
Haxial KDX/IRC Client". <!-- [S client `FUN_08082b90`, `FUN_0809767c`,
`FUN_08097c30`, `FUN_08097f78`] --> So the IRC bridge is in the client. Server
1.620 has no IRC code.

A new server starts with three chats: <!-- [S `FUN_08058140`] -->

1. Public Chat, "For all users"
2. Voice Chat, with the voice flag
3. Admin Lounge, "For the administrators only!", for class 0 only

Each time the server starts, there is a chance of 10 in 300 that it sets the
topic of Public Chat to one of ten joke lines, from a made-up user such as
`Kevin`, `A. Beaver`, or `Diskartes`.

Chat 1, Public Chat, always exists and cannot be deleted. Privilege bit 0
overrides the class bitmaps.
