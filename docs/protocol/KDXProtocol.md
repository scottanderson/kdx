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
| 10600 | UDP | voice chat; the client advertises its port in the user record [O] |
| tracker | UDP/TCP | server registration and listing (not yet documented) |

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
| 0x0c | 4 | len | payload length, at most 0x20000 |

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
**[S `FUN_080865c0`]**.

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

## Times

Times are u64 milliseconds. The epoch looks like 1900-01-01: 0x3a36c4895ba
fits a date in 2026. This is not yet verified **[I]**.

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
least 6 bytes. Otherwise the server raises 0x1000a. The server joins the
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
| 0x0a | 2 | voice UDP port | 10600 in the capture [O] |
| 0x0c | 2 | icon length | < 0x709 |
| 0x0e | 6 | | from the client; `00 00 00 ff cc 00` in the capture, probably two RGB colours [I] |
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
| 0x2c | 1 | third string length | ≤ 0x7f; probably the topic [I] |
| 0x2d | 1 | fourth string length | ≤ 0x3f |
| 0x30 | | strings | |

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
| 117 | SKPK | ? | |
| 118 | (unnamed) | Client | inline |
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
| 700 | Get Tracker Group List | Client | (tracker) |
| 701 | Tracker Group List | Tracker | |
| 702 | Get Tracker Server List | Client | (tracker) |
| 703 | Tracker Server List | Tracker | |
| 710–713 | Get / Info / Set / Done TrackerServ Settings | | (tracker) |
| 720 | Get TrackerServ Statistics | Client | (tracker) |
| 721 | TrackerServ Statistics | Tracker | |
| 800 | Open Process Monitor | Client | `FUN_0805b394` |
| 801 | Close Process Monitor | Client | `FUN_0805b418` |
| 802 | Get Process List | Client | `FUN_0805b454` |
| 803 | Process List | Server | |
| 804 | Exit/Terminate Process | Client | `FUN_0805b8f4` |
| 900 | Open View Display | Client | |
| 901 | Close View Display | Client | |
| 902 | View Display Params | | |
| 903 | View Display Update | Server | |
| 904 | View Display Mouse | Client | |
| 905 | View Display Keyboard | Client | |
| 1000 | Start Voice | Client | |
| 1001 | Start Voice (R) | Server | |
| 1002 | Set Voice Format | | |
| 1003 | Voice Data | | (UDP) |
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
  record.
- A later 100 Set Identity broadcasts 102 User Changed.

**101 User List**
Initiator: Server. **[S `FUN_0804af04`; O]**

A sequence of user records, one per user who is logged in (all of them on
login).

**102 User Changed** / **103 User Joined**
Initiator: Server. **[S]**

One user record.

**104 User Departed**
Initiator: Server. **[O `captures/kdx-cli.pcapng` frames 398, 429]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | user ID | |
| 0x04 | 1 | | 7 in both samples |
| 0x05 | 3 | 0 | |

**105 Server Info**
Initiator: Server. **[S `FUN_0804b304`; O 86 bytes]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 0x20 | | 32 bytes from the user object (+0x74), probably a session ID for transfer connections [I] |
| 0x20 | 4 | | server value (+0x260) |
| 0x24 | 4 | your user ID | [O] 7 and 8 in successive probes |
| 0x28 | 2 | server version | 0x0654 = 1620 |
| 0x2a | 2 | can modify settings | 1 with Modify Server Settings |
| 0x46 | 2 | | 2 |
| 0x48 | 1 | name length | ≤ 0x3f |
| 0x49 | 1 | description length | ≤ 0xff |
| 0x4a | 1 | third string length | ≤ 0x7f |
| 0x4c | | name, description, third string | "New Server" [O] |

The whole message is at most 0x400 bytes.

**106 Server Icon**
Initiator: Server.

The server's icon bitmap. The layout is not read yet.

**107 Server Greeting**
Initiator: Server. **[S `FUN_0804b738`; O]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | 0 |
| 0x04 | | greeting text, to the end of the payload |

When no greeting is set, the server sends `(No greeting.)` **[O]**.

**108 Ping** → **109 Pong**
Initiator: Client. **[S]**

The request is empty, and so is the reply (109 Pong). Ping does not reset
the idle timer.

**110 Set Mini User Icon**
Initiator: Client. **[S `FUN_0804b6d4`]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 1 | status | |
| 0x01 | 3 | | the payload must be at least 4 bytes |

There is no reply. If the value changed, the server broadcasts **111 Mini
User Icon Changed** to everyone, the sender included:

| Off | Size | Field |
|---|---|---|
| 0x00 | 2 | user ID |
| 0x02 | 2 | 0 |
| 0x04 | 4 | status |

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
Initiator: Client. **[S `FUN_0804fdac`]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | value |

