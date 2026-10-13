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
([handshake.md](handshake.md)): sub-protocol `%KDX`, sub-version 0x10 (the
only one it accepts), the same challenge table, and messages framed and
encrypted as in [session.md](session.md) **[S `FUN_0808f77c`]**. The
listener accepts a payload of at most 0x80000 bytes and fewer than 6
connections from one address.

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

## Open questions

- Capture 1003 Voice Data, which could not be started in the sandbox; see
  `.scratch/capture-gaps/issues/07-dcc-voice.md`.
