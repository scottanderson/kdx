# `Accounts.dat`

**Status:** verified (lab). Byte-exact over eleven lab files; an account
written by `kdx-legacy` logs in to KDX Server 1.620, which also reads back the
rewritten file (OpenKDX's `../OpenKDX/lab/legacy.sh`).

Account classes, accounts and Folder Access items. The file is XORed with a
fixed keystream (below); unscrambled it is a `%KDXAKNT` container of three
list sections whose records are the wire structures of
[503 Account Class Info](../protocol/messages/accounts.md#m503),
[510 Account Info](../protocol/messages/accounts.md#m510) and
[1105 Set Folder Access](../protocol/messages/folder-access.md#m1105).

Evidence: "file NN" is a diff between consecutive files in
`crates/kdx-legacy/tests/fixtures/lab-2026-11-13/server/accounts/` (one CLI
command each, listed in that directory's README), written by the OpenKDX
scenario `../OpenKDX/lab/scenarios/legacy-samples.sh`.

## Scrambling

The whole file is XORed, big-endian 32-bit word by word, with the states of a
linear congruential generator; a trailing partial word uses the leading bytes
of the next state. Nothing but the contents goes in: the same accounts give
the same bytes on another host, in another directory, under another admin
login (a second first run under a different hostname). <!-- [O] -->

| Program and file | multiplier | increment | first state (over bytes 0..4) |
|---|---|---|---|
| Server, `Accounts.dat` | `0x41C64E6D` | `12345` | `0x9ad22861` |
| Tracker, `KDXTrackerServ.stg` | `0x41a609fd` | `0x1379` | `0xacb8fa74` |
| Client, `KDX.stg` | `0x41a609fb` | `0x2319` | `0x35662cb6` |

How it was found (files 22, 23, 24): renaming an account from `Name Y` to
`Name Z` changed one byte of the file by `0x03` (`'Y' ^ 'Z'`), so the
scrambling is a position-keyed XOR; the two files before and after a password
change differ in 32 bytes whose plaintexts are the two derived keys
(`AccountKey::from_password`), which gave 32 keystream bytes; those satisfy
`s[i+1] = 0x41C64E6D * s[i] + 12345 mod 2^32`, the ANSI C `rand()`
recurrence; running it backwards gives the first state. The tracker and client
constants came the same way from the long zero runs in their headers
(ciphertext = keystream where plaintext is zero), fitted over the whole file.
<!-- [O] -->

`kdx_legacy::scramble::{Stream, scramble, unscramble}`.

## Plain layout

Struct `AccountsFile` (dynamic):<a id="sAccountsFile"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 8 | `char[8]` | `magic` | `%KDXAKNT` |
| 0x08 | 2 | `u16` | `version` | 2 |
| 0x0a | 2 | `u8[2]` | `ignored_0a` | 0 |
| 0x0c | 4 | `u8[4]` | `ignored_0c` | 0 |
| 0x10 | 56 | `u32[14]` | `table` | 7 pairs of (offset from the file start, length); `(0, 0)` when absent |
| 0x48 | 8 | `u8[8]` | `ignored_48` | `00 00 01 01 02 00 00 00` in every file |
| 0x50 |  | `u8[]` | `sections` | In index order, each starting 8-aligned (file 23). <!-- [O] --> |
|  | 4 | `u8[4]` | `trailer` | Zero, or the keystream (so zero in the file); the server writes either (files 20 to 26). <!-- [O] --> |

The bytes between a section's end and the next 8-aligned offset are not zeroed
by the server (`4f 72 87 dc` in one file) and are kept as read (file 23).
<!-- [O] -->

| Section | Records | Wire twin |
|---|---|---|
| 0 | account classes, 12 | [`AccountClassInfo`](../protocol/messages/accounts.md#sAccountClassInfo): 0x38-byte header, name / second string / editor login, pad 4 |
| 1 | accounts | [`Account`](../protocol/messages/accounts.md#sAccount): 0x80-byte header, login / name / third / editor, pad 4; the key is the 32-byte derived key |
| 2 | Folder Access items, sorted by name | [`FolderAccess`](../protocol/messages/folder-access.md#sFolderAccess) without the current-name string: 0xb8-byte header, name / owner / description, pad 4 |
| 3 to 6 | never seen | |

A list section: `47 fd db 01`, `u32` count, `u32` 0, `u32[count]` cumulative
end offsets of the records (from the first record's start), then the records
(files 20, 25). Record padding bytes are not zeroed by the server either
(`40 00 00` after `Administrator`); the wire decoders keep them. <!-- [O] -->

Observations: a deleted account is removed, not tombstoned (file 26); the
server rewrites the whole file on every change; admin's `ignored_58` bytes
change on every write (a counter) (files 20 to 26); a class rename grew the
class record by the editor login (file 23). <!-- [O] -->

## Not yet seen

Sections 3 to 6; more than 12 classes (the client allows 12); an account with
the third string set.
