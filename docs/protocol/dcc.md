# Client-to-client (DCC) and voice

KDX Client 1.620 beta 5 for Linux runs its own listener for direct
client-to-client connections, which the client calls DCC. Private chat and
voice use it. Everything here is read from the client's code **[S]**; no
capture has a DCC connection yet (`captures/linux-files.md` notes the
listener on TCP 10600, with no connection to it). Tags and number formats
follow [KDXProtocol.md](KDXProtocol.md).

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

## Primary connection

After login, each side sends **117 SKPK** and then **100 Set Identity**
(a normal user record) **[S `FUN_0808f3e0`]**. The primary connection then
handles **[S]**:

| Type | Handler | Action |
|---:|---|---|
| 1 | | status |
| 100 Set Identity | `FUN_0808fbc0` | the peer's nick, icon, and info |
| 117 SKPK | `FUN_0808ff64` | see below |
| 300 Send Private Message | | a chat line: u32, 12 bytes, then the text (more than 0x10 bytes in all) |

How a user opens a private DCC chat in the UI, and which side connects, has
not been traced. The peer address would come from 223 Chat DCC Addresses
or the user's address [I].

**117 SKPK** (0x30 bytes) **[S `FUN_0808eefc`, `FUN_0808ff64`]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 0x20 | key K | the sender's key for secondary connections to it |
| 0x20 | 4 | DCC port | the sender's listener |
| 0x24 | 4 | connection ID | the sender's ID for this primary connection |
| 0x28 | 2 | client version | 0x0654 (1620); the receiver warns if it is older or newer |
| 0x2a | 2 | 0 | |
| 0x2c | 4 | 0 | |

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

- The challenge table is a 3193-byte text that starts `The following story
  is Copyright 2004`, its trailing NUL included. The Linux, Windows, and
  Mac clients hold the same bytes (SHA-256 `b43f2222…dc029c36`).
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

The peer sends **[S `FUN_08095c84`]**:

- **4000** opens the window, at least 0x18 bytes: u16 width
  (0x40..0x1000), u16 height (0x10..0x1000), then a u32 the window is
  filled with **[S `FUN_08094be8`]**. A second 4000 fails with 0x10003.
- **4001** is a stream of draw records **[S `FUN_08095ac4`]**. Each starts
  with a u32 whose top 10 bits give the kind and low 22 bits the body
  length; the next record starts at the length plus 4, rounded up to 4.

Every body except kind 2's starts with the u16 ID of an object that kind 2
created. Coordinates are i16.

| Kind | Body | Handler |
|---:|---|---|
| 2 | create: rect (left, top, right, bottom) at 0x00, ID at 0x0c, u16 at 0x0e, object type at 0x12 (only 1, an off-screen image, is known); at least 0x14 bytes | `FUN_08094d50` |
| 3 | delete | `FUN_08094f2c` |
| 4 | move: x, y at 0x02 | `FUN_08094fb8` |
| 100 | rect at 0x04, u32 at 0x0c | `FUN_08095058` |
| 101 | u16 at 0x02, rect at 0x04, u32s at 0x0c and 0x10 | `FUN_0809511c` |
| 102 | u16 at 0x02, rect at 0x04, u32 colour at 0x0c | `FUN_080951fc` |
| 103 | point at 0x04, u32 at 0x08 | `FUN_080952e4` |
| 104 | byte at 0x03, two points at 0x04, u32 colour at 0x0c | `FUN_08095390` |
| 105, 106 | u16 at 0x02, rect at 0x04, text from 0x14 | `FUN_08095480`, `FUN_08095590` |
| 107 | u16 at 0x02, u32 colour at 0x04, point at 0x08, three or more points from 0x0c | `FUN_080956a0` |
| 108 | point at 0x04, u32 at 0x08 | `FUN_08095904` |
| 109 | destination point at 0x04, source point at 0x08, u16 width and height at 0x0c | `FUN_080959a4` |

What each kind draws is [I] from its arguments: 100 fill, 102 frame, 103
pixel, 104 line, 105/106 text, 107 polygon, 109 copy.

The client sends back two 16-byte messages **[S]**:

- **4200** (`FUN_0809607c`): u32, u32, u32, u16, a byte that is 1, 2, or 3
  for window events 5, 6, and 7, and a zero byte. [I] mouse.
- **4201** (`FUN_08096188`): u32, u32, u16, u16, a byte, a byte that is 5,
  6, or 10 for window events 9, 10, and 14, and a u16. [I] keyboard.

## Open questions

- Capture a private DCC chat and a voice call between two clients; see
  `.scratch/capture-gaps/issues/07-dcc-voice.md`.
- Which side opens the primary connection, and from which UI action.