No reply. The server stores 1 if the value is nonzero and 2 if it is zero
(user +0x3b). The purpose is unknown.

### Monitoring and administration

**140 Open Connection Monitor** / **141 Close Connection Monitor**
Initiator: Client. **[S]**

These need Get User Info (bit 40) and turn a per-user flag on or off. There
is no reply.

**142 Get Connection Monitor Info** → **143 Connection Monitor Info**
Initiator: Client. **[S `FUN_0804b9b0`; O]**

The request is empty. Without a 140 Open Connection Monitor first, the
server replies with 1 Status 0x10003 (protocol breach) **[O]**. The reply is
one record per connection, with no count in front, each padded to 4:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | user ID | |
| 0x04 | 4 | | |
| 0x08 | 8 | login time | |
| 0x10 | 8 | time | |
| 0x18 | 8 | time | probably the last activity [I] |
| 0x20 | 0x40 | | transfer progress [I]; all zero with no transfers |
| 0x60 | 1 | nick length | |
| 0x61 | 1 | login length | |
| 0x62 | 1 | account name length | |
| 0x63 | 1 | address length | |
| 0x68 | | nick, login, account name, address | `probe`, `admin`, `Administrator`, `127.0.0.1:50984` |

The padding after the address is uninitialised **[O]**.

The server also sends per-transfer records [S]; that layout is not read yet.

**150 Disconnect User**
Initiator: Client. Privilege: Disconnect Users (41), and Ban Users (43) to
ban. **[S `FUN_0804c42c`]**

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

**152 Get User Info** → **153 User Info**
Initiator: Client. Privilege: Get User Info (40). **[S `FUN_0804c8a8`; O]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | user ID |

If the user doesn't exist, the server replies with 1 Status 0x1000c **[O]**.

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
Initiator: Client. Privilege: Broadcast (33). **[S `FUN_0804d1a8`]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 2 | |
| 0x02 | 2 | (copied through) |
| 0x04 | | text, ≤ 0x800 |

Everyone receives **155 Rcv Broadcast Message**: {u32 sender ID, 8 zero
bytes, u16 0, u16 (copied from the request), text}.

**170 Get Server Settings** → **171 Server Settings Info**
Initiator: Client. Privilege: Modify Server Settings (61). **[S
`FUN_0805f598`; O 216 bytes]**

The reply is the server's settings file. It starts with the magic `%KDXSSTG`
and a version, followed by a table of section offsets and the sections. The
server name appears near the end. **172 Set Server Settings** sends the same
blob back, and the server answers with 173 Set Server Settings Done. The
field layout is not read yet.

**174 Shutdown Server**
Initiator: Client. Privilege: Shutdown Server (64). **[S `FUN_08061598`]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 1 | mode | low 3 bits nonzero |
| 0x04 | | message | ≤ 0xff |

The server broadcasts 151 Disconnect Message with flag 4, then exits after 5
seconds.

**175 Upgrade Server Software**
Initiator: Client. Privileges: Shutdown Server (64) and Launch Programs
(82). **[S `FUN_0806172c`]**

The server broadcasts "Upgrading server software, be back in 5 seconds!".

**180 Server History Join** / **182 Server History Leave**
Initiator: Client. Privilege: View Server History (63). **[S `FUN_08062674`,
`FUN_080627f8`, `FUN_08061d80`]**

The reply (181 Server History Join (R)) is {u32 0, u32 count, buffered
records}. While joined, the client receives **183 Server History Added** for
each new record. A record's kind is one of 184–191 (Login, Disconnected,
Abort File Download, …), or the type of the logged request (e.g. 400 Get
File List for a failed file list).

### Chat

**200 Create Private Chat** → **216 Create Private Chat (R)**
Initiator: Client. Privilege: Create Private Chat (34). **[S
`FUN_08058420`]**

A user can have at most 10 private chats. Reply:

| Off | Size | Field |
|---|---|---|
| 0x08 | 4 | chat ID |
| 0x0c | 4 | key |
| 0x16 | 2 | member count |
| 0x20 | 1 | name length |
| 0x24 | 4 | creator user ID |
| … | | name, e.g. `Private 1` |

**201 Delete Chat**
Initiator: Client. Privilege: Create/Modify Chats (37). **[S
`FUN_08059774`]**

The request is a u32 chat ID. Every member receives **221 Chat Deleted**
{u32 chat ID, u32 user ID}.

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
| 0x1e–0x22 | 1 each | five string lengths; the third is the name [O] |
| 0x24 | 4 × count | member user IDs |
| … | | the five strings |

In frame 68 of `captures/probe.pcapng` only the name (`Public Chat`) is
set; the payload is not padded.

