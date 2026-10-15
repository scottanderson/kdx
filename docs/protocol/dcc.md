# Client-to-client (DCC) and voice

KDX Client 1.620 beta 5 for Linux runs its own listener for direct
client-to-client connections, which the client calls DCC. Private chat and voice
use it. Most of this is read from the client's code **[S]**.
`captures/linux-dcc-voice-failed.pcapng` has two copies of that client talking
to each other, up to a voice call that fails before any voice data **[O]**; its
frames are cited below. Tags and number formats follow
[the protocol overview](README.md).

## Listener

The client listens on TCP 10600 by default when its preference for it is on **[S
`FUN_0808fae4`]**. It puts that port in its user record (offset 0x0a, see
[encodings.md](encodings.md#user-record)), so the server can hand it out in
[223 Chat DCC Addresses](messages/chat.md#m223). With the listener off, the port
is 0 **[S `FUN_0804d654`]**. The client can also ask a UPnP router for a TCP
mapping named `KDXClientDCC` **[S `FUN_08090344`]**; that is the client's only
use of UDP.

A DCC connection uses the same handshake as a server connection
([handshake.md](handshake.md)): sub-protocol `%KDX`, sub-version 0x10 (the
only one it accepts), the same challenge table, and messages framed and
encrypted as in [framing.md](framing.md) **[S `FUN_0808f77c`]**. The
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
with the peer's K from [117 SKPK](messages/session.md#m117) (30-42, 125-141)
**[O]**.

## Primary connection

After login, each side sends [117 SKPK](messages/session.md#m117) and then
**[100 Set Identity](messages/session.md#m100)** (a normal user record) **[S
`FUN_0808f3e0`; O]** (frames 22-26). The primary connection then handles
**[S]**:

| Type | Handler | Action |
|---:|---|---|
| [1 Status](messages/status.md#m1) | | status |
| 100 Set Identity | `FUN_0808fbc0` | the peer's nick, icon, and info |
| 117 SKPK | `FUN_0808ff64` | see [117 SKPK](messages/session.md#m117) |
| [300 Send Private Message](messages/private-messages.md#m300) | | a chat line: u32, 12 bytes, then the text (more than 0x10 bytes in all) |

A 300 in frame 28 is u32 1, 12 zero bytes, then "dcc message body" **[O]**.

A user opens a primary connection from the Connect dialog with Type "Client".
Joining a voice chat also opens one to each address in
[223 Chat DCC Addresses](messages/chat.md#m223): the member who joins calls the
members already there (frames 108-123). Joining again while those connections
stay open opens none (148-155) **[O]**. The joiner's own listener is not needed.

## Voice

A voice chat is a server chat with flag bit 1 set (see the chat record in
[encodings.md](encodings.md#chat-record)). On joining, the client sends
[222 Get Chat DCC Addresses](messages/chat.md#m222) and gets each other member's
`ip:port`. Voice then runs on TCP between the clients, not over UDP.

1. The caller opens a secondary connection to the peer's DCC port, with the
   login `00` + the connection ID from the peer's
   [117 SKPK](messages/session.md#m117) (0x24) and the peer's key K **[S
   `FUN_08092240`]**.
2. On login it sends [1000 Start Voice](messages/voice.md#m1000).
3. The peer answers with [1001 Start Voice (R)](messages/voice.md#m1001).
4. On 1001 the caller sends its [1002 Set Voice Format](messages/voice.md#m1002)
   if the value is 0, or gives up.
5. Both sides then send [1003 Voice Data](messages/voice.md#m1003) as they
   record.

The capture confirms steps 1-3 **[O]**: 1000 and 1001 0x00640007 with the
peer's Voice Controls closed (frames 39-40), and 1001 0 then both 1002s with
it open (134-138). In the lab sandbox, which has no sound devices, each
client answered the other's 1002 with status 0x10004 (unimplemented) and
logged "Operation is unimplemented. (ID 1/4 @ S101)" (139-141), so no 1003
flowed.

## Open questions

- Capture [1003 Voice Data](messages/voice.md#m1003), which could not be started
  in the sandbox; see `.scratch/capture-gaps/issues/07-dcc-voice.md`.
