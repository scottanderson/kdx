# KDX Protocol

The reference for the Haxial KDX protocol as spoken by KDX Server 1.620, KDX
Client 1.600 and 1.620 beta 5, and Tracker 1.110. It is built from lab
captures and static reading of the binaries.

This page covers the protocol's shape: the connection types, the message
header, and the catalog of every message type. Each message is described in
detail on the page for its area, linked below.

## Where to read

| To learn about | Read |
|---|---|
| How a connection starts: hello, proof, challenge tables | [handshake.md](handshake.md) |
| Message encryption, framing, and the header scramble | [session.md](session.md) |
| Strings, times, paths, and the user and chat records | [encodings.md](encodings.md) |
| The client-server connection | [server.md](server.md) |
| Login, user list, server info, ping | [messages/session.md](messages/session.md) |
| The connection monitor, broadcasts, settings, history | [messages/administration.md](messages/administration.md) |
| Chat rooms and private messages | [messages/chat.md](messages/chat.md) |
| File listings, folders, volumes, search | [messages/files.md](messages/files.md) |
| Downloads and uploads | [messages/transfers.md](messages/transfers.md) |
| Accounts, classes, and passwords | [messages/accounts.md](messages/accounts.md) |
| News | [messages/news.md](messages/news.md) |
| Processes | [messages/processes.md](messages/processes.md) |
| Folder access rules | [messages/folder-access.md](messages/folder-access.md) |
| View display (remote screen) | [messages/view-display.md](messages/view-display.md) |
| Error codes carried by 1 Status | [errors.md](errors.md) |
| Privilege bits | [privileges.md](privileges.md) |
| Whole exchanges: login, flood ban, posting news | [sequences.md](sequences.md) |
| The tracker: UDP registration and the browse protocol | [tracker.md](tracker.md) |
| Client to client (DCC): private chat and voice | [dcc.md](dcc.md) |
| GConsole, the remote console | [gconsole.md](gconsole.md) |
| How the work is planned | [PLAN.md](PLAN.md) |

Each claim carries its evidence: **[O]** observed in a capture or lab probe,
**[S]** read in a binary (with the function address), **[I]** inferred. The
lab probes are in `captures/probe.pcapng` (see `captures/probe.md`).
Unmarked rows in a layout table share the tag in the message's heading line.
Offsets and lengths are hex. All multi-byte numbers are big-endian.
A message is named by its type and title, as in 412 Download File.

# Protocol overview

KDX is a client/server system for chat, private messages, file sharing,
news, and server administration. Trackers list servers.

Every connection opens with a hello from each side that names a protocol, a
sub-protocol, and a sub-version. The protocol is always `%TXP`. The sub-protocol
is `%KDX`, or `%HGC` for GConsole. The sub-version says what the sender is,
and so what kind of connection the receiver has:

| Sub-protocol | Sub-version | The sender is | Connection | Page |
|---|---|---|---|---|
| `%KDX` | 0x10 | a client | client to client (DCC) | [dcc.md](dcc.md) |
| `%KDX` | 0x20 | a server | client to server | [server.md](server.md) |
| `%KDX` | 0x30 | a tracker | client to tracker | [tracker.md](tracker.md) |
| `%HGC` | 0x40 | a GConsole peer | client to GConsole | [gconsole.md](gconsole.md) |

A client always sends 0x10 and decides what it has reached from the hello
that comes back. Each server type accepts only the sub-versions it serves and
rejects the rest with 0x40071, 0x40072 or 0x40073
([handshake.md](handshake.md#connection-types)).

| Port | Transport | Use |
|---|---|---|
| 10600 | TCP | client to client (DCC): private chat and voice; the client advertises its port in the user record **[O; S]**; see [dcc.md](dcc.md) |
| 10700 | TCP | client to server: every message, and file transfers on extra connections **[O]**; see [server.md](server.md) |
| 10800 | UDP, TCP | tracker: server registration (UDP) and listing (TCP); see [tracker.md](tracker.md) **[S]** |
| 10900 | TCP | GConsole, a Haxial remote console the client can open; no KDX server or tracker serves it **[S]**; see [gconsole.md](gconsole.md) |

A connection has two phases:

1. **Handshake.** Both sides exchange a cleartext hello (protocol `%TXP`,
   a sub-protocol, a version, a sub-version, a nonce, and the login name).
   Each side then proves knowledge of the account key and of a challenge
   table built into the programs. See [handshake.md](handshake.md).
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
| 0x0a | 2 | len high | bits 32–47 of the length; nonzero only on transfer data headers |
| 0x0c | 4 | len | payload length: at most 0x20000 to the server, 0x100000 to the client |

The payload is padded to a multiple of 16 before encryption; the length
field gives the real size. The length is 48 bits, but only transfer data
headers exceed 4 GiB; see [session.md](session.md).

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
| 0x04 | 4 | request type | e.g. 0x98 for a failed 152 Get User Info **[O]** |
| 0x08 | 4 | module | source module tag of the raise site, ASCII packed into an integer (`0x54524e` = `TRN`; 0x46 for the path code) |
| 0x0c | 4 | line | source line of the raise site |
| 0x10 | 4 | extra | a value the raise site passes along |
| 0x14 | 4 | 0 | |

**[O]** examples:

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

# Message types

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
| 4000 | Open Window | GConsole peer | client `FUN_08094be8` **[S]**; frame 10 of `linux-gconsole` **[O]** |
| 4001 | Draw | GConsole peer | client `FUN_08095ac4` **[S]**; frame 14 of `linux-gconsole` **[O]** |
| 4200 | Key | Client (to GConsole peer) | the peer; client sender `FUN_0809607c` **[S]**; frame 18 of `linux-gconsole` **[O]** |
| 4201 | Mouse | Client (to GConsole peer) | the peer; client sender `FUN_08096188` **[S]**; frame 32 of `linux-gconsole` **[O]** |

A type the dispatcher doesn't know is ignored without a reply **[S]**.
4000 to 4201 belong to GConsole, not the server, so their names are
provisional; see [gconsole.md](gconsole.md).


# Message descriptions

The messages a server speaks are described by area, starting from
[server.md](server.md). The tracker, DCC and GConsole have their own pages:
[tracker.md](tracker.md) (700 to 721), [dcc.md](dcc.md) (117, 300 on DCC, and
1000 to 1003) and [gconsole.md](gconsole.md) (4000 to 4201).

## Not yet documented

- The tracker: 700–721 and UDP registration are read statically but not
  captured. See [tracker.md](tracker.md).
- Client-to-client (DCC) messages 117, 300 on DCC, and 1000-1003 are read
  statically but not captured. See [dcc.md](dcc.md).
