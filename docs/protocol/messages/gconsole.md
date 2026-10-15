# GConsole messages

Tags, hex, and field notation: see [Conventions](../README.md#conventions).

Message types 4000 to 4201, on the connection described in
[gconsole.md](../gconsole.md).

`captures/linux-gconsole.pcapng` shows every message below between Client
1.620 beta 5 and `kdx-probe --listen` as the peer **[O]**. Each layout is
in `kdx_proto::msg::gconsole`, tested against that capture. Names are
provisional.

The peer sends [4000 Open Window](#m4000) and [4001 Draw](#m4001) **[S
`FUN_08095c84`]**.

## <a id="m4000"></a>4000 Open Window

Initiator: GConsole peer. **[S `FUN_08094be8`]**

At least 0x18 bytes: u16 width (0x40..0x1000), u16 height (0x10..0x1000),
then a u32 colour the window is filled with. A second [4000 Open Window](#m4000)
fails with 0x10003. Frame 10 opened a 0x200 × 0x180 window filled with
0x00336699 **[O]**. The colour goes through `FUN_08114bc8` first; colours are
described under [4001 Draw](#m4001).

## <a id="m4001"></a>4001 Draw

Initiator: GConsole peer. **[S `FUN_08095ac4`]**

A stream of draw records. Each starts with a u32 whose top 10 bits give the
kind and low 22 bits the body length; the next record starts at the length
plus 4, rounded up to 4. Frame 14 holds 21 records **[O]**.

A colour is a u32: a transparency byte, then red, green, and blue
(0x00RRGGBB is opaque). 0 is opaque, 0x01-0xfe blend, and 0xff draws
nothing **[S `FUN_080ba8d0`]**.

Every body except kind 2's starts with the u16 ID of an object that kind 2
created. Coordinates are i16, in the object; rects exclude their right and
bottom edges. What each kind drew in frame 14 is **[O]**:

| Kind | Body | Drew | Handler |
|---:|---|---|---|
| 2 | create: rect (left, top, right, bottom) at 0x00 in the window, ID at 0x0c, flags at 0x0e (0x8 shows the object), object type at 0x12 (only 1, an off-screen image, is known); at least 0x14 bytes | an object; with flags 8 and type 1 it showed | `FUN_08094d50` |
| 3 | delete | removed object 3 and its drawing | `FUN_08094f2c` |
| 4 | move: x, y at 0x02, in the window | moved object 2 | `FUN_08094fb8` |
| 100 | u16 at 0x02, rect at 0x04, colour at 0x0c | a filled rect; 0, 1, and 2 at 0x02 drew the same | `FUN_08095058` |
| 101 | u16 at 0x02, rect at 0x04, colours at 0x0c and 0x10 | a fill from the first colour at the left to the second at the right | `FUN_0809511c` |
| 102 | u16 line width at 0x02, rect at 0x04, colour at 0x0c | a rect's outline; widths 1 and 3 drew, 0 drew nothing | `FUN_080951fc` |
| 103 | u16 at 0x02, point at 0x04, colour at 0x08 | one pixel | `FUN_080952e4` |
| 104 | byte at 0x02, line width byte at 0x03, two points at 0x04, colour at 0x0c | a line, 2 wide | `FUN_08095390` |
| 105, 106 | u16 at 0x02, rect at 0x04, 8 bytes at 0x0c, text from 0x14 to the end | the text, the same for both kinds | `FUN_08095480`, `FUN_08095590` |
| 107 | u16 at 0x02, colour at 0x04, point at 0x08, three or more points from 0x0c | a filled polygon; the point at 0x08 was (0, 0) | `FUN_080956a0` |
| 108 | u16 at 0x02, point at 0x04, colour at 0x08 | a flood fill of the area of like colour around the point | `FUN_08095904` |
| 109 | u16 at 0x02, destination point at 0x04, source point at 0x08, u16 width and height at 0x0c | a copy within the object | `FUN_080959a4` |

The client sends back two 16-byte messages, [4200 Key](#m4200) and
[4201 Mouse](#m4201) **[S `FUN_0809607c`, `FUN_08096188`]**, one for each
key or button event in the window. Mouse moves send nothing **[O]** (frames
28-30, a drag).

## <a id="m4200"></a>4200 Key

Initiator: Client (to GConsole peer). Sent by `FUN_0809607c` **[S]**:

| Offset | Size | Field |
|---:|---:|---|
| 0x00 | 4 | the client's clock, in ms |
| 0x04 | 4 | modifiers held before the event: 8 Shift (frame 54), 2 Alt (18) |
| 0x08 | 4 | the character: `a` 0x61, `A` 0x41, Return 0x0a, BackSpace 0x08; 0 for keys with none and on release |
| 0x0c | 2 | the key's USB HID usage: `a` 4, Return 0x28, Left 0x50, Left Shift 0xe1; 0 on most releases |
| 0x0e | 1 | 1 pressed, 2 released, 3 for window event 7 (never seen) |
| 0x0f | 1 | 0 |

All **[O]** but kind 3. A held key repeats as a release and a press
together, every 40 ms after 661 ms (frames 78-154). Ctrl+B sent Ctrl's
press, a release with modifiers 0x21, and Ctrl's release, but no press for
B (68-72). F1 opened the client's window menu instead of reaching the peer.

## <a id="m4201"></a>4201 Mouse

Initiator: Client (to GConsole peer). Sent by `FUN_08096188` **[S]**:

| Offset | Size | Field |
|---:|---:|---|
| 0x00 | 4 | the client's clock, in ms |
| 0x04 | 4 | 0x00040000 on a right button press (frame 32), else 0 |
| 0x08 | 2 | x in the window |
| 0x0a | 2 | y in the window |
| 0x0c | 1 | button: 1 left, 2 right, 3 middle, 4 wheel up, 5 wheel down |
| 0x0d | 1 | 5 pressed, 6 released, 10 wheel |
| 0x0e | 2 | i16 wheel movement: 0x78 up, −0x78 down; 0 for buttons |

All **[O]**, frames 20-46. Each wheel step sends two identical messages
(40, 42).

The modifier bits are Client 1.600's (`FUN_00463870`): Ctrl 0x21, Alt 2, Shift
8, Caps Lock 4, Windows 0x30 **[S client `FUN_00463870`]**. Client 1.600 sends
its own key codes at 0x0c instead of HID usages; see
[905 View Display Keyboard](view-display.md#m905) **[S `FUN_00448830`]**.
