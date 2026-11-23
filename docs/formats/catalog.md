# `Catalog.dat`

**Status:** read/write. Byte-exact over three lab files; our encoder rebuilds
the server's file from its entries, index included.

The file catalog that [452 File Search](../protocol/messages/files.md#m452)
reads, written next to the server by
[450 Generate Catalog](../protocol/messages/files.md#m450) (the client's
"Generate File Catalog"). Clear text, magic `%FlK`. Evidence: files 50 to 52
in `crates/kdx-legacy/tests/fixtures/lab-2026-11-13/server/catalog/`,
generated on a fresh server, after one upload, and after a second upload and a
new folder. <!-- [O] -->

## Layout

Struct `CatalogFile` (dynamic):<a id="sCatalogFile"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `char[4]` | `magic` | `%FlK` |
| 0x04 | 2 | `u16` | `version` | 1 |
| 0x06 | 2 | `u16` | `ignored_06` | 1 |
| 0x08 | 4 | `u32` | `entry_count` |  |
| 0x0c | 4 | `u32` | `index_offset` | Where the index starts. |
| 0x10 | 8 | `u64` | `generated` | Generation time in milliseconds since 1900. |
| 0x18 | 24 | `u8[24]` | `ignored_18` | 0 |
| 0x30 |  | [`CatalogEntry`](#sCatalogEntry)`[]` | `entries` | `entry_count` entries. |
|  | `entry_count` | `u32[]` | `masks` | One search mask per entry. |
|  | `entry_count` | `u32[]` | `offsets` | The entries' offsets. |

### An entry

Every folder and file under `Bases/`, depth first, children sorted by name;
entry 0 is `Bases` itself.

Struct `CatalogEntry` (dynamic):<a id="sCatalogEntry"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 8 | `u64` | `size` | Bytes; 0 for a folder. |
| 0x08 | 8 | `u64` | `created` | Milliseconds since 1900. |
| 0x10 | 8 | `u64` | `modified` | The same. |
| 0x18 | 4 | `u32` | `kind` | 1 file, 2 folder. |
| 0x1c | 4 | `u32` | `descendants` | Folders: items below at any depth; files: 0 (files 51, 52). <!-- [O] --> |
| 0x20 | 4 | `u32` | `parent` | Index of the parent entry; 0 for `Bases`. |
| 0x24 | 4 | `u32` | `ignored_24` | 0 |
| 0x28 | 1 | `u8` | `flags` | `0x03` folder, `0x10` file. |
| 0x29 | 1 | `u8` | `ignored_29` | 0 |
| 0x2a | 1 | `u8` | `name_len` |  |
| 0x2b | 1 | `u8` | `ignored_2b` | 0 |
| 0x2c | `name_len` | `str` | `name` |  |
|  | pad4 | `u8[]` | `padding` | Zero. |

### The search mask

One `u32` per entry, in entry order: bit *n* is set when the name holds the
*n*-th letter of the alphabet, case-insensitively (`Bases` has bits 0, 1, 4,
18 set, `0x00040013`); a space sets bit 27, `.` bit 28, `[` and `]` bit 31
(every fixture name in files 50 to 52). <!-- [O] --> Digits and other
punctuation have not been seen; `name_mask` gives them bit 31. <!-- [I] -->
Search presumably tests the query's mask against each entry's before
comparing names.

`kdx_legacy::catalog::{Catalog, CatalogEntry, name_mask, decode_catalog,
encode_catalog}`, `LegacyDir::{load_catalog, save_catalog}` (no `.orig`: the
server regenerates it on request).
