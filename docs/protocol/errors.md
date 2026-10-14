# Error codes

The codes carried by 1 Status.

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
| 0x20064–0x20066 | memory errors |
| 0x30064–0x30065 | file system errors |
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
| 0x40064–0x4006f | network errors |
| 0x40070 | Incorrect login and/or password. |
| 0x40071 | The network protocol is unknown. |
| 0x40072 | the peer's protocol is too old |
| 0x40073 | the peer's protocol is more recent |
| 0x40074 | Received data that was invalid according to the protocol. |
| 0x40075 | A limit in the net protocol was exceeded. |
| 0x40076 | can't send fast enough |
| 0x40077 | Banned for flooding. |
| 0x50064–0x50066 | sound errors |

Codes outside the table also occur. 507 Create Account and 511 Set Account
raise 0x640005 when the account would hold a privilege the requester lacks
**[S `FUN_080551b0`]**.