The other members receive **205 Chat User Joined** {u32 chat ID, u32 user
ID, u32 0} (`captures/kdx-cli.pcapng` frame 417) **[O]**.

**204 Leave Chat**
Initiator: Client. **[S `FUN_08058c0c`]**

The request is a u32 chat ID. The other members receive **206 Chat User
Departed** {u32 chat ID, u32 user ID, u8 1, 3 zero bytes}
(`captures/kdx-cli.pcapng` frame 424) **[O]**. A private chat that becomes empty
is deleted.

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
Initiator: Client. **[S `FUN_080591ac`]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | chat ID |
| 0x08 | 2 | invitee count (≤ 20) |
| 0x0f | 1 | message length |
| 0x10 | 4 × count | invitee user IDs |
| … | | message |

**213 Must Join Chat**
Initiator: Server.

Sent after login to accounts with Join Chat On Connect. It tells the client
which chat to join.

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

[O]: Public Chat (1), Voice Chat (2, flags 2), Admin Lounge (3).

**217 Create Chat** / **218 Get Chat** → **219 Chat Info** / **220 Set Chat**
Initiator: Client. Privilege: Create/Modify Chats (37) for 217 Create Chat
and 220. **[S; O 219 Chat Info]**

217 Create Chat and 220 Set Chat carry a chat record. 218 Get Chat is a u32
chat ID, and 219 Chat Info is a chat record. 220 Set Chat refuses private
chats.

**222 Get Chat DCC Addresses** → **223 Chat DCC Addresses**
Initiator: Client. **[S `FUN_0805a110`]**

The sender must be a member of a voice chat. Reply entries: {u32 user ID, …,
0x0e u8 length, `"ip:port"`}. Voice data then goes peer-to-peer over UDP.

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
| 0x30 | 2 | folder access item | 0x2c, 0x2b for the folders, 0 for a file (see 1107 Folder Access List) |
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

**402 Create Folder**, **403 Create Alias**, **405 Move To Trash**, **407
Move**, **411 Set File/Folder Info**, **420 Launch File**, **421 Empty
Trash**, **423 Set Volume Info**
Initiator: Client. **[S]**

Handlers are listed in the table above. Each one checks privileges (see
Access Privileges) and takes one or two file paths. Their layouts are not
read yet.

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

One 414 header covers the whole fork, even past the 0x20000 limit on
normal payloads: a 600000-byte file came as a single 414 of length 600000,
byte for byte, and a resume from 300000 as one of 300992 from 299008
(`captures/kdx-cli.pcapng` frames 133 and 190) **[O]**. Data is read in
chunks starting at 0x10000 and growing to 0x80000. With a
speed limit, chunks are multiples of 0x1000. At the end the server writes a
188 Finished File Download history record.

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

**450 Generate Catalog**, **452 File Search**
These are not read yet.

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
| 0x25 | 1 | | 0x50 for Administrator, 0x28 for Guest |
| 0x26 | 0xa | | |
| 0x30 | 1 | class index | 0, 2 |
| 0x31 | 1 | | 0xff for Administrator, 0 for Guest; probably the colour [I] |
| 0x34 | 1 | name length | |
| 0x38 | | name | |

The padding after the name is uninitialised (`@`, `.` in the lab) **[O]**.

**509 Get Account** → **510 Account Info**
Initiator: Client. Privilege: View Accounts (50). **[S `FUN_08055e98`; O]**

The request is the account's login, with no length byte. Reply (148 bytes
for `admin`):

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 0x10 | | `80 00 00 00 00 00 02 00…`; probably a mask of privileges set on the account itself [I] |
| 0x10 | 0x10 | | the same bytes; probably the privilege values [I] |
| 0x20 | 1 | | 1 |
| 0x40 | 8 | created | |
| 0x48 | 8 | modified | |
| 0x50 | 8 | last login | |
| 0x7a | 1 | login length | |
| 0x7b | 1 | name length | |
| 0x80 | | login, name | `admin`, `Administrator` |

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

504 Set Account Class, 507 Create Account, 508 Delete Account, 511 Set
Account, and 550 Change Own Password are not read yet. Their handlers are
listed in the table above.

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
`FUN_0805cc24`; O 76 bytes]**

The request is a u32 group ID. An unknown group gets 1 Status 0x1000c. The
reply is padded to 4:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | group ID | |
| 0x04 | 0xc | | 0 |
| 0x10 | 8 | time of the last post | 611 Post News Message sets it |
| 0x18 | 4 | read classes | a class bitmap, class 0 in the top bit; `ffffffff` |
| 0x1c | 4 | post classes | the same; `ffffffff` |
| 0x20 | 4 | | `01 00 00 00` |
| 0x24 | 0xc | | 0 |
| 0x30 | 2 | message count | 1 |
| 0x32 | 2 | | a stored value shifted right by 10, so probably a size limit in KiB [I]; 0x32 |
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
| 0x04 | 8 | | not read |
| 0x0c | 2 | body length | the server keeps at most 0x4000 bytes |
| 0x0e | 1 | subject length | the server keeps at most 0x80 bytes |
| 0x0f | 1 | | not read |
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

