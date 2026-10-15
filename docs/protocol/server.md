# Client to server

The main connection: a client talks to a KDX server on TCP 10700. Every message
in the [index](messages/README.md) except the tracker and GConsole ones goes
over it, and so do file transfers, on extra connections.

## Connecting

1. The client opens TCP 10700 and sends its hello with sub-protocol `%KDX` and
   sub-version 0x10. The server answers with sub-version 0x20 and a nonce, and
   both sides prove knowledge of the account key and the challenge table
   ([handshake.md](handshake.md)).
   **[O `linux-first-connect.pcapng` frames 8, 10]**
2. The login name picks the account. An empty name is the `guest` account. A
   name of `0x00` and a 4-byte user ID attaches a transfer connection to a
   logged-in user ([transfers.md](transfers.md)).
   **[S `FUN_0805270c`]**
3. Messages follow, each a 16-byte header and a payload encrypted under the
   account key ([framing.md](framing.md)). **[S, O]**
4. The client sends [100 Set Identity](messages/session.md#m100), and the server
   answers with the user list and server info. The full exchange is in
   [sequences.md](sequences.md#login). **[O `linux-first-connect.pcapng` frames
   23, 24]**

## Requests and replies

Request N gets reply N+1, or a [1 Status](messages/status.md#m1) when it fails.
Replies reuse the request's txid. Notifications carry the server's own. The
rules, and the header, are in
[the protocol overview](README.md#requests-replies-and-notifications). Encodings
that many messages share are in [encodings.md](encodings.md).

## Messages

[messages/README.md](messages/README.md) lists every message this connection
carries, by area, with the page that describes each.

## Reference

- [Error codes](messages/status.md#error-codes) carried by
  [1 Status](messages/status.md#m1).
- [privileges.md](privileges.md): the privilege bits of classes and accounts.
- [sequences.md](sequences.md): login, flood ban, posting news, logout.
