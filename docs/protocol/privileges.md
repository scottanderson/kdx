# Access privileges

For hex and field notation, see [Conventions](README.md#conventions).

This page lists the privilege bits of account classes and accounts.

A set of privileges is a 128-bit big-endian number. Bit n has the mask `1 << 127
>> n`, so bit 0 is the high bit of the first byte, and bit n is in byte n/8 with
the mask 0x80 >> (n % 8). The server stores a user's privileges at user +0x44 to
+0x53. It tests one privilege as a single byte, and combines sets with AND, OR,
and NOT over all 16 bytes (`FUN_080551b0`). <!-- [S] --> Bits 83 to 127 have no
known use.

Struct `Privileges` (16 bytes):<a id="sPrivileges"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 16 | `u128` | `bits` | A bit set. The table below names the bits. |

The table gives the bits that server handlers test, found by a scan of all
handlers for tests of user +0x44 to +0x4e. <!-- [S] --> "Login" is
`FUN_0804b114`, the step after login that sends [105 Server
Info](messages/session.md#m105), [107 Server
Greeting](messages/session.md#m107), and [213 Must Join
Chat](messages/chat.md#m213).

| Bit | Privilege | Tested in |
|---:|---|---|
| 0 | Override. It overrides the class lists of chats, and the limits on news and accounts. It is not in the client's list. | [202 Join Chat](messages/chat.md#m202), [207 Send Chat Text](messages/chat.md#m207), [209 Set Chat Topic](messages/chat.md#m209), [400 Get File List](messages/files.md#m400), [508 Delete Account](messages/accounts.md#m508), [511 Set Account](messages/accounts.md#m511), [607 Get News Messages](messages/news.md#m607), [609 Get News Message](messages/news.md#m609), [611 Post News Message](messages/news.md#m611), [612 Delete News Message](messages/news.md#m612) |
| 3 | Color Overrides Class | `FUN_08054900` account lookup |
| 4 | Can Login | set in all lab classes |
| 10 | Download | [412 Download File](messages/files.md#m412) |
| 11 | Upload | [402 Create Folder](messages/files.md#m402), [415 Upload File](messages/files.md#m415) |
| 13 | Empty Trash | [421 Empty Trash](messages/files.md#m421) |
| 15 | Create Aliases | [403 Create Alias](messages/files.md#m403) |
| 16 | Move | [407 Move File/Folder](messages/files.md#m407) |
| 17 | Rename | [409 Get File/Folder Info](messages/files.md#m409), [411 Set File/Folder Info](messages/files.md#m411) |
| 18 | Delete | [405 Move To Trash](messages/files.md#m405) |
| 20 | Set File/Folder Attributes | 409 Get File/Folder Info, 411 Set File/Folder Info |
| 22 | See All Volumes | 400 Get File List, [423 Set Volume Info](messages/files.md#m423) |
| 23 | Has Custom Base Folder | |
| 24 | See All Bases | |
| 25 | Allow Simultaneous Downloads | 412 Download File (large files) |
| 26 | Ignore Speed Limits | |
| 27 | View Folder Access | [505 Get Account Class List](messages/accounts.md#m505), [1103 Get Folder Access](messages/folder-access.md#m1103), [1106 Get Folder Access List](messages/folder-access.md#m1106) |
| 28 | Modify Folder Access | 402 Create Folder, 403 Create Alias, 411 Set File/Folder Info, [1100 Create Folder Access](messages/folder-access.md#m1100), [1101 Delete Folder Access](messages/folder-access.md#m1101), [1105 Set Folder Access](messages/folder-access.md#m1105) |
| 30 | Join Chat | 202 Join Chat, [214 Get Chat List](messages/chat.md#m214), login |
| 31 | Send Chat | 207 Send Chat Text |
| 32 | Send Messages | [300 Send Private Message](messages/private-messages.md#m300) |
| 33 | Broadcast | [154 Send Broadcast Message](messages/administration.md#m154) |
| 34 | Create Private Chat | [200 Create Private Chat](messages/chat.md#m200) |
| 35 | Set Chat Topic | 209 Set Chat Topic |
| 36 | Join Chat On Connect | login |
| 37 | Create/Modify Chats | [201 Delete Chat](messages/chat.md#m201), [217 Create Chat](messages/chat.md#m217), [218 Get Chat](messages/chat.md#m218), [220 Set Chat](messages/chat.md#m220), 505 Get Account Class List |
| 40 | Get User Info | [140 Open Connection Monitor](messages/monitoring.md#m140), [141 Close Connection Monitor](messages/monitoring.md#m141), [142 Get Connection Monitor Info](messages/monitoring.md#m142), [152 Get User Info](messages/administration.md#m152) |
| 41 | Disconnect Users | [150 Disconnect User](messages/administration.md#m150) |
| 42 | Cannot Be Disconnected | 150 (target) |
| 43 | Ban Users | 150 Disconnect User |
| 50 | View Accounts | [502 Get Account Class](messages/accounts.md#m502), 505 Get Account Class List, [509 Get Account](messages/accounts.md#m509), [512 Get Account List](messages/accounts.md#m512) |
| 51 | Create Accounts | [507 Create Account](messages/accounts.md#m507) |
| 52 | Modify Accounts | [504 Set Account Class](messages/accounts.md#m504), 508 Delete Account, 511 Set Account |
| 53 | Change Own Password | [550 Change Own Password](messages/accounts.md#m550) |
| 54 | Protected Account | 508 Delete Account, 511 Set Account (target) |
| 60 | Use Any Name | [100 Set Identity](messages/session.md#m100), login |
| 61 | Modify Server Settings | [113 Get Extra Server Info](messages/session.md#m113), [170 Get Server Settings](messages/administration.md#m170), [172 Set Server Settings](messages/administration.md#m172), [450 Generate Catalog](messages/files.md#m450), login (105 Server Info) |
| 62 | Don't Show Greeting | login |
| 63 | View Server History | [180 Server History Join](messages/administration.md#m180) |
| 64 | Shutdown Server | [174 Shutdown Server](messages/administration.md#m174), [175 Upgrade Server Software](messages/administration.md#m175) |
| 65 | Disable Encryption | 412 Download File, 415 Upload File (plain mode) |
| 66 | View Display | [900 Open View Display](messages/view-display.md#m900) |
| 67 | Control Display | [902 View Display Params](messages/view-display.md#m902), [904 View Display Mouse](messages/view-display.md#m904), [905 View Display Keyboard](messages/view-display.md#m905) |
| 70 | View News | [605 Get Newsgroup List](messages/news.md#m605), 607 Get News Messages, 609 Get News Message |
| 71 | Post News | 607 Get News Messages, 611 Post News Message |
| 72 | Delete News Messages | 607 Get News Messages, 612 Delete News Message |
| 73 | Create/Modify Newsgroups | 505 Get Account Class List, [600 Create Newsgroup](messages/news.md#m600), [601 Delete Newsgroup](messages/news.md#m601), [602 Get Newsgroup](messages/news.md#m602), [604 Set Newsgroup](messages/news.md#m604) |
| 74 | See Logins In News | 609 Get News Message |
| 80 | View Processes | [800 Open Process Monitor](messages/processes.md#m800), [802 Get Process List](messages/processes.md#m802) |
| 81 | Terminate Processes | [804 Exit/Terminate Process](messages/processes.md#m804) |
| 82 | Launch Programs | 175 Upgrade Server Software, [420 Launch File](messages/files.md#m420) |

All classes have bits 2 and 4 set (0x44&0x20 and 0x44&0x08). Bit 4 is Can Login,
and bit 2 has no name in the client's table. <!-- [S] --> If the administrator
answers no to guest access in the server's first-run setup, the server clears
Can Login on the `guest` account and sets it in that account's mask. <!-- [S
`FUN_08054394`] --> The client never adds the 0x17-byte info trailer that sets
user flag bit 1 (see [User record](encodings.md#user-record)), so the program
that sends it is not Client 1.620 beta 5. <!-- [S client `FUN_0804d654`] -->

The client's account editor lists 50 privileges in this order, from the
resources of KDXClient.exe: <!-- [S] -->

1. Download, Upload, Create Aliases, Move, Rename, Delete, Empty Trash, Set
   Attributes, Has Custom Base Folder, See All Bases, See All Volumes, Allow
   Simultaneous Downloads, Ignore Speed Limits
2. View Folder Access, Modify Folder Access
3. Create/Modify Chats, Join Chat, Send Chat, Create Private Chat, Set Chat
   Topic, Join Chat On Connect
4. Send Messages, Broadcast, Get User Info, Disconnect Users, Cannot Be
   Disconnected, Ban Users
5. View Accounts, Create Accounts, Modify Accounts, Change Own Password,
   Protected Account
6. View News, Post News, Delete News Messages, Create/Modify Newsgroups, See
   Logins In News
7. View Processes, Terminate Processes, Launch Programs, View Display, Control
   Display
8. Can Login, Color Overrides Class, Use Any Name, Modify Server Settings, Don't
   Show Greeting, View Server History, Shutdown Server, Disable Encryption

The client's editor table at 0x0812c760 (file offset 0xe4760) pairs each label
with its bit number. <!-- [S] --> The names in the table above come from it. The
other bits have no label.
