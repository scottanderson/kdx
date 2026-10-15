# Connection monitor messages

Tags, hex, and field notation: see [Conventions](../README.md#conventions).

Message types 140 to 143: watching the list of connections to the server.

## <a id="m140"></a>140 Open Connection Monitor
Initiator: Client. **[S]**
Handled inline in the dispatcher **[S]**.

No payload.

The monitor is one open, any number of polls, and one close:

```mermaid
%%{init: {"sequence": {"mirrorActors": false}}}%%
sequenceDiagram
    participant C as Client
    participant S as Server
    C->>S: 140 Open Connection Monitor
    loop while the monitor is open
        C->>S: 142 Get Connection Monitor Info
        S->>C: 143 Connection Monitor Info
    end
    C->>S: 141 Close Connection Monitor
```

[140 Open Connection Monitor](#m140) and [141 Close Connection Monitor](#m141)
need Get User Info (bit 40) and turn a per-user flag on or off. There is no
reply.

## <a id="m141"></a>141 Close Connection Monitor
Initiator: Client. **[S]**
Handled inline in the dispatcher **[S]**.

No payload.

The counterpart of [140 Open Connection Monitor](#m140): it turns the flag
off. There is no reply.

## <a id="m142"></a>142 Get Connection Monitor Info
Initiator: Client. **[S `FUN_0804b9b0`; O]**

No payload.

The request needs Get User Info (bit 40); without a
[140 Open Connection Monitor](#m140) first, the server replies with
[1 Status](status.md#m1) 0x10003 (protocol breach) **[O]**.

## <a id="m143"></a>143 Connection Monitor Info
Initiator: Server. **[S `FUN_0804b9b0`; O]**

Reply to [142 Get Connection Monitor Info](#m142). The reply is one record per
connection, each followed by one record per file transfer of that user, with no
count in front, each padded to 4 **[S `FUN_0804b9b0`]**:

Struct `ConnectionMonitorInfo` (dynamic):

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | rest | MonitoredConnection[] | records |  |

Struct `MonitoredConnection` (dynamic):

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | u32 | user_id |  |
| 0x04 | 4 | u32 | transfer_id | 0 in a connection record; the transfer's ID in a transfer record **[S]** |
| 0x08 | 8 | u64 | login_time | the transfer's start time in a transfer record **[S]** |
| 0x10 | 8 | u64 | time_10 | 0 in a transfer record **[S]** |
| 0x18 | 8 | u64 | time_18 | **[S]** |
| 0x20 | 8 | u64 | bytes_done | transfer record only: bytes moved minus bytes still queued to send **[S]** |
| 0x28 | 8 | u64 | bytes_total | transfer record only **[S]** |
| 0x30 | 4 | u32 | rate | transfer record only: bytes per second since the start, not counting a resumed offset **[S]**; 0x51a1 under a 0x5000 limit **[O]** |
| 0x34 | 4 | u32 | seconds_left | transfer record only: (total − done) / rate **[S]** |
| 0x38 | 4 | u32 | value_38 | transfer record only; a per-transfer value **[S]**; 0x5000, the server's speed limit, in every record **[O]** |
| 0x3c | 4 | u32 | state | transfer record only; 0x90 while data is queued and the state is 0 **[S]**; 0x19c ([412 Download File](files.md#m412)) for a download **[O]** |
| 0x40 | 2 | u8[2] | ignored_40 | 0 as sent. |
| 0x42 | 1 | u8 | class_index | **[S]** |
| 0x43 | 29 | u8[29] | ignored_43 |  |
| 0x60 | 1 | u8 | nick_len |  |
| 0x61 | 1 | u8 | login_len |  |
| 0x62 | 1 | u8 | account_len |  |
| 0x63 | 1 | u8 | address_len |  |
| 0x64 | 2 | u8[2] | ignored_64 |  |
| 0x66 | 2 | u16 | path_len |  |
| 0x68 | nick_len | str | nick | `probe` **[O]** |
|  | login_len | str | login | `admin` **[O]** |
|  | account_len | str | account | `Administrator` **[O]** `?` if the class is gone **[S]** |
|  | address_len | str | address | `127.0.0.1:50984` **[O]** the connection's peer address; the transfer connection's in a transfer record **[S]** |
|  | path_len | FilePath | path | transfer record only; the root in a connection record, as in the file messages **[S; O]** |
|  | pad4 | u8[] | padding | padding to a multiple of 4 |

The padding after the strings is uninitialised **[O]**; `kdx-proto` keeps
it on a round trip. Fields marked transfer record only are 0 in a connection
record.

In `captures/linux-catalog-volumes.pcapng`, the replies from frame 104 to 161
hold three records: the operator's connection, the `kdx-get` connection
(user 2), and its download of `speed.bin` (transfer 3) **[O]**. In frame
104 the transfer has done 0x14000 of 0x200000 bytes at 0x6aaa per second,
with 0x49 seconds left; in frame 161, 0x1f4000 at 0x51a1, with 2 left.
