# Client-to-client (DCC) and voice

KDX Client 1.620 beta 5 for Linux runs its own listener for direct
client-to-client connections, which the client calls DCC. Private chat and voice
use it. Most of this is read from the client's code **[S]**.
`captures/linux-dcc-voice-failed.pcapng` has two copies of that client talking
to each other, up to a voice call that fails before any voice data **[O]**; its
frames are cited below. Tags and number formats follow
[KDXProtocol.md](KDXProtocol.md).

## Listener

The client listens on TCP 10600 by default when its preference for it is on
**[S `FUN_0808fae4`]**. It puts that port in its user record (offset 0x0a,
see [KDXProtocol.md](KDXProtocol.md#user-record)), so the server can hand
it out in 223 Chat DCC Addresses. With the listener off, the port is 0
**[S `FUN_0804d654`]**. The client can also ask a UPnP router for a TCP
mapping named `KDXClientDCC` **[S `FUN_08090344`]**; that is the client's
only use of UDP.

A DCC connection uses the same handshake as a server connection
([handshake.md](handshake.md)): service `%KDX`, version 0x10 (the only one
it accepts), the same challenge table, and messages framed and encrypted as
in [session.md](session.md) **[S `FUN_0808f77c`]**. The listener accepts a
payload of at most 0x80000 bytes and fewer than 6 connections from one
address.

## Logins

The login callback is `FUN_0808efcc` **[S]**:

- **Empty login: a primary connection.** The proof is checked with the
  empty-password key. The listener then makes the connection's own key K:
  the empty-password key with its first u32 replaced by a random value.
- **`00` and a u32 connection ID (5 bytes): a secondary connection.** It
  attaches to the primary connection with that ID. The proof is checked
  with that connection's K, and the peer must connect from the same
  address. Voice uses this.

This mirrors the server's transfer connections, which attach to a session
with a `00`+ID login.

In the capture, a login named `admin` gets a proof with status 1 and closes
(frames 1-11); the empty login opens a primary connection (13-28); and
`00 00000002` and `00 00000004` open secondary connections whose proofs match
with the peer's K from 117 SKPK (30-42, 125-141) **[O]**.

## Primary connection

After login, each side sends **117 SKPK** and then **100 Set Identity** (a
normal user record) **[S `FUN_0808f3e0`; O]** (frames 22-26). The primary
connection then handles **[S]**:

| Type | Handler | Action |
|---:|---|---|
| 1 | | status |
| 100 Set Identity | `FUN_0808fbc0` | the peer's nick, icon, and info |
| 117 SKPK | `FUN_0808ff64` | see below |
| 300 Send Private Message | | a chat line: u32, 12 bytes, then the text (more than 0x10 bytes in all) |

A 300 in frame 28 is u32 1, 12 zero bytes, then "dcc message body" **[O]**.

A user opens a primary connection from the Connect dialog with Type "Client".
Joining a voice chat also opens one to each address in 223 Chat
DCC Addresses: the member who joins calls the members already there
(frames 108-123). Joining again while those connections stay open opens none
(148-155) **[O]**. The joiner's own listener is not needed.

**117 SKPK** (0x30 bytes) **[S `FUN_0808eefc`, `FUN_0808ff64`]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 0x20 | key K | the sender's key for secondary connections to it |
| 0x20 | 4 | DCC port | the sender's listener |
| 0x24 | 4 | connection ID | the sender's ID for this primary connection; each client numbers its own |
| 0x28 | 2 | client version | 0x0654 (1620); the receiver warns if it is older or newer |
| 0x2a | 2 | 0 | |
| 0x2c | 4 | 0 | |

Frames 22, 25, 119, and 122 match this layout, with ports 0x2968 and 0x2969 and
connection IDs 2 and 4 **[O]**. `kdx_proto::msg::dcc::Skpk` re-encodes frames
22 and 25 byte for byte.

## Voice

A voice chat is a server chat with flag bit 1 set (see the chat record in
[KDXProtocol.md](KDXProtocol.md#chat-record)). On joining, the client sends
222 Get Chat DCC Addresses and gets each other member's `ip:port`. Voice
then runs on TCP between the clients, not over UDP.

1. The caller opens a secondary connection to the peer's DCC port, with the
   login `00` + the connection ID from the peer's 117 SKPK (0x24) and the
   peer's key K **[S `FUN_08092240`]**.
2. On login it sends **1000 Start Voice**, empty **[S `FUN_0808f1ac`]**.
3. The peer answers with **1001 Start Voice (R)**, a u32 **[S
   `FUN_08094074`]**:
   - 0x00640007 if its Voice Controls window is not open;
   - 0, followed by its own 1002, if it is.
4. On 1001 the caller sends its 1002 if the value is 0. Otherwise it shows
   "The remote user does not have his/her Voice Controls window open." and
   closes the connection **[S `FUN_08094218`]**.
5. Both sides then send **1003 Voice Data** as they record.

The capture confirms steps 1-3 **[O]**: 1000 and 1001 0x00640007 with the peer's
Voice Controls closed (frames 39-40), and 1001 0 then both 1002s with it open
(134-138). In the lab sandbox, which has no sound devices, each client answered
the other's 1002 with status 0x10004 (unimplemented) and logged "Operation is
unimplemented. (ID 1/4 @ S101)" (139-141), so no 1003 flowed.

**1002 Set Voice Format** (0x18 bytes) **[S `FUN_08093ae0`,
`FUN_080943a0`]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 0xc | 0 | |
| 0x0c | 4 | sample rate | Hz |
| 0x10 | 4 | codec | `ADPC` |
| 0x14 | 2 | buffer | ms; the receiver divides by 10 and clamps to 35..500 |
| 0x16 | 1 | channels | 1 |
| 0x17 | 1 | bits per sample | 16 |

Both sides sent 11025 Hz (0x2b11), `ADPC`, 500 ms (0x01f4), 1 channel, and 16
bits (frames 136, 138) **[O]**.

**1003 Voice Data** **[S `FUN_08093670`, `FUN_0809455c`, `FUN_080e15e4`]**

The payload is IMA ADPCM, two samples per byte, high nibble first. The
decoder state starts fresh for each message. It decodes to big-endian
signed 16-bit samples, four times the payload size. The sender drops audio
while more than 0x100000 bytes wait in its send queue.

## GConsole

The client's connection dialog has a Type menu **[S `FUN_0804bac0`]**:

| Value | Type | Default port |
|---:|---|---:|
| 0 | Auto-Detect | none; needs a port |
| 1 | Server | 10700 |
| 2 | Client (DCC) | 10600 |
| 3 | Tracker | 10800 |
| 4 | GConsole | 10900 |

GConsole is a Haxial remote console, handled by `FUN_08095c84` **[S]**.
No KDX server or tracker contains it, and the clients only play the display
end: they check for a `%HGC` hello but never send one, and handle 4000/4001
but never send them **[S]**.

### Handshake

The client sends its usual hello (`%KDX`, version 0x10) with the login
from the GConsole Login dialog. The peer's hello decides the connection
kind **[S `FUN_0804b824`, `FUN_0804b938`]**:

| Service | Version | Connection |
|---|---:|---|
| `%KDX` | 0x10 | Client (DCC) |
| `%KDX` | 0x20 | Server |
| `%KDX` | 0x30 | Tracker |
| `%HGC` (0x25484743) | 0x40 | GConsole |

`%HGC` with any other version fails with 0x40072 (lower) or 0x40073
(higher). On `%HGC` the client switches the connection to a second
challenge table and a second header scramble before it checks the peer's
proof **[S `FUN_0804b824`]**:

- The challenge table is the GConsole story, which only the clients hold
  ([handshake.md](handshake.md#the-gconsole-story)). The client uses it
  exactly as it uses the KDX story
  ([handshake.md](handshake.md#challenge-tables)). In
  `captures/linux-gconsole.pcapng` the peer's challenge, 2832 (frame 6), is
  past the end of the KDX story, and the client's proof answers it from
  this one (frame 8) **[O]**.
- The header scramble (`FUN_0809aefc`) has the shape of the `%KDX` one in
  [session.md](session.md#header-scramble) with other rotations and masks.
  Scrambling header words h0–h3 before encryption gives:

  | Word | Value |
  |---|---|
  | 0 | `rotl(h1, 15) ^ 0x5f547a17` |
  | 1 | `rotr(h2, 4) ^ 0x69c83e35` |
  | 2 | `rotl(h0, 7) ^ 0x1b20e200` |
  | 3 | `rotr(h3, 5) ^ 0x8022e8d1` |

Everything else follows the `%KDX` handshake and session. "Logged in.
Opening GConsole..." shows once the proofs check out.

### Messages

`captures/linux-gconsole.pcapng` shows every message below between Client
1.620 beta 5 and `kdx-probe --listen` as the peer **[O]**. Each layout is
in `kdx_proto::msg::gconsole`, tested against that capture. Names are
provisional.

The peer sends **[S `FUN_08095c84`]**:

- **4000 Open Window**, at least 0x18 bytes: u16 width (0x40..0x1000),
  u16 height (0x10..0x1000), then a u32 colour the window is filled with
  **[S `FUN_08094be8`]**. A second 4000 fails with 0x10003. Frame 10 opened
  a 0x200 × 0x180 window filled with 0x00336699 **[O]**.
- **4001 Draw** is a stream of draw records **[S `FUN_08095ac4`]**. Each
  starts with a u32 whose top 10 bits give the kind and low 22 bits the
  body length; the next record starts at the length plus 4, rounded up to
  4. Frame 14 holds 21 records **[O]**.

A colour is a u32: a transparency byte, then red, green, and blue
(0x00RRGGBB is opaque). 0 is opaque, 0x01–0xfe blend, and 0xff draws
nothing **[S `FUN_080ba8d0`]**. The 4000 colour goes through
`FUN_08114bc8` first.

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

The client sends back two 16-byte messages **[S `FUN_0809607c`,
`FUN_08096188`]**, one for each key or button event in the window. Mouse
moves send nothing **[O]** (frames 28–30, a drag).

**4200 Key** (`FUN_0809607c`):

| Offset | Size | Field |
|---:|---:|---|
| 0x00 | 4 | the client's clock, in ms |
| 0x04 | 4 | modifiers held before the event: 8 Shift (frame 54), 2 Alt (18) |
| 0x08 | 4 | the character: `a` 0x61, `A` 0x41, Return 0x0a, BackSpace 0x08; 0 for keys with none and on release |
| 0x0c | 2 | the key's USB HID usage: `a` 4, Return 0x28, Left 0x50, Left Shift 0xe1; 0 on most releases |
| 0x0e | 1 | 1 pressed, 2 released, 3 for window event 7 (never seen) |
| 0x0f | 1 | 0 |

All **[O]** but kind 3. A held key repeats as a release and a press
together, every 40 ms after 661 ms (frames 78–154). Ctrl+B sent Ctrl's
press, a release with modifiers 0x21, and Ctrl's release, but no press for
B (68–72). F1 opened the client's window menu instead of reaching the peer.

**4201 Mouse** (`FUN_08096188`):

| Offset | Size | Field |
|---:|---:|---|
| 0x00 | 4 | the client's clock, in ms |
| 0x04 | 4 | 0x00040000 on a right button press (frame 32), else 0 |
| 0x08 | 2 | x in the window |
| 0x0a | 2 | y in the window |
| 0x0c | 1 | button: 1 left, 2 right, 3 middle, 4 wheel up, 5 wheel down |
| 0x0d | 1 | 5 pressed, 6 released, 10 wheel |
| 0x0e | 2 | i16 wheel movement: 0x78 up, −0x78 down; 0 for buttons |

All **[O]**, frames 20–46. Each wheel step sends two identical messages
(40, 42).

The modifier bits are Client 1.600's: Ctrl 0x21, Alt 2, Shift 8, Caps Lock
4, Windows 0x30 **[S client 1.600 `FUN_00463870`]**. Client 1.600 sends its
own key codes at 0x0c instead of HID usages; see 905 View Display Keyboard
in [KDXProtocol.md](KDXProtocol.md#view-display) **[S `FUN_00448830`]**.

### Compared with view display

View display (900–905) and GConsole both put a remote screen in a client
window and send input back, but share little else **[S]**:

| | View display | GConsole |
|---|---|---|
| Peer | a KDX server, on a transfer connection to its port | anything that answers `%HGC` on 10900 |
| Access | privileges 66 and 67; 902 says whether input is allowed | the GConsole login |
| Size | in every 903; a new size reallocates the frame buffer | once, in 4000 |
| Picture | the server's whole screen as RGB555 pixels, diffed against the last frame, compressed with runs and two-colour codes | draw records into objects: fills, gradients, lines, polygons, text, copies; colours are u32 with a transparency byte |
| Updates | a timer every 1500 ms, and within 100 ms of a click or key | whenever the peer sends 4001 |
| Mouse | 904: kinds 1 down, 2 up, 3 move, 4 wheel | 4201: kinds 5 down, 6 up, 10 wheel; no moves |
| Keyboard | 905: kinds 1, 2, 3 | 4200: kinds 1, 2, 3 |

The input messages carry the same client input event **[S `FUN_0806fc68`,
`FUN_0806f1f4`, `FUN_08096188`, `FUN_0809607c`]**: 904 and 4201 send its
modifiers, x, y, button, and wheel; 905 and 4200 send its modifiers,
character, and key code. GConsole adds the client's clock and a 4-byte
character, and leaves moves out. The servers rebuild that event from 904
and 905 **[S `FUN_0805c344`, `FUN_0805c498`]**. No GConsole record uses the
903 decoder, which only the 903 handler calls **[S `FUN_0806f864`]**.

## Open questions

- Capture 1003 Voice Data, which could not be started in the sandbox; see
  `.scratch/capture-gaps/issues/07-dcc-voice.md`.
- GConsole: what the u16 at 0x02 of kinds 100, 101, 103, and 105–109 and
  the 8 bytes at 0x0c of 105/106 mean; what kind 3 of 4200 (window event
  7) is.
