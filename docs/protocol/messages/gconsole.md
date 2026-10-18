# GConsole messages

For hex and field notation, see [Conventions](../README.md#conventions).

Message types 4000 to 4201 run on the connection that
[gconsole.md](../gconsole.md) describes.

`captures/linux-gconsole.pcapng` shows all the messages below, between Client
1.620 beta 5 and `kdx-probe --listen` as the peer. <!-- [O] -->
`kdx_proto::msg::gconsole` has each layout, tested against that capture. The
names are provisional.

The peer sends [4000 Open Window](#m4000) and [4001 Draw](#m4001). <!-- [S
`FUN_08095c84`] -->

## <a id="m4000"></a>4000 Open Window

Initiator: GConsole peer. <!-- [S `FUN_08094be8`] --> Part of the [GConsole
exchange](../gconsole.md#messages).

The payload is at least 0x18 bytes. If the peer sends more, the payload is
longer than the struct.

Struct `OpenWindow` (24 bytes):<a id="sOpenWindow"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 2 | `u16` | `width` | 0x40 to 0x1000 |
| 0x02 | 2 | `u16` | `height` | 0x10 to 0x1000 |
| 0x04 | 4 | `u32` | `color` | The client fills the window with it. |
| 0x08 | 16 | `u8[16]` | `rest` | The client does not read these bytes. |

A second 4000 Open Window fails with 0x10003. Frame 10 opened a window of 0x200
× 0x180, filled with 0x00336699. <!-- [O] --> The color goes through
`FUN_08114bc8` first. [4001 Draw](#m4001) describes colors.

## <a id="m4001"></a>4001 Draw

Initiator: GConsole peer. <!-- [S `FUN_08095ac4`] --> Part of the [GConsole
exchange](../gconsole.md#messages).

The payload is a stream of draw records. Each record starts with a u32: the top
10 bits give the kind, and the low 22 bits give the body length. The next record
starts at the length plus 4, rounded up to a multiple of 4. Frame 14 holds 21
records. <!-- [O] -->

Struct `Draw` (dynamic):<a id="sDraw"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | rest | [`DrawRecord`](#sDrawRecord)`[]` | `records` | Rust `padding`: the padding after each record. |

Struct `DrawRecord` (dynamic):<a id="sDrawRecord"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `u32` | `body_len` |  |
| 0x04 | `body_len` | `u8[]` | `body` | The length is the low 22 bits of `body_len`, and the top 10 bits are the kind. The table below gives the layout of each kind. |
|  | pad4 | `u8[]` | `padding` | Padding to a multiple of 4. Keep it as received. |

A color is a u32: a transparency byte, then red, green, and blue. A transparency
of 0 is opaque, 0x01 to 0xfe blend, and 0xff draws nothing. <!-- [S
`FUN_080ba8d0`] -->

Each body, except that of kind 2, starts with the u16 ID of an object that kind
2 made. Coordinates are i16, in the object. A rect does not include its right
and bottom edges. The table shows what each kind drew in frame 14: <!-- [O] -->

| Kind | Body | Drew | Handler |
|---:|---|---|---|
| 2 | Create: a rect (left, top, right, bottom) at 0x00 in the window, the ID at 0x0c, flags at 0x0e (0x8 shows the object), and the object type at 0x12. The only known type is 1, an off-screen image. At least 0x14 bytes. | an object, shown with flags 8 and type 1 | `FUN_08094d50` |
| 3 | Delete. | removed object 3 and its drawing | `FUN_08094f2c` |
| 4 | Move: x and y at 0x02, in the window. | moved object 2 | `FUN_08094fb8` |
| 100 | u16 at 0x02, rect at 0x04, color at 0x0c | a filled rect. 0, 1, and 2 at 0x02 drew the same. | `FUN_08095058` |
| 101 | u16 at 0x02, rect at 0x04, colors at 0x0c and 0x10 | a fill from the first color at the left to the second at the right | `FUN_0809511c` |
| 102 | u16 line width at 0x02, rect at 0x04, color at 0x0c | a rect's outline. Widths 1 and 3 drew, and 0 drew nothing. | `FUN_080951fc` |
| 103 | u16 at 0x02, point at 0x04, color at 0x08 | one pixel | `FUN_080952e4` |
| 104 | byte at 0x02, line width byte at 0x03, two points at 0x04, color at 0x0c | a line, 2 wide | `FUN_08095390` |
| 105, 106 | u16 at 0x02, rect at 0x04, 8 bytes at 0x0c, text from 0x14 to the end | the text, the same for both kinds | `FUN_08095480`, `FUN_08095590` |
| 107 | u16 at 0x02, color at 0x04, point at 0x08, three or more points from 0x0c | a filled polygon. The point at 0x08 was (0, 0). | `FUN_080956a0` |
| 108 | u16 at 0x02, point at 0x04, color at 0x08 | a flood fill of the area of the same color around the point | `FUN_08095904` |
| 109 | u16 at 0x02, destination point at 0x04, source point at 0x08, u16 width and height at 0x0c | a copy within the object | `FUN_080959a4` |

The client sends back two 16-byte messages, [4200 Key](#m4200) and [4201
Mouse](#m4201), one for each key or button event in the window. <!-- [S
`FUN_0809607c`, `FUN_08096188`] --> Mouse moves send nothing. <!-- [O frames 28
to 30] a drag -->

## <a id="m4200"></a>4200 Key

Initiator: Client, to the GConsole peer. <!-- [S `FUN_0809607c`] --> Part of the
[GConsole exchange](../gconsole.md#messages).

Struct `Key` (16 bytes):<a id="sKey"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `u32` | `time` | The client's clock, in ms. |
| 0x04 | 4 | `u32` | `modifiers` | The modifiers held before the event: 8 Shift (frame 54), 2 Alt (frame 18). |
| 0x08 | 4 | `u32` | `character` | `a` 0x61, `A` 0x41, Return 0x0a, BackSpace 0x08. 0 for keys with no character, and on release. |
| 0x0c | 2 | `u16` | `usage` | The key's USB HID usage: `a` 4, Return 0x28, Left 0x50, Left Shift 0xe1. 0 on most releases. |
| 0x0e | 1 | `u8` | `kind` | 1 pressed, 2 released, 3 for window event 7 (not seen). |
| 0x0f | 1 | `u8` | `value_0f` |  |

<!-- [O] all but kind 3 -->
A held key repeats as a release and a press together, every 40 ms after the
first 661 ms (frames 78 to 154). Ctrl+B sent Ctrl's press, a release with
modifiers 0x21, and Ctrl's release, but no press for B (frames 68 to 72). F1
opened the client's window menu, and did not go to the peer.

## <a id="m4201"></a>4201 Mouse

Initiator: Client, to the GConsole peer. <!-- [S `FUN_08096188`] --> Part of the
[GConsole exchange](../gconsole.md#messages).

Struct `Mouse` (16 bytes):<a id="sMouse"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `u32` | `time` | The client's clock, in ms. |
| 0x04 | 4 | `u32` | `value_04` | 0x00040000 on a right button press (frame 32), and 0 otherwise. |
| 0x08 | 2 | `u16` | `x` | In the window. |
| 0x0a | 2 | `u16` | `y` | In the window. |
| 0x0c | 1 | `u8` | `button` | 1 left, 2 right, 3 middle, 4 wheel up, 5 wheel down |
| 0x0d | 1 | `u8` | `kind` | 5 pressed, 6 released, 10 wheel |
| 0x0e | 2 | `i16` | `wheel` | 0x78 up, −0x78 down, and 0 for buttons. |

<!-- [O frames 20 to 46] -->
Each wheel step sends two identical messages (frames 40 and 42).

The modifier bits are those of Client 1.600: Ctrl 0x21, Alt 2, Shift 8, Caps
Lock 4, and Windows 0x30. <!-- [S client `FUN_00463870`] --> Client 1.600 sends
its own key codes at 0x0c, not HID usages; see [905 View Display
Keyboard](view-display.md#m905). <!-- [S `FUN_00448830`] -->
