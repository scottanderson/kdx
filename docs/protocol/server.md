# Client to server

The main connection: a client talks to a KDX server on TCP 10700. Every message
in the catalog of [KDXProtocol.md](KDXProtocol.md) except the tracker and
GConsole ones goes over it, and so do file transfers, on extra connections.

## Connecting

1. The client opens TCP 10700 and sends its hello with sub-protocol `%KDX` and
   sub-version 0x10. The server answers with sub-version 0x20 and a nonce, and
   both sides prove knowledge of the account key and the challenge table
   ([handshake.md](handshake.md)).
   **[O `linux-first-connect.pcapng` frames 8, 10]**
2. The login name picks the account. An empty name is the `guest` account. A
   name of `0x00` and a 4-byte user ID attaches a transfer connection to a
   logged-in user ([messages/transfers.md](messages/transfers.md)).
   **[S `FUN_0805270c`]**
3. Messages follow, each a 16-byte header and a payload encrypted under the
   account key ([session.md](session.md)). **[S, O]**
4. The client sends 100 Set Identity, and the server answers with the user list
   and server info. The full exchange is in [sequences.md](sequences.md#login).
   **[O `linux-first-connect.pcapng` frames 23, 24]**

## Requests and replies

Request N gets reply N+1, or a 1 Status when it fails. Replies reuse the
request's txid. Notifications carry the server's own. The rules, and the header,
are in [KDXProtocol.md](KDXProtocol.md#requests-replies-and-notifications).
Encodings that many messages share are in [encodings.md](encodings.md).

## Messages by area

| Area | Types | Page |
|---|---|---|
| Session, users, server info | 100 to 118 | [messages/session.md](messages/session.md) |
| Monitoring and administration | 140 to 189 | [messages/administration.md](messages/administration.md) |
| Chat and private messages | 200 to 223, 300, 301 | [messages/chat.md](messages/chat.md) |
| Files | 400 to 423, 450 to 453 | [messages/files.md](messages/files.md) |
| Downloads and uploads | 412 to 419 | [messages/transfers.md](messages/transfers.md) |
| Accounts | 502 to 551 | [messages/accounts.md](messages/accounts.md) |
| News | 600 to 613 | [messages/news.md](messages/news.md) |
| Processes | 800 to 804 | [messages/processes.md](messages/processes.md) |
| View display | 900 to 905 | [messages/view-display.md](messages/view-display.md) |
| Folder access | 1100 to 1107 | [messages/folder-access.md](messages/folder-access.md) |

## Reference

- [errors.md](errors.md): the codes carried by 1 Status.
- [privileges.md](privileges.md): the privilege bits of classes and accounts.
- [sequences.md](sequences.md): login, flood ban, posting news, logout.
