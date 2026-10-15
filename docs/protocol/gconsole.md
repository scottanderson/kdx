# GConsole

Tags, hex, and field notation: see [Conventions](README.md#conventions).

GConsole is a Haxial remote console: connection type 4 in the client's
Connect dialog, on TCP 10900 ([handshake.md](handshake.md#connection-types)).
The client's side is handled by `FUN_08095c84` **[S]**. No KDX server or
tracker contains it, and the clients only play the display end: they check
for a `%HGC` hello but never send one, and handle
[4000 Open Window](messages/gconsole.md#m4000)/
[4001 Draw](messages/gconsole.md#m4001) but never send them **[S]**. Tags and
number formats follow [the protocol overview](README.md).

## Handshake

The client sends its usual hello (`%KDX`, sub-version 0x10) with the login
from the GConsole Login dialog. The peer's hello decides the connection
kind ([handshake.md](handshake.md#connection-types)); a GConsole peer sends
`%HGC` (0x25484743) with sub-version 0x40 **[S `FUN_0804b824`]**. `%HGC`
with any other sub-version fails with 0x40072 (lower) or 0x40073 (higher).
On `%HGC` the client switches the connection to a second challenge table
and a second header scramble before it checks the peer's proof
**[S `FUN_0804b824`]**:

- The challenge table is the GConsole story, which only the clients hold
  ([handshake.md](handshake.md#the-gconsole-story)). The client uses it
  exactly as it uses the KDX story
  ([handshake.md](handshake.md#challenge-tables)). In
  `captures/linux-gconsole.pcapng` the peer's challenge, 2832 (frame 6), is
  past the end of the KDX story, and the client's proof answers it from
  this one (frame 8) **[O]**.
- The header scramble (`FUN_0809aefc`) has the shape of the `%KDX` one in
  [framing.md](framing.md#header-scramble) with other rotations and masks.
  Scrambling header words h0-h3 before encryption gives:

  | Word | Value |
  |---|---|
  | 0 | `rotl(h1, 15) ^ 0x5f547a17` |
  | 1 | `rotr(h2, 4) ^ 0x69c83e35` |
  | 2 | `rotl(h0, 7) ^ 0x1b20e200` |
  | 3 | `rotr(h3, 5) ^ 0x8022e8d1` |

Everything else follows the `%KDX` handshake and session. "Logged in.
Opening GConsole..." shows once the proofs check out.

## Messages

The messages are described in [messages/gconsole.md](messages/gconsole.md).

## Compared with view display

View display (900-905) and GConsole both put a remote screen in a client
window and send input back, but share little else **[S]**:

| | View display | GConsole |
|---|---|---|
| Peer | a KDX server, on a transfer connection to its port | anything that answers `%HGC` on 10900 |
| Access | privileges 66 and 67; [902 View Display Params](messages/view-display.md#m902) says whether input is allowed | the GConsole login |
| Size | in every [903 View Display Update](messages/view-display.md#m903); a new size reallocates the frame buffer | once, in [4000 Open Window](messages/gconsole.md#m4000) |
| Picture | the server's whole screen as RGB555 pixels, diffed against the last frame, compressed with runs and two-colour codes | draw records into objects: fills, gradients, lines, polygons, text, copies; colours are u32 with a transparency byte |
| Updates | a timer every 1500 ms, and within 100 ms of a click or key | whenever the peer sends [4001 Draw](messages/gconsole.md#m4001) |
| Mouse | [904 View Display Mouse](messages/view-display.md#m904): kinds 1 down, 2 up, 3 move, 4 wheel | [4201 Mouse](messages/gconsole.md#m4201): kinds 5 down, 6 up, 10 wheel; no moves |
| Keyboard | [905 View Display Keyboard](messages/view-display.md#m905): kinds 1, 2, 3 | [4200 Key](messages/gconsole.md#m4200): kinds 1, 2, 3 |

The input messages carry the same client input event **[S `FUN_0806fc68`,
`FUN_0806f1f4`, `FUN_08096188`, `FUN_0809607c`]**: 904 and 4201 send its
modifiers, x, y, button, and wheel; 905 and 4200 send its modifiers, character,
and key code. GConsole adds the client's clock and a 4-byte character, and
leaves moves out. The servers rebuild that event from 904 and 905 **[S
`FUN_0805c344`, `FUN_0805c498`]**. No GConsole record uses the 903 decoder,
which only the 903 handler calls **[S `FUN_0806f864`]**.

The servers grab the screen into 32-bit pixels laid out as a zero byte,
then red, green, and blue **[S `FUN_0041ffd0`]**, the layout of a 4001
colour; both are likely Haxial's one image format **[I]**.

## Open questions

- What the u16 at 0x02 of kinds 100, 101, 103, and 105-109 and the 8 bytes
  at 0x0c of 105/106 mean; what kind 3 of [4200 Key](messages/gconsole.md#m4200)
  (window event 7) is.
