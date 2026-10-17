# Client to server

For hex and field notation, see [Conventions](README.md#conventions).

This is the main connection, between a client and a KDX server on TCP port
10700. It carries all the messages in the [index](messages/README.md) except the
       tracker and GConsole messages. File transfers use more connections of the
       same kind.

## Connecting

1. The client connects to TCP port 10700 and sends its hello, with sub-protocol
   `%KDX` and sub-version 0x10. The server answers with sub-version 0x20 and a
   nonce. Then both sides prove that they know the account key and the challenge
   table; see [handshake.md](handshake.md). <!-- [O
   `captures/linux-first-connect.pcapng` frames 8, 10] -->
2. The login name selects the account. An empty name is the `guest` account. A
   name of `0x00` and a 4-byte user ID attaches a transfer connection to a user
   who is logged in; see [transfers.md](transfers.md). <!-- [S `FUN_0805270c`]
   -->
3. Messages follow. Each is a 16-byte header and a payload, encrypted with the
   account key; see [framing.md](framing.md). <!-- [S; O] -->
4. The client sends [100 Set Identity](messages/session.md#m100), and the server
   answers with the user list and the server info.
   [sequences.md](sequences.md#login) gives the full exchange. <!-- [O
   `captures/linux-first-connect.pcapng` frames 23, 24] -->

## Requests and replies

Request N gets reply N+1, or [1 Status](messages/status.md#m1) if it fails. A
reply has the txid of its request, and a notification has the server's own txid.
[The protocol overview](README.md#requests-replies-and-notifications) gives the
rules and the header. [encodings.md](encodings.md) gives the encodings that many
messages share.

## Messages

[messages/README.md](messages/README.md) lists all messages on this connection
by area, with the page that describes each one.

## Reference

- [Error codes](messages/status.md#error-codes) in [1
  Status](messages/status.md#m1)
- [privileges.md](privileges.md): the privilege bits of classes and accounts
- [sequences.md](sequences.md): login, flood ban, posting news, and logout
