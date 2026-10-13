# KDX Protocol

This is a single reference for the Haxial KDX protocol as spoken by KDX
Server 1.620 and KDX Client 1.600. It is built from lab captures and static
reading of the binaries. The deeper write-ups are in
[handshake.md](handshake.md) (connection setup and authentication) and
[session.md](session.md) (message encryption and framing).

Each claim carries its evidence: **[O]** observed in a capture or lab probe,
**[S]** read in a binary (with the function address), **[I]** inferred. The
lab probes are in `captures/probe.pcapng` (see `captures/probe.md`).
Unmarked rows in a layout table share the tag in the message's heading line.
Offsets and lengths are hex. All multi-byte numbers are big-endian.
A message is named by its type and title, as in 412 Download File.

# Protocol Overview

KDX is a client/server system for chat, private messages, file sharing,
news, and server administration. Trackers list servers.

| Port | Transport | Use |
|---|---|---|
| 10700 | TCP | client ↔ server: every message, and file transfers on extra connections [O] |
| 10600 | TCP | client ↔ client (DCC): private chat and voice; the client advertises its port in the user record [O; S]; see [dcc.md](dcc.md) |
| 10800 | UDP, TCP | tracker: server registration (UDP) and listing (TCP); see [tracker.md](tracker.md) [S] |
| 10900 | TCP | GConsole, a Haxial remote console the client can open; no KDX server or tracker serves it [S]; see [dcc.md](dcc.md) |

A connection has two phases:

1. **Handshake.** Both sides exchange a cleartext hello (`%TXP` magic, the
   service `%KDX`, a version, a nonce, and the login name). Each side then
   proves knowledge of the account key and of a 1739-byte challenge table
   built into the programs. See [handshake.md](handshake.md).
2. **Session.** Every message after that is a 16-byte header plus a payload,
   each block encrypted with Twofish-256 in ECB mode under the account key.
   See [session.md](session.md).

# Messages

## Header

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | txid | nonzero. Each side counts up from its own random start |
| 0x04 | 4 | type | the message type, listed below |
| 0x08 | 1 | 0 | |
| 0x09 | 1 | flags | bit 0: this message is a reply |
| 0x0a | 2 | 0 | |
| 0x0c | 4 | len | payload length: at most 0x20000 to the server, 0x100000 to the client |

The payload is padded to a multiple of 16 before encryption; the length
field gives the real size.

## Requests, replies, and notifications

