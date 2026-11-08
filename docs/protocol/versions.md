# Protocol versions

For hex and field notation, see [Conventions](README.md#conventions).

The rest of this spec describes KDX 1.6. This page tells how the protocol got
there, from the 0.9x builds of 2001 to 1.620. It comes from two sources:

- The binaries that still exist; see `vendor/MANIFEST.md`. The Mac builds have
  a function that names each message type, which `re/name_table.py` reads, and
  the x86 builds keep the names in a block of strings.
- Haxial's version history pages for KDX, TrackerServ, and RemoteAdminTool,
  which give a date for each release. `vendor/MANIFEST.md` tells where to get
  them. Where a fact comes only from these pages, this page says so.

No capture of a build older than 1.600 exists, so nothing on this page is
verified on the wire.

## Timeline

| Version | Date | Protocol change | Binary |
|---|---|---|---|
| 0.915 to 0.921 | 2001 | The [0.9x handshake](handshake.md#version-09x) | Mac |
| 0.920 | 2001 | Trackers, server settings, and the greeting | Mac |
| 0.950 | 7 Jun 2003 | "Security enhancements"; not compatible with older builds | none |
| 0.951 | 18 Jun 2003 | Get Server Info, and an option to send file transfers without encryption | none |
| 0.990 | 8 Aug 2003 | Folder uploads, Change Own Password, the Can Login privilege | none |
| 1.000 | 20 Aug 2003 | The 1.x handshake with one sub-version; RAT's features | Mac |
| 1.100 | 16 Nov 2003 | A sub-version for each role, DCC, voice; not compatible with older builds | none |
| 1.110 | 17 Nov 2003 | Bug fixes | Windows |
| 1.200 | 23 Feb 2004 | Unicode text | none |
| 1.300 | 15 Mar 2004 | Many chats, voice chats | Windows |
| 1.500 | 31 Jul 2004 | Folder access, flood protection | Windows |
| 1.520 | 29 Aug 2004 | Flood protection for each account class, reused transfer connections | Windows, Linux |
| 1.600 | 3 Oct 2004 | Can See All Volumes, volume info | Windows, Linux, Mac |
| 1.620 | 23 Nov 2004 | Linux fixes only | Linux |

<!-- [I] dates and changes from Haxial's version history pages in vendor/refs;
the Binary column from vendor/MANIFEST.md -->

## 0.9x
<!-- [S client `FUN_100013f0`] -->

The 0.9x builds use their own handshake, with no XOR layer and no challenge
table, and a session that is Twofish from the start; see [Version
0.9x](handshake.md#version-09x) and [framing.md](framing.md#across-versions).
The header is the same as in 1.x.

Client 0.921 names 79 message types, and 70 of them have the same number and
name in 1.x. The differences:

| Type | 0.921 name | 1.000 name |
|---:|---|---|
| 106 | Server Banner | [106 Server Icon](messages/session.md#m106) |
| 607 | Join Newsgroup | [607 Get News Messages](messages/news.md#m607) |
| 608 | Join Newsgroup (R) | [608 News Messages](messages/news.md#m608) |
| 609 | Leave Newsgroup | [609 Get News Message](messages/news.md#m609) |
| 610 | Post News Message | [610 News Message](messages/news.md#m610) |
| 611 | Delete News Message | [611 Post News Message](messages/news.md#m611) |
| 612 | News Message Posted | [612 Delete News Message](messages/news.md#m612) |
| 613 | News Message Deleted | [613 News Message Posted](messages/news.md#m613) |
| 614 | News Activity Sound | [614 News Message Deleted](messages/news.md#m614) |

So in 0.921, a client joined a newsgroup and then got each change as it came,
as with [180 Server History Join](messages/administration.md#m180). In 1.000,
the client asks for the list and for each message. This probably came with the
many newsgroups of 0.950. <!-- [I] -->

0.921 has none of these types, which 1.000 has: <!-- [S client `FUN_100013f0`,
server `FUN_1000ff90`] -->

- 107 to 114: greeting, ping, mini icons, and extra server info.
- 140 to 143: the connection monitor.
- 154 and 155: broadcasts.
- 174 to 191: shutdown, upgrade, and the server history.
- 418 to 421 and 450 to 453: resumed uploads, speed limits, launch, empty trash,
  catalogs, and search.
- 514, 515, 550, and 551: account notices, and Change Own Password.
- 710 to 713: tracker settings.
- 800 to 804 and 900 to 905: processes and view display.

The version history dates most of these between 0.922 and 1.000; for example,
ping came in 0.937, and the connection monitor in 0.945. <!-- [I] Haxial's
version history in vendor/refs -->

The tracker had its own sub-protocol, `%KDt`. Tracker 0.902 listens on TCP port
10699 and UDP port 10698. <!-- [S tracker `FUN_10001960`] --> Server 0.921 adds
`:10698` to a tracker address that has no port, and Client 0.921 adds
`:10699`. <!-- [S] strings in the binaries --> The registration datagram
already has the encryption of [udp.md](udp.md#encryption), with the same LCG
and CRC seed. <!-- [S server `FUN_10007e90`, `FUN_10007d50`] -->

## 0.950

The version history says that 0.950 made "security enhancements to the KDX
protocol", and that it cannot connect to an older server or tracker. TrackerServ
0.909 was "updated to new protocol", for Client 0.950. <!-- [I] Haxial's version
history in vendor/refs --> No build from 0.950 to 0.990 is known. RAT 1.080,
from May 2003, still uses the 0.9x handshake, and 1.000, from August 2003, has
the 1.x handshake. So the 1.x handshake probably came in 0.950. <!-- [I] -->

## 1.000
<!-- [S server `FUN_10006d50`, `FUN_1005ebe0`, `FUN_1005ee10`, `FUN_1005fcb0`,
client `FUN_10000bc0`, `FUN_1002d080`] -->

Server 1.000 and Client 1.000 have the 1.x handshake: the XOR layer, `%TXP`, the
proof, and the challenge table. They have the same header scramble constants as
the later builds. <!-- [S] re/scan_consts.py --> Each new connection gets the
magic 0x2aa24aec and a receive cap of 0x40000, and the server sets 0x20000 on
each connection that it accepts.

But every peer sends sub-version 1, and the receiver needs the same value:

- A sub-version below its own gives 0x40072, and above it gives 0x40073.
- The other checks are the same as in [HELLO](handshake.md#hello), including the
  0x800 limit on the length.

So a 1.000 server gives 0x40073 to the 0x10 of a 1.1 client. The client still
browses a tracker with `%KDt` and sub-version 1. Server 1.000 adds `:10698` to
a tracker address, and Client 1.000 adds `:10699`. <!-- [S] strings in the
binaries -->

Server 1.000 and Client 1.000 name the same 134 types. They match the names of
1.620, except for the types that came later. <!-- [S server `FUN_1000ff90`,
client `FUN_10034ba0`] --> The new types since 0.921 include RAT's features:
[800 Open Process Monitor](messages/processes.md#m800) to [804 Exit/Terminate
Process](messages/processes.md#m804), [900 Open View
Display](messages/view-display.md#m900) to [905 View Display
Keyboard](messages/view-display.md#m905), [420 Launch
File](messages/files.md#m420), [421 Empty Trash](messages/files.md#m421), [113
Get Extra Server Info](messages/session.md#m113), and [175 Upgrade Server
Software](messages/administration.md#m175). For RAT's own types, see
[rat.md](rat.md). [191 Login Failed](messages/administration.md#m191) is also
new in 1.000. <!-- [I] Haxial's version history in vendor/refs -->

## 1.100 and 1.110

The version history says that 1.100 changed the protocol and cannot connect to
older builds, including trackers. It added: <!-- [I] Haxial's version history
in vendor/refs -->

- DCC, a connection from client to client, with voice from NetFone; see
  [dcc.md](dcc.md).
- An Auto-Detect connection type, which tells a server, a tracker, and a client
  apart.
- Show My Address, which a server or tracker can answer.
- Remote tracker statistics.
- A server option for a separate port for file transfers.

The sub-version for each role does what Auto-Detect needs: 0x10 for a client,
0x20 for a server, and 0x30 for a tracker; see [Connection
types](handshake.md#connection-types). Client 1.110, the oldest 1.1 build, sends
0x10. <!-- [S] re/README.md lists its connection functions --> So the role
sub-versions probably came in 1.100, and the tracker then moved from `%KDt` to
`%KDX` with 0x30. <!-- [I] -->

TrackerServ 1.100 moved to port 10800 for both TCP and UDP; see
[tracker.md](tracker.md#ports). <!-- [I] Haxial's version history in vendor/refs
-->

Client 1.110 and Tracker 1.110 name these types, which 1.000 does not: <!-- [S]
name blocks in the binaries -->

- [115 Get My Address](messages/session.md#m115), [116 Your
  Address](messages/session.md#m116), and [117 SKPK](messages/session.md#m117).
- [720 Get TrackerServ Statistics](messages/tracker.md#m720) and [721
  TrackerServ Statistics](messages/tracker.md#m721).
- [1000 Start Voice](messages/voice.md#m1000) to [1003 Voice
  Data](messages/voice.md#m1003).

## 1.200

1.200 changed all text to Unicode. It can still connect to 1.1, but the text
can be wrong. <!-- [I] Haxial's version history in vendor/refs --> See
[encodings.md](encodings.md).

## 1.300

Server 1.300 adds [213 Must Join Chat](messages/chat.md#m213) to [223 Chat DCC
Addresses](messages/chat.md#m223), for permanent chats and voice chats. The name
of type 200 changes from "Create Chat" to "Create Private Chat", and the new
[217 Create Chat](messages/chat.md#m217) makes a permanent chat. <!-- [S] name
blocks in the binaries --> Client 1.320 and Client 1.400 have the same names.

## 1.500 and 1.520

Server 1.500 and Client 1.520 add [1100 Create Folder
Access](messages/folder-access.md#m1100) to [1107 Folder Access
List](messages/folder-access.md#m1107). <!-- [S] name blocks in the binaries -->

1.500 also added protection against connection floods, and 1.520 made flood
protection a setting of each account class. 1.520 sends the files of a queue
over one transfer connection, where before each file had its own. <!-- [I]
Haxial's version history in vendor/refs --> See [Flood
ban](sequences.md#flood-ban) and [transfers.md](transfers.md).

## 1.600 and 1.620

Server 1.600 for Windows adds [422 Volume Info](messages/files.md#m422) and
[423 Set Volume Info](messages/files.md#m423), for the Can See All Volumes
privilege. <!-- [S] name blocks in the binaries --> 1.620 changed only the Linux
builds. <!-- [I] Haxial's version history in vendor/refs -->

No name table has [1 Status](messages/status.md#m1), or types 112 and 118.
<!-- [S] -->
