# Access privileges

The privilege bits of account classes and accounts.

An account's privileges are a bit string of at least 11 bytes. It is stored
at user +0x44 and sent in the class records (506 Account Class List). Bit n
is byte n/8, mask 0x80 >> (n % 8) **[S]**.

Bits that server handlers test, found by scanning every handler for tests of
user +0x44..+0x4e **[S]**. `FUN_0804b114` is the post-login step that sends
105 Server Info, 107 Server Greeting, and 213.

| Bit | Privilege | Tested in |
|---:|---|---|
| 0 | override: bypasses chat class lists and news/account restrictions; not in the client's list | 202 Join Chat, 207 Send Chat Text, 209 Set Chat Topic, 400 Get File List, 508 Delete Account, 511 Set Account, 607 Get News Messages, 609 Get News Message, 611 Post News Message, 612 Delete News Message |
| 3 | Color Overrides Class | `FUN_08054900` account lookup |
| 4 | Can Login | set in every lab class |
| 10 | Download | 412 Download File |
| 11 | Upload | 402 Create Folder, 415 Upload File |
| 13 | Empty Trash | 421 Empty Trash |
| 15 | Create Aliases | 403 Create Alias |
| 16 | Move | 407 Move File/Folder |
| 17 | Rename | 409 Get File/Folder Info, 411 Set File/Folder Info |
| 18 | Delete | 405 Move To Trash |
| 20 | Set File/Folder Attributes | 409 Get File/Folder Info, 411 Set File/Folder Info |
| 22 | See All Volumes | 400 Get File List, 423 Set Volume Info |
| 23 | Has Custom Base Folder | |
| 24 | See All Bases | |
| 25 | Allow Simultaneous Downloads | 412 Download File (large files) |
| 26 | Ignore Speed Limits | |
| 27 | View Folder Access | 505 Get Account Class List, 1103 Get Folder Access, 1106 Get Folder Access List |
| 28 | Modify Folder Access | 402 Create Folder, 403 Create Alias, 411 Set File/Folder Info, 1100 Create Folder Access, 1101 Delete Folder Access, 1105 Set Folder Access |
| 30 | Join Chat | 202 Join Chat, 214 Get Chat List, login |
| 31 | Send Chat | 207 Send Chat Text |
| 32 | Send Messages | 300 Send Private Message |
| 33 | Broadcast | 154 Send Broadcast Message |
| 34 | Create Private Chat | 200 Create Private Chat |
| 35 | Set Chat Topic | 209 Set Chat Topic |
| 36 | Join Chat On Connect | login |
| 37 | Create/Modify Chats | 201 Delete Chat, 217 Create Chat, 218 Get Chat, 220 Set Chat, 505 Get Account Class List |
| 40 | Get User Info | 140 Open Connection Monitor, 141 Close Connection Monitor, 142 Get Connection Monitor Info, 152 Get User Info |
| 41 | Disconnect Users | 150 Disconnect User |
| 42 | Cannot Be Disconnected | 150 (target) |
| 43 | Ban Users | 150 Disconnect User |
| 50 | View Accounts | 502 Get Account Class, 505 Get Account Class List, 509 Get Account, 512 Get Account List |
| 51 | Create Accounts | 507 Create Account |
| 52 | Modify Accounts | 504 Set Account Class, 508 Delete Account, 511 Set Account |
| 53 | Change Own Password | 550 Change Own Password |
| 54 | Protected Account | 508 Delete Account, 511 Set Account (target) |
| 60 | Use Any Name | 100 Set Identity, login |
| 61 | Modify Server Settings | 113 Get Extra Server Info, 170 Get Server Settings, 172 Set Server Settings, 450 Generate Catalog, login (105 Server Info) |
| 62 | Don't Show Greeting | login |
| 63 | View Server History | 180 Server History Join |
| 64 | Shutdown Server | 174 Shutdown Server, 175 Upgrade Server Software |
| 65 | Disable Encryption |
| 66 | View Display | 900 Open View Display |
| 67 | Control Display | 902 View Display Params, 904, 905 | 412 Download File, 415 Upload File (plain mode) |
| 70 | View News | 605 Get Newsgroup List, 607 Get News Messages, 609 Get News Message |
| 71 | Post News | 607 Get News Messages, 611 Post News Message |
| 72 | Delete News Messages | 607 Get News Messages, 612 Delete News Message |
| 73 | Create/Modify Newsgroups | 505 Get Account Class List, 600 Create Newsgroup, 601 Delete Newsgroup, 602 Get Newsgroup, 604 Set Newsgroup |
| 74 | See Logins In News | 609 Get News Message |
| 80 | View Processes | 800 Open Process Monitor, 802 Get Process List |
| 81 | Terminate Processes | 804 Exit/Terminate Process |
| 82 | Launch Programs | 175 Upgrade Server Software, 420 Launch File |

Bits 2 and 4 (0x44&0x20, 0x44&0x08) are set in every class. Bit 4 is Can
Login; bit 2 has no name in the client's table **[S]**. Answering no to
guest access at the server's first-run setup clears Can Login on the
`guest` account and sets it in that account's mask **[S `FUN_08054394`]**.
The client never appends the 0x17-byte info trailer that sets user flag bit
1 (see User record), so whatever sends it is not Client 1.620 beta 5 **[S
client `FUN_0804d654`]**.

The client's account editor lists 50 privileges in this order **[S
KDXClient.exe resources]**:

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
7. View Processes, Terminate Processes, Launch Programs, View Display,
   Control Display
8. Can Login, Color Overrides Class, Use Any Name, Modify Server Settings,
   Don't Show Greeting, View Server History, Shutdown Server, Disable
   Encryption

The client's editor table at 0x0812c760 (file offset 0xe4760) pairs each
label with its bit number **[S]**, which gives the names in the table
above. The other bits have no label.