The newsgroup management messages (600 Create Newsgroup, 601 Delete
Newsgroup, 604 Set Newsgroup) are not read yet.

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
`FUN_0805b8f4`]**

Not probed.

### Folder access

A folder access item is a named set of rules. A folder refers to one by ID
(the u16 at offset 0x30 of a 401 File List entry). Item `[Default]` applies
when a folder and its parents name no item.

**1106 Get Folder Access List** → **1107 Folder Access List**
Initiator: Client. Privilege: View Folder Access (27). **[S `FUN_080578d8`;
O]**

The request is empty or a u32. If the u32's top byte has bit 0 set, a user
without the privilege gets no reply instead of 1 Status 0x10014.

Reply: a u32 count, then one entry per item, each padded to 4:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | | 1, 2, 0 in the lab |
| 0x04 | 4 | item ID | 0 for `[Default]`, 0x2c `DB`, 0x2b `UL` |
| 0x08 | 8 | | not written by the server |
| 0x10 | 1 | | |
| 0x11 | 1 | name length | |
| 0x12 | 1 | second string length | 0 in the lab |
| 0x13 | 1 | description length | |
| 0x14 | | name, second string, description | `[Default]`, empty, `If a folder and its parents…` |

1100 Create Folder Access, 1101 Delete Folder Access, 1103 Get Folder
Access, and 1105 Set Folder Access are not read yet.

### Not yet documented

- View display: 900–905.
- Voice: 1000–1003.
- The tracker: 700–721, using the LCG XOR at `0x0804a454`/`0x0804a4b4`.
- 117 SKPK.

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
| 3 | (login) | `FUN_08054900` account lookup |
| 10 | Download | 412 Download File |
| 11 | Upload | 402 Create Folder, 415 Upload File |
| 13 | Empty Trash | 421 Empty Trash |
| 15 | Create Aliases | 403 Create Alias |
| 16 | Move | 407 Move File/Folder |
| 17 | Rename [I] | 409 Get File/Folder Info, 411 Set File/Folder Info |
| 18 | Delete | 405 Move To Trash |
| 20 | Set Attributes [I] | 409 Get File/Folder Info, 411 Set File/Folder Info |
| 22 | See All Volumes [I] | 400 Get File List, 423 Set Volume Info |
| 25 | Allow Simultaneous Downloads [I] | 412 Download File (large files) |
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
| 60 | Use Any Name | 100 Set Identity, login |
| 61 | Modify Server Settings | 113 Get Extra Server Info, 170 Get Server Settings, 172 Set Server Settings, 450 Generate Catalog, login (105 Server Info) |
| 62 | Don't Show Greeting | login |
| 63 | View Server History | 180 Server History Join |
| 64 | Shutdown Server | 174 Shutdown Server, 175 Upgrade Server Software |
| 65 | Disable Encryption [I] | 412 Download File, 415 Upload File (plain mode) |
| 70 | View News | 605 Get Newsgroup List, 607 Get News Messages, 609 Get News Message |
| 71 | Post News | 607 Get News Messages, 611 Post News Message |
| 72 | Delete News Messages | 607 Get News Messages, 612 Delete News Message |
| 73 | Create/Modify Newsgroups | 505 Get Account Class List, 600 Create Newsgroup, 601 Delete Newsgroup, 602 Get Newsgroup, 604 Set Newsgroup |
| 74 | See Logins In News | 609 Get News Message |
| 80 | View Processes | 800 Open Process Monitor, 802 Get Process List |
| 81 | Terminate Processes | 804 Exit/Terminate Process |
| 82 | Launch Programs | 175 Upgrade Server Software, 420 Launch File |

Bits 2 and 4 (0x44&0x20, 0x44&0x08) are set in every class. One of them is
probably Can Login **[I]**.

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

The file bits (8–23), Can Login, Color Overrides Class, the display bits,
and Disable Encryption are not yet mapped to bit numbers.

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

## Posting news

A client confirms a request that has no success reply, such as 611 Post News
Message, by sending 115 Get My Address after it. The server handles one
connection's requests in order, so an error for the post arrives before 116
Your Address **[O `captures/kdx-news.pcapng` frames 356–361]**.

## Logout

When a connection closes, the other users receive **104 User Departed**, and
the server writes a 185 Disconnected history record.
