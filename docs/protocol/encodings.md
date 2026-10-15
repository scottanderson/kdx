# Common encodings

Tags, hex, and field notation: see [Conventions](README.md#conventions).

Strings, times, paths, and the user and chat records that many messages share.
Evidence tags and byte order are explained in
[the protocol reference](README.md).

## Strings

Strings have no terminator, and their lengths are not stored with them. A
fixed-size part of the record holds the lengths (usually u8). The strings follow
that part in a fixed order. Most variable records are then padded to a multiple
of 4. The padding bytes are uninitialized memory, so ignore them (the `86 ab` at
the end of the first entry in the [401 File List](messages/files.md#m401)
example) **[O]**. `kdx-proto` still stores them, with any other bytes it doesn't
model, and writes them back, so a decoded payload encodes to the same bytes
(`kdx-dissect --audit` over every capture) **[O]**.

Text is UTF-8 in the 1.6 builds, but the Linux server passes file names
through as raw bytes:

- Server 1.600 for Windows lists files with `FindFirstFileW` and encodes
  each UTF-16 name as UTF-8 (`FUN_00428640`, `FUN_00428380`). It decodes an
  incoming path as UTF-8 before `CreateFileW` (`FUN_0042a740`,
  `FUN_004286e0`), so an invalid byte becomes U+FFFD and the file is not
  found **[S]**.
- Client 1.600 for Windows keeps text as UTF-8 (decoder at 0x00457190)
  **[S]**.
- All three Windows builds (client, server, tracker) use only the wide
  Win32 calls and `CF_UNICODETEXT` (0xd) for the clipboard. Their only
  `MultiByteToWideChar` calls convert font and device names (client
  `FUN_0045ecd0`, server `FUN_0041b460`), so no Windows code page touches
  protocol text **[S]**.
- Server 1.600 for Mac OS X converts `readdir` names to NFC UTF-8 with
  `ConvertFromUnicodeToText` (`FUN_000348cc`, encoding 0x08000100), and the
  Carbon client 1.600 converts HFS+ names the same way **[S]**.
- The Carbon server 1.600 (PEF) reads names with `FSGetCatalogInfoBulk` and
  creates files with `FSCreateFileUnicode`, so it gets UTF-16 names from the
  File Manager even on Mac OS 9. `FUN_10032a80` normalizes a name to NFC
  (`CreateTextEncoding(0x100,3,0)`) and encodes it as UTF-8, or encodes it
  without normalizing when the Text Encoding Converter is older than 0x1020
  **[S]**. The Mac builds use the system script encoding
  (`UpgradeScriptInfoToTextEncoding`, `FUN_10060990`) only for local
  keyboard and clipboard text **[S]**.
- Client 1.620 beta 5 for Linux reads keys with `Xutf8LookupString` and the
  clipboard as `UTF8_STRING`, and has no other codec **[S]**. It keeps local
  file names as raw bytes (`readdir64` in `FUN_0810e6c0`) **[S]**.
- Server 1.620 for Linux copies `readdir64` names unchanged
  (`FUN_08091b6c`), so a name is whatever bytes are on disk **[S]**. Names
  copied from a classic Mac or uploaded by an older client may be Mac OS
  Roman, such as `Erik’s` with byte 0xd5 (user report). The Mach-O and Linux
  trackers read their folders with raw `readdir` too **[S]**.

Chat, nicks, and news follow from this: every client builds text from UTF-16 or
UTF-8 input and has no other codec, so it sends UTF-8 **[I]**. Servers relay
chat bytes without decoding them; [207 Send Chat Text](messages/chat.md#m207)
copies the text into [208 Receive Chat Text](messages/chat.md#m208) after
capping the message at 0x1000 bytes, which can split a multi-byte character
(`FUN_08058d74`) **[S]**.

The Linux pair bears this out. Client 1.620 beta 5 sends UTF-8 in 207,
[300 Send Private Message](messages/private-messages.md#m300),
[611 Post News Message](messages/news.md#m611),
[100 Set Identity](messages/session.md#m100),
[402 Create Folder](messages/files.md#m402), and
[415 Upload File](messages/files.md#m415), and Server 1.620 stores and relays
the bytes unchanged in 208,
[301 Incoming Private Message](messages/private-messages.md#m301),
[610 News Message](messages/news.md#m610), and
[102 User Changed](messages/session.md#m102) and on disk **[O
`captures/linux-text-encoding.pcapng` frames 90, 103, 118 to 121, 249 to 250,
319 to 325, 327 to 330]**. String lengths count bytes. A name matches only its
own bytes: with `caf\xc3\xa9.txt` and `caf\x8e.txt` in one folder, 401 lists
both and [412 Download File](messages/files.md#m412) fetches each by its own
spelling **[O frames 41, 52, 71]**.

The decoders never fail. Each invalid sequence becomes one U+FFFD: a stray
continuation byte (0x80 to 0xbf) or a 0xf8+ lead uses up 1 byte, but a lead byte
with bad continuations or an overlong form uses up the whole length it claims,
and a sequence cut off at the end uses up the rest. So `Erik\xd5s` shows as
`Erik�`, losing the `s` (Windows client 0x00457190, Windows server
`FUN_00428490`, Linux client `FUN_080e75d8`) **[S]**. The Linux client shows
`caf\x8e \xd5 \xd1` as `caf� ��`: the 0xd5 lead byte uses up the space after it,
so the client draws three boxes **[O `captures/linux-text-encoding.pcapng`
frames 195, 302]**. Encoding the other way, a lone UTF-16 surrogate becomes
U+FFFD (`FUN_00428640`) **[S]**.

KDX switched to Unicode in 1.2. Each client and server archive in
`vendor/MANIFEST.md` ships Haxial's release notes as `UPGRADE WARNINGS.html`
(clients) or `NEW UPGRADE WARNINGS.html` (servers), and their "KDX 1.2 --
UNICODE" entry says earlier versions supported only Latin text, that 1.2
can still connect to the previous version but the text may be "somewhat
mutilated", and that text created by earlier versions has the same problem
**[S]**. The Windows client's `WINDOWS XP REQUIRED.html` gives Unicode as the
reason it needs Windows NT **[S]**. The tracker archives' notes predate the
entry. Pre-1.2 builds are not vendored, so their charset is unknown;
probably Mac OS Roman from a Mac and a Windows code page from a PC **[I]**.

Nothing negotiates the charset. The handshake's sub-version is 0x10 for
every 1.6 client, and Server 1.620 accepts only 0x10 (see
[handshake.md](handshake.md)); since 1.2 could still reach 1.1, the switch
probably kept that byte **[I]**. No build branches on the peer's version
for text, so a 1.2+ client decodes pre-1.2 bytes as UTF-8 (showing U+FFFD as
above), and a pre-1.2 client shows UTF-8 as two or three Latin characters
per letter **[I]**. Old text stored on a server, such as file names or
news, can still hold pre-1.2 bytes.

No 1.6 build has a Mac OS Roman table. No capture has non-ASCII text yet.

## Times

Times are u64 milliseconds since 1900-01-01 00:00 UTC. The server builds
them as (tv_sec + 2208988800) × 1000 + ms **[S `FUN_08090eb0`]**, so
0x3a36c4895ba is 2026-10-08 **[O]**.

## File paths

A path is a list of names. Each name gives one folder level below the user's
base folder. **[S `FUN_08069880`, `FUN_08096b4c`; O]**

| Size | Field |
|---|---|
| 2 | count of names (1..0x400) |
| 2 | 0, per name |
| 1 | name length, per name (nonzero) |
| n | name, per name |

The root is an empty payload, not a count of 0. A path with names must be at
least 6 bytes. Otherwise the server raises 0x1000a: a
[400 Get File List](messages/files.md#m400) of `00 00` gets it in frames 106-107
of `captures/probe.pcapng` **[O]**, so `kdx-dissect` flags that request as
invalid. The server joins the names with `/` and checks each one with
`FUN_08094084` **[S]**. For example, `00 01 00 00 0c "Uploads [UL]"` is the root
folder `Uploads [UL]` **[O]**.

## User record

**[S `FUN_0804af04`, `FUN_0804b304`; O `captures/first-connect.pcapng`]**

This record is used in [100 Set Identity](messages/session.md#m100) (client to
server, user ID 0) and in [101 User List](messages/session.md#m101),
[102 User Changed](messages/session.md#m102), and
[103 User Joined](messages/session.md#m103).

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | user ID | assigned by the server; 0 in 100 Set Identity |
| 0x04 | 1 | status | the value set by [110 Set Mini User Icon](messages/session.md#m110) |
| 0x05 | 5 | 0 | |
| 0x0a | 2 | DCC TCP port | the client's client-to-client listener, 10600 by default; 0 if it has none (10600 in the capture **[O]**; **[S client `FUN_0804d654`]**) |
| 0x0c | 2 | icon length | < 0x709 |
| 0x0e | 6 | colours | from the client: two 3-byte RGB colours **[S client `FUN_0804d654`]**; `00 00 00 ff cc 00` in the capture |
| 0x14 | 1 | server byte | set by the server; 0 from the client, 0xff for admin **[O]** |
| 0x15 | 2 | server word | set by the server |
| 0x17 | 1 | flags | bit 2 is from the client; bit 1 is set by the server (see below) |
| 0x18 | 1 | extra length | skipped by the server |
| 0x19 | 1 | nick length | |
| 0x1a | 1 | info length | ≤ 0x7f |
| 0x1b | 1 | 0 | |
| 0x1c | | icon, extra, nick, info | in that order |

- The client sends 100 Set Identity unpadded (565 bytes in the capture). The
  server pads each record of 101 User List with zeros to a multiple of 4
  (568 bytes) **[O]**. `kdx_proto::msg::UserRecord` re-encodes both.

- The icon is a 2-bit-per-pixel bitmap (about 0x200 bytes in the capture)
  **[O]**.
- The server strips `\r`, `\n`, and `\t` from the info text.
- If the info text ends with 0xac, the server hashes its last 0x17 bytes
  (FNV, seed 0x496c). A result of 0x9652a79d means a hidden marker. The
  server removes those 0x17 bytes and sets flag bit 1. This is probably a
  client-identification trailer **[S `FUN_0804b304`; I]**.

## Chat record

The record of [219 Chat Info](messages/chat.md#m219)
**[S `FUN_08059944`, `FUN_08059be8`, `FUN_08059e24`; O]**

This record is used in [217 Create Chat](messages/chat.md#m217),
[219 Chat Info](messages/chat.md#m219), and
[220 Set Chat](messages/chat.md#m220).

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | chat ID | |
| 0x04 | 4 | key | |
| 0x08 | 0xc | | |
| 0x14 | 4 | join classes | bitmap of account classes, MSB = class 0 |
| 0x18 | 4 | send classes | |
| 0x1c | 4 | topic classes | |
| 0x20 | 8 | | |
| 0x28 | 2 | flags | bit 0 private, bit 1 voice |
| 0x2a | 1 | name length | ≤ 0x3f |
| 0x2b | 1 | description length | ≤ 0x7f |
| 0x2c | 1 | IRC server length | ≤ 0x7f; `host:port`, the client adds `:6667` if there is no port **[S client `FUN_080823a0`]** |
| 0x2d | 1 | IRC channel length | ≤ 0x3f; the client adds `#` if the name has no channel prefix **[S]** |
| 0x30 | | strings | |

The two IRC strings make the chat an IRC gateway (the client's chat editor calls
them Make IRC Gateway: IRC Server, Channel). The server only stores them. Each
client that joins the chat opens its own connection to the IRC server, joins the
channel, and uses a nick made from the user's nick (at most 9 characters). The
client's IRC sender treats a line starting with `/` as an IRC command and one
starting with `\` as raw IRC; how a chat window splits text between KDX and IRC
is not traced **[I]**. The client answers CTCP VERSION with "Unholy Haxial
KDX/IRC Client" **[S client `FUN_08082b90`, `FUN_0809767c`, `FUN_08097c30`,
`FUN_08097f78`]**. So the "IRC bridge" is client-side; Server 1.620 has no IRC
code.

A new server starts with three chats **[S `FUN_08058140`]**: 1 Public Chat
("For all users"), 2 Voice Chat (voice flag), and 3 Admin Lounge ("For the
administrators only!", class 0 only). At each start, with a chance of 10 in
300, Public Chat's topic is set to one of ten joke lines, with a made-up
setter such as `Kevin`, `A. Beaver`, or `Diskartes`.

Chat 1 (Public Chat) always exists and can't be deleted. Privilege bit 0
bypasses the class bitmaps.
