# Missing message types

Message types that the protocol reference does not yet cover: types a program
handles or sends that have no handler or layout in
`docs/protocol/messages/README.md`. Static findings only, tagged as in the docs;
move each one into `docs/protocol/` once a capture or test confirms it.

## Server 1.620: session dispatcher

`FUN_0804fdac` is a compare tree on the message type. <!-- [S] --> Its cases
match the Handler column of the catalog exactly. Inline cases:

| Type | Inline action |
|---:|---|
| [108 Ping](../docs/protocol/messages/session.md#m108) | replies [109 Pong](../docs/protocol/messages/session.md#m109) |
| [115 Get My Address](../docs/protocol/messages/session.md#m115) | replies [116 Your Address](../docs/protocol/messages/session.md#m116) with the peer address string at conn +0x58 |
| [118 (unnamed)](../docs/protocol/messages/session.md#m118) | needs 4 bytes; sets conn byte +0x3b to 2 if the u32 is 0, else 1 (licence report; see the protocol reference) |
| [140 Open Connection Monitor](../docs/protocol/messages/monitoring.md#m140) / [141 Close Connection Monitor](../docs/protocol/messages/monitoring.md#m141) | set / clear conn byte +0x3c, then log a history record |
| [421 Empty Trash](../docs/protocol/messages/files.md#m421) | needs privilege bit at conn +0x45 & 4; replies [1 Status](../docs/protocol/messages/status.md#m1) success 0x1a5 |

Named types with no case, so the server ignores them on a session: <!-- [S] -->
[117 SKPK](../docs/protocol/messages/session.md#m117),
[404 Delete File/Folder](../docs/protocol/messages/files.md#m404),
[406 Rename File/Folder](../docs/protocol/messages/files.md#m406),
[408 Copy File/Folder](../docs/protocol/messages/files.md#m408), 900-905, and
1000-1003.

## Server 1.620: transfer dispatcher

`FUN_08052194` handles a transfer connection's requests: <!-- [S] -->

| Type | Handler |
|---:|---|
| [400 Get File List](../docs/protocol/messages/files.md#m400) | `FUN_0804d938` |
| [402 Create Folder](../docs/protocol/messages/files.md#m402) | `FUN_0804e0f8` |
| [412 Download File](../docs/protocol/messages/files.md#m412) | `FUN_08050984` |
| [415 Upload File](../docs/protocol/messages/files.md#m415) | `FUN_08051750` |
| [900 Open View Display](../docs/protocol/messages/view-display.md#m900) | `FUN_0805c0f0` |
| [904 View Display Mouse](../docs/protocol/messages/view-display.md#m904) | `FUN_0805c344` |
| [905 View Display Keyboard](../docs/protocol/messages/view-display.md#m905) | `FUN_0805c498` |

So View Display runs on a transfer connection, not on the session.

The Linux build cannot serve it. <!-- [S] --> 900 checks the View Display
privilege (conn +0x4c & 0x20), then `FUN_0805bb54` asks `FUN_080996d8` for
the screen, and that only raises 0x10004 (unimplemented). The mouse and keyboard
injectors behind 904 and 905 (`FUN_080996f4`, `FUN_080996fc`) are empty. So
against Server 1.620 for Linux, 900 can only get an error, and
[902 View Display Params](../docs/protocol/messages/view-display.md#m902)/[903 View Display Update](../docs/protocol/messages/view-display.md#m903)
need a Windows or Mac server.

## Server 1.620: name table

`FUN_08049d14` looks up types below 0x454 in the jump table at 0x080abc6c;
944 of its 1108 entries go to the "Unknown!" stub at 0x0804a310. <!-- [S] -->
The named entries are exactly the catalog's types, so no named type is missing
from the catalog. [112 (unnamed)](../docs/protocol/messages/session.md#m112) and
[118 (unnamed)](../docs/protocol/messages/session.md#m118) are handled but
unnamed. `re/scripts/jumptable.py 0x080abc6c 0x454` dumps it.

## Client 1.620 beta 5 (Linux): name table

The client's copy indexes its jump table at 0x08126aac by type − 8 (entry for
[100 Set Identity](../docs/protocol/messages/session.md#m100) at 0x08126c1c,
string at 0x08127be2). <!-- [S] --> Its names and types match the server's table
exactly.

## Tracker 1.110 (Linux)

The tracker's name table has the same names (entry for
[100 Set Identity](../docs/protocol/messages/session.md#m100) at 0x080c5b1c).
<!-- [S] --> Its TCP dispatcher `FUN_0804b378` handles: <!-- [S] -->

| Type | Handler |
|---:|---|
| [115 Get My Address](../docs/protocol/messages/session.md#m115) | inline: replies [116 Your Address](../docs/protocol/messages/session.md#m116) with the peer address string at conn +0x10 |
| [700 Get Tracker Group List](../docs/protocol/messages/tracker.md#m700) | `FUN_0804abf4` |
| [702 Get Tracker Server List](../docs/protocol/messages/tracker.md#m702) | `FUN_0804adcc` |
| [710 Get TrackerServ Settings](../docs/protocol/messages/tracker.md#m710) | `FUN_0804cb90` |
| [712 Set TrackerServ Settings](../docs/protocol/messages/tracker.md#m712) | `FUN_0804cc10` |
| [720 Get TrackerServ Statistics](../docs/protocol/messages/tracker.md#m720) | `FUN_0804b180` |

Servers register with the tracker over UDP, not through this dispatcher.
<!-- [S `FUN_0804c048`] --> See `docs/protocol/udp.md`.

## Client 1.620 beta 5 (Linux): dispatcher

`FUN_0804d7b4` handles these from a server: <!-- [S] -->

- notifications:
  [101 User List](../docs/protocol/messages/session.md#m101)-[108 Ping](../docs/protocol/messages/session.md#m108),
  [111 Mini User Icon Changed](../docs/protocol/messages/session.md#m111),
  [151 Disconnect Message](../docs/protocol/messages/administration.md#m151),
  [155 Rcv Broadcast Message](../docs/protocol/messages/administration.md#m155),
  [183 Server History Added](../docs/protocol/messages/administration.md#m183),
  [205 Chat User Joined](../docs/protocol/messages/chat.md#m205),
  [206 Chat User Departed](../docs/protocol/messages/chat.md#m206),
  [208 Receive Chat Text](../docs/protocol/messages/chat.md#m208),
  [210 Chat Topic Changed](../docs/protocol/messages/chat.md#m210),
  [212 Chat Invitation](../docs/protocol/messages/chat.md#m212),
  [213 Must Join Chat](../docs/protocol/messages/chat.md#m213),
  [221 Chat Deleted](../docs/protocol/messages/chat.md#m221),
  [301 Incoming Private Message](../docs/protocol/messages/private-messages.md#m301),
  [419 Notify Speed Limit](../docs/protocol/messages/files.md#m419),
  [451 Generate Catalog (S)](../docs/protocol/messages/files.md#m451),
  [515 Account Deleted](../docs/protocol/messages/accounts.md#m515),
  [613 News Message Posted](../docs/protocol/messages/news.md#m613),
  [614 News Message Deleted](../docs/protocol/messages/news.md#m614),
  [803 Process List](../docs/protocol/messages/processes.md#m803),
  [1102 Folder Access Deleted](../docs/protocol/messages/folder-access.md#m1102);
- replies: [1 Status](../docs/protocol/messages/status.md#m1),
  [109 Pong](../docs/protocol/messages/session.md#m109),
  [114 Extra Server Info](../docs/protocol/messages/session.md#m114),
  [116 Your Address](../docs/protocol/messages/session.md#m116),
  [143 Connection Monitor Info](../docs/protocol/messages/monitoring.md#m143),
  [153 User Info](../docs/protocol/messages/administration.md#m153),
  [171 Server Settings Info](../docs/protocol/messages/administration.md#m171),
  [173 Set Server Settings Done](../docs/protocol/messages/administration.md#m173),
  [181 Server History Join (R)](../docs/protocol/messages/administration.md#m181),
  [203 Chat Window Info](../docs/protocol/messages/chat.md#m203),
  [215 Chat List](../docs/protocol/messages/chat.md#m215),
  [216 Create Private Chat (R)](../docs/protocol/messages/chat.md#m216),
  [219 Chat Info](../docs/protocol/messages/chat.md#m219),
  [223 Chat DCC Addresses](../docs/protocol/messages/chat.md#m223),
  [401 File List](../docs/protocol/messages/files.md#m401),
  [410 File/Folder Info](../docs/protocol/messages/files.md#m410),
  [422 Volume Info](../docs/protocol/messages/files.md#m422),
  [453 File Search (R)](../docs/protocol/messages/files.md#m453),
  [503 Account Class Info](../docs/protocol/messages/accounts.md#m503),
  [506 Account Class List](../docs/protocol/messages/accounts.md#m506),
  [510 Account Info](../docs/protocol/messages/accounts.md#m510),
  [513 Account List](../docs/protocol/messages/accounts.md#m513),
  [551 Change Own Password Done](../docs/protocol/messages/accounts.md#m551),
  [603 Newsgroup Info](../docs/protocol/messages/news.md#m603),
  [606 Newsgroup List](../docs/protocol/messages/news.md#m606),
  [608 News Messages](../docs/protocol/messages/news.md#m608),
  [610 News Message](../docs/protocol/messages/news.md#m610), 803,
  [1104 Folder Access Info](../docs/protocol/messages/folder-access.md#m1104),
  [1107 Folder Access List](../docs/protocol/messages/folder-access.md#m1107).

The DCC connection (`FUN_0808f3e0`) handles 1,
[100 Set Identity](../docs/protocol/messages/session.md#m100),
[117 SKPK](../docs/protocol/messages/session.md#m117), and
[300 Send Private Message](../docs/protocol/messages/private-messages.md#m300);
the voice connection handles 1000-1003. See `docs/protocol/dcc.md`.

## Still to read

- [404 Delete File/Folder](../docs/protocol/messages/files.md#m404)/[406 Rename File/Folder](../docs/protocol/messages/files.md#m406)/[408 Copy File/Folder](../docs/protocol/messages/files.md#m408):
  no handler in the server or tracker, and no sender found in Client 1.620
  beta 5.
- The connection flood ban seen in `captures/kdx-accounts.pcapng` (30
  connections a minute); the per-request flood check is read.
- GConsole: the unknown fields of
  [4001 Draw](../docs/protocol/messages/gconsole.md#m4001)'s records and of
  [4200 Key](../docs/protocol/messages/gconsole.md#m4200) (see the open
  questions in `docs/protocol/gconsole.md`).

## Other behaviour found in the sweep

Not protocol, but worth knowing when running the originals: <!-- [S] -->

- Server console: besides `exit`, `quit`, `about`, `info`, and `setport`,
  there is an unlisted `prod` command. It marks every socket readable and
  writable so the event loop polls them all again (`FUN_08056b24`,
  `FUN_080972e4`).
- Server ports: the Alternate port and the File Transfer port are extra
  listeners with the same accept function as the main port (`FUN_0805f34c`,
  `FUN_080531f0`). [105 Server Info](../docs/protocol/messages/session.md#m105)
  offset 0x20 tells the client the File Transfer port.
- Tracker 1.110's about text says version 1.111. Its default groups are
  General, Trackers, Macintosh, Windows, Education, Business, and Games
  (`FUN_0804c7a8`).
- Client: a bookmark `Official Haxial Server` at `kdxserver.haxial.com`
  (`FUN_08058ce0`); a built-in IRC client for chat gateways (see the chat
  record in the protocol reference); an evaluation period and a signed
  Haxial Purchase Certificate, whose only protocol effect is the
  [118 (unnamed)](../docs/protocol/messages/session.md#m118) value.
