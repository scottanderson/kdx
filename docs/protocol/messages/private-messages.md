# Private messages

For hex and field notation, see [Conventions](../README.md#conventions).

Message types 300 and 301 carry a private message to one user. A client also
sends [300 Send Private Message](#m300) to another client on a DCC connection;
see [dcc.md](../dcc.md#primary-connection).

## <a id="m300"></a>300 Send Private Message
Initiator: Client. Privilege: Send Messages (32). <!-- [S `FUN_0804d2d0`; O] -->

Struct `PrivateMessage` (dynamic):<a id="sPrivateMessage"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `u32` | `user_id` | The target's ID in 300, and the sender's in [301](#m301). |
| 0x04 | 12 | `u8[12]` | `ignored` |  |
| 0x10 | rest | `str` | `text` |  |

The payload must be at least 0x11 bytes. The server forwards it to the target as
[301 Incoming Private Message](#m301).

## <a id="m301"></a>301 Incoming Private Message
Initiator: Server. Forwarded from [300 Send Private Message](#m300).

The server sends the layout of 300 on to the target, with the sender's ID in
place of the target's.

Struct `IncomingPrivateMessage` (dynamic):<a id="sIncomingPrivateMessage"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 |  | [`PrivateMessage`](#sPrivateMessage) | `message` | `user_id` is the sender's. |
