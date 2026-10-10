# Missing opcodes

Message types that the protocol reference does not yet cover: types a program
handles or sends that have no handler or layout in
`docs/protocol/KDXProtocol.md`. Static findings only, tagged as in the docs;
move each one into `docs/protocol/` once a capture or test confirms it.

## Server 1.620: session dispatcher

`FUN_0804fdac` is a compare tree on the message type **[S]**. Its cases match
the Handler column of the catalog exactly. Inline cases:

| Type | Inline action |
|---:|---|
| 108 Ping | replies 109 Pong |
| 115 Get My Address | replies 116 with the peer address string at conn +0x58 |
| 118 (unnamed) | needs 4 bytes; sets conn byte +0x3b to 2 if the u32 is 0, else 1 |
| 140 / 141 | set / clear conn byte +0x3c, then log a history record |
| 421 Empty Trash | needs privilege bit at conn +0x45 & 4; replies 1 Status success 0x1a5 |

Named types with no case, so the server ignores them on a session **[S]**:
117 SKPK, 404 Delete File/Folder, 406 Rename File/Folder, 408 Copy File/Folder,
900–905, and 1000–1003.

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
**[S `FUN_0804c048`]**. See `docs/protocol/tracker.md`.

## Still to read

- The client-to-client service the clients run on TCP 10600 (DCC): which
  types it handles. 222/223 Chat DCC Addresses and 1000–1003 voice point
  there [I].
- 117 SKPK and 404/406/408: no handler in the server or tracker; look for
  senders in the clients.
- 901 Close View Display: no case in either server dispatcher.
