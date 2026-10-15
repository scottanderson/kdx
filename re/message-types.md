# Missing message types

Message types that the protocol reference does not yet cover: types a program
handles or sends that have no handler or layout in
`docs/protocol/messages/README.md`. Static findings only, tagged as in the docs;
move each one into `docs/protocol/` once a capture or test confirms it.

## Server 1.620: session dispatcher

`FUN_0804fdac` is a compare tree on the message type **[S]**. Its cases match
the Handler column of the catalog exactly. Inline cases:

| Type | Inline action |
|---:|---|
| 108 Ping | replies 109 Pong |
| 115 Get My Address | replies 116 with the peer address string at conn +0x58 |
| 118 (unnamed) | needs 4 bytes; sets conn byte +0x3b to 2 if the u32 is 0, else 1 (licence report; see the protocol reference) |
| 140 / 141 | set / clear conn byte +0x3c, then log a history record |
| 421 Empty Trash | needs privilege bit at conn +0x45 & 4; replies 1 Status success 0x1a5 |

Named types with no case, so the server ignores them on a session **[S]**:
117 SKPK, 404 Delete File/Folder, 406 Rename File/Folder, 408 Copy File/Folder,
900-905, and 1000-1003.

## Server 1.620: transfer dispatcher

`FUN_08052194` handles a transfer connection's requests **[S]**:

| Type | Handler |
|---:|---|
| 400 Get File List | `FUN_0804d938` |
| 402 Create Folder | `FUN_0804e0f8` |
| 412 Download File | `FUN_08050984` |
| 415 Upload File | `FUN_08051750` |
| 900 Open View Display | `FUN_0805c0f0` |
| 904 View Display Mouse | `FUN_0805c344` |
| 905 View Display Keyboard | `FUN_0805c498` |

So View Display runs on a transfer connection, not on the session.

The Linux build cannot serve it **[S]**. 900 checks the View Display
privilege (conn +0x4c & 0x20), then `FUN_0805bb54` asks `FUN_080996d8` for
the screen, and that only raises 0x10004 (unimplemented). The mouse and keyboard injectors
behind 904 and 905 (`FUN_080996f4`, `FUN_080996fc`) are empty. So against
Server 1.620 for Linux, 900 can only get an error, and 902/903 need a
Windows or Mac server.

## Server 1.620: name table

`FUN_08049d14` looks up types below 0x454 in the jump table at 0x080abc6c;
944 of its 1108 entries go to the "Unknown!" stub at 0x0804a310 **[S]**. The
named entries are exactly the catalog's types, so no named type is missing
from the catalog. 112 and 118 are handled but unnamed.
`re/scripts/jumptable.py 0x080abc6c 0x454` dumps it.

## Client 1.620 beta 5 (Linux): name table

The client's copy indexes its jump table at 0x08126aac by type − 8 (entry for
100 Set Identity at 0x08126c1c, string at 0x08127be2) **[S]**. Its names and
types match the server's table exactly.

## Tracker 1.110 (Linux)

The tracker's name table has the same names (entry for 100 at 0x080c5b1c)
**[S]**. Its TCP dispatcher `FUN_0804b378` handles **[S]**:

| Type | Handler |
|---:|---|
| 115 Get My Address | inline: replies 116 with the peer address string at conn +0x10 |
| 700 Get Tracker Group List | `FUN_0804abf4` |
| 702 Get Tracker Server List | `FUN_0804adcc` |
| 710 Get TrackerServ Settings | `FUN_0804cb90` |
| 712 Set TrackerServ Settings | `FUN_0804cc10` |
| 720 Get TrackerServ Statistics | `FUN_0804b180` |

Servers register with the tracker over UDP, not through this dispatcher
**[S `FUN_0804c048`]**. See `docs/protocol/udp.md`.

## Client 1.620 beta 5 (Linux): dispatcher

`FUN_0804d7b4` handles these from a server **[S]**:

- notifications: 101-108, 111, 151, 155, 183, 205, 206, 208, 210, 212, 213,
  221, 301, 419, 451, 515, 613, 614, 803, 1102;
- replies: 1, 109, 114, 116, 143, 153, 171, 173, 181, 203, 215, 216, 219,
  223, 401, 410, 422, 453, 503, 506, 510, 513, 551, 603, 606, 608, 610, 803,
  1104, 1107.

The DCC connection (`FUN_0808f3e0`) handles 1, 100, 117, and 300; the voice
connection handles 1000-1003. See `docs/protocol/dcc.md`.

## Still to read

- 404/406/408: no handler in the server or tracker, and no sender found in
  Client 1.620 beta 5.
- The connection flood ban seen in `captures/kdx-accounts.pcapng` (30
  connections a minute); the per-request flood check is read.
- GConsole: the unknown fields of 4001's records and of 4200 (see the open
  questions in `docs/protocol/gconsole.md`).

## Other behaviour found in the sweep

Not protocol, but worth knowing when running the originals **[S]**:

- Server console: besides `exit`, `quit`, `about`, `info`, and `setport`,
  there is an unlisted `prod` command. It marks every socket readable and
  writable so the event loop polls them all again (`FUN_08056b24`,
  `FUN_080972e4`).
- Server ports: the Alternate port and the File Transfer port are extra
  listeners with the same accept function as the main port (`FUN_0805f34c`,
  `FUN_080531f0`). 105 Server Info offset 0x20 tells the client the File
  Transfer port.
- Tracker 1.110's about text says version 1.111. Its default groups are
  General, Trackers, Macintosh, Windows, Education, Business, and Games
  (`FUN_0804c7a8`).
- Client: a bookmark `Official Haxial Server` at `kdxserver.haxial.com`
  (`FUN_08058ce0`); a built-in IRC client for chat gateways (see the chat
  record in the protocol reference); an evaluation period and a signed
  Haxial Purchase Certificate, whose only protocol effect is the 118 value.
