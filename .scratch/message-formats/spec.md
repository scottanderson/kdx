# Message formats

Goal: every byte of every message payload, on the server, the tracker, and
client-to-client connections, has a name and a meaning backed by **[S]** or
**[O]** evidence, or is proven unused. Then `kdx-dissect` can print each
payload as its struct, in wire order, and fall back to a hexdump only when
a payload doesn't fit (issue 15).

## What "understood" means

A field is understood when the protocol docs give it one of these:

- a name and meaning, tagged **[S]** with a function address or **[O]**
  with a capture and frame;
- "unused", with **[S]** evidence that no reader in Server 1.620, Client
  1.620 beta 5, or Tracker 1.110 looks at it, and what the writer puts
  there (zero, a copy, or uninitialised memory);
- "not decidable from the Linux builds", with the reason, when only a
  Windows or Mac build could say more and that build can't be run or read
  usefully.

An **[I]** alone does not count. Unnamed table rows (an empty Field
column), rows that are just `0`, and offsets missing from a table are gaps.

## Which builds

Use the Linux builds first, for both static analysis and captures:

1. Server 1.620 (`kdx`), Client 1.620 beta 5 (`kdxlnxclient`), and Tracker
   1.110 (`kdxtracker`) in Ghidra, and the lab's Linux server, client, and
   tracker for captures.
2. Server 1.600 for Windows under Wine (`lab/kdx-lab server win`), Client
   1.600 for Windows (`kdxclient`), or Tracker 1.110 for Windows, only when
   the Linux builds can't settle a gap: a stub, a handler they lack, a
   field they never write or read. Write down the Linux evidence that makes
   the Windows build necessary. Tracker 1.110 for Windows is not in
   `kdx-lab` yet, and the Windows server has no Ghidra project in
   `re/README.md` yet; import it the same way as the others.
3. Mac builds, statically only (the lab can't run PowerPC), and only when
   no Windows build can settle the gap either.

The Windows and Mac builds are 1.600, older than the Linux 1.620 builds.
Note any difference between versions instead of assuming one layout.

## How to work an issue

Issues here are static analysis plus doc work, for an agent alone.

1. Set `Status: claimed` in the issue.
2. Re-read the existing captures before making new ones: run `kdx-dissect`
   on `captures/*.pcapng` (passwords in
   `crates/kdx-dissect/tests/captures.rs`) and look for the type.
3. Read the handlers and builders in Ghidra (`re/README.md`). Start from the
   function addresses the doc already cites.
4. When a gap needs a capture, write a capture-gaps issue (below) instead of
   driving the GUI from here. Link it from the issue's comments, and keep
   working on the static gaps.
5. Put each finding in `docs/protocol/`, tagged as `AGENTS.md` says, and add
   new landmarks to `re/README.md`. When a byte gets a name, rename the
   `kdx-proto` field that holds it, and keep its round-trip tests byte for
   byte.
6. Tick the gap off in the issue. When none are left, set `Status: done`
   and add a comment summarising the result.

## Handing off a capture

A capture needs the operator, so it goes to `.scratch/capture-gaps/`:

- Create `.scratch/capture-gaps/issues/NN-<slug>.md` with the next free
  number, and add a row for it to the table in
  `.scratch/capture-gaps/spec.md`.
- Header lines: `Status: ready-for-human`, `Scenario:` (`linux-…`, or
  `win-…` for the Windows server), `Targets:` (types and offsets), and
  `Origin: message-formats NN`.
- Give the setup the agent does, then numbered GUI steps with the types
  each step should produce, as the other capture-gaps issues do. Mark
  guessed results [I].
- Prefer one scenario per issue here, with as many gaps as fit in one
  session.

## Gap inventory

Collected on 2026-10-13 from `docs/protocol/KDXProtocol.md`, `tracker.md`,
`dcc.md`, `handshake.md`, and `session.md`, and from `kdx-dissect` runs
over all 22 captures. Every type in the catalog has a section; each issue
lists its gaps one by one.

| Issue | Area | Types |
|---|---|---|
| 01 | codec coverage audit | every captured type |
| 02 | common encodings: Status, user record, icon bitmap, handshake, session | 1, 100–103, 106 |
| 03 | session | 104–118 |
| 04 | monitoring and administration | 140–155, 174, 175, 180–191 |
| 05 | server settings blob | 170–173 |
| 06 | chat and private messages, chat record | 200–223, 300, 301 |
| 07 | files and transfers | 400–453 |
| 08 | accounts and privileges | 502–515, 550, 551 |
| 09 | news | 600–614 |
| 10 | processes | 800–804 |
| 11 | folder access | 1100–1107 |
| 12 | view display | 900–905 |
| 13 | tracker, UDP and TCP | registration, 107/115/116 on a tracker, 700–721 |
| 14 | client-to-client (DCC) and voice | 117, 100/300 on DCC, 1000–1003 |
| 15 | `kdx-dissect` prints structs | all; blocked by 01–14 |

## Contradictions

The sweep found places where two docs, or a doc and `kdx-proto`, disagree.
Each is a gap in its issue as well; settle it there, fix both sides, and
strike it here.

| Issue | Where | One says | The other says |
|---|---|---|---|
| 03 | 105 Server Info 0x00 | Session table: "probably a session ID for transfer connections" [I] | Transfer connections: the account key for an attached login |
| 03 | 105 Server Info 0x20 | Session table: File Transfer port [S client `FUN_08052cc0`] | `ServerInfo::server_value`: "unknown meaning" |
| 06 | chat record 0x04..0x13 | Chat record table: key at 0x04, then 0xc bytes at 0x08 | 217 Create Chat table: 0x10 bytes of 0 at 0x04 |
| 06 | chat record 0x2c..0x2d | Chat record table: IRC server and channel lengths | 217 Create Chat table: a u32 0 |
| 07 | 412 Download File options | 412: privilege bits 25 and 65 [I] | Access Privileges table: Allow Simultaneous Downloads (25) and Disable Encryption (65) |
| 08 | 503 Account Class Info 0x25 | Doc: flood protection | `AccountClass::unknown_25` |
| 11 | 1107 Folder Access List 0x00, 0x04 | Doc: flags [S], icon [S] | `FolderAccessItem::unknown_00`, `item_id` |

Types never captured: 117, 404, 406, 408, 514, 901, 1000–1003, and 108/109
from a GUI client (only `kdx-probe` sent them). 223 has been captured only
empty. 184–191 are history record kinds, not messages.

Out of scope: GConsole (4000, 4001, 4200, 4201), a remote-console
connection type in the client's connection dialog (port 10900). The client
is the display; no KDX server or tracker serves the other end, so nothing
in the lab can produce it (`dcc.md`, GConsole).
