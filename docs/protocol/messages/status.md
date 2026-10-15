# Status

Message type 1: the reply that carries a result code, for an error or for a
success. The codes are in [Error codes](#error-codes).

## <a id="m1"></a>1 Status

Initiator: Server. A tracker and a DCC peer send it too. Sent as a reply. **[S
`FUN_08086640`, `FUN_080865c0`; O]**

The dispatcher (`FUN_0804fdac`) runs each handler inside a C++ `try`. The
catch at 0x0805090c turns the exception into a 1 Status reply to the
request's txid. Some handlers also send 1 Status on success.

Error form, 24 bytes:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | code | an error code (see [Error codes](#error-codes)) |
| 0x04 | 4 | request type | e.g. 0x98 for a failed [152 Get User Info](administration.md#m152) **[O]** |
| 0x08 | 4 | module | source module tag of the raise site, ASCII packed into an integer (`0x54524e` = `TRN`; 0x46 for the path code) |
| 0x0c | 4 | line | source line of the raise site |
| 0x10 | 4 | extra | a value the raise site passes along |
| 0x14 | 4 | 0 | |

**[O]** examples:

- `152:00000003` (no user 3) returns `00 01 00 0c 00 00 00 98 00…`, which is
  0x1000c item does not exist.
- `400:0000` (malformed path) returns `00 01 00 0a 00 00 01 90 00 00 00 46
  00 00 00 98…`, which is 0x1000a corrupt, raised in module 0x46 at line
  0x98 of `FUN_080969f8`.

Success form, 8 bytes: `u32 0`, then `u32 value`, often the request type **[S
`FUN_080865c0`]**. [507 Create Account](accounts.md#m507) gets value 0x1fb and
[511 Set Account](accounts.md#m511) 0x1ff **[O `captures/kdx-accounts.pcapng`
frames 235, 413]**. [201 Delete Chat](chat.md#m201) gets 0xc9,
[217 Create Chat](chat.md#m217) 0xd9, [220 Set Chat](chat.md#m220) 0xdc, and
[504 Set Account Class](accounts.md#m504) 0x1f8 **[O
`captures/linux-chat-rooms.pcapng` frames 81, 45, 55, 148]**.

The client renders an error as `ID ^0/^1 [^2] @ ^3 #^4`, filling in the
code's class and number, the module, and the line **[S]**.

### Tracker form

The tracker's error Status is 8 bytes: the error code, then the request type as
a u32, like the first 8 bytes of the server's 24-byte error form above **[O
`captures/linux-tracker.pcapng` frames 61, 164; `kdx-cli-tracker` 39, 53]**:

```
00 01 00 14  00 00 02 c6    0x10014 for 710
```

Verified: `kdx-proto` decodes frame 61 as `Status::TrackerError` and
re-encodes it byte for byte **[O]**.

### Client to client

A DCC primary connection handles 1 as a status reply, with no other detail
known (see [dcc.md](../dcc.md#primary-connection)) **[S]**.

## Error codes

The codes carried by [1 Status](#m1).

The server's name table is `FUN_08066d04`. It has 79 entries: the codes are
at 0x080b1dc0 and the text pointers at 0x080b1f00 **[S]**. The high 16 bits
give the class.

| Code | Meaning |
|---|---|
| 0x10001 | unknown error |
| 0x10002 | invalid value |
| 0x10003 | protocol breach |
| 0x10004 | unimplemented |
| 0x10005 | unsupported |
| 0x10006 | timed out |
| 0x10007 | aborted |
| 0x10008 | data version incompatible |
| 0x10009 | data format incompatible |
| 0x1000a | corrupt data |
| 0x1000b | out of range |
| 0x1000c | item does not exist |
| 0x1000d | item already exists |
| 0x1000e | limit cannot be exceeded |
| 0x1000f | locked |
| 0x10010 | data file missing |
| 0x10011 | resource missing |
| 0x10012 | process does not exist |
| 0x10013 | IPC queue full |
| 0x10014 | Access Denied (a missing privilege) |
| 0x10015 | hardware error |
| 0x10016 | hardware busy |
| 0x10017 | unknown view class |
| 0x10018 | view does not exist |
| 0x10019 | invalid object |
| 0x1001a | buffer too small |
| 0x1001b | unknown OS error |
| 0x1001c | not initialized |
| 0x1001d | re-entry not allowed |
| 0x1001e | invalid name |
| 0x20064-0x20066 | memory errors |
| 0x30064-0x30065 | file system errors |
| 0x30066 | file, folder, or item missing |
| 0x30067 | file missing |
| 0x30068 | folder missing |
| 0x30069 | alias missing |
| 0x3006a | expected a file |
| 0x3006b | expected a folder |
| 0x3006c | expected an alias |
| 0x3006d | alias target missing |
| 0x3006e | read-only volume |
| 0x3006f | volume closed |
| 0x30070 | no space |
| 0x30071 | volume limit |
| 0x30072 | wrong passphrase |
| 0x30073 | name already exists |
| 0x30074 | end of file |
| 0x30075 | file is open |
| 0x30076 | invalid name |
| 0x30077 | access denied by the file system |
| 0x30078 | cross-volume operation |
| 0x30079 | folder too deep or alias loop |
| 0x3007a | volume missing |
| 0x40064-0x4006f | network errors |
| 0x40070 | Incorrect login and/or password. |
| 0x40071 | The network protocol is unknown. |
| 0x40072 | the peer's protocol is too old |
| 0x40073 | the peer's protocol is more recent |
| 0x40074 | Received data that was invalid according to the protocol. |
| 0x40075 | A limit in the net protocol was exceeded. |
| 0x40076 | can't send fast enough |
| 0x40077 | Banned for flooding. |
| 0x50064-0x50066 | sound errors |

Codes outside the table also occur. [507 Create Account](accounts.md#m507) and
[511 Set Account](accounts.md#m511) raise 0x640005 when the account would hold a
privilege the requester lacks **[S `FUN_080551b0`]**.