- A **request** carries the sender's next txid and flag 0.
- A **reply** reuses the request's txid and sets flag bit 0. The server sets
  the flag exactly when it answers a request (`FUN_080863b4` receives the
  request's txid) **[S, O]**.
- A **notification** is a message the server sends on its own, with its own
  txid and flag 0. For example, 103 User Joined goes to everyone else when a
  user logs in **[O]**.
- The server only handles messages with flag 0. Replies arriving from a
  client are dropped **[S `FUN_080528c8`]**.
- Every message except 108 Ping, 142 Get Connection Monitor Info, and 609
  Get News Message resets the sender's idle timer **[S `FUN_080528c8`]**.

Message types come in pairs: request N gets reply N+1. A request that fails
gets the generic **1 Status** reply instead. Some requests have no reply on
success **[S, O]**.

## 1 Status

Initiator: Server. Sent as a reply. **[S `FUN_08086640`, `FUN_080865c0`;
O]**

The dispatcher (`FUN_0804fdac`) runs each handler inside a C++ `try`. The
catch at 0x0805090c turns the exception into a 1 Status reply to the
request's txid. Some handlers also send 1 Status on success.

Error form, 24 bytes:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | code | an error code (see Error Codes) |
| 0x04 | 4 | request type | e.g. 0x98 for a failed 152 Get User Info [O] |
| 0x08 | 4 | module | source module tag of the raise site, ASCII packed into an integer (`0x54524e` = `TRN`; 0x46 for the path code) |
| 0x0c | 4 | line | source line of the raise site |
| 0x10 | 4 | extra | a value the raise site passes along |
| 0x14 | 4 | 0 | |

[O] examples:

- `152:00000003` (no user 3) returns `00 01 00 0c 00 00 00 98 00…`, which is
  0x1000c item does not exist.
- `400:0000` (malformed path) returns `00 01 00 0a 00 00 01 90 00 00 00 46
  00 00 00 98…`, which is 0x1000a corrupt, raised in module 0x46 at line
  0x98 of `FUN_080969f8`.

Success form, 8 bytes: `u32 0`, then `u32 value`, often the request type
**[S `FUN_080865c0`]**. 507 Create Account gets value 0x1fb and 511 Set
Account 0x1ff **[O `captures/kdx-accounts.pcapng` frames 235, 413]**.
201 Delete Chat gets 0xc9, 217 Create Chat 0xd9, 220 Set Chat 0xdc, and 504
Set Account Class 0x1f8 **[O `captures/linux-chat-rooms.pcapng` frames 81,
45, 55, 148]**.

The client renders an error as `ID ^0/^1 [^2] @ ^3 #^4`, filling in the
code's class and number, the module, and the line **[S]**.

# Common Encodings

## Strings

Strings have no terminator, and their lengths are not stored with them. A
fixed-size part of the record holds the lengths (usually u8). The strings
follow that part in a fixed order. Most variable records are then padded to
a multiple of 4. The padding bytes are uninitialized memory, so ignore them
(the `86 ab` at the end of the first entry in the 401 File List example)
**[O]**.

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

Chat, nicks, and news follow from this: every client builds text from UTF-16
or UTF-8 input and has no other codec, so it sends UTF-8 **[I]**. Servers
relay chat bytes without decoding them; 207 Send Chat Text copies the text
into 208 after capping the message at 0x1000 bytes, which can split a
multi-byte character (`FUN_08058d74`) **[S]**.

The Linux pair bears this out. Client 1.620 beta 5 sends UTF-8 in 207, 300,
611, 100, 402, and 415, and Server 1.620 stores and relays the bytes
unchanged in 208, 301, 610, and 102 and on disk **[O `linux-text-encoding`
90, 103, 118–121, 249–250, 319–325, 327–330]**. String lengths count bytes.
A name matches only its own bytes: with `caf\xc3\xa9.txt` and `caf\x8e.txt`
in one folder, 401 lists both and 412 fetches each by its own spelling
**[O 41, 52, 71]**.

The decoders never fail. Each invalid sequence becomes one U+FFFD: a stray
continuation byte (0x80 to 0xbf) or a 0xf8+ lead uses up 1 byte, but a lead
byte with bad continuations or an overlong form uses up the whole length it
claims, and a sequence cut off at the end uses up the rest. So `Erik\xd5s`
shows as `Erik�`, losing the `s` (Windows client 0x00457190, Windows server
`FUN_00428490`, Linux client `FUN_080e75d8`) **[S]**. The Linux client shows
`caf\x8e \xd5 \xd1` as `caf� ��`: the 0xd5 lead byte uses up the space after
it, so the client draws three boxes **[O `linux-text-encoding` 195, 302]**.
Encoding the other way, a lone UTF-16 surrogate becomes U+FFFD
(`FUN_00428640`) **[S]**.

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

Nothing negotiates the charset. The handshake's version byte is 0x10 for
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
least 6 bytes. Otherwise the server raises 0x1000a: a 400 Get File List of
`00 00` gets it in frames 106-107 of `captures/probe.pcapng` **[O]**, so
`kdx-dissect` flags that request as invalid. The server joins the
names with `/` and checks each one with `FUN_08094084` **[S]**. For example,
`00 01 00 00 0c "Uploads [UL]"` is the root folder `Uploads [UL]` **[O]**.

## User record

**[S `FUN_0804af04`, `FUN_0804b304`; O `captures/first-connect.pcapng`]**

This record is used in 100 Set Identity (client to server, user ID 0) and in
101 User List, 102 User Changed, and 103 User Joined.

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | user ID | assigned by the server; 0 in 100 Set Identity |
| 0x04 | 1 | status | the value set by 110 Set Mini User Icon |
| 0x05 | 5 | 0 | |
| 0x0a | 2 | DCC TCP port | the client's client-to-client listener, 10600 by default; 0 if it has none (10600 in the capture [O]; [S client `FUN_0804d654`]) |
| 0x0c | 2 | icon length | < 0x709 |
| 0x0e | 6 | | from the client: two 3-byte RGB colours [S client `FUN_0804d654`]; `00 00 00 ff cc 00` in the capture |
| 0x14 | 1 | | set by the server; 0 from the client, 0xff for admin [O] |
| 0x15 | 2 | | set by the server |
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

**[S `FUN_08059944`, `FUN_08059be8`, `FUN_08059e24`; O 219 Chat Info]**

This record is used in 217 Create Chat, 219 Chat Info, and 220.

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
| 0x2c | 1 | IRC server length | ≤ 0x7f; `host:port`, the client adds `:6667` if there is no port [S client `FUN_080823a0`] |
| 0x2d | 1 | IRC channel length | ≤ 0x3f; the client adds `#` if the name has no channel prefix [S client] |
| 0x30 | | strings | |

The two IRC strings make the chat an IRC gateway (the client's chat editor
calls them Make IRC Gateway: IRC Server, Channel). The server only stores
them. Each client that joins the chat opens its own connection to the IRC
server, joins the channel, and uses a nick made from the user's nick (at most
9 characters). The client's IRC sender treats a line starting with `/` as an
IRC command and one starting with `\` as raw IRC; how a chat window splits
text between KDX and IRC is not traced [I]. The client
answers CTCP VERSION with "Unholy Haxial KDX/IRC Client" **[S client
`FUN_08082b90`, `FUN_0809767c`, `FUN_08097c30`, `FUN_08097f78`]**. So the
"IRC bridge" is client-side; Server 1.620 has no IRC code.

A new server starts with three chats **[S `FUN_08058140`]**: 1 Public Chat
("For all users"), 2 Voice Chat (voice flag), and 3 Admin Lounge ("For the
administrators only!", class 0 only). At each start, with a chance of 10 in
300, Public Chat's topic is set to one of ten joke lines, with a made-up
setter such as `Kevin`, `A. Beaver`, or `Diskartes`.

Chat 1 (Public Chat) always exists and can't be deleted. Privilege bit 0
bypasses the class bitmaps.

# Message Types (with Type ID)

"Handler" is the server function that handles the request. The names come
from the server's name table (`FUN_08049d14`, jump table at 0x080abc6c)
**[S]**. In that table, (R) means reply, (D) data, (C) complete, and (S)
status.

| Type | Name | Initiator | Handler |
|---:|---|---|---|
| 1 | Status (this project's name; the server's table has none) | Server | `FUN_08086640` |
| 100 | Set Identity | Client | `FUN_0804b304` |
| 101 | User List | Server | |
| 102 | User Changed | Server | |
| 103 | User Joined | Server | |
| 104 | User Departed | Server | |
| 105 | Server Info | Server | |
| 106 | Server Icon | Server | |
| 107 | Server Greeting | Server | |
| 108 | Ping | Client | inline |
| 109 | Pong | Server | |
| 110 | Set Mini User Icon | Client | `FUN_0804b6d4` |
| 111 | Mini User Icon Changed | Server | |
| 112 | (unnamed; requests the greeting) | Client | `FUN_0804b738` |
| 113 | Get Extra Server Info | Client | `FUN_080619d8` |
| 114 | Extra Server Info | Server | |
| 115 | Get My Address | Client | inline |
| 116 | Your Address | Server | |
| 117 | SKPK | Client (to client) | client `FUN_0808ff64` (DCC only) |
| 118 | (unnamed; licence report) | Client | inline |
| 140 | Open Connection Monitor | Client | inline |
| 141 | Close Connection Monitor | Client | inline |
| 142 | Get Connection Monitor Info | Client | `FUN_0804b9b0` |
| 143 | Connection Monitor Info | Server | |
| 150 | Disconnect User | Client | `FUN_0804c42c` |
| 151 | Disconnect Message | Server | |
| 152 | Get User Info | Client | `FUN_0804c8a8` |
| 153 | User Info | Server | |
| 154 | Send Broadcast Message | Client | `FUN_0804d1a8` |
| 155 | Rcv Broadcast Message | Server | |
| 170 | Get Server Settings | Client | `FUN_0805f598` |
| 171 | Server Settings Info | Server | |
| 172 | Set Server Settings | Client | `FUN_0805f638` |
| 173 | Set Server Settings Done | Server | |
| 174 | Shutdown Server | Client | `FUN_08061598` |
| 175 | Upgrade Server Software | Client | `FUN_0806172c` |
| 180 | Server History Join | Client | `FUN_08062674` |
| 181 | Server History Join (R) | Server | |
| 182 | Server History Leave | Client | `FUN_080627f8` |
| 183 | Server History Added | Server | `FUN_08061d80` |
| 184–191 | history event kinds: Login, Disconnected, Abort File Download, Abort File Upload, Finished File Download, Finished File Upload, Server Started, Login Failed | (record kinds) | |
| 200 | Create Private Chat | Client | `FUN_08058420` |
| 201 | Delete Chat | Client | `FUN_08059774` |
| 202 | Join Chat | Client | `FUN_08058710` |
| 203 | Chat Window Info | Server | |
| 204 | Leave Chat | Client | `FUN_08058c0c` |
| 205 | Chat User Joined | Server | |
| 206 | Chat User Departed | Server | |
| 207 | Send Chat Text | Client | `FUN_08058d74` |
| 208 | Receive Chat Text | Server | |
| 209 | Set Chat Topic | Client | `FUN_08058eac` |
| 210 | Chat Topic Changed | Server | |
| 211 | Chat Invite | Client | `FUN_080591ac` |
| 212 | Chat Invitation | Server | |
| 213 | Must Join Chat | Server | |
| 214 | Get Chat List | Client | `FUN_080594dc` |
| 215 | Chat List | Server | |
| 216 | Create Private Chat (R) | Server | |
| 217 | Create Chat | Client | `FUN_08059944` |
| 218 | Get Chat | Client | `FUN_08059be8` |
| 219 | Chat Info | Server | |
| 220 | Set Chat | Client | `FUN_08059e24` |
| 221 | Chat Deleted | Server | |
| 222 | Get Chat DCC Addresses | Client | `FUN_0805a110` |
| 223 | Chat DCC Addresses | Server | |
| 300 | Send Private Message | Client | `FUN_0804d2d0` |
| 301 | Incoming Private Message | Server | |
| 400 | Get File List | Client | `FUN_0804d938` |
| 401 | File List | Server | |
| 402 | Create Folder | Client | `FUN_0804e0f8` |
| 403 | Create Alias | Client | `FUN_0804e4ec` |
| 404 | Delete File/Folder | Client | |
| 405 | Move To Trash | Client | `FUN_0804e980` |
| 406 | Rename File/Folder | Client | |
| 407 | Move File/Folder | Client | `FUN_0804ebd8` |
| 408 | Copy File/Folder | Client | |
| 409 | Get File/Folder Info | Client | `FUN_0804ef00` |
| 410 | File/Folder Info | Server | |
| 411 | Set File/Folder Info | Client | `FUN_0804f684` |
| 412 | Download File | Client | `FUN_08050984` (transfer connection) |
| 413 | Download File (R) | Server | |
| 414 | Download File (D) | Server | |
| 415 | Upload File | Client | `FUN_08051750` (transfer connection) |
| 416 | Upload File (R) | Server | |
| 417 | Upload File (D) | Client | |
| 418 | Upload File (C) | Server | |
| 419 | Notify Speed Limit | Server | |
| 420 | Launch File | Client | `FUN_0804fa90` |
| 421 | Empty Trash | Client | inline |
| 422 | Volume Info | Server | |
| 423 | Set Volume Info | Client | `FUN_0804f8d0` |
| 450 | Generate Catalog | Client | `FUN_0805a728` |
| 451 | Generate Catalog (S) | Server | |
| 452 | File Search | Client | `FUN_0805abc0` |
| 453 | File Search (R) | Server | |
| 502 | Get Account Class | Client | `FUN_080552b8` |
| 503 | Account Class Info | Server | |
| 504 | Set Account Class | Client | `FUN_080553c8` |
| 505 | Get Account Class List | Client | `FUN_0805572c` |
| 506 | Account Class List | Server | |
| 507 | Create Account | Client | `FUN_08055928` |
| 508 | Delete Account | Client | `FUN_08055d50` |
| 509 | Get Account | Client | `FUN_08055e98` |
| 510 | Account Info | Server | |
| 511 | Set Account | Client | `FUN_08055ff4` |
| 512 | Get Account List | Client | `FUN_080566bc` |
| 513 | Account List | Server | |
| 514 | Account Changed | Server | |
| 515 | Account Deleted | Server | |
| 550 | Change Own Password | Client | `FUN_08056a04` |
| 551 | Change Own Password Done | Server | |
| 600 | Create Newsgroup | Client | `FUN_0805c8f8` |
| 601 | Delete Newsgroup | Client | `FUN_0805caf4` |
| 602 | Get Newsgroup | Client | `FUN_0805cc24` |
| 603 | Newsgroup Info | Server | |
| 604 | Set Newsgroup | Client | `FUN_0805ce60` |
| 605 | Get Newsgroup List | Client | `FUN_0805d038` |
| 606 | Newsgroup List | Server | |
| 607 | Get News Messages | Client | `FUN_0805d2e8` |
| 608 | News Messages | Server | |
| 609 | Get News Message | Client | `FUN_0805d54c` |
| 610 | News Message | Server | |
| 611 | Post News Message | Client | `FUN_0805d758` |
| 612 | Delete News Message | Client | `FUN_0805db44` |
| 613 | News Message Posted | Server | |
| 614 | News Message Deleted | Server | |
| 700 | Get Tracker Group List | Client | tracker `FUN_0804abf4` |
| 701 | Tracker Group List | Tracker | |
| 702 | Get Tracker Server List | Client | tracker `FUN_0804adcc` |
| 703 | Tracker Server List | Tracker | |
| 710–713 | Get / Info / Set / Done TrackerServ Settings | | tracker `FUN_0804cb90`, `FUN_0804cc10` |
| 720 | Get TrackerServ Statistics | Client | tracker `FUN_0804b180` |
| 721 | TrackerServ Statistics | Tracker | |
| 800 | Open Process Monitor | Client | `FUN_0805b394` |
| 801 | Close Process Monitor | Client | `FUN_0805b418` |
| 802 | Get Process List | Client | `FUN_0805b454` |
| 803 | Process List | Server | |
| 804 | Exit/Terminate Process | Client | `FUN_0805b8f4` |
| 900 | Open View Display | Client | `FUN_0805c0f0` (transfer connection) |
| 901 | Close View Display | Client | none |
| 902 | View Display Params | Server | |
| 903 | View Display Update | Server | |
| 904 | View Display Mouse | Client | `FUN_0805c344` |
| 905 | View Display Keyboard | Client | `FUN_0805c498` |
| 1000 | Start Voice | Client (to client) | client `FUN_08094074` (DCC only) |
| 1001 | Start Voice (R) | Client (to client) | client `FUN_08094218` |
| 1002 | Set Voice Format | Client (to client) | client `FUN_080943a0` |
| 1003 | Voice Data | Client (to client) | client `FUN_0809455c` (TCP) |
| 1100 | Create Folder Access | Client | `FUN_080572f0` |
| 1101 | Delete Folder Access | Client | `FUN_08057460` |
| 1102 | Folder Access Deleted | Server | |
| 1103 | Get Folder Access | Client | `FUN_08057534` |
| 1104 | Folder Access Info | Server | |
| 1105 | Set Folder Access | Client | `FUN_08057634` |
| 1106 | Get Folder Access List | Client | `FUN_080578d8` |
| 1107 | Folder Access List | Server | |

A type the dispatcher doesn't know is ignored without a reply **[S]**.

## Message Description

### Session

**100 Set Identity**
Initiator: Client. **[S `FUN_0804b304`; O]**

The client's first message after the handshake. It is also sent again
whenever the user changes nick, icon, or info. The payload is a user record
with user ID 0.

- The nick takes effect only with the Use Any Name privilege; otherwise the
  account name is used. Changes are logged as `Set Name: <…> (…)`.
- No direct reply. On the first 100 Set Identity, the server sends, in
  order:
  - 101 User List;
  - 105 Server Info;
  - 107 Server Greeting, unless the account has Don't Show Greeting;
  - 106 Server Icon, if one is set;
  - 213 Must Join Chat, when Join Chat On Connect and Join Chat are both
    set.

  It also sends 103 User Joined to everyone else and writes a 184 Login history
  record. [O]: 105, 107, 106 in that order
  (`captures/linux-server-settings.pcapng` frames 80, 82, 83), and 213
  after 106 (`captures/linux-chat-rooms.pcapng` frame 167).
- A later 100 Set Identity broadcasts 102 User Changed to everyone, the
  sender included (`captures/linux-server-settings.pcapng` frames 196, 199,
  201) **[O]**.

**101 User List**
Initiator: Server. **[S `FUN_0804af04`; O]**

A sequence of user records, one per user who is logged in (all of them on
login).

**102 User Changed** / **103 User Joined**
Initiator: Server. **[S; O]**

One user record.

**104 User Departed**
Initiator: Server. **[O `captures/kdx-cli.pcapng` frames 398, 429;
`captures/linux-chat-admin.pcapng` frames 59, 116]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | user ID | |
| 0x04 | 1 | | 7 in every capture |
| 0x05 | 3 | 0 | |

`kdx_proto::msg::user::UserDeparted` re-encodes it byte for byte (test
`linux_chat_admin`).

**105 Server Info**
Initiator: Server. **[S `FUN_0804b304`; O 86 bytes]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 0x20 | | 32 bytes from the user object (+0x74), probably a session ID for transfer connections [I] |
| 0x20 | 4 | file transfer port | the server's optional File Transfer port (settings +0x260); 0 if none. The client opens transfer connections there when it is nonzero [S client `FUN_08052cc0`] |
| 0x24 | 4 | your user ID | [O] 7 and 8 in successive probes |
| 0x28 | 2 | server version | 0x0654 = 1620 |
| 0x2a | 2 | can modify settings | 1 with Modify Server Settings |
| 0x46 | 2 | | 2. Unless it is 2, the client compares the version and warns about an older or newer server [S client]; Server 1.620 always sends 2, so the warning never shows |
| 0x48 | 1 | name length | ≤ 0x3f |
| 0x49 | 1 | description length | ≤ 0xff |
| 0x4a | 1 | third string length | ≤ 0x7f |
| 0x4c | | name, description, third string | "New Server" [O] |

The whole message is at most 0x400 bytes.

**106 Server Icon**
Initiator: Server. **[O]**

The server's icon bitmap, the same bytes as the icon section of the settings
blob (see 171 Server Settings Info). [O]: 524 bytes for a 32×32 icon pasted
from the user icon, starting `01 20 20 04 00 01 00 00 00 ff cc 00`
(`captures/linux-server-settings.pcapng` frame 83), equal to the icon
section of the 171 at frame 221. `kdx_proto::msg::server_info::ServerIcon`
holds the bytes.

**107 Server Greeting**
Initiator: Server. **[S `FUN_0804b738`; O]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | 0 |
| 0x04 | | greeting text, to the end of the payload |

When no greeting is set, the server sends `(No greeting.)`
(`captures/probe.pcapng` frame 163) **[O]**. When a greeting is set but not
enabled, it sends `(Greeting disabled.)`
(`captures/linux-server-settings.pcapng` frame 46) **[O]**. The enable
option is bit 1 of byte 0x93 of the settings blob.
`kdx_proto::msg::server_info::ServerGreeting` re-encodes frames 46, 62, and
82 byte for byte (test `linux_server_settings`).

**108 Ping** → **109 Pong**
Initiator: Client. **[S]**

The request is empty, and so is the reply (109 Pong). Ping does not reset
the idle timer.

**110 Set Mini User Icon**
Initiator: Client. **[S `FUN_0804b6d4`; O]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 1 | status | 1 after `/away`, 0 after `/back` [O] |
| 0x01 | 3 | 0 | the payload must be at least 4 bytes |

There is no reply. If the value changed, the server broadcasts **111 Mini
User Icon Changed** to everyone, the sender included:

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | user ID |
| 0x04 | 4 | the 110 payload |

[O]: `/away` sent `01 00 00 00`, and the peer received `00 00 00 03 01 00 00
00` for user 3 (`captures/linux-server-settings.pcapng` frames 210, 211,
213). `kdx_proto::msg::user::MiniUserIcon` and `MiniUserIconChanged`
re-encode frames 210 to 218 byte for byte.

**112 (unnamed)**
Initiator: Client. **[S `FUN_0804b738`; O]**

The request is empty. The server answers with 107 Server Greeting as a new
message (its own txid), not as a reply. The client uses this to re-read the
greeting.

**113 Get Extra Server Info** → **114 Extra Server Info**
Initiator: Client. **[S `FUN_080619d8`; O]**

The request is empty. Reply:

| Off | Size | Field |
|---|---|---|
| 0x00 | 0x10 | |
| 0x10 | 8 | server start time (ms) |
| 0x18 | 8 | current server time (ms) |
| 0x20 | 4 | uptime in seconds |
| 0x24 | 4 | OS value |
| 0x28 | 4 | OS value |
| 0x2c | 1 | OS string length |
| 0x30 | | OS string |

**115 Get My Address** → **116 Your Address**
Initiator: Client. **[S; O]**

The request is empty. The reply is a u8 length followed by `"a.b.c.d:port"`:
the client's address as the server sees it.

**118 (unnamed)**
Initiator: Client. **[S `FUN_0804fdac`; O `captures/probe.pcapng` frame
165]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | value |

No reply. The server stores 1 if the value is nonzero and 2 if it is zero
(user +0x3b). The probe sent `00 00 00 01` and got no reply **[O]**.
`kdx_proto::msg::user::RegistrationNotice` re-encodes it.

The client sends it on a timer, 9000000 ms plus a random 0..0x4fffff ms
after login **[S client `FUN_0804c01c`]**. The value comes from the
client's stored purchase certificate (its length; 0 without one) **[S
client `FUN_08107e04`]**. So it reports whether the client is registered
**[I]**.

The server acts on it **[S `FUN_0804b7c8`]**: every 900000 ms plus a random
0..0x8ffff ms, it sends 155 Rcv Broadcast Message "Please help fund the further
development of KDX by purchasing a license." to each user whose stored value is
2 and who has been connected less than 2 hours (7200000 ms). The first round
runs 217500000 ms (about 60 hours) after the server starts **[S
`FUN_080531f0`]**, so a short lab run never shows it. The text is obfuscated in
the binary at 0x080adc00 (each byte is ~b + 0xbe). The 155 payload is 16 zero
bytes then the text.

### Monitoring and administration

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
| 0x04 | 4 | transfer ID | 0 in a connection record; the transfer's ID in a transfer record [S] |
| 0x08 | 8 | login time | the transfer's start time in a transfer record [S] |
| 0x10 | 8 | time | 0 in a transfer record [S] |
| 0x18 | 8 | current server time | [S] |
| 0x20 | 8 | bytes done | transfer record only: bytes moved minus bytes still queued to send [S] |
| 0x28 | 8 | total bytes | transfer record only [S] |
| 0x30 | 4 | rate | transfer record only: bytes per second since the start, not counting a resumed offset [S]; 0x51a1 under a 0x5000 limit [O] |
| 0x34 | 4 | seconds left | transfer record only: (total − done) / rate [S] |
| 0x38 | 4 | | transfer record only; a per-transfer value [S]; 0x5000, the server's speed limit, in every record [O] |
| 0x3c | 4 | state | transfer record only; 0x90 while data is queued and the state is 0 [S]; 0x19c (412) for a download [O] |
| 0x40 | 2 | 0 | |
| 0x42 | 1 | account class index | [S] |
| 0x60 | 1 | nick length | |
| 0x61 | 1 | login length | |
| 0x62 | 1 | account class name length | `?` if the class is gone [S] |
| 0x63 | 1 | address length | the connection's peer address; the transfer connection's in a transfer record [S] |
| 0x66 | 2 | path length | transfer record only: the file path, as in the file messages [S, O] |
| 0x68 | | nick, login, account class name, address, then the path | `probe`, `admin`, `Administrator`, `127.0.0.1:50984` |

The padding after the strings is uninitialised **[O]**. Fields marked
transfer record only are 0 in a connection record.

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
| 0x3f | 1 | name length | `Administrator`; the class or the account name [I] |
| 0x40 | 1 | login length | |
| 0x41 | 1 | name length | `Administrator`; the class or the account name [I] |
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
| 0x0c | 0x58 | section table | 11 entries of u32 offset, u32 length [I] |
| 0x0c | 8 | main section | entry 0: 0x70, 0x68 |
| 0x24 | 8 | greeting section | entry 3: 0xd8, 0x17; length 0 when no greeting is set |
| 0x34 | 8 | icon section | entry 5: 0xf0, 0x20c; 0xd8, 0 when none is set; 0xf0, 0 in a 172 without it |
| 0x44 | 8 | | entry 7: offset the end of the blob and length 0 in a 171; 0, 0 in a 172 |
| 0x64 | 12 | | `01 01 01 01 00 01 00 02 00 00 00 00` in a 171; 0x6b is 0 in a 172, and 0x69 too in a 172 without the icon |
| 0x93 | 1 | flags | bit 1: greeting enabled |
| 0xa8 | 4 | total outgoing speed limit | bytes per second; 0x5000, or 0 when off |
| 0xc2 | 1 | | 0x0a, the length of both names seen [I] |
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
`kdx_proto::msg::admin::HistoryRecord` re-encodes every record byte for
byte.

### Chat

**200 Create Private Chat** → **216 Create Private Chat (R)**
Initiator: Client. Privilege: Create Private Chat (34). **[S
`FUN_08058420`; O `captures/linux-chat-admin.pcapng` frames 84-85]**

A user can have at most 10 private chats. The request is empty. The reply
has the 203 Chat Window Info layout, with the creator as the only member
**[O]**:

| Off | Size | Field |
|---|---|---|
| 0x08 | 4 | chat ID |
| 0x0c | 4 | key |
| 0x14 | 2 | flags; 0 at 85 |
| 0x16 | 2 | member count |
| 0x20 | 1 | name length |
| 0x24 | 4 × count | member user IDs: the creator |
| … | | name, e.g. `Private 1` |

[O]: the 49-byte reply at 85 holds time 0, chat ID 0xb06d05e6, key
0x4ca93520, 1 member, user 2, and `Private 1`. The u32 at 0x24 is the member
list of 203, not a separate creator field.
`kdx_proto::msg::chat::ChatWindowInfo` re-encodes it byte for byte (test
`linux_chat_admin`).

**201 Delete Chat**
Initiator: Client. Privilege: Create/Modify Chats (37). **[S
`FUN_08059774`; O]**

The request is a u32 chat ID. Every member, the requester included,
receives **221 Chat Deleted** {u32 chat ID, u32 user ID of the requester}
before the requester's 1 Status success. Users outside the chat receive
nothing.

[O] `captures/linux-chat-rooms.pcapng`: deleting `lab-chat` (0xe8df4d29)
with two members sent `e8 df 4d 29 00 00 00 02` to both (frames 125, 127),
then Status at 129. Deleting the empty `lab-room` sent no 221 to a user in
Public Chat (frames 77–83). The client deleted a chat whose window was open
without sending 204 Leave Chat first (frame 122). `ChatId` and `ChatDeleted`
in `kdx_proto::msg::chat` re-encode 201 (frames 77, 122) and 221 (frames
125, 127) byte for byte (test `linux_chat_rooms`).

**202 Join Chat** → **203 Chat Window Info**
Initiator: Client. Privilege: Join Chat (30) and the chat's join classes.
**[S `FUN_08058710`; O]**

Request: {u32 chat ID, u32 key}.

Reply (51 bytes in the lab):

| Off | Size | Field |
|---|---|---|
| 0x00 | 8 | time |
| 0x08 | 4 | chat ID |
| 0x0c | 4 | key |
| 0x14 | 2 | flags |
| 0x16 | 2 | member count |
| 0x1e | 1 | IRC server length | [S client `FUN_08082b90`] |
| 0x1f | 1 | IRC channel length | |
| 0x20 | 1 | name length | [O] |
| 0x21 | 1 | topic length | |
| 0x22 | 1 | topic setter length | the nick that set the topic |
| 0x24 | 4 × count | member user IDs | |
| | | name, topic, topic setter, IRC server, IRC channel | |

In frame 68 of `captures/probe.pcapng` only the name (`Public Chat`) is
set; the payload is not padded.

[O] `captures/linux-chat-rooms.pcapng`: flags is 2 for a voice chat (frames
70, 115) and 0 for Public Chat (frame 176). The time is set for Public Chat
and 0 for chats made with 217 Create Chat.

The other members receive **205 Chat User Joined** {u32 chat ID, u32 user
ID, u32 0} (`captures/kdx-cli.pcapng` frame 417; `linux-chat-rooms.pcapng`
frames 117, 177; `linux-chat-admin.pcapng` frame 68) **[O]**.
`kdx_proto::msg::chat::ChatUserChange` re-encodes it byte for byte.

**204 Leave Chat**
Initiator: Client. **[S `FUN_08058c0c`; O `captures/linux-chat-rooms.pcapng`
frame 75, `captures/kdx-cli.pcapng` frames 423-424]**

The request is a u32 chat ID. The other members receive **206 Chat User
Departed** {u32 chat ID, u32 user ID, u8 1, 3 zero bytes}
(`captures/kdx-cli.pcapng` frame 424) **[O]**. The meaning of the 1 is
unknown; 205 has 0 there. A private chat that becomes empty is deleted.
`ChatId` and `ChatUserChange` in `kdx_proto::msg::chat` re-encode 204 and
206 byte for byte (tests `linux_chat_rooms`, `kdx_cli`).

**207 Send Chat Text** → **208 Receive Chat Text**
Initiator: Client. Privilege: Send Chat (31) and the chat's send classes.
**[S `FUN_08058d74`; O]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | chat ID |
| 0x04 | 4 | user ID (the server fills in the sender) |
| 0x08 | 4 | |
| 0x0c | | text, ≤ 0x1000 |

The server forwards the same layout as 208 Receive Chat Text to every
member, the sender included.

**209 Set Chat Topic** → **210 Chat Topic Changed**
Initiator: Client. Privilege: Set Chat Topic (35) and the chat's topic
classes. **[S `FUN_08058eac`; O]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | chat ID |
| 0x0a | 1 | topic length |
| 0x0c | | topic |

[O]: a probe that put `Hello` at 0x10 with length 5 got the topic
`00 00 00 00 48`, so the server reads from 0x0c (`captures/probe.pcapng`
frames 73–74).

Members receive 210 Chat Topic Changed:

| Off | Size | Field |
|---|---|---|
| 0x00 | 8 | time |
| 0x08 | 4 | chat ID |
| 0x0c | 4 | setter user ID |
| 0x10 | 8 | 0 |
| 0x18 | 1 | topic length |
| 0x19 | 1 | setter name length |
| 0x1a | 2 | 0 |
| 0x1c | | topic, setter name |

**211 Chat Invite** → **212 Chat Invitation**
Initiator: Client. **[S `FUN_080591ac`; O `captures/linux-chat-admin.pcapng`
frames 87-88, 118-120]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | chat ID |
| 0x08 | 2 | invitee count (≤ 20) |
| 0x0f | 1 | message length |
| 0x10 | 4 × count | invitee user IDs |
| … | | message |

Each invitee receives 212 Chat Invitation, not padded **[O]**:

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | chat ID |
| 0x04 | 4 | key |
| 0x08 | 4 | inviter user ID |
| 0x0c | 4 | 1 at 88, 2 at 120 |
| 0x10 | 2 | 1 |
| 0x18 | 1 | probably the message length [I]; 0 |
| 0x19 | 1 | chat name length |
| 0x1c | | message [I], chat name |

[O]: the client invited user 1 to `Private 1` (frame 87, 20 bytes), then
users 1 and 4 (frame 118, 24 bytes), with no message. User 4 had left, so
only user 1 got a 212 each time (frames 88 and 120, 37 bytes each). The u32
at 0x0c may be the request's invitee count or a running count [I]. The u16
at 0x10 is 1 although 216 gave the chat flags 0. `ChatInvite` and
`ChatInvitation` in `kdx_proto::msg::chat` re-encode both byte for byte
(test `linux_chat_admin`).

**213 Must Join Chat**
Initiator: Server. **[S `FUN_0804b114`; O]**

Sent after login when the account has Join Chat (30) and Join Chat On
Connect (36). The payload is empty, so it names no chat. The client always
joins chat 1, Public Chat: its 213 handler ignores the payload and calls the
join helper with chat ID 1 and key 0, which opens the chat window and sends
202 Join Chat {1, 0} **[S Client 1.620 beta 5 `FUN_08084020`,
`FUN_08080734`]**. If the window for chat 1 is already open, the helper only
brings it to the front and sends nothing.

[O] `captures/linux-chat-rooms.pcapng`: with bit 36 set in the
Administrator class (504 Set Account Class at frame 145), 213 arrived after
106 Server Icon (frame 167), and the client sent 202 for chat 1 (frame 175).

**214 Get Chat List** → **215 Chat List**
Initiator: Client. **[S `FUN_080594dc`; O]**

The request is empty. Reply: a u32 count, then one entry per chat, each
padded to 4:

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | chat ID |
| 0x04 | 4 | key |
| 0x0e | 2 | flags (bit 1 voice) |
| 0x10 | 2 | member count |
| 0x12 | 1 | name length |
| 0x13 | 1 | description length |
| 0x14 | | name, description |

[O]: Public Chat (1), Voice Chat (2, flags 2), Admin Lounge (3). Chats
made with 217 Create Chat follow, with a random chat ID (0xc217598c and
0xe8df4d29 in `captures/linux-chat-rooms.pcapng` frames 47, 90).

**217 Create Chat** / **218 Get Chat** → **219 Chat Info** / **220 Set Chat**
Initiator: Client. Privilege: Create/Modify Chats (37) for 217 Create Chat
and 220. **[S; O `captures/linux-chat-rooms.pcapng` frames 42-62, 85,
136-137]**

217 Create Chat and 220 Set Chat carry a chat record. 218 Get Chat is a u32
chat ID, and 219 Chat Info is a chat record. 220 Set Chat refuses private
chats. 217 and 220 get 1 Status success.

Chat record, not padded:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | chat ID | 0 in 217 Create Chat |
| 0x04 | 0x10 | | 0 |
| 0x14 | 4 | class mask | |
| 0x18 | 4 | class mask | |
| 0x1c | 4 | class mask | |
| 0x20 | 8 | | 0 |
| 0x28 | 2 | flags | bit 1 voice |
| 0x2a | 1 | name length | |
| 0x2b | 1 | description length | |
| 0x2c | 4 | | 0 |
| 0x30 | | name, description | |

[O] `captures/linux-chat-rooms.pcapng`: 217 for `lab-room` / `lab` (frame
42), 219 and 220 for `lab edited` (frames 50, 52), 220 with Voice Chat ticked
(flags 2, frame 62), and 219 for Public Chat (frame 137). The three class
masks are `ff ff ff ff` for Public Chat and `80 00 00 00` for a chat whose
Access was left at the default. Their bits are probably account classes,
MSB first (Administrator is class 0), for join, send, and topic [I]. The
client sends 505 Get Account Class List before it opens the new-chat window
(frame 39). 218 is the chat ID alone (frames 49, 59, 136). 217 gets 1 Status
success 0xd9 and 220 gets 0xdc (frames 45, 55). `ChatRecord` and `ChatId` in
`kdx_proto::msg::chat` re-encode 217 to 220 byte for byte (test
`linux_chat_rooms`).

**222 Get Chat DCC Addresses** → **223 Chat DCC Addresses**
Initiator: Client. **[S `FUN_0805a110`; O]**

The request is a u32 chat ID. The chat must have the voice flag (bit 1)
and the sender must be a member; otherwise 1 Status 0x10014. An unknown
chat gets 0x1000c **[S]**. The reply has one entry for each other member
whose user record has a nonzero DCC port (0x0a), each padded to 4 **[S]**:

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | user ID |
| 0x04 | 0xa | 0 |
| 0x0e | 1 | address length |
| 0x0f | 1 | 0 |
| 0x10 | | `"ip:port"`: the member's address and DCC port |

The client then opens a TCP connection to each address for voice; see
[dcc.md](dcc.md).

[O] `captures/linux-chat-rooms.pcapng`: the client sends 222 right after
joining a voice chat. The reply was empty (len 0) when the client was alone
(frames 72–73) and when a `kdx` CLI peer was also a member (frames
119-120). The CLI sends DCC port 0, so the server leaves it out **[S]**.
An entry has not been seen, so 223 has no codec yet.
`kdx_proto::msg::chat::ChatId` re-encodes 222 byte for byte (test
`linux_chat_rooms`).

### Private messages

**300 Send Private Message** → **301 Incoming Private Message**
Initiator: Client. Privilege: Send Messages (32). **[S `FUN_0804d2d0`; O]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | target user ID |
| 0x04 | 0xc | |
| 0x10 | | text |

The payload must be at least 0x11 bytes. The server forwards the same layout
to the target as 301 Incoming Private Message, with the sender's ID in place
of the target's.

### Files

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
| 0x20 | 4 | attributes | 1 for a file, 0x01000002 for a folder [O]; the server clears bit 24 and tests 0x42 [S] |
| 0x24 | 0xa | | |
| 0x2e | 2 | | 0xffff for folders, 0 for a file |
| 0x30 | 2 | folder icon | the folder access entry's icon: 0x2c Dropbox, 0x2b Uploads; 0 for a file (see 1100 Create Folder Access) [S] |
| 0x32 | 2 | | |
| 0x34 | 1 | | 4 for `Admin Dropbox [DB]`, 5 for `Uploads [UL]`, 0 for a file |
| 0x35 | 1 | name length | |
| 0x36 | 1 | kind length | 0 for folders; `exttype/bin` for a `.bin` file, as in 413 Download File (R) |
| 0x37 | 1 | | |
| 0x38 | | name, kind | |

[O] `captures/kdx-cli.pcapng`: a 600000-byte `cli-big.bin` is listed with
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
| 0x26 | 1 | disk type | 1 RAM, 2 Network, 3 Cartridge, 4 Hard Disk, 5 CD, 6 DVD, 7 Floppy; 0 on Linux [O] |
| 0x2c | 4 | | 0x88 [O] |
| 0x30 | 4 | icon | 0x3d [O] |
| 0x44 | 4 | flags | bit 0 name editable, bit 1 comments editable |
| 0x48 | 4 | flags mask | the client ANDs it with 0x44 |
| 0x4e | 1 | name length | |
| 0x50 | 1 | format length | e.g. the file system name |
| 0x51 | 1 | comments length | |
| 0x54 | | name, format, comments | `/tmp` and `tmpfs` at 51 [O] |

On Linux the sizes and item count are all ones and the times are 0 [O].
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

#### Transfer connections

Files move on a second connection to the same port, attached to a user who
is already logged in. **[S `FUN_0805270c`, `FUN_080528c8`, `FUN_08052194`;
O kdx-probe `--attach`]**

1. The client connects and runs the normal handshake with a special login:
   - the login name is 5 bytes: `00`, then the u32 user ID from 105 Server
     Info offset 0x24;
   - the account key is the 32 bytes at 105 Server Info offset 0x00, not a
     password hash.
2. The server accepts it only if that user is logged in, is not itself an
   attached connection, and connects from the same IP address. The attached
   connection gets its own user ID, but it acts with the parent's
   privileges and base folder. The server log calls it a transfer, and on
   close it logs `Transfer Disconnect` **[O]**.
3. On this connection, the server handles only 400 Get File List, 402
   Create Folder, 412 Download File, 415 Upload File, and 900, 904, 905
   (view display). It ignores everything else, including 108 Ping **[S, O]**.
4. One transfer runs at a time per connection. A second 412 Download File or
   415 Upload File during a transfer gets 1 Status 0x10003. Afterwards the
   connection takes requests again **[S]**.

The stream is encrypted unless the client asks for plain mode (request
offset 0x17 bit 0, which needs privilege bit 65, probably Disable
Encryption [I]). Encrypted data is framed exactly like a message: a header
whose length gives the byte count, followed by that many bytes in 16-byte
Twofish blocks. A client can read it with its normal receive path **[O]**.
Plain mode has not been tried.

**412 Download File**
Initiator: Client, on a transfer connection. Privilege: Download (10).
**[S `FUN_08050984`, `FUN_080512b4`; O]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | data fork offset | where to resume |
| 0x08 | 8 | resource fork offset | |
| 0x10 | 7 | | |
| 0x17 | 1 | options | bit 0: plain (unencrypted) transfer |
| 0x18 | | file path | |

The payload must be at least 0x1a bytes. Each offset is rounded down to a
multiple of 0x1000. An offset at or past the end still gets its 414
Download File (D), with length 0. [O]: offset 5000 resumed at 4096, and the
65904 bytes that came back equal the full download from 0x1000; offset
70000 on the 70000-byte file got an empty 414 (`captures/probe.pcapng`
frames 271–281, 383–389, and 348–351; test `downloads`).

A file over 0x40000 bytes is refused with code 0x640003 while the same
user already has a download running, unless the user has privilege bit 25
(probably Allow Simultaneous Downloads [I]).

The server answers with **413 Download File (R)**, 99 bytes for a plain
Linux file:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | data fork size | 0x11170 = 70000 |
| 0x08 | 8 | resource fork size | |
| 0x10 | 8 | 0 | |
| 0x18 | 8 | created | |
| 0x20 | 8 | modified | |
| 0x28 | 0xc | | 0, 0, 1 |
| 0x34 | 0x14 | 0 | |
| 0x48 | 1 | name length | |
| 0x49 | 1 | second string length | |
| 0x4a | 1 | third string length | |
| 0x4b | 1 | fourth string length | at most 0x41 |
| 0x4c | | name, strings 2–4 | `probe-dl.bin`, `exttype/bin`, empty, empty [O] |

The second string is probably a type that the server derives from the file
extension, since Linux files have no type code [I]. Then it streams:

1. **414 Download File (D)** with the request's txid and the reply flag.
   The length is the data-fork bytes that remain. The data follows.
2. **414 Download File (D)** with the server's own txid and no reply flag.
   The length is the resource-fork bytes that remain (0 here). The data
   follows.

One 414 header covers the whole fork, even past the limits on
normal payloads: a 600000-byte file came as a single 414 of length 600000,
byte for byte, and a resume from 300000 as one of 300992 from 299008
(`captures/kdx-cli.pcapng` frames 133 and 190) **[O]**. Data is read in
chunks starting at 0x10000 and growing to 0x80000. With a
speed limit, chunks are multiples of 0x1000. At the end the server writes a
188 Finished File Download history record.

**419 Notify Speed Limit**
Initiator: Server. **[O]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | speed limit, in bytes per second |

It came on the session, not on the transfer connection, when a download
started under the total outgoing speed limit. [O]: `00 00 50 00`, the
20480 bytes per second set in the server settings, came right after the 413
(`captures/linux-server-settings.pcapng` frames 106, 110).
`kdx_proto::msg::transfer::NotifySpeedLimit` re-encodes it.

**415 Upload File**
Initiator: Client, on a transfer connection. Privilege: Upload (11).
**[S `FUN_08051750`, `FUN_08051dec`; O]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | data fork size | |
| 0x08 | 8 | resource fork size | |
| 0x10 | 7 | | |
| 0x17 | 1 | options | bit 0: plain (unencrypted) transfer |
| 0x18 | 0x2e | | |
| 0x46 | 2 | path length | of the destination folder path |
| 0x48 | 1 | name length | nonzero |
| 0x49 | 1 | second string length | |
| 0x4a | 1 | third string length | |
| 0x4b | 1 | fourth string length | |
| 0x4c | | name, strings 2–4, folder path | |

The server writes to a temporary name until the upload finishes. If the
name already exists, it fails with 0x30073. An earlier partial upload of
the same name is resumed. At the end, the server applies the second to
fourth strings to the file. They probably carry type and comment
information [I].

The server answers with **416 Upload File (R)**, 24 bytes:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | bytes already on the server | from a partial upload; the server resumes at this value rounded down to 0x1000 |
| 0x08 | 8 | resource fork | always all ones; resource forks don't resume |
| 0x10 | 8 | 0 | |

The client then sends:

1. A **417 Upload File (D)** header. Its length must equal the data-fork
   bytes that remain, or the server fails with 0x40074. The data follows.
2. A **417 Upload File (D)** header for the resource fork, then its data.
   This build expects length 0 here [S].

When both forks are complete, the server renames the file into place,
writes a 189 Finished File Upload history record, and sends an empty
**418 Upload File (C)** with its own txid **[O]**. [O]: 100 bytes uploaded
to `Uploads [UL]` arrived byte for byte, and so did 300000 bytes sent under
one 417 header (`captures/kdx-cli.pcapng` frame 244).

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
| 0x34 | 1 | 0x10 for a file [O] |
| 0x36 | 1 | name length |
| 0x38 | | name, then the item's full file path (root first) |

Items the user can't see through folder access are left out. No hit gives
an empty 453.

Frame 29 of `captures/linux-catalog-volumes.pcapng` has one hit for `a.txt`:
size 6, attributes 1, name length 0, and a path length of 0x16 for the
path `lab-files`, `a.txt` **[O]**. So the name can be empty, with the item's
name only in its path. `kdx_proto::msg::file` re-encodes 420 and 450 to 453
byte for byte (test `linux_catalog_volumes`).

### Accounts

**505 Get Account Class List** → **506 Account Class List**
Initiator: Client. Any one of View Folder Access (27), Create/Modify Chats
(37), View Accounts (50), or Create/Modify Newsgroups (73), since every
class-list editor uses it [I]. **[S `FUN_0805572c`; O]**

Reply: a u32 count (12 classes in the lab), then one record per class,
padded to 4 with zeros. A record is the 503 Account Class Info record
without the 8 bytes at 0x28 **[O probe.pcapng, port 50770]**:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 0x10 | privileges | 11 bytes used (see Access Privileges) |
| 0x10 | 8 | time | |
| 0x18 | 0x10 | | 0 |
| 0x28 | 1 | class index | 0 to 11, in list order |
| 0x29 | 3 | | `ff 00 00` for Administrator, `00 00 ff` for User, else 0 |
| 0x2c | 1 | name length | |
| 0x30 | | name | |

[O] privilege bytes:

- Admin: `28 31 e8 5b f4 d0 3c 0d c3 e0 80`
- User: `28 30 00 03 f0 c0 04 08 03 20`
- Guest: `28 30 00 03 80 00 00 08 02`

**502 Get Account Class** → **503 Account Class Info**
Initiator: Client. Privilege: View Accounts (50). **[S `FUN_080552b8`; O]**

The request is a u8 class index. The reply is the stored class record,
padded to 4:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 0x10 | privileges | 11 bytes used |
| 0x10 | 8 | time | |
| 0x18 | 0xd | | |
| 0x25 | 1 | flood protection | commands per 2 minutes; 0x50 (Relaxed) for Administrator, 0x28 (Strict) for Guest; see Flood ban [S]; 0x3c (Average) for User, which 504 sets to 0x14 (Very Strict) in `linux-accounts-folder-access` frame 43 [O] |
| 0x26 | 2 | | stored as given [S] |
| 0x28 | 8 | | |
| 0x30 | 1 | class index | 0, 2 |
| 0x31 | 1 | | 0xff for Administrator, 0 for Guest; probably the colour [I] |
| 0x34 | 1 | name length | |
| 0x35 | 1 | second string length | [S] |
| 0x36 | 1 | editor login length | set by the server on 504 [S] |
| 0x38 | | name, second string, editor login | |

The padding after the name is uninitialised (`@`, `.` in the lab) **[O]**.

**509 Get Account** → **510 Account Info**
Initiator: Client. Privilege: View Accounts (50). **[S `FUN_08055e98`; O]**

The request is the account's login, with no length byte. An unknown login
gets 1 Status 0x1000c (`captures/kdx-accounts.pcapng` frame 554). The reply
is the stored account record, with the key replaced by a flag. Its padding is
uninitialised (frame 590) **[O]**.

**Account record.** 507 Create Account, 510 Account Info, and 511 Set
Account share this layout **[S `FUN_08055928`, `FUN_08055ff4`; O]**:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 0x10 | privileges | values of the bits set in the mask; `80 00 00 00 00 00 02 00…` for admin, 0 for new accounts **[S `FUN_080551b0`]** |
| 0x10 | 0x10 | privilege mask | bits the account sets itself; the rest come from its class **[S `FUN_080551b0`]** |
| 0x20 | 0x20 | account key | see below |
| 0x40 | 8 | created | set by the server |
| 0x48 | 8 | modified | set by the server |
| 0x50 | 8 | last login | 0 if never |
| 0x58 | 0x1e | | 0 |
| 0x76 | 1 | class index | must name an existing class, else 0x10002 |
| 0x77 | 3 | | stored as given |
| 0x7a | 1 | login length | |
| 0x7b | 1 | name length | the server filters the name (`FUN_08081ffc`) |
| 0x7c | 1 | third string length | stored as given; 0 in the lab |
| 0x7d | 1 | editor length | see below |
| 0x80 | | login, name, third string, editor | |

The key field:

- 507 Create Account: the new account's key (see
  [handshake.md](handshake.md)). The CLI sends the empty password's key for an
  account with no password.
- 511 Set Account: the new key. All zero keeps the current key.
- 510 Account Info: byte 0 is 1 if the account has a password, i.e. its key
  is not the empty password's key (`FUN_08066a90`), and 0 otherwise. The
  rest is zero (frames 253 and 590).

The editor field:

- In a request it is ignored by 507, and in 511 it names the account to
  change.
- The server stores the requester's login there. 510 shows `admin` for the
  accounts admin created (frame 253). The setup-created `admin` account shows
  an empty editor.

**507 Create Account**
Initiator: Client. Privilege: Create Accounts (51). **[S `FUN_08055928`; O]**

The request is an account record. The server sets the times and the
editor, then replies with 1 Status success 0x1fb (frame 235). Errors:

- 0x1000d if the login exists (frame 289);
- 0x10002 for an invalid login (`FUN_08057f88`) or class;
- 0x640005 when the account would hold a privilege the requester lacks
  (`FUN_080551b0`, below).

The privilege check (`FUN_080551b0`) **[S]**: the account's effective
privileges are its class's bits, with the bits in the mask taken from the
record instead. Bits 3 and 36 are ignored, and so is bit 23 if the requester
has bit 24. The request fails if any remaining bit is one the requester lacks.
A requester with bit 0 passes.

**511 Set Account**
Initiator: Client. Privilege: Modify Accounts (52). **[S `FUN_08055ff4`; O]**

The request is an account record whose editor field holds the current login.
If the login field differs, the server renames the account, failing with
0x1000d if the new login exists. It keeps the creation and last-login times,
sets the modified time, and replies with 1 Status success 0x1ff (frames
412–413). Users logged in to the account are updated in place with the new
login and privileges.

Errors:

- 0x1000c for an unknown account;
- 0x10014 if the account has Protected Account (54) set in both the mask
  and the values and the requester lacks privilege 0;
- the same checks as 507 Create Account otherwise.

**508 Delete Account**
Initiator: Client. Privilege: Modify Accounts (52). **[S `FUN_08055d50`; O]**

The request is the login, with no length byte. On success the server sends
**515 Account Deleted**, whose payload is the same login, to the requester
as a notification (its own txid, no reply flag). It sends no reply (frames
607–610).

Errors:

- 0x1000c for an unknown login (frame 630);
- 0x10014 without the privilege (frame 391);
- 0x10014 if the target has privilege 0 set in both its mask and its
  values;
- 0x10014 if the target has Protected Account (54) set in both and the
  requester lacks privilege 0.

**550 Change Own Password** → **551 Change Own Password Done**
Initiator: Client. Privilege: Change Own Password (53). **[S `FUN_08056a04`;
O]**

The request is the 32-byte key of the new password, not the password. The
server stores it as the account's key and replies with an empty 551
(frames 323–324). The old password stops working at once.

**512 Get Account List** → **513 Account List**
Initiator: Client. Privilege: View Accounts (50). **[S `FUN_080566bc`; O]**

The request is empty. Reply: a u32 count, then one entry per account, each
padded to 4:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | created | |
| 0x08 | 8 | last login | 0 if never |
| 0x10 | 0xb | | |
| 0x1b | 1 | | 1 for admin, 0 for guest |
| 0x1c | 4 | | |
| 0x20 | 1 | class index | 0 for admin, 2 for guest |
| 0x21 | 1 | | 0xff for admin; matches 503 Account Class Info offset 0x31 |
| 0x24 | 1 | login length | |
| 0x25 | 1 | name length | |
| 0x28 | | login, name | |

**504 Set Account Class**
Initiator: Client. Privilege: Modify Accounts (52). **[S `FUN_080553c8`; O]**

The request is a 503 Account Class Info record (at least 0x38 bytes) with
the class index at 0x30. Client 1.620 for Linux sends it unpadded, with the
time 0 and the second string and editor login empty (69 bytes for
Administrator, `captures/linux-catalog-volumes.pcapng` frames 40 and 59;
61 bytes for Guest, `captures/linux-accounts-news.pcapng` frames 20 and 30).
The server filters the name (`FUN_08081ffc`), writes the requester's login
as the editor, sets the time at 0x10, and saves the class. Privileges the
requester lacks can't be granted (`FUN_080551b0`; failure 1 Status
0x640005). Success is 1 Status 0x1f8 (frames 43 and 62); the history record
0x1f8 carries the old class name. Logged-in users of that class get the new
privileges at once (`FUN_08054900`).

**514 Account Changed**
Server 1.620 never sends it and Client 1.620 beta 5 doesn't handle it
**[S]**.

### News

**605 Get Newsgroup List** → **606 Newsgroup List**
Initiator: Client. Privilege: View News (70). **[S `FUN_0805d038`; O]**

Reply: a u32 count, then one entry per group:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | time | |
| 0x08 | 4 | group ID | |
| 0x0c | 4 | | |
| 0x10 | 2 | message count | 1 [I] |
| 0x12 | 4 | | |
| 0x16 | 1 | name length | |
| 0x17 | 1 | | 0, probably the description length [I] |
| 0x18 | | name | `General News` |

**602 Get Newsgroup** → **603 Newsgroup Info**
Initiator: Client. Privilege: Create/Modify Newsgroups (73). **[S
`FUN_0805cc24`; O 76 bytes; O `captures/linux-accounts-news.pcapng` frames
68-69]**

The request is a u32 group ID. An unknown group gets 1 Status 0x1000c. The
reply is not padded: frame 69 of `captures/linux-accounts-news.pcapng` is 82
bytes, for the 14-byte name `test newsgroup` and the description `desc`
**[O]**. The record is:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | group ID | |
| 0x04 | 0xc | | 0 |
| 0x10 | 8 | time of the last post | 611 Post News Message sets it |
| 0x18 | 4 | read classes | a class bitmap, class 0 in the top bit; `ffffffff` |
| 0x1c | 4 | post classes | the same; `ffffffff` |
| 0x20 | 4 | | a third class bitmap; `01 00 00 00`; the client sends 0 [S] |
| 0x24 | 0xc | | 0 |
| 0x30 | 2 | message count | 1 |
| 0x32 | 2 | archive size | KiB, clamped to 4..1024; 0x32 (50 KiB) by default [S `FUN_0805c8f8`] |
| 0x34 | 8 | | 0 |
| 0x3c | 1 | name length | at most 0x3f |
| 0x3d | 1 | description length | at most 0x7f; 0 in the lab |
| 0x3e | 2 | | 0 |
| 0x40 | | name, description | `General News` |

The read and post class bitmaps are the ones 607 Get News Messages and 611
Post News Message check **[S]**.

**607 Get News Messages** → **608 News Messages**
Initiator: Client. **[S `FUN_0805d2e8`; O]**

The request is a u32 group ID. Reply:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | group ID | |
| 0x04 | 4 | permissions | 1 post, 2 delete any message, 4 read; 7 for admin, 4 for guest |
| 0x08 | 4 | | |
| 0x0c | 4 | count | |
| 0x10 | | news message records | same as 610 News Message |

The server computes the permissions from the user's privileges and the
group's class bitmaps **[S `FUN_0805d2e8`; O `captures/kdx-news.pcapng`
frames 77 and 204]**. Without read access but with post access, the reply is
just the 16-byte header with permissions 1 and count 0 **[S]**.

**609 Get News Message** → **610 News Message**
Initiator: Client. **[S `FUN_0805d54c`; O]**

Request: {u32 group ID, u32 message ID}. 609 Get News Message does not reset
the idle timer.

The reply is one news message record, padded to 4:

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

**611 Post News Message**
Initiator: Client. Privilege: Post News (71), and the user's class must be in
the group's post classes unless the user has privilege 0. **[S
`FUN_0805d758`; O `captures/kdx-news.pcapng`]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | group ID | |
| 0x04 | 8 | | ignored by the server [S] |
| 0x0c | 2 | body length | the server keeps at most 0x4000 bytes |
| 0x0e | 1 | subject length | the server keeps at most 0x80 bytes |
| 0x0f | 1 | | ignored by the server [S] |
| 0x10 | | subject, body | lines end in CR |

There is no reply on success (frames 356–361). The server stores a 610 News
Message record, with the user's nick and login, under the group's next
message ID, then sends 613 News Message Posted to every user whose class may
read the group, the poster included. Errors **[O]**: 0x1000c for an unknown
group (frame 160) and 0x10014 for a guest (frame 182).

**612 Delete News Message**
Initiator: Client. **[S `FUN_0805db44`; O]**

Request: {u32 group ID, u32 message ID}. The user's class must be in the
group's read classes unless the user has privilege 0. Without Delete News
Messages (72), the user may delete only the group's last message, and only
if they posted it (the login at 0x1d matches); otherwise 1 Status 0x10014
(frame 226, a guest).

There is no reply on success. The server broadcasts 614 News Message Deleted
to every user (frames 409–411). An unknown group or message gets no reply
and no 614 (frame 435).

**613 News Message Posted**, **614 News Message Deleted**
Initiator: Server. **[S; O]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | group ID |
| 0x04 | 4 | message ID |
| 0x08 | 4 | messages in the group afterwards |

**600 Create Newsgroup**
Initiator: Client. Privilege: Create/Modify Newsgroups (73). **[S
`FUN_0805c8f8`, client `FUN_08089034`; O
`captures/linux-accounts-news.pcapng` frames 58-63]**

The request is a 603 Newsgroup Info record. The server ignores the ID at
0x00 and picks one. The name must be 1..0x3f bytes and the description at
most 0x7f. Success is 1 Status 0x258, and a history record. The client's
editor has Name, Description, Archive Size, and the class access list; it
sends 600 or 604, then 605 Get Newsgroup List.

Frame 58 creates `test newsgroup` (description `desc`, archive size 0x1a4,
read and post classes `80000000`, so Administrator only). It is 82 bytes,
unpadded, with the ID, time, message count, and the bitmap at 0x20 all 0.
The server gives it ID 2 (frame 63) **[O]**. `NewsgroupInfo` re-encodes 600,
603, and 604 byte for byte (test `linux_accounts_news`).

When a group's stored messages reach 1.5 times the archive size, the server
moves the oldest ones into a file under `News Archives` named after the
group **[S `FUN_0805ed88`]**.

**601 Delete Newsgroup**
Initiator: Client. Privilege: Create/Modify Newsgroups (73). **[S
`FUN_0805caf4`; O `captures/linux-accounts-news.pcapng` frames 81-84]**

The request is a u32 group ID. An unknown group gets 0x1000c. Success is
1 Status 0x259 (frame 84). 602 Get Newsgroup also writes history record 0x25a.

**604 Set Newsgroup**
Initiator: Client. Privilege: Create/Modify Newsgroups (73). **[S
`FUN_0805ce60`; O `captures/linux-accounts-news.pcapng` frames 71-74]**

The request is a 603 Newsgroup Info record with the group ID at 0x00.
Success is 1 Status 0x25c (frame 74). Frame 71 renames group 2 to `Lab
Group`, with the description `edited description` and archive size 0x32. It
is 91 bytes, unpadded **[O]**.

### Processes

**800 Open Process Monitor** / **801 Close Process Monitor**
Initiator: Client. Privilege: View Processes (80). **[S `FUN_0805b394`,
`FUN_0805b418`]**

**802 Get Process List** → **803 Process List**
Initiator: Client. Privilege: View Processes (80). **[S `FUN_0805b454`; O]**

The request is empty. The server runs `/bin/ps auxwww` (its own child shows
up in the list) and replies:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | | about 33 GB; host memory [I] |
| 0x08 | 8 | | about 33 GB [I] |
| 0x10 | 0xc | | |
| 0x1c | 4 | count | |
| 0x20 | | process entries | |

Each process entry is padded to 4:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | PID | |
| 0x04 | 4 | | |
| 0x08 | 8 | start time | |
| 0x10 | 0x29 | | |
| 0x39 | 1 | user length | |
| 0x3a | 1 | command length | at most 0xff; longer commands are cut |
| 0x3c | | command, user | |

**804 Exit/Terminate Process**
Initiator: Client. Privilege: Terminate Processes (81). **[S
`FUN_0805b8f4`, `FUN_08090c6c`]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | flags | bit 0 kill, bit 1 exit, bit 2 extra flag; one of bits 0 and 1 is required, else 0x10002 |
| 0x04 | 4 | PID | |
| 0x08 | 4 | | must be 0, or nothing happens |

On Linux, kill sends SIGKILL and exit sends SIGTERM to the PID. Bit 2 gets
0x10005 (unsupported) on Linux. A failed `kill()` returns the OS error.
There is no reply on success; the server logs `Terminate Process` and
writes history record 0x324 **[S]**. The client's Exit Process sends flags
2 (exit) **[O `captures/linux-shutdown.pcapng` frame 25]**.
`kdx_proto::msg::admin::ExitTerminateProcess` re-encodes it.

### Folder access

A folder access item is a named set of rules. A folder names its item with
a ` [tag]` suffix, such as `Uploads [UL]`. Item `[Default]` applies when a
folder and its parents name no item. The u16 at offset 0x30 of a 401 File
List entry is the item's icon, not an ID **[S]**.

**1106 Get Folder Access List** → **1107 Folder Access List**
Initiator: Client. Privilege: View Folder Access (27). **[S `FUN_080578d8`;
O]**

The request is empty or a u32. If the u32's top byte has bit 0 set, a user
without the privilege gets no reply instead of 1 Status 0x10014. Client
1.620 for Linux sends `00000001` when it opens the file window and an empty
request after 1100 (`captures/linux-files.pcapng` frames 23 and 232) **[O]**.

Reply: a u32 count, then one entry per item, each padded to 4:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | flags | 1 built-in (`[Default]`), 2 Owner Login In Folder Name; 1, 2, 0 in the lab [S] |
| 0x04 | 4 | icon | 0 for `[Default]`, 0x2c Dropbox (`DB`), 0x2b Uploads (`UL`) [S] |
| 0x08 | 8 | | not written by the server |
| 0x10 | 1 | | |
| 0x11 | 1 | name length | |
| 0x12 | 1 | second string length | 0 in the lab |
| 0x13 | 1 | description length | |
| 0x14 | | name, second string, description | `[Default]`, empty, `If a folder and its parents…` |

**Folder access record**
**[S `FUN_080572f0`, `FUN_08057634`, client folder access editor; O
`captures/linux-files.pcapng` frames 226, 229, 231;
`captures/linux-accounts-folder-access.pcapng` 1105;
`captures/linux-folder-access-guest.pcapng` 1105]**

1100, 1104, and 1105 share this record. Its total length, padded to 4, must
equal the payload length. `kdx_proto::msg::admin::FolderAccess` re-encodes
the three captured records byte for byte, padding included (test
`linux_files`).

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | | |
| 0x04 | 4 | flags | byte 7: bit 0 built-in, bit 1 Owner Login In Folder Name |
| 0x08 | 4 | icon | 0x28 Normal, 0x2b Uploads, 0x2c Dropbox, 0x34 Pictures, 0x35 Audio |
| 0x0c | 0x80 | class rules | 32 entries at 0x0c + 4 × class index: u16 permission bits, MSB = bit 0, then u16 0. User (class 1) is at 0x10 and Guest (class 2) at 0x14 [O]. The first, 0xfffe0000 (every permission) in both lab items, is probably Administrator's [I] |
| 0x8c | 4 | | 0xfffe0000 in `DB`, 0 in `[Default]`; probably the owner's rule [I] |
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

**1100 Create Folder Access**
Initiator: Client. Privilege: Modify Folder Access (28). **[S
`FUN_080572f0`; O `captures/linux-files.pcapng` frames 231-234]**

The request is a folder access record. An invalid name gets 0x1001e, an
existing one 0x1000d. Success is 1 Status 0x44c. Frame 231 creates `test`
with every other field 0, and frame 234 is the success.

**1101 Delete Folder Access** → **1102 Folder Access Deleted**
Initiator: Client. Privilege: Modify Folder Access (28). **[S
`FUN_08057460`; O `captures/linux-accounts-folder-access.pcapng` frames
70–71]**

The request is the item name, with no length byte. An unknown name gets
0x1000c. There is no Status reply: the server sends 1102, payload the name,
to the requester and writes history record 0x44d. 1102 carries the server's
own next txid, not the request's (`f2cfecc3` at frame 71) **[O]**.

**1103 Get Folder Access** → **1104 Folder Access Info**
Initiator: Client. Privilege: View Folder Access (27). **[S
`FUN_08057534`; O `captures/linux-files.pcapng` frames 225-229]**

The request is the item name, with no length byte (`DB` at 225, `[Default]`
at 228). The reply is the stored folder access record. After a 1105, that is
the 1105's record as sent, with its current name
(`captures/linux-folder-access-guest.pcapng` frames 30, 38) **[O]**.

**1105 Set Folder Access**
Initiator: Client. Privilege: Modify Folder Access (28). **[S
`FUN_08057634`; O `captures/linux-accounts-folder-access.pcapng` frames
63, 66]**

The request is a folder access record whose current name (0xb1) picks the
item; it is renamed if the name differs. Success is 1 Status 0x451. The
client sends the name and the current name even when they are equal: frame
63 is 200 bytes, with `lab-item` twice and length 8 at 0xad and 0xb1. It
sets byte 0x14 to 0x80, bit 0 (Can See Folder) of the Guest rule **[O]**.
Ticking Can See Folder for Guest alone and then for User alone sets 0x14
and then 0x10 (`captures/linux-folder-access-guest.pcapng` frames 30, 40)
**[O]**.

### View display

View display shares the server's screen. It runs on a transfer connection
(see Transfer connections), not on the session **[S `FUN_08052194`]**.
Server 1.620 for Linux can't capture its screen (`FUN_080996d8` raises
0x10004), so 900 there only gets an error, and its input injectors are
empty. The layouts below are read from the server's code and the client's
decoder **[S]**. Server 1.600 for Windows under Wine serves it to Client
1.620 beta 5 in `captures/win-view-display.pcapng` **[O]**.

**900 Open View Display** → **902 View Display Params**, **903 View
Display Update**
Initiator: Client, on a transfer connection. Privilege: View Display (66).
**[S `FUN_0805c0f0`, client `FUN_0806f9d8`]**

The request is empty. A connection already busy gets 0x10003. The reply is
902, then a full 903 frame. After that the server sends a 903 to every
viewer every 1500 ms when the screen has changed. 901 Close View Display has
no handler: the client closes the connection, and sends no 901 (frame 420)
**[O]**.

902 View Display Params is 0x10 bytes: u32 flags, bit 0 = the viewer may
control the display (Control Display, 67). The rest is 0. Admin gets 1
(frame 107) **[O]**.

903 View Display Update:

| Off | Size | Field |
|---|---|---|
| 0x00 | 2 | screen width |
| 0x02 | 2 | screen height |
| 0x04 | 2 | left of the changed rectangle |
| 0x06 | 2 | top |
| 0x08 | 2 | right (exclusive) |
| 0x0a | 2 | bottom (exclusive) |
| 0x0c | | pixel stream for the whole screen, row by row |

A pixel is a u16 RGB555 (0RRRRRGGGGGBBBBB) **[S `FUN_0805bb54`]**. The
value 0x6c59 means unchanged; a real 0x6c59 is sent as 0x6c5a. A frame
after the first marks every unchanged pixel this way. A new screen size
starts a fresh full frame.

The stream is a sequence of codes, read as u16 words **[S server
`FUN_0805ba44`, `FUN_0805c5bc`; client `FUN_0806f36c`]**:

- A word with bit 15 clear is one pixel.
- A word with bit 15 set gives colour A = word & 0x7fff, and the next word
  C decides the rest:
  - C bit 15 clear: a run of C pixels of A.
  - C bit 15 set: a third word D gives colour B = D & 0x7fff, and
    n = ((C >> 12) & 7) | ((D >> 15) << 3).
    - n > 0: n pixels (4..12) from the mask C & 0xfff, highest used bit
      first; 1 = B, 0 = A.
    - n = 0: count = C & 0xfff, then a bitmap of ((count + 15) / 16) × 2
      bytes, MSB first; 1 = B, 0 = A.

The server uses runs for 10 or more equal pixels, the two-colour codes for
spans of 4 or more pixels with only two colours, and single pixels
otherwise.

Wine's black 3840×2160 screen comes as one full 903 (frame 108): width
0x0f00, height 0x0870, the rectangle 0, 0, 0x0f00, 0x0870, then 253 runs
`8000 7fff` and one `8000 10fd`, 3840 × 2160 pixels in all. The screen
never changed, so no later 903 came **[O]**.

A viewer whose send queue passes 0x800000 bytes is dropped with 0x40076.

**904 View Display Mouse**
Initiator: Client, on the view display connection. Privilege: Control
Display (67). **[S `FUN_0805c344`, client `FUN_0806fc68`]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | modifiers | from the client's event [I] |
| 0x04 | 2 | x | |
| 0x06 | 2 | y | |
| 0x08 | 2 | | from the client's event; wheel or click count [I]; 0 for clicks and moves [O] |
| 0x0a | 1 | button | |
| 0x0b | 1 | kind | 1 down, 2 up, 3 move, 4 wheel [I from the client's event codes 9, 10, 11/13, 14] |

A click is kind 1, then kind 2, both with button 1; moves have button 0
(frames 307–310) **[O]**. Other kinds are ignored. After kind 2, the server
sends the next 903 within 100 ms. The client sends nothing when 902 denied
control, and shows "You do not have access to control" on a click.

**905 View Display Keyboard**
Initiator: Client, on the view display connection. Privilege: Control
Display (67). **[S `FUN_0805c498`, client `FUN_0806f1f4`]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | modifiers | [I] |
| 0x04 | 2 | character | [O] |
| 0x06 | 2 | key code | [O]; `a` = 4, `b` = 5, `c` = 6 |
| 0x08 | 3 | 0 | |
| 0x0b | 1 | kind | 1 key down, 2 key up, 3 key repeat [I from the client's event codes 5, 6, 7] |

Typing `abc` sends `0061 0004`, `0062 0005`, `0063 0006` with kind 1, each
followed by a kind 2 that is zero apart from the kind (frames 312–322)
**[O]**. After kind 1, the server sends the next 903 within 100 ms.

### Not yet documented

- The tracker: 700–721 and UDP registration are read statically but not
  captured. See [tracker.md](tracker.md).
- Client-to-client (DCC) messages 117, 300 on DCC, and 1000-1003 are read
  statically but not captured. See [dcc.md](dcc.md).

# Error Codes

The server's name table is `FUN_08066d04`. It has 79 entries: the codes are
at 0x080b1dc0 and the text pointers at 0x080b1f00 **[S]**. The high 16 bits
give the class.

| Code | Meaning |
|---|---|
| 0x10001 | unknown error |
| 0x10002 | invalid value |
| 0x10003 | protocol breach |
| 0x10004 | unimplemented |
| 0x10005 | unsupported |
| 0x10006 | timed out |
| 0x10007 | aborted |
| 0x10008 | data version incompatible |
| 0x10009 | data format incompatible |
| 0x1000a | corrupt data |
| 0x1000b | out of range |
| 0x1000c | item does not exist |
| 0x1000d | item already exists |
| 0x1000e | limit cannot be exceeded |
| 0x1000f | locked |
| 0x10010 | data file missing |
| 0x10011 | resource missing |
| 0x10012 | process does not exist |
| 0x10013 | IPC queue full |
| 0x10014 | Access Denied (a missing privilege) |
| 0x10015 | hardware error |
| 0x10016 | hardware busy |
| 0x10017 | unknown view class |
| 0x10018 | view does not exist |
| 0x10019 | invalid object |
| 0x1001a | buffer too small |
| 0x1001b | unknown OS error |
| 0x1001c | not initialized |
| 0x1001d | re-entry not allowed |
| 0x1001e | invalid name |
| 0x20064–0x20066 | memory errors |
| 0x30064–0x30065 | file system errors |
| 0x30066 | file, folder, or item missing |
| 0x30067 | file missing |
| 0x30068 | folder missing |
| 0x30069 | alias missing |
| 0x3006a | expected a file |
| 0x3006b | expected a folder |
| 0x3006c | expected an alias |
| 0x3006d | alias target missing |
| 0x3006e | read-only volume |
| 0x3006f | volume closed |
| 0x30070 | no space |
| 0x30071 | volume limit |
| 0x30072 | wrong passphrase |
| 0x30073 | name already exists |
| 0x30074 | end of file |
| 0x30075 | file is open |
| 0x30076 | invalid name |
| 0x30077 | access denied by the file system |
| 0x30078 | cross-volume operation |
| 0x30079 | folder too deep or alias loop |
| 0x3007a | volume missing |
| 0x40064–0x4006f | network errors |
| 0x40070 | Incorrect login and/or password. |
| 0x40071 | The network protocol is unknown. |
| 0x40072 | the peer's protocol is too old |
| 0x40073 | the peer's protocol is more recent |
| 0x40074 | Received data that was invalid according to the protocol. |
| 0x40075 | A limit in the net protocol was exceeded. |
| 0x40076 | can't send fast enough |
| 0x40077 | Banned for flooding. |
| 0x50064–0x50066 | sound errors |

Codes outside the table also occur. 507 Create Account and 511 Set Account
raise 0x640005 when the account would hold a privilege the requester lacks
**[S `FUN_080551b0`]**.

# Access Privileges

An account's privileges are a bit string of at least 11 bytes. It is stored
at user +0x44 and sent in the class records (506 Account Class List). Bit n
is byte n/8, mask 0x80 >> (n % 8) **[S]**.

Bits that server handlers test, found by scanning every handler for tests of
user +0x44..+0x4e **[S]**. `FUN_0804b114` is the post-login step that sends
105 Server Info, 107 Server Greeting, and 213.

| Bit | Privilege | Tested in |
|---:|---|---|
| 0 | override: bypasses chat class lists and news/account restrictions; not in the client's list | 202 Join Chat, 207 Send Chat Text, 209 Set Chat Topic, 400 Get File List, 508 Delete Account, 511 Set Account, 607 Get News Messages, 609 Get News Message, 611 Post News Message, 612 Delete News Message |
| 3 | Color Overrides Class | `FUN_08054900` account lookup |
| 4 | Can Login | set in every lab class |
| 10 | Download | 412 Download File |
| 11 | Upload | 402 Create Folder, 415 Upload File |
| 13 | Empty Trash | 421 Empty Trash |
| 15 | Create Aliases | 403 Create Alias |
| 16 | Move | 407 Move File/Folder |
| 17 | Rename | 409 Get File/Folder Info, 411 Set File/Folder Info |
| 18 | Delete | 405 Move To Trash |
| 20 | Set File/Folder Attributes | 409 Get File/Folder Info, 411 Set File/Folder Info |
| 22 | See All Volumes | 400 Get File List, 423 Set Volume Info |
| 23 | Has Custom Base Folder | |
| 24 | See All Bases | |
| 25 | Allow Simultaneous Downloads | 412 Download File (large files) |
| 26 | Ignore Speed Limits | |
| 27 | View Folder Access | 505 Get Account Class List, 1103 Get Folder Access, 1106 Get Folder Access List |
| 28 | Modify Folder Access | 402 Create Folder, 403 Create Alias, 411 Set File/Folder Info, 1100 Create Folder Access, 1101 Delete Folder Access, 1105 Set Folder Access |
| 30 | Join Chat | 202 Join Chat, 214 Get Chat List, login |
| 31 | Send Chat | 207 Send Chat Text |
| 32 | Send Messages | 300 Send Private Message |
| 33 | Broadcast | 154 Send Broadcast Message |
| 34 | Create Private Chat | 200 Create Private Chat |
| 35 | Set Chat Topic | 209 Set Chat Topic |
| 36 | Join Chat On Connect | login |
| 37 | Create/Modify Chats | 201 Delete Chat, 217 Create Chat, 218 Get Chat, 220 Set Chat, 505 Get Account Class List |
| 40 | Get User Info | 140 Open Connection Monitor, 141 Close Connection Monitor, 142 Get Connection Monitor Info, 152 Get User Info |
| 41 | Disconnect Users | 150 Disconnect User |
| 42 | Cannot Be Disconnected | 150 (target) |
| 43 | Ban Users | 150 Disconnect User |
| 50 | View Accounts | 502 Get Account Class, 505 Get Account Class List, 509 Get Account, 512 Get Account List |
| 51 | Create Accounts | 507 Create Account |
| 52 | Modify Accounts | 504 Set Account Class, 508 Delete Account, 511 Set Account |
| 53 | Change Own Password | 550 Change Own Password |
| 54 | Protected Account | 508 Delete Account, 511 Set Account (target) |
| 60 | Use Any Name | 100 Set Identity, login |
| 61 | Modify Server Settings | 113 Get Extra Server Info, 170 Get Server Settings, 172 Set Server Settings, 450 Generate Catalog, login (105 Server Info) |
| 62 | Don't Show Greeting | login |
| 63 | View Server History | 180 Server History Join |
| 64 | Shutdown Server | 174 Shutdown Server, 175 Upgrade Server Software |
| 65 | Disable Encryption |
| 66 | View Display | 900 Open View Display |
| 67 | Control Display | 902 View Display Params, 904, 905 | 412 Download File, 415 Upload File (plain mode) |
| 70 | View News | 605 Get Newsgroup List, 607 Get News Messages, 609 Get News Message |
| 71 | Post News | 607 Get News Messages, 611 Post News Message |
| 72 | Delete News Messages | 607 Get News Messages, 612 Delete News Message |
| 73 | Create/Modify Newsgroups | 505 Get Account Class List, 600 Create Newsgroup, 601 Delete Newsgroup, 602 Get Newsgroup, 604 Set Newsgroup |
| 74 | See Logins In News | 609 Get News Message |
| 80 | View Processes | 800 Open Process Monitor, 802 Get Process List |
| 81 | Terminate Processes | 804 Exit/Terminate Process |
| 82 | Launch Programs | 175 Upgrade Server Software, 420 Launch File |

Bits 2 and 4 (0x44&0x20, 0x44&0x08) are set in every class. Bit 4 is Can
Login; bit 2 has no name in the client's table **[S]**. Answering no to
guest access at the server's first-run setup clears Can Login on the
`guest` account and sets it in that account's mask **[S `FUN_08054394`]**.
The client never appends the 0x17-byte info trailer that sets user flag bit
1 (see User record), so whatever sends it is not Client 1.620 beta 5 **[S
client `FUN_0804d654`]**.

The client's account editor lists 50 privileges in this order **[S
KDXClient.exe resources]**:

1. Download, Upload, Create Aliases, Move, Rename, Delete, Empty Trash, Set
   Attributes, Has Custom Base Folder, See All Bases, See All Volumes, Allow
   Simultaneous Downloads, Ignore Speed Limits
2. View Folder Access, Modify Folder Access
3. Create/Modify Chats, Join Chat, Send Chat, Create Private Chat, Set Chat
   Topic, Join Chat On Connect
4. Send Messages, Broadcast, Get User Info, Disconnect Users, Cannot Be
   Disconnected, Ban Users
5. View Accounts, Create Accounts, Modify Accounts, Change Own Password,
   Protected Account
6. View News, Post News, Delete News Messages, Create/Modify Newsgroups, See
   Logins In News
7. View Processes, Terminate Processes, Launch Programs, View Display,
   Control Display
8. Can Login, Color Overrides Class, Use Any Name, Modify Server Settings,
   Don't Show Greeting, View Server History, Shutdown Server, Disable
   Encryption

The client's editor table at 0x0812c760 (file offset 0xe4760) pairs each
label with its bit number **[S]**, which gives the names in the table
above. The other bits have no label.

# Transaction Sequences

## Login

1. Client and server exchange hellos and proofs
   ([handshake.md](handshake.md)).
2. The client sends **100 Set Identity**.
3. The server sends 101 User List and 105 Server Info, and may send 106
   Server Icon, 107 Server Greeting, and 213 Must Join Chat (see 100 Set
   Identity). Everyone else receives 103 User Joined.
4. The client then asks for whatever it shows. In
   `captures/first-connect.pcapng` the server sends only 101 and 105 (frames
   23–24), and the user's later actions bring 300 Send Private Message
   twice, 605 Get Newsgroup List, and 607 Get News Messages **[O]**.

## Flood ban

After 30 connections from 127.0.0.1 in about a minute, the server logged
"Banned for flooding. (ID 4/119)" and then closed each new connection
before sending its hello (`captures/kdx-accounts.pcapng`, the connections
after frame 467) **[O]**. The ban lasted at least five minutes and ended
when the server restarted.

That limit is in the network layer that Server, Tracker, and Client share
(the client applies it to its DCC listener) **[S server `FUN_08071c90`,
called from accept `FUN_08097e80`; tracker `FUN_0806bbac`; client
`FUN_080ca7cc`]**. Per source address it counts accepted connections in a
window of 300 seconds. The 31st within a window adds the address to a block
list for 3600 seconds and logs 0x40077; while blocked, each new connection
is closed at once. The limits (30 and 300) are defaults that nothing
changes. Unlike the server's own ban list, this one also blocks 127.0.0.1.

Two more limits apply at accept **[S `FUN_08052ad0`]**. An address with 6
connections already open gets its new one closed without a word. And an
address on the server's ban list (bans from 150 Disconnect User, and the
flood bans below) is rejected and logged, except 127.0.0.1, which the ban
list never matches **[S `FUN_08061398`]**.

The server also counts each connection's requests **[S `FUN_0804fc40`,
called per message from the session loop]**. The limit L is the account
class's flood protection byte (503 offset 0x25); a connection with no class
yet gets 40. The client's menu names the values **[S client resources at
file offset 0xf6cd8]**: Very Strict 20, Strict 40, Average 60, Relaxed 80,
Very Relaxed 120, Total Slacker 200, Off 0. With 0 there is no check.

Each request adds 1 to the count, starting with the login's 100 Set
Identity **[O `captures/linux-accounts-folder-access.pcapng`]**. When the
count reaches L, the server
looks at the time since the window started. At 121 seconds or more, it
resets the count and starts a new window. Otherwise:

- at count L, it sends 155 Rcv Broadcast Message "WARNING: The server has
  detected that you are flooding it. If you continue to flood the server,
  you will be banned.";
- at L + 4, "Final warning: If you continue to flood the server, you will
  be banned.";
- above L + 8, it adds a 2-hour ban (7200000 ms) on the address, with the
  reason "Auto-banned this bastard for flooding.", and closes the
  connection with 0x640004 (`FUN_0804a8c0`).

With Very Strict (0x14), a client that logged in and then sent 115 Get My
Address every 300 ms got the WARNING before the reply to its 19th request,
the Final warning before the reply to its 23rd, and no reply to its 28th:
the server closed the connection (frames 147, 161, 178) **[O]**. Each 155 is
16 zero bytes and the text. The server logged "Auto-banned for flooding!"
and "Banned for flooding!". The ban did not block 127.0.0.1: the same
account logged in again 12 seconds later (frame 188) **[O]**.

The server's 1.52 upgrade notes say that flood protection became a
per-class setting then, and is off for classes that predate it.

## Posting news

A client confirms a request that has no success reply, such as 611 Post News
Message, by sending 115 Get My Address after it. The server handles one
connection's requests in order, so an error for the post arrives before 116
Your Address **[O `captures/kdx-news.pcapng` frames 356–361]**.

## Logout

When a connection closes, the other users receive **104 User Departed**, and
the server writes a 185 Disconnected history record.
