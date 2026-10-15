# Process messages

Tags, hex, and field notation: see [Conventions](../README.md#conventions).

Message types 800 to 804: listing and ending the processes of the server host.

## <a id="m800"></a>800 Open Process Monitor
Initiator: Client. Privilege: View Processes (80). **[S `FUN_0805b394`]**

## <a id="m801"></a>801 Close Process Monitor
Initiator: Client. Privilege: View Processes (80). **[S `FUN_0805b418`]**

## <a id="m802"></a>802 Get Process List
Initiator: Client. Privilege: View Processes (80). **[S `FUN_0805b454`; O]**

The request is empty. The server runs `/bin/ps auxwww` (its own child shows
up in the list) and replies with [803 Process List](#m803).

## <a id="m803"></a>803 Process List
Initiator: Server. Reply to [802 Get Process List](#m802).

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | | about 33 GB; host memory **[I]** |
| 0x08 | 8 | | about 33 GB **[I]** |
| 0x10 | 0xc | | |
| 0x1c | 4 | count | |
| 0x20 | | process entries | |

Each process entry is padded to 4:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | PID | |
| 0x04 | 4 | | |
| 0x08 | 8 | start time | |
| 0x10 | 0x29 | | |
| 0x39 | 1 | user length | |
| 0x3a | 1 | command length | at most 0xff; longer commands are cut |
| 0x3c | | command, user | |

## <a id="m804"></a>804 Exit/Terminate Process
Initiator: Client. Privilege: Terminate Processes (81). **[S `FUN_0805b8f4`,
`FUN_08090c6c`]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | flags | bit 0 kill, bit 1 exit, bit 2 extra flag; one of bits 0 and 1 is required, else 0x10002 |
| 0x04 | 4 | PID | |
| 0x08 | 4 | | must be 0, or nothing happens |

On Linux, kill sends SIGKILL and exit sends SIGTERM to the PID. Bit 2 gets
0x10005 (unsupported) on Linux. A failed `kill()` returns the OS error.
There is no reply on success; the server logs `Terminate Process` and
writes history record 0x324 **[S]**. The client's Exit Process sends flags
2 (exit) **[O `captures/linux-shutdown.pcapng` frame 25]**.
`kdx_proto::msg::admin::ExitTerminateProcess` re-encodes it.
