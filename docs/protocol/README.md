# KDX Protocol

The reference for the Haxial KDX protocol as spoken by KDX Server 1.620, KDX
Client 1.600 and 1.620 beta 5, and Tracker 1.110. It is built from lab
captures and static reading of the binaries.

This page covers the protocol's shape: the connection types, the message
header, and the catalog of every message type. Each message is described in
detail on the page for its area, linked below.

## Read in this order

1. [Conventions](#conventions), below: tags, hex, and field notation.
2. [handshake.md](handshake.md): how a connection starts, then
   [framing.md](framing.md): the encryption and the message header.
3. [sequences.md](sequences.md): whole exchanges, such as login.
4. [messages/README.md](messages/README.md): the index of every message type,
   then the page for the area you need.
5. [encodings.md](encodings.md) and [privileges.md](privileges.md): the
   shared record formats and the privilege bits the messages refer to.
6. [server.md](server.md), [transfers.md](transfers.md),
   [tracker.md](tracker.md) with [udp.md](udp.md), [dcc.md](dcc.md), and
   [gconsole.md](gconsole.md): one page for each kind of connection.

## Where to read

### Connections

| To learn about | Read |
|---|---|
| How a connection starts: hello, proof, challenge tables | [handshake.md](handshake.md) |
| Message encryption, framing, and the header scramble over TCP | [framing.md](framing.md) |
| The UDP datagram a server sends to register with a tracker | [udp.md](udp.md) |
| File transfers and view display on their own connection | [transfers.md](transfers.md) |
| The client-server connection | [server.md](server.md) |
| The tracker and its TCP browse connection | [tracker.md](tracker.md) |
| Client to client (DCC): private chat and voice | [dcc.md](dcc.md) |
| GConsole, the remote console | [gconsole.md](gconsole.md) |

### Message pages

| To learn about | Read |
|---|---|
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
| Tracker messages (700 to 721) | [messages/tracker.md](messages/tracker.md) |
| GConsole messages (4000 to 4201) | [messages/gconsole.md](messages/gconsole.md) |

### Reference

| To learn about | Read |
|---|---|
| Privilege bits | [privileges.md](privileges.md) |
| Whole exchanges: login, flood ban, posting news | [sequences.md](sequences.md) |
| Strings, times, paths, and the user and chat records | [encodings.md](encodings.md) |
| How the work is planned | [PLAN.md](../PLAN.md) |

## Conventions

These apply to every page of the spec.

**Evidence tags.** Each claim carries its evidence in a bold tag:

- **[O]** observed in a capture or lab probe. The lab probes are in
  `captures/probe.pcapng` (see `captures/probe.md`).
- **[S]** read in a binary, with the function address when there is one.
- **[I]** inferred, from neither of the above directly.

Unmarked rows in a layout table share the tag in the message's heading line.

A tag lists its evidence in the order S, I, O, separated by semicolons, as in
``**[S `FUN_08085a04`, `FUN_0808584c`; I; O `captures/probe.pcapng` frames 3
to 9]**``. The S part names the functions, separated by commas; a binary name
(client, server, tracker) may come before a function. Each O part names a
capture and its frames, written `frame 7`, `frames 7, 9`, or `frames 7 to 9`.
Leave out the capture when the page already names it, and the frames when the
whole capture shows the fact. Start a second O part for a second capture.
Write no commas between the letters and no remark inside a tag; put a remark
such as "not captured" in the text before the tag.

**Hex.** A number written with a `0x` prefix is hex, as in 0x40071. Offsets and
lengths in layout tables are hex too. Ports, counts, and the message type
numbers are decimal. A run of bytes is written as hex pairs in a code span, as
in `00 00`. A message is named by its type and title, as in
[412 Download File](messages/files.md#m412).

**Endianness.** All multi-byte numbers are big-endian unless the page says
otherwise. The one exception is a placeholder in the UDP datagram, which
[udp.md](udp.md) says is stored little-endian.

**Fields.** A layout table lists each field's offset (Off), size in bytes
(Size), name, and notes. A blank size means the field's length varies. Integer
widths are written `u8`, `u16`, `u32`, and `u64`, for 1, 2, 4, and 8 bytes. An
integer with a value is written `u32 0`.

**Blank and 0.** In a layout table's Field column, `0` means the bytes are zero
as sent, and a blank means the field has no name yet, so its use is unknown or
it is padding. Every field with a known use has a name. In the dot tables of
[messages/README.md](messages/README.md), a blank means no handler there.

## Protocol overview

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
| 10600 | TCP | client to client (DCC): private chat and voice; the client advertises its port in the user record **[S; O]**; see [dcc.md](dcc.md) |
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

## Messages

### Header

Every message starts with a 16-byte header: the txid (4 bytes, nonzero, counted
up by each side from its own random start), the message type (4), a zero byte,
a flags byte (bit 0 set on a reply), 2 bytes of high length bits that only
transfer data headers use, and the payload length (4). The payload is padded to
a multiple of 16 before encryption, and the length gives the real size. The
full table, the limits and the 48-bit length are in
[framing.md](framing.md#header).

### Requests, replies, and notifications

- A **request** carries the sender's next txid, with the reply bit (bit 0 of
  the flags byte) clear.
- A **reply** reuses the request's txid and sets the reply bit. The server
  sets it exactly when it answers a request (`FUN_080863b4` receives the
  request's txid) **[S; O]**.
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
have no reply on success **[S; O]**.

## Message types

[messages/README.md](messages/README.md) lists every message type with the
connection that carries it, who sends it, and the page that describes it.
Each message's page gives its handler. The names come from the server's name
table (`FUN_08049d14`, jump table at 0x080abc6c) **[S]**. In that table, (R)
means reply, (D) data, (C) complete, and (S) status. A type the dispatcher
doesn't know is ignored without a reply **[S]**. 4000 to 4201 belong to
GConsole, not the server, so their names are provisional; see
[gconsole.md](gconsole.md).

## Not yet documented

- Client-to-client (DCC) [1003 Voice Data](messages/voice.md#m1003) is read
  statically but not captured: the voice call in
  `captures/linux-dcc-voice-failed.pcapng` fails before any voice data. See
  [dcc.md](dcc.md).
