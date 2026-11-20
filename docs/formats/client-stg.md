# `KDX.stg`

**Status:** verified (lab) for the identity and the address book (byte-exact
over six lab files; Client 1.600 starts on a file we wrote and logs in with
the name we set, OpenKDX's `../OpenKDX/lab/scenarios/client-samples.sh`).
Window positions, settings and registration are kept as bytes.

The client's settings, written on a clean exit (palette, Exit; a killed client
loses changes). Scrambled with the client's stream
([accounts-dat.md](accounts-dat.md#scrambling): `0x41a609fb`, `0x2319`,
`0x35662cb6`).

Evidence: files 60 to 64 in
`crates/kdx-legacy/tests/fixtures/lab-2026-11-13/client/`, one change per
exit (listed in that directory's README). <!-- [O] -->

## Plain layout

Struct `ClientSettingsFile` (dynamic):<a id="sClientSettingsFile"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 8 | `char[8]` | `magic` | `%KDXSTNG` |
| 0x08 | 2 | `u16` | `version` | 2 |
| 0x0a | 2 | `u8[2]` | `ignored_0a` | 0 |
| 0x0c | 4 | `u8[4]` | `ignored_0c` | 0 |
| 0x10 | 96 | `u32[24]` | `table` | 12 pairs of (offset, length). |
| 0x70 | 16 | `u8[16]` | `ignored_70` | `00 00 00 00 03 01 00 00 00 00 00 02 01 01 01 01` |
| 0x80 |  | `u8[]` | `sections` | Contiguous, in the order 0, 1, 7, 8, 10, 9, 11 (not index order; the model keeps `order`). |

| Section | Length (fresh) | Holds |
|---|---|---|
| 0 | 0x214 | window positions and sizes, opaque |
| 1 | 0x234 + name + description | identity, below |
| 7 | 0xe8 + entries | address book, below |
| 8 | 0x74 | opaque (starts `b7 1a 00 02`) |
| 9 | 0x38 | zero |
| 10 | 0x24 | 36 bytes that look random; registration or a machine id <!-- [I] --> |
| 11 | 0x18 | six 4-byte values `FD\0d`, `FD\0f`, and so on; sound or volume settings <!-- [I] --> |

### Section 1: identity

Struct `ClientIdentity` (dynamic):<a id="sClientIdentity"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 12 | `u8[12]` | `ignored_00` | 0 |
| 0x0c | 4 | `u8[4]` | `ignored_0c` | `00 ff cc 00` (the background color again?) |
| 0x10 | 12 | `u8[12]` | `ignored_10` | `00 00 02 0b` then zero. |
| 0x1c | 1 | `u8` | `name_len` | (file 61) <!-- [O] --> |
| 0x1d | 1 | `u8` | `description_len` | (file 62) <!-- [O] --> |
| 0x1e | 2 | `u8[2]` | `ignored_1e` | 0 |
| 0x20 | 2 | `u8[2]` | `ignored_20` | `01 02` |
| 0x22 | 3 | `u8[3]` | `ignored_22` | `ff ff ff` |
| 0x25 | 3 | `u8[3]` | `background_color` | `ff cc 00`, the client's default. <!-- [I] --> |
| 0x28 | 3 | `u8[3]` | `text_color` | `00 00 00` <!-- [I] --> |
| 0x2b | 512 | `u8[512]` | `icon` | The 32 by 32 icon, 4 bits per pixel, 16 bytes per row. |
| 0x22b | `name_len` | `str` | `name` |  |
|  | `description_len` | `str` | `description` |  |
|  | pad4 | `u8[]` | `padding` | Zero. |

The two colors are inferred from the client's defaults (text black on
`#ffcc00`, as it sends in
[100 Set Identity](../protocol/messages/session.md#m100)); a color-change
sample is still to be made.

### Section 7: address book

A list: `38 fd db 01`, `u32` count, `u32` (equal to the count in every file;
kept as read), `u32` 0, `u32[count]` entry ids (1, 2, 3 and on), `u32` 0,
`u32[count]` cumulative end offsets, then the records (files 63, 64).
<!-- [O] -->

Struct `AddressBookEntry` (dynamic):<a id="sAddressBookEntry"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 16 | `u8[16]` | `ignored_00` | 0 |
| 0x10 | 4 | `u32` | `flags` | `01 00 00 00` on the stock entry, `00 00 00 80` on one made in New Address Book Item with its default check boxes; not pinned. |
| 0x14 | 4 | `u32` | `ignored_14` | `0x16` on the stock entry, 0 otherwise. |
| 0x18 | 4 | `u32` | `ignored_18` | 0 |
| 0x1c | 32 | `u8[32]` | `key` | The password as `AccountKey::from_password`; the empty-password key when empty (file 63). <!-- [O] --> |
| 0x3c | 64 | `u8[64]` | `ignored_3c` | 0 |
| 0x7c | 5 | `u8[5]` | `lengths` | Name, description, comments, address, login. |
| 0x81 | 3 | `u8[3]` | `ignored_81` | 0 |
| 0x84 |  | `str[5]` | `strings` | Name, description, comments, address, login. |
|  | pad4 | `u8[]` | `padding` | As read. |

The stock entry (`Official Haxial Server`, `kdxserver.haxial.com`) keeps its
one-line text in *comments* (the Description column is empty in the window);
an entry from the window puts the Description field at 0x7d (files 60, 63).
<!-- [O] -->

`kdx_legacy::client_stg::{ClientSettings, ClientIdentity, AddressBookEntry,
decode_client_stg, encode_client_stg}`.
