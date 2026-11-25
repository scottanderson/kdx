# `.hap` appearance files

**Status:** read/write. Byte-exact over all 111 files of Haxial's appearance
pack (`Appearances041205.sit`, lab-only: `~/haxial/legacy/appearances/`);
`kdx_legacy::hap::{Theme, decode_hap, encode_hap}`. Which color or image slot
the client draws where is not known yet (the editor's lists give the names,
below).

Evidence: the 111 files, read with `lab_every_pack_file_round_trips`
(`crates/kdx-legacy/tests/hap.rs`, `KDX_LAB_HAP_DIR`). <!-- [O] --> No file
of the pack is committed: it is Haxial's and third parties' artwork. The
constructed-theme tests cover the writer with pixels of ours. Appearance
Editor 1.240 could not be used for one-field diffs: it ignores a file given on
the command line and its Haxial-drawn Save/Open dialog lists no volumes under
Wine, so nothing can be saved from it.

## Layout

Struct `HapFile` (dynamic):<a id="sHapFile"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `char[4]` | `magic` | `%HAP` |
| 0x04 | 2 | `u16` | `version` | 1 |
| 0x06 | 6 | `u8[6]` | `ignored_06` | 0 |
| 0x0c | 104 | `u32[26]` | `table` | 13 pairs of (offset, length). |
| 0x74 | 20 | `u8[20]` | `ignored_74` | Eight 0, `01 01 01 01`, eight 0, in every file. |
| 0x88 |  | `u8[]` | `sections` | Contiguous and in index order; every length is a multiple of 4. The decoder accepts only this shape (and an empty section at offset 0 or the running offset), so a successful decode means the file rewrites byte for byte. |

| Section | Holds |
|---|---|
| 4 | information: created, modified (milliseconds since 1900), 18 zero bytes, four length bytes (name, version, creator, description), 14 zero bytes, the strings, pad 4 |
| 5 | images: a 528-slot table, then bitmap records with the 12-byte extra header |
| 6 | 204 colors, `00 RR GG BB` each |
| 7 | icons: a 768-slot table, then bitmap records without the extra header |
| others | never seen non-empty; empty 5 and 7 carry the running offset, the others `(0, 0)` (`Theme::default` marks 4 to 7 as placeholders; 4 and 6 are never empty) |

A slot table is one `u32` per UI element: 0 for none, else the record's
offset from the section start; several slots may share one record, and every
record is used by at least one slot (a table of zeros with no records is
rejected). Records follow the table contiguously in offset order.

### A bitmap record

Struct `HapBitmap` (dynamic):<a id="sHapBitmap"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 2 | `u16` | `width` |  |
| 0x02 | 2 | `u16` | `height` |  |
| 0x04 | 1 | `u8` | `transparent_flag` | 1 when byte 7 names a transparent palette index. <!-- [I] --> 0 or 1; anything else is rejected. |
| 0x05 | 1 | `u8` | `depth` | Bits per pixel: 1, 2, 4 or 8. |
| 0x06 | 1 | `u8` | `palette_last` | Palette entries minus 1. |
| 0x07 | 1 | `u8` | `transparent` | The transparent index when the flag is set; otherwise usually 0, kept as read. <!-- [I] --> |
| 0x08 | 12 | `u8[12]` | `extra` | Images only: a color (`00 RR GG BB`), four bytes that look like insets (`08 08 07 07`, `0b 0b 0a 0a`), a word (0, `04 00 00 00`, and so on). <!-- [I] --> |
|  |  | `u32[]` | `palette` | `palette_last + 1` entries, `00 RR GG BB`. |
|  | rest | `u8[]` | `pixels` | `stride * height` bytes, most significant bits first, rows padded to 4 bytes. |

Every images and icons section of the pack tiles exactly with this record.
<!-- [O] -->

## The editor's names

Appearance Editor 1.240's Colors, Images and Icons pages list the slots by
name in order (screenshots `~/captures/hap-col-*.png`, `hap-images-*.png`,
`hap-icons-*.png` on the lab). Colors, as listed, with the group headers
(each header has its own swatch): Primary Group (Light, Background, Dark,
Frame, Label, Disable Frame, Disable Label); Miscellaneous (Important Label,
Focus Box, Text Box Background/Foreground, Text Hilite Background/Foreground,
Text Insertion Point, List Background, List Label, List Hilite
Background/Foreground, List Sort Column Background, List Separator, Workspace
Background 1 to 4); Button Group, Button Hilite Group, Button Disable Group
(Light 2, Light 1, face, Dark 1, Dark 2, Frame, Label each); Default Button
Group (Light, face, Dark, Frame); Window Frame (Light 2, Light 1, Window, Dark
1, Dark 2, Frame, Label); Window Frame Transition 1 to 18; Window Frame Focus
(as Window Frame); Window Frame Focus Transition 1 to 18; Menu Group (Light,
Background, Dark, Label, Disable Label); Menu Hilite (Light, Background, Dark,
Label); Progress Bar Group (Transition 1 to 10, Frame, Label); Progress
Background (Light, Bkgnd, Dark); Scroll Bar Group (Frame, Light, ScrollBar,
Dark, Label), Scroll Bar Hilite (Light, Hilite, Dark, Label), Scroll Bar
Indicator (Light, Indicator, Dark), Scroll Bar Indicator Hilite (Light,
Hilite, Dark), Scroll Bar Background (Light 2, Light 1, Bkgnd, Dark 1, Dark
2), Scroll Bar Disable (Light, Disable, Dark, Frame, Label); Slider Group
(Bar, Bar Frame, Bar Hilite, Bar Hilite Frame), Slider Indicator (Light,
Indicator, Dark, Frame), Slider Indicator Hilite (same), Slider Disable
(Light, Disable, Dark, Frame); Column Header Group (Frame, Light, Header,
Dark, Label, Hilite Light, Hilite, Hilite Dark, Hilite Label); File Label
Colors 0 to 15. Counting each header row with a swatch as one row gives more
rows (about 215) than the 204 entries in the file, so the row-to-slot mapping
is not a plain 1:1 and is left to whoever renders a theme, who can read it
from the client's rendering.

## Not supported (yet)

Nothing is rendered; `Bitmap::rgba` gives RGBA with the transparent index at
alpha 0 (a 65535 by 65535 bitmap would need 17 GB there; the renderer should
cap what it draws). Fonts: none seen; sections 0 to 3 and 8 to 12 have never
been non-empty. `Bitmap`'s fields are public; `pixel` and `rgba` assume the
invariants `Bitmap::new` and `decode_hap` establish.
