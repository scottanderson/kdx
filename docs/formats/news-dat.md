# `News.dat`

**Status:** read/write. Byte-exact over seven lab files; a post appended by
`kdx-legacy` to the file before a post produces the file the server wrote
after it.

Newsgroups and their messages. Clear text, `%KDX` + `NUUZ`. A fresh server
creates it empty (0 bytes) and writes it on the first change; an empty file
decodes to an empty store (files 30, 31). <!-- [O] -->

Evidence: "file NN" is a diff between consecutive files in
`crates/kdx-legacy/tests/fixtures/lab-2026-11-13/server/news/` (the lab's
legacy samples scenario, one news command of the command-line client
each, listed in that directory's README).

## Layout

Struct `NewsFile` (dynamic):<a id="sNewsFile"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 8 | `char[8]` | `magic` | `%KDXNUUZ` |
| 0x08 | 2 | `u16` | `version` | 2 |
| 0x0a | 2 | `u16` | `group_count` | (file 33) <!-- [O] --> |
| 0x0c | 4 | `u8[4]` | `ignored_0c` | 0 |
| 0x10 | 40 | `u32[10]` | `table` | 5 pairs of (offset, length); only entry 1 is used: `(0x40, groups section length)`. |
| 0x38 | 8 | `u8[8]` | `ignored_38` | `00 00 00 00 01 00 00 00` |
| 0x40 |  | [`NewsGroupEntry`](#sNewsGroupEntry)`[]` | `groups` | `group_count` entries, each padded to 4. |
|  |  | `u8[]` | `lists` | One message list per group, in group order, each at an 8-aligned offset. |
|  | rest | `u8[]` | `trailer` | 0 to 7 bytes to a multiple of 8; whatever the server's buffer held (`9d 40 00 00`, `00 00 00 02`, zeros). |

The bytes between the groups section and the first list, and between lists,
are the same kind of uninitialized padding (`om C` in one file) and are kept
as read (files 32, 33). <!-- [O] -->

### A group entry

Struct `NewsGroupEntry` (dynamic):<a id="sNewsGroupEntry"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `u32` | `list_offset` | Of this group's message list. |
| 0x04 | 4 | `u32` | `list_len` |  |
| 0x08 | 4 | `u32` | `last_message_id` | The last id handed out: 1 in a new group, so the first post gets 2; a deleted id is not reused (files 32, 34, 35, 37). <!-- [O] --> |
| 0x0c | 12 | `u8[12]` | `ignored_0c` | 0 |
| 0x18 |  | [`NewsgroupInfo`](../protocol/messages/news.md#sNewsgroupInfo) | `info` | As [603 Newsgroup Info](../protocol/messages/news.md#m603) carries it. |
|  | pad4 | `u8[]` | `padding` |  |

`ignored_20` of the `NewsgroupInfo` is `01 00 00 00` for `General News` and
zero for a created group; probably the built-in flag. <!-- [I] -->

### A message list

`47 fd db 01`, `u32` count, `u32` 0, `u32[count]` cumulative end offsets, then
the wire [`NewsMessage`](../protocol/messages/news.md#sNewsMessage) records
(time, group id, message id, body length, poster, login, subject, body with CR
line ends), each padded to 4 (files 32, 34). <!-- [O] -->

## Behavior seen

- A deleted message is removed from the list and the count; the id counter
  keeps going (file 35). <!-- [O] -->
- When a group exceeds its size limit the oldest messages are moved to
  `News Archives/<group>/<group> <yymmdd>.txt`
  ([news-archives.md](news-archives.md)) and removed from the file; four
  1.5 KB posts into a 4 KiB group left one (file 37). <!-- [O] -->
- The stock welcome post of a fresh server is Haxial's text; the fixtures
  start after its deletion.

`kdx_legacy::news::{NewsStore, NewsGroupRecord, decode_news, encode_news}`
and `LegacyDir::{load_news, save_news}`; `NewsStore::{post, delete_message,
create_group, delete_group}` keep the counters as the server does.
