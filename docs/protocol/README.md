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
| Message encryption, framing, and the header scramble over TCP | [framing.md](framing.md) |
| The UDP datagram a server sends to register with a tracker | [udp.md](udp.md) |
| File transfers and view display on their own connection | [transfers.md](transfers.md) |
| Strings, times, paths, and the user and chat records | [encodings.md](encodings.md) |
| The client-server connection | [server.md](server.md) |
| Every message type, with the connections that use it | [messages/README.md](messages/README.md) |
| Login, user list, server info, ping, SKPK (100 to 118) | [messages/session.md](messages/session.md) |
| The connection monitor (140 to 143) | [messages/monitoring.md](messages/monitoring.md) |
| Disconnects, broadcasts, settings, history (150 to 191) | [messages/administration.md](messages/administration.md) |
| Chat rooms (200 to 223) | [messages/chat.md](messages/chat.md) |
| Private messages (300 to 301) | [messages/private-messages.md](messages/private-messages.md) |
| File listings, folders, downloads, uploads, volumes, and search (400 to 453) | [messages/files.md](messages/files.md) |
| Accounts, classes, and passwords (502 to 551) | [messages/accounts.md](messages/accounts.md) |
| News (600 to 614) | [messages/news.md](messages/news.md) |
| Processes (800 to 804) | [messages/processes.md](messages/processes.md) |
| View display, the remote screen (900 to 905) | [messages/view-display.md](messages/view-display.md) |
| Voice between clients (1000 to 1003) | [messages/voice.md](messages/voice.md) |
| Folder access rules (1100 to 1107) | [messages/folder-access.md](messages/folder-access.md) |
| The [1 Status](messages/status.md#m1) reply, on every connection (1) | [messages/status.md](messages/status.md) |
| Error codes carried by 1 Status | [Error codes](messages/status.md#error-codes) |
| Privilege bits | [privileges.md](privileges.md) |
| Whole exchanges: login, flood ban, posting news | [sequences.md](sequences.md) |
| The tracker and its TCP browse connection | [tracker.md](tracker.md) |
| Tracker messages (700 to 721) | [messages/tracker.md](messages/tracker.md) |
| Client to client (DCC): private chat and voice | [dcc.md](dcc.md) |
| GConsole, the remote console | [gconsole.md](gconsole.md) |
| GConsole messages (4000 to 4201) | [messages/gconsole.md](messages/gconsole.md) |
| How the work is planned | [PLAN.md](../PLAN.md) |

Each claim carries its evidence: **[O]** observed in a capture or lab probe,
**[S]** read in a binary (with the function address), **[I]** inferred. The lab
probes are in `captures/probe.pcapng` (see `captures/probe.md`). Unmarked rows
in a layout table share the tag in the message's heading line. Offsets and
lengths are hex. All multi-byte numbers are big-endian. A message is named by
its type and title, as in [412 Download File](messages/files.md#m412).

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
| 10800 | UDP, TCP | tracker: server registration (UDP, see [udp.md](udp.md)) and listing (TCP, see [tracker.md](tracker.md)) **[S]** |
| 10900 | TCP | GConsole, a Haxial remote console the client can open; no KDX server or tracker serves it **[S]**; see [gconsole.md](gconsole.md) |

A connection has two phases, and the second carries every message:

1. **Handshake.** Both sides exchange a cleartext hello (protocol `%TXP`,
   a sub-protocol, a version, a sub-version, a nonce, and the login name).
   Each side then proves knowledge of the account key and of a challenge
   table built into the programs. See [handshake.md](handshake.md).
2. **Session.** Every message after that is a 16-byte header plus a payload,
   each block encrypted with Twofish-256 in ECB mode under the account key.
   See [framing.md](framing.md).

# Messages

## Header

Every message starts with a 16-byte header: the txid (4 bytes, nonzero, counted
up by each side from its own random start), the message type (4), a zero byte,
a flags byte (bit 0 set on a reply), 2 bytes of high length bits that only
transfer data headers use, and the payload length (4). The payload is padded to
a multiple of 16 before encryption, and the length gives the real size. The
full table, the limits and the 48-bit length are in
[framing.md](framing.md#header).

## Requests, replies, and notifications

- A **request** carries the sender's next txid, with the reply bit (bit 0 of
  the flags byte) clear.
- A **reply** reuses the request's txid and sets the reply bit. The server
  sets it exactly when it answers a request (`FUN_080863b4` receives the
  request's txid) **[S, O]**.
- A **notification** is a message the server sends on its own, with its own txid
  and the reply bit clear. For example,
  [103 User Joined](messages/session.md#m103) goes to everyone else when a user
  logs in **[O]**.
- The server only handles messages with the reply bit clear. Replies
  arriving from a client are dropped **[S `FUN_080528c8`]**.
- Every message except [108 Ping](messages/session.md#m108),
  [142 Get Connection Monitor Info](messages/monitoring.md#m142), and
  [609 Get News Message](messages/news.md#m609) resets the sender's idle timer
  **[S `FUN_080528c8`]**.

Message types come in pairs: request N gets reply N+1. A request that fails gets
the generic **[1 Status](messages/status.md#m1)** reply instead. Some requests
have no reply on success **[S, O]**.

# Message types

[messages/README.md](messages/README.md) lists every message type with the
connection that carries it, who sends it, and the page that describes it.
Each message's page gives its handler. The names come from the server's name
table (`FUN_08049d14`, jump table at 0x080abc6c) **[S]**. In that table, (R)
means reply, (D) data, (C) complete, and (S) status. A type the dispatcher
doesn't know is ignored without a reply **[S]**. 4000 to 4201 belong to
GConsole, not the server, so their names are provisional; see
[gconsole.md](gconsole.md).

# Not yet documented

- Client-to-client (DCC) [1003 Voice Data](messages/voice.md#m1003) is read
  statically but not captured: the voice call in
  `captures/linux-dcc-voice-failed.pcapng` fails before any voice data. See
  [dcc.md](dcc.md).
