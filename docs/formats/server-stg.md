# `KDXServer.stg`

**Status:** verified (lab). Byte-exact over 18 lab files; Server 1.620 starts
on our rewrite and shows the name we set (OpenKDX's `../OpenKDX/lab/legacy.sh`).

The server's settings. The file is the same `%KDXSSTG` blob the wire carries
in [171 Server Settings Info](../protocol/messages/administration.md#m171)
and [172 Set Server Settings](../protocol/messages/administration.md#m172),
so [`ServerSettings`](../protocol/messages/administration.md#sServerSettings)
of `kdx-proto` is the one model for both, with its typed
[`MainSettings`](../protocol/messages/administration.md#sMainSettings),
[`TrackerEntry`](../protocol/messages/administration.md#sTrackerEntry) and
[`AddressRule`](../protocol/messages/administration.md#sAddressRule);
`kdx_legacy::server_stg` adds the file checks, `validate_settings` and the
live-store methods. The server rewrites the whole file on every 172 it
accepts. <!-- [O] -->

Evidence: "file NN" is a diff between consecutive snapshots in
`crates/kdx-legacy/tests/fixtures/lab-2026-11-13/server/stg/`, each after one
change in the Windows client's Server Settings window (OpenKDX's
`../OpenKDX/lab/scenarios/legacy-samples.sh` lists the steps). The byte
layout is on the protocol page; this page records what the files add to it.

## The header

The version at 0x08 is 1; `value_64` is `01 01 01 01 00 01 00 02 00 00 00` in
every file, and the byte at 0x6f is the number of tracker entries in section
1 (files 12, 15). <!-- [O] -->

Table entries for an empty section: sections 1, 2, 3, 5 and 7 carry the
running offset with length 0; 4, 6, 8, 9 and 10 carry `(0, 0)` and have never
been seen non-empty (files 00 to 16). <!-- [O] --> A client's 172 differs: 7
is `(0, 0)` and 5 is `(offset, 0)` when the icon is left out. The model keeps
a `placeholders` flag per section so both forms re-encode exactly.

| Section | Holds | Page in the client |
|---|---|---|
| 0 | main | General Settings, Port Numbers, Server History, flags |
| 1 | trackers | Trackers |
| 2 | allow/deny list | Allow/Deny Addresses |
| 3 | greeting: 4 bytes then the text, as [107 Server Greeting](../protocol/messages/session.md#m107) carries it | Greeting |
| 5 | the server icon, as [106 Server Icon](../protocol/messages/session.md#m106) carries it | General Settings |
| 7 | chat rooms (`Public Chat`, `Voice Chat`, `Admin Lounge` on a fresh server; 0x104 bytes) | none |

## Section 0: main

Which window field is which byte, from one change per file: <!-- [O] -->

| Byte | Field | File |
|---|---|---|
| 0x12 | file transfer port, 0 = none | 14 |
| 0x16 | alternate port, 0 = none | 05 |
| 0x18 | history kinds, a bitmask in the window's order: bit 0 Login, 1 Disconnected, then Download File, Upload File, Delete File, Get File List, Miscellaneous File, Modify Account, Miscellaneous Account, Server Settings, Miscellaneous | 07, 08 (bits 2 to 10 inferred <!-- [I] -->) |
| 0x1c | history: start a new file when size exceeds, bytes (1 MiB default) | 09 |
| 0x23 | flags: bit 0 register with trackers, bit 1 greeting on | 13, 06 |
| 0x2c | twelve bytes that differ per server directory; an identifier? <!-- [I] --> | |
| 0x38 | total outgoing speed limit, bytes/s; 0 = off | 04 |
| 0x52 | name length | 01 |
| 0x53 | description length | 02 |
| 0x54 | 0 in every file; sits among the length bytes, so the decoder refuses a non-zero value rather than misread the strings | |
| 0x55 | address length | 03 |

The zero after the name is the section's `pad4`, which is absent when the
strings end on a multiple of 4 (file 03: 0x5c + 34 = 0x7e, two zero bytes; a
63-byte name alone gives one) (files 02, 03). <!-- [O] -->

## Section 1: a tracker entry

0x20-byte header, strings, pad to 4 (files 12, 15). <!-- [O] --> The bytes at
0x18, 0x19 and 0x1b are zero in every file and refused when set, like 0x54
of the main section.

The password typed into Edit Tracker (five characters, visible as dots in the
lab's screenshot `legacy-samples-tracker-dialog.png`) does not appear in the
file written afterwards, and no length byte changes: the server does not keep
it in `KDXServer.stg` (file 15). <!-- [O] -->

## Section 2: an allow/deny entry

0x1c-byte header, strings, pad to 4 (files 10, 11, 16). <!-- [O] --> Byte
0x1b is zero in every file and refused when set.

An entry whose expiry has passed is dropped the next time the server writes
the file: the allow entry of file 11 (expiry = the minute it was made) is gone
from file 12. <!-- [O] -->

Strings are capped at 255 bytes by their length byte;
`kdx_legacy::server_stg::validate_settings` rejects longer ones before a save,
and the encoder never writes more bytes than the length byte says. Bytes
after the last section are kept in `trailing` and written back.

## Not yet seen

Sections 4, 6, 8 to 10; the icon in a file (only on the wire so far); a
description over 255 bytes.
