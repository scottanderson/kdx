# Messages

Every message type, in order, grouped by the page that describes it. Each page
covers one contiguous range of type numbers, so a type's page can be found from
its number alone.

A KDX connection is one of four kinds, set by the sub-version of the sender's
hello (see [handshake.md](../handshake.md#connection-types)): client to client
(DCC), client to server, client to tracker, and client to GConsole. The
DCC, Server, Tracker and GConsole columns show which kinds use each message.
A dot means the kind handles or sends the message **[S]**; a blank means it has
no handler there, or nothing shows one. Most messages belong to the server.

Each message has a stable link made of the page that describes it and its
type number: [chat.md#m200](chat.md#m200) is 200 Create Private Chat. The last
column links to it. The header and the rules for requests and replies are in
[the protocol overview](../README.md#messages), and where the names come from is
in [its message types section](../README.md#message-types).

## How to read the tables

- **Request:** sent by the one that wants something done. The Role column
  names the reply, when there is one.
- **Reply:** sent back for a request, with the request's transaction ID and
  the reply bit set.
- **Notification:** sent on its own, without a request from the receiver. It
  may follow some other message, as the Role column says.
- **Data:** the bytes of a transfer or a stream, sent after a request.
- **Sent by** is the end of the connection that sends the message.

## Messages by page

### [Status](status.md), 1

| Type | Message | Sent by | Role | DCC | Server | Tracker | GConsole | Details |
|---:|---|---|---|:---:|:---:|:---:|:---:|---|
| 1 | Status | Server, tracker, or client | Reply that carries a result code | ● | ● | ● |  | [Status](status.md#m1) |

### [Session](session.md), 100 to 118

| Type | Message | Sent by | Role | DCC | Server | Tracker | GConsole | Details |
|---:|---|---|---|:---:|:---:|:---:|:---:|---|
| 100 | Set Identity | Client | Request | ● | ● |  |  | [Session](session.md#m100) |
| 101 | User List | Server | Notification after login |  | ● |  |  | [Session](session.md#m101) |
| 102 | User Changed | Server | Notification: a user changed their identity |  | ● |  |  | [Session](session.md#m102) |
| 103 | User Joined | Server | Notification: a user logged in |  | ● |  |  | [Session](session.md#m103) |
| 104 | User Departed | Server | Notification: a user left |  | ● |  |  | [Session](session.md#m104) |
| 105 | Server Info | Server | Notification after login |  | ● |  |  | [Session](session.md#m105) |
| 106 | Server Icon | Server | Notification after login |  | ● |  |  | [Session](session.md#m106) |
| 107 | Server Greeting | Server or tracker | Notification after login |  | ● | ● |  | [Session](session.md#m107) |
| 108 | Ping | Client | Request, reply 109 |  | ● |  |  | [Session](session.md#m108) |
| 109 | Pong | Server | Reply to 108 |  | ● |  |  | [Session](session.md#m109) |
| 110 | Set Mini User Icon | Client | Request |  | ● |  |  | [Session](session.md#m110) |
| 111 | Mini User Icon Changed | Server | Notification: a user changed their mini icon |  | ● |  |  | [Session](session.md#m111) |
| 112 | (unnamed; requests the greeting) | Client | Request |  | ● |  |  | [Session](session.md#m112) |
| 113 | Get Extra Server Info | Client | Request, reply 114 |  | ● |  |  | [Session](session.md#m113) |
| 114 | Extra Server Info | Server | Reply to 113 |  | ● |  |  | [Session](session.md#m114) |
| 115 | Get My Address | Client | Request, reply 116 |  | ● | ● |  | [Session](session.md#m115) |
| 116 | Your Address | Server or tracker | Reply to 115 |  | ● | ● |  | [Session](session.md#m116) |
| 117 | SKPK | Either client | Sent first, after the login | ● |  |  |  | [Session](session.md#m117) |
| 118 | (unnamed; licence report) | Client | Request |  | ● |  |  | [Session](session.md#m118) |

### [Connection monitor](monitoring.md), 140 to 143

| Type | Message | Sent by | Role | DCC | Server | Tracker | GConsole | Details |
|---:|---|---|---|:---:|:---:|:---:|:---:|---|
| 140 | Open Connection Monitor | Client | Request |  | ● |  |  | [Monitoring](monitoring.md#m140) |
| 141 | Close Connection Monitor | Client | Request |  | ● |  |  | [Monitoring](monitoring.md#m141) |
| 142 | Get Connection Monitor Info | Client | Request, reply 143 |  | ● |  |  | [Monitoring](monitoring.md#m142) |
| 143 | Connection Monitor Info | Server | Reply to 142 |  | ● |  |  | [Monitoring](monitoring.md#m143) |

### [Administration](administration.md), 150 to 191

| Type | Message | Sent by | Role | DCC | Server | Tracker | GConsole | Details |
|---:|---|---|---|:---:|:---:|:---:|:---:|---|
| 150 | Disconnect User | Client | Request |  | ● |  |  | [Administration](administration.md#m150) |
| 151 | Disconnect Message | Server | Notification to the target of 150, and broadcast by 174 and 175 |  | ● |  |  | [Administration](administration.md#m151) |
| 152 | Get User Info | Client | Request, reply 153 |  | ● |  |  | [Administration](administration.md#m152) |
| 153 | User Info | Server | Reply to 152 |  | ● |  |  | [Administration](administration.md#m153) |
| 154 | Send Broadcast Message | Client | Request |  | ● |  |  | [Administration](administration.md#m154) |
| 155 | Rcv Broadcast Message | Server | Notification: a broadcast from 154 |  | ● |  |  | [Administration](administration.md#m155) |
| 170 | Get Server Settings | Client | Request, reply 171 |  | ● |  |  | [Administration](administration.md#m170) |
| 171 | Server Settings Info | Server | Reply to 170 |  | ● |  |  | [Administration](administration.md#m171) |
| 172 | Set Server Settings | Client | Request, reply 173 |  | ● |  |  | [Administration](administration.md#m172) |
| 173 | Set Server Settings Done | Server | Reply to 172 |  | ● |  |  | [Administration](administration.md#m173) |
| 174 | Shutdown Server | Client | Request |  | ● |  |  | [Administration](administration.md#m174) |
| 175 | Upgrade Server Software | Client | Request |  | ● |  |  | [Administration](administration.md#m175) |
| 180 | Server History Join | Client | Request, reply 181 |  | ● |  |  | [Administration](administration.md#m180) |
| 181 | Server History Join (R) | Server | Reply to 180 |  | ● |  |  | [Administration](administration.md#m181) |
| 182 | Server History Leave | Client | Request |  | ● |  |  | [Administration](administration.md#m182) |
| 183 | Server History Added | Server | Notification while joined with 180 |  | ● |  |  | [Administration](administration.md#m183) |
| 184 to 191 | History record kinds | Server | Kinds of record in 181 and 183 | | ● | | | [Administration](administration.md#m184) |

### [Chat](chat.md), 200 to 223

| Type | Message | Sent by | Role | DCC | Server | Tracker | GConsole | Details |
|---:|---|---|---|:---:|:---:|:---:|:---:|---|
| 200 | Create Private Chat | Client | Request, reply 216 |  | ● |  |  | [Chat](chat.md#m200) |
| 201 | Delete Chat | Client | Request |  | ● |  |  | [Chat](chat.md#m201) |
| 202 | Join Chat | Client | Request, reply 203 |  | ● |  |  | [Chat](chat.md#m202) |
| 203 | Chat Window Info | Server | Reply to 202 |  | ● |  |  | [Chat](chat.md#m203) |
| 204 | Leave Chat | Client | Request |  | ● |  |  | [Chat](chat.md#m204) |
| 205 | Chat User Joined | Server | Notification: a member joined (after 202) |  | ● |  |  | [Chat](chat.md#m205) |
| 206 | Chat User Departed | Server | Notification: a member left (after 204) |  | ● |  |  | [Chat](chat.md#m206) |
| 207 | Send Chat Text | Client | Request |  | ● |  |  | [Chat](chat.md#m207) |
| 208 | Receive Chat Text | Server | Notification: chat text from 207 |  | ● |  |  | [Chat](chat.md#m208) |
| 209 | Set Chat Topic | Client | Request |  | ● |  |  | [Chat](chat.md#m209) |
| 210 | Chat Topic Changed | Server | Notification: a new topic (after 209) |  | ● |  |  | [Chat](chat.md#m210) |
| 211 | Chat Invite | Client | Request |  | ● |  |  | [Chat](chat.md#m211) |
| 212 | Chat Invitation | Server | Notification: an invitation (after 211) |  | ● |  |  | [Chat](chat.md#m212) |
| 213 | Must Join Chat | Server | Notification after login |  | ● |  |  | [Chat](chat.md#m213) |
| 214 | Get Chat List | Client | Request, reply 215 |  | ● |  |  | [Chat](chat.md#m214) |
| 215 | Chat List | Server | Reply to 214 |  | ● |  |  | [Chat](chat.md#m215) |
| 216 | Create Private Chat (R) | Server | Reply to 200 |  | ● |  |  | [Chat](chat.md#m216) |
| 217 | Create Chat | Client | Request |  | ● |  |  | [Chat](chat.md#m217) |
| 218 | Get Chat | Client | Request, reply 219 |  | ● |  |  | [Chat](chat.md#m218) |
| 219 | Chat Info | Server | Reply to 218 |  | ● |  |  | [Chat](chat.md#m219) |
| 220 | Set Chat | Client | Request |  | ● |  |  | [Chat](chat.md#m220) |
| 221 | Chat Deleted | Server | Notification: a chat was deleted (after 201) |  | ● |  |  | [Chat](chat.md#m221) |
| 222 | Get Chat DCC Addresses | Client | Request, reply 223 |  | ● |  |  | [Chat](chat.md#m222) |
| 223 | Chat DCC Addresses | Server | Reply to 222 |  | ● |  |  | [Chat](chat.md#m223) |

### [Private messages](private-messages.md), 300 to 301

| Type | Message | Sent by | Role | DCC | Server | Tracker | GConsole | Details |
|---:|---|---|---|:---:|:---:|:---:|:---:|---|
| 300 | Send Private Message | Client | Request | ● | ● |  |  | [Private messages](private-messages.md#m300) |
| 301 | Incoming Private Message | Server | Notification: a message from 300 |  | ● |  |  | [Private messages](private-messages.md#m301) |

### [Files, transfers, volumes and search](files.md), 400 to 453

| Type | Message | Sent by | Role | DCC | Server | Tracker | GConsole | Details |
|---:|---|---|---|:---:|:---:|:---:|:---:|---|
| 400 | Get File List | Client | Request, reply 401 |  | ● |  |  | [Files](files.md#m400) |
| 401 | File List | Server | Reply to 400 |  | ● |  |  | [Files](files.md#m401) |
| 402 | Create Folder | Client | Request |  | ● |  |  | [Files](files.md#m402) |
| 403 | Create Alias | Client | Request |  | ● |  |  | [Files](files.md#m403) |
| 404 | Delete File/Folder | Client | Request |  | ● |  |  | [Files](files.md#m404) |
| 405 | Move To Trash | Client | Request |  | ● |  |  | [Files](files.md#m405) |
| 406 | Rename File/Folder | Client | Request |  | ● |  |  | [Files](files.md#m406) |
| 407 | Move File/Folder | Client | Request |  | ● |  |  | [Files](files.md#m407) |
| 408 | Copy File/Folder | Client | Request |  | ● |  |  | [Files](files.md#m408) |
| 409 | Get File/Folder Info | Client | Request, reply 410, 422 |  | ● |  |  | [Files](files.md#m409) |
| 410 | File/Folder Info | Server | Reply to 409 |  | ● |  |  | [Files](files.md#m410) |
| 411 | Set File/Folder Info | Client | Request |  | ● |  |  | [Files](files.md#m411) |
| 412 | Download File | Client | Request, reply 413 |  | ● |  |  | [Files](files.md#m412) |
| 413 | Download File (R) | Server | Reply to 412 |  | ● |  |  | [Files](files.md#m413) |
| 414 | Download File (D) | Server | Data for 412, one message per fork |  | ● |  |  | [Files](files.md#m414) |
| 415 | Upload File | Client | Request, reply 416 |  | ● |  |  | [Files](files.md#m415) |
| 416 | Upload File (R) | Server | Reply to 415 |  | ● |  |  | [Files](files.md#m416) |
| 417 | Upload File (D) | Client | Data for 415, sent by the client |  | ● |  |  | [Files](files.md#m417) |
| 418 | Upload File (C) | Server | Completes 415 |  | ● |  |  | [Files](files.md#m418) |
| 419 | Notify Speed Limit | Server | Notification |  | ● |  |  | [Files](files.md#m419) |
| 420 | Launch File | Client | Request |  | ● |  |  | [Files](files.md#m420) |
| 421 | Empty Trash | Client | Request |  | ● |  |  | [Files](files.md#m421) |
| 422 | Volume Info | Server | Reply to 409 |  | ● |  |  | [Files](files.md#m422) |
| 423 | Set Volume Info | Client | Request |  | ● |  |  | [Files](files.md#m423) |
| 450 | Generate Catalog | Client | Request |  | ● |  |  | [Files](files.md#m450) |
| 451 | Generate Catalog (S) | Server | Progress of 450 |  | ● |  |  | [Files](files.md#m451) |
| 452 | File Search | Client | Request, reply 453 |  | ● |  |  | [Files](files.md#m452) |
| 453 | File Search (R) | Server | Reply to 452 |  | ● |  |  | [Files](files.md#m453) |

### [Accounts](accounts.md), 502 to 551

| Type | Message | Sent by | Role | DCC | Server | Tracker | GConsole | Details |
|---:|---|---|---|:---:|:---:|:---:|:---:|---|
| 502 | Get Account Class | Client | Request, reply 503 |  | ● |  |  | [Accounts](accounts.md#m502) |
| 503 | Account Class Info | Server | Reply to 502 |  | ● |  |  | [Accounts](accounts.md#m503) |
| 504 | Set Account Class | Client | Request |  | ● |  |  | [Accounts](accounts.md#m504) |
| 505 | Get Account Class List | Client | Request, reply 506 |  | ● |  |  | [Accounts](accounts.md#m505) |
| 506 | Account Class List | Server | Reply to 505 |  | ● |  |  | [Accounts](accounts.md#m506) |
| 507 | Create Account | Client | Request |  | ● |  |  | [Accounts](accounts.md#m507) |
| 508 | Delete Account | Client | Request |  | ● |  |  | [Accounts](accounts.md#m508) |
| 509 | Get Account | Client | Request, reply 510 |  | ● |  |  | [Accounts](accounts.md#m509) |
| 510 | Account Info | Server | Reply to 509 |  | ● |  |  | [Accounts](accounts.md#m510) |
| 511 | Set Account | Client | Request |  | ● |  |  | [Accounts](accounts.md#m511) |
| 512 | Get Account List | Client | Request, reply 513 |  | ● |  |  | [Accounts](accounts.md#m512) |
| 513 | Account List | Server | Reply to 512 |  | ● |  |  | [Accounts](accounts.md#m513) |
| 514 | Account Changed | Server | Notification, never sent by Server 1.620 |  | ● |  |  | [Accounts](accounts.md#m514) |
| 515 | Account Deleted | Server | Notification after 508 |  | ● |  |  | [Accounts](accounts.md#m515) |
| 550 | Change Own Password | Client | Request, reply 551 |  | ● |  |  | [Accounts](accounts.md#m550) |
| 551 | Change Own Password Done | Server | Reply to 550 |  | ● |  |  | [Accounts](accounts.md#m551) |

### [News](news.md), 600 to 614

| Type | Message | Sent by | Role | DCC | Server | Tracker | GConsole | Details |
|---:|---|---|---|:---:|:---:|:---:|:---:|---|
| 600 | Create Newsgroup | Client | Request |  | ● |  |  | [News](news.md#m600) |
| 601 | Delete Newsgroup | Client | Request |  | ● |  |  | [News](news.md#m601) |
| 602 | Get Newsgroup | Client | Request, reply 603 |  | ● |  |  | [News](news.md#m602) |
| 603 | Newsgroup Info | Server | Reply to 602 |  | ● |  |  | [News](news.md#m603) |
| 604 | Set Newsgroup | Client | Request |  | ● |  |  | [News](news.md#m604) |
| 605 | Get Newsgroup List | Client | Request, reply 606 |  | ● |  |  | [News](news.md#m605) |
| 606 | Newsgroup List | Server | Reply to 605 |  | ● |  |  | [News](news.md#m606) |
| 607 | Get News Messages | Client | Request, reply 608 |  | ● |  |  | [News](news.md#m607) |
| 608 | News Messages | Server | Reply to 607 |  | ● |  |  | [News](news.md#m608) |
| 609 | Get News Message | Client | Request, reply 610 |  | ● |  |  | [News](news.md#m609) |
| 610 | News Message | Server | Reply to 609 |  | ● |  |  | [News](news.md#m610) |
| 611 | Post News Message | Client | Request |  | ● |  |  | [News](news.md#m611) |
| 612 | Delete News Message | Client | Request |  | ● |  |  | [News](news.md#m612) |
| 613 | News Message Posted | Server | Notification after 611 |  | ● |  |  | [News](news.md#m613) |
| 614 | News Message Deleted | Server | Notification after 612 |  | ● |  |  | [News](news.md#m614) |

### [Tracker](tracker.md), 700 to 721

| Type | Message | Sent by | Role | DCC | Server | Tracker | GConsole | Details |
|---:|---|---|---|:---:|:---:|:---:|:---:|---|
| 700 | Get Tracker Group List | Client | Request, reply 701 |  |  | ● |  | [Tracker](tracker.md#m700) |
| 701 | Tracker Group List | Tracker | Reply to 700 |  |  | ● |  | [Tracker](tracker.md#m701) |
| 702 | Get Tracker Server List | Client | Request, reply 703 |  |  | ● |  | [Tracker](tracker.md#m702) |
| 703 | Tracker Server List | Tracker | Reply to 702 |  |  | ● |  | [Tracker](tracker.md#m703) |
| 710 | Get TrackerServ Settings | Client | Request, reply 711 |  |  | ● |  | [Tracker](tracker.md#m710) |
| 711 | TrackerServ Settings Info | Tracker | Reply to 710 |  |  | ● |  | [Tracker](tracker.md#m711) |
| 712 | Set TrackerServ Settings | Client | Request, reply 713 |  |  | ● |  | [Tracker](tracker.md#m712) |
| 713 | Set TrackerServ Settings Done | Tracker | Reply to 712 |  |  | ● |  | [Tracker](tracker.md#m713) |
| 720 | Get TrackerServ Statistics | Client | Request, reply 721 |  |  | ● |  | [Tracker](tracker.md#m720) |
| 721 | TrackerServ Statistics | Tracker | Reply to 720 |  |  | ● |  | [Tracker](tracker.md#m721) |

### [Processes](processes.md), 800 to 804

| Type | Message | Sent by | Role | DCC | Server | Tracker | GConsole | Details |
|---:|---|---|---|:---:|:---:|:---:|:---:|---|
| 800 | Open Process Monitor | Client | Request |  | ● |  |  | [Processes](processes.md#m800) |
| 801 | Close Process Monitor | Client | Request |  | ● |  |  | [Processes](processes.md#m801) |
| 802 | Get Process List | Client | Request, reply 803 |  | ● |  |  | [Processes](processes.md#m802) |
| 803 | Process List | Server | Reply to 802 |  | ● |  |  | [Processes](processes.md#m803) |
| 804 | Exit/Terminate Process | Client | Request |  | ● |  |  | [Processes](processes.md#m804) |

### [View display](view-display.md), 900 to 905

| Type | Message | Sent by | Role | DCC | Server | Tracker | GConsole | Details |
|---:|---|---|---|:---:|:---:|:---:|:---:|---|
| 900 | Open View Display | Client | Request, reply 902 |  | ● |  |  | [View display](view-display.md#m900) |
| 901 | Close View Display | Client | Request |  | ● |  |  | [View display](view-display.md#m901) |
| 902 | View Display Params | Server | Reply to 900 |  | ● |  |  | [View display](view-display.md#m902) |
| 903 | View Display Update | Server | Screen updates after 902 |  | ● |  |  | [View display](view-display.md#m903) |
| 904 | View Display Mouse | Client | Request |  | ● |  |  | [View display](view-display.md#m904) |
| 905 | View Display Keyboard | Client | Request |  | ● |  |  | [View display](view-display.md#m905) |

### [Voice](voice.md), 1000 to 1003

| Type | Message | Sent by | Role | DCC | Server | Tracker | GConsole | Details |
|---:|---|---|---|:---:|:---:|:---:|:---:|---|
| 1000 | Start Voice | Caller | Request, reply 1001 | ● |  |  |  | [Voice](voice.md#m1000) |
| 1001 | Start Voice (R) | Callee | Reply to 1000 | ● |  |  |  | [Voice](voice.md#m1001) |
| 1002 | Set Voice Format | Either client | Sent by either side | ● |  |  |  | [Voice](voice.md#m1002) |
| 1003 | Voice Data | Either client | Voice data, sent by either side | ● |  |  |  | [Voice](voice.md#m1003) |

### [Folder access](folder-access.md), 1100 to 1107

| Type | Message | Sent by | Role | DCC | Server | Tracker | GConsole | Details |
|---:|---|---|---|:---:|:---:|:---:|:---:|---|
| 1100 | Create Folder Access | Client | Request |  | ● |  |  | [Folder access](folder-access.md#m1100) |
| 1101 | Delete Folder Access | Client | Request |  | ● |  |  | [Folder access](folder-access.md#m1101) |
| 1102 | Folder Access Deleted | Server | Notification after 1101 |  | ● |  |  | [Folder access](folder-access.md#m1102) |
| 1103 | Get Folder Access | Client | Request, reply 1104 |  | ● |  |  | [Folder access](folder-access.md#m1103) |
| 1104 | Folder Access Info | Server | Reply to 1103 |  | ● |  |  | [Folder access](folder-access.md#m1104) |
| 1105 | Set Folder Access | Client | Request |  | ● |  |  | [Folder access](folder-access.md#m1105) |
| 1106 | Get Folder Access List | Client | Request, reply 1107 |  | ● |  |  | [Folder access](folder-access.md#m1106) |
| 1107 | Folder Access List | Server | Reply to 1106 |  | ● |  |  | [Folder access](folder-access.md#m1107) |

### [GConsole](gconsole.md), 4000 to 4201

| Type | Message | Sent by | Role | DCC | Server | Tracker | GConsole | Details |
|---:|---|---|---|:---:|:---:|:---:|:---:|---|
| 4000 | Open Window | GConsole | Opens a window |  |  |  | ● | [GConsole](gconsole.md#m4000) |
| 4001 | Draw | GConsole | Draws in the window |  |  |  | ● | [GConsole](gconsole.md#m4001) |
| 4200 | Key | Client | A key event |  |  |  | ● | [GConsole](gconsole.md#m4200) |
| 4201 | Mouse | Client | A mouse event |  |  |  | ● | [GConsole](gconsole.md#m4201) |
