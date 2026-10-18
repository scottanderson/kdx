# Messages

For hex and struct tables, see [Conventions](../README.md#conventions). Field
names are the `kdx-proto` field names. Other bytes have the [special field
names](../README.md#special-field-names).

This page lists all message types in order, grouped by the page that describes
them. Each page covers one range of types, so the type number alone tells you
the page.

A KDX connection is one of four kinds, set by the sub-version of the sender's
hello; see [handshake.md](../handshake.md#connection-types). The kinds are
client to client (DCC), client to server, client to tracker, and client to
GConsole. Each kind has a column. A cell names the sender of the message on that
kind of connection: Client, Server, Tracker, or GConsole. On a DCC connection,
Caller and Callee are the two ends of a voice call, and Either means that both
sides send it. A blank cell means that the kind has no handler for the message,
or that no evidence shows one. <!-- [S] --> Most messages belong to the server.
A server tells a tracker that it exists with a UDP datagram, which has no
message type; see [udp.md](../udp.md).

Each message has a stable link, made of its page and its type number:
[chat.md#m200](chat.md#m200) is [200 Create Private Chat](chat.md#m200). The
Message column links to it. [The protocol overview](../README.md#messages) gives
the header and the rules for requests and replies, and [Message
types](../README.md#message-types) tells where the names come from.

## How to read the tables

- **Request:** a message that asks for something. The Role column names the
  reply, if there is one.
- **Reply:** the answer to a request. It has the request's transaction ID, and
  the reply bit is set.
- **Notification:** a message that the receiver did not ask for. The Role column
  says if it follows some other message.
- **Data:** the bytes of a transfer or a stream, sent after a request.
- A `↳` comes before the name of a reply, and the Role column links the request
  and the reply to each other.

## Messages by page

### [Status](status.md), 1

| Type | Message | Role | Client↔Client (DCC) | Client↔Server | Client↔Tracker | Client↔GConsole |
|---:|---|---|:---:|:---:|:---:|:---:|
| 1 | [Status](status.md#m1) | Reply that carries a result code | Either | Server | Tracker |  |

### [Session](session.md), 100 to 118

| Type | Message | Role | Client↔Client (DCC) | Client↔Server | Client↔Tracker | Client↔GConsole |
|---:|---|---|:---:|:---:|:---:|:---:|
| 100 | [Set Identity](session.md#m100) | Request | Either | Client |  |  |
| 101 | [User List](session.md#m101) | Notification after login |  | Server |  |  |
| 102 | [User Changed](session.md#m102) | Notification: a user changed their identity |  | Server |  |  |
| 103 | [User Joined](session.md#m103) | Notification: a user logged in |  | Server |  |  |
| 104 | [User Departed](session.md#m104) | Notification: a user left |  | Server |  |  |
| 105 | [Server Info](session.md#m105) | Notification after login |  | Server |  |  |
| 106 | [Server Icon](session.md#m106) | Notification after login |  | Server |  |  |
| 107 | [Server Greeting](session.md#m107) | Notification after login |  | Server | Tracker |  |
| 108 | [Ping](session.md#m108) | Request, reply [109 Pong](session.md#m109) |  | Client |  |  |
| 109 | ↳ [Pong](session.md#m109) | Reply to [108 Ping](session.md#m108) |  | Server |  |  |
| 110 | [Set Mini User Icon](session.md#m110) | Request |  | Client |  |  |
| 111 | [Mini User Icon Changed](session.md#m111) | Notification: a user changed their mini icon |  | Server |  |  |
| 112 | [(unnamed; requests the greeting)](session.md#m112) | Request |  | Client |  |  |
| 113 | [Get Extra Server Info](session.md#m113) | Request, reply [114 Extra Server Info](session.md#m114) |  | Client |  |  |
| 114 | ↳ [Extra Server Info](session.md#m114) | Reply to [113 Get Extra Server Info](session.md#m113) |  | Server |  |  |
| 115 | [Get My Address](session.md#m115) | Request, reply [116 Your Address](session.md#m116) |  | Client | Client |  |
| 116 | ↳ [Your Address](session.md#m116) | Reply to [115 Get My Address](session.md#m115) |  | Server | Tracker |  |
| 117 | [SKPK](session.md#m117) | Sent first, after the login | Either |  |  |  |
| 118 | [(unnamed; license report)](session.md#m118) | Request |  | Client |  |  |

### [Connection monitor](monitoring.md), 140 to 143

| Type | Message | Role | Client↔Client (DCC) | Client↔Server | Client↔Tracker | Client↔GConsole |
|---:|---|---|:---:|:---:|:---:|:---:|
| 140 | [Open Connection Monitor](monitoring.md#m140) | Request |  | Client |  |  |
| 141 | [Close Connection Monitor](monitoring.md#m141) | Request |  | Client |  |  |
| 142 | [Get Connection Monitor Info](monitoring.md#m142) | Request, reply [143 Connection Monitor Info](monitoring.md#m143) |  | Client |  |  |
| 143 | ↳ [Connection Monitor Info](monitoring.md#m143) | Reply to [142 Get Connection Monitor Info](monitoring.md#m142) |  | Server |  |  |

### [Administration](administration.md), 150 to 191

| Type | Message | Role | Client↔Client (DCC) | Client↔Server | Client↔Tracker | Client↔GConsole |
|---:|---|---|:---:|:---:|:---:|:---:|
| 150 | [Disconnect User](administration.md#m150) | Request |  | Client |  |  |
| 151 | [Disconnect Message](administration.md#m151) | Notification to the target of [150 Disconnect User](administration.md#m150), and broadcast by [174 Shutdown Server](administration.md#m174) and [175 Upgrade Server Software](administration.md#m175) |  | Server |  |  |
| 152 | [Get User Info](administration.md#m152) | Request, reply [153 User Info](administration.md#m153) |  | Client |  |  |
| 153 | ↳ [User Info](administration.md#m153) | Reply to [152 Get User Info](administration.md#m152) |  | Server |  |  |
| 154 | [Send Broadcast Message](administration.md#m154) | Request |  | Client |  |  |
| 155 | [Rcv Broadcast Message](administration.md#m155) | Notification: a broadcast from [154 Send Broadcast Message](administration.md#m154) |  | Server |  |  |
| 170 | [Get Server Settings](administration.md#m170) | Request, reply [171 Server Settings Info](administration.md#m171) |  | Client |  |  |
| 171 | ↳ [Server Settings Info](administration.md#m171) | Reply to [170 Get Server Settings](administration.md#m170) |  | Server |  |  |
| 172 | [Set Server Settings](administration.md#m172) | Request, reply [173 Set Server Settings Done](administration.md#m173) |  | Client |  |  |
| 173 | ↳ [Set Server Settings Done](administration.md#m173) | Reply to [172 Set Server Settings](administration.md#m172) |  | Server |  |  |
| 174 | [Shutdown Server](administration.md#m174) | Request |  | Client |  |  |
| 175 | [Upgrade Server Software](administration.md#m175) | Request |  | Client |  |  |
| 180 | [Server History Join](administration.md#m180) | Request, reply [181 Server History Join (R)](administration.md#m181) |  | Client |  |  |
| 181 | ↳ [Server History Join (R)](administration.md#m181) | Reply to [180 Server History Join](administration.md#m180) |  | Server |  |  |
| 182 | [Server History Leave](administration.md#m182) | Request |  | Client |  |  |
| 183 | [Server History Added](administration.md#m183) | Notification while joined with [180 Server History Join](administration.md#m180) |  | Server |  |  |
| 184 to 191 | History record kinds | Server | Kinds of record in 181 and 183 | | ● | | | [Administration](administration.md#m184) |

### [Chat](chat.md), 200 to 223

| Type | Message | Role | Client↔Client (DCC) | Client↔Server | Client↔Tracker | Client↔GConsole |
|---:|---|---|:---:|:---:|:---:|:---:|
| 200 | [Create Private Chat](chat.md#m200) | Request, reply [216 Create Private Chat (R)](chat.md#m216) |  | Client |  |  |
| 201 | [Delete Chat](chat.md#m201) | Request |  | Client |  |  |
| 202 | [Join Chat](chat.md#m202) | Request, reply [203 Chat Window Info](chat.md#m203) |  | Client |  |  |
| 203 | ↳ [Chat Window Info](chat.md#m203) | Reply to [202 Join Chat](chat.md#m202) |  | Server |  |  |
| 204 | [Leave Chat](chat.md#m204) | Request |  | Client |  |  |
| 205 | [Chat User Joined](chat.md#m205) | Notification: a member joined (after [202 Join Chat](chat.md#m202)) |  | Server |  |  |
| 206 | [Chat User Departed](chat.md#m206) | Notification: a member left (after [204 Leave Chat](chat.md#m204)) |  | Server |  |  |
| 207 | [Send Chat Text](chat.md#m207) | Request |  | Client |  |  |
| 208 | [Receive Chat Text](chat.md#m208) | Notification: chat text from [207 Send Chat Text](chat.md#m207) |  | Server |  |  |
| 209 | [Set Chat Topic](chat.md#m209) | Request |  | Client |  |  |
| 210 | [Chat Topic Changed](chat.md#m210) | Notification: a new topic (after [209 Set Chat Topic](chat.md#m209)) |  | Server |  |  |
| 211 | [Chat Invite](chat.md#m211) | Request |  | Client |  |  |
| 212 | [Chat Invitation](chat.md#m212) | Notification: an invitation (after [211 Chat Invite](chat.md#m211)) |  | Server |  |  |
| 213 | [Must Join Chat](chat.md#m213) | Notification after login |  | Server |  |  |
| 214 | [Get Chat List](chat.md#m214) | Request, reply [215 Chat List](chat.md#m215) |  | Client |  |  |
| 215 | ↳ [Chat List](chat.md#m215) | Reply to [214 Get Chat List](chat.md#m214) |  | Server |  |  |
| 216 | ↳ [Create Private Chat (R)](chat.md#m216) | Reply to [200 Create Private Chat](chat.md#m200) |  | Server |  |  |
| 217 | [Create Chat](chat.md#m217) | Request |  | Client |  |  |
| 218 | [Get Chat](chat.md#m218) | Request, reply [219 Chat Info](chat.md#m219) |  | Client |  |  |
| 219 | ↳ [Chat Info](chat.md#m219) | Reply to [218 Get Chat](chat.md#m218) |  | Server |  |  |
| 220 | [Set Chat](chat.md#m220) | Request |  | Client |  |  |
| 221 | [Chat Deleted](chat.md#m221) | Notification: a chat was deleted (after [201 Delete Chat](chat.md#m201)) |  | Server |  |  |
| 222 | [Get Chat DCC Addresses](chat.md#m222) | Request, reply [223 Chat DCC Addresses](chat.md#m223) |  | Client |  |  |
| 223 | ↳ [Chat DCC Addresses](chat.md#m223) | Reply to [222 Get Chat DCC Addresses](chat.md#m222) |  | Server |  |  |

### [Private messages](private-messages.md), 300 to 301

| Type | Message | Role | Client↔Client (DCC) | Client↔Server | Client↔Tracker | Client↔GConsole |
|---:|---|---|:---:|:---:|:---:|:---:|
| 300 | [Send Private Message](private-messages.md#m300) | Request | Either | Client |  |  |
| 301 | [Incoming Private Message](private-messages.md#m301) | Notification: a message from [300 Send Private Message](private-messages.md#m300) |  | Server |  |  |

### [Files, transfers, volumes and search](files.md), 400 to 453

| Type | Message | Role | Client↔Client (DCC) | Client↔Server | Client↔Tracker | Client↔GConsole |
|---:|---|---|:---:|:---:|:---:|:---:|
| 400 | [Get File List](files.md#m400) | Request, reply [401 File List](files.md#m401) |  | Client |  |  |
| 401 | ↳ [File List](files.md#m401) | Reply to [400 Get File List](files.md#m400) |  | Server |  |  |
| 402 | [Create Folder](files.md#m402) | Request |  | Client |  |  |
| 403 | [Create Alias](files.md#m403) | Request |  | Client |  |  |
| 404 | [Delete File/Folder](files.md#m404) | Request |  | Client |  |  |
| 405 | [Move To Trash](files.md#m405) | Request |  | Client |  |  |
| 406 | [Rename File/Folder](files.md#m406) | Request |  | Client |  |  |
| 407 | [Move File/Folder](files.md#m407) | Request |  | Client |  |  |
| 408 | [Copy File/Folder](files.md#m408) | Request |  | Client |  |  |
| 409 | [Get File/Folder Info](files.md#m409) | Request, reply [410 File/Folder Info](files.md#m410), [422 Volume Info](files.md#m422) |  | Client |  |  |
| 410 | ↳ [File/Folder Info](files.md#m410) | Reply to [409 Get File/Folder Info](files.md#m409) |  | Server |  |  |
| 411 | [Set File/Folder Info](files.md#m411) | Request |  | Client |  |  |
| 412 | [Download File](files.md#m412) | Request, reply [413 Download File (R)](files.md#m413) |  | Client |  |  |
| 413 | ↳ [Download File (R)](files.md#m413) | Reply to [412 Download File](files.md#m412) |  | Server |  |  |
| 414 | [Download File (D)](files.md#m414) | Data for [412 Download File](files.md#m412), one message per fork |  | Server |  |  |
| 415 | [Upload File](files.md#m415) | Request, reply [416 Upload File (R)](files.md#m416) |  | Client |  |  |
| 416 | ↳ [Upload File (R)](files.md#m416) | Reply to [415 Upload File](files.md#m415) |  | Server |  |  |
| 417 | [Upload File (D)](files.md#m417) | Data for [415 Upload File](files.md#m415), sent by the client |  | Client |  |  |
| 418 | [Upload File (C)](files.md#m418) | Completes [415 Upload File](files.md#m415) |  | Server |  |  |
| 419 | [Notify Speed Limit](files.md#m419) | Notification |  | Server |  |  |
| 420 | [Launch File](files.md#m420) | Request |  | Client |  |  |
| 421 | [Empty Trash](files.md#m421) | Request |  | Client |  |  |
| 422 | ↳ [Volume Info](files.md#m422) | Reply to [409 Get File/Folder Info](files.md#m409) |  | Server |  |  |
| 423 | [Set Volume Info](files.md#m423) | Request |  | Client |  |  |
| 450 | [Generate Catalog](files.md#m450) | Request |  | Client |  |  |
| 451 | [Generate Catalog (S)](files.md#m451) | Progress of [450 Generate Catalog](files.md#m450) |  | Server |  |  |
| 452 | [File Search](files.md#m452) | Request, reply [453 File Search (R)](files.md#m453) |  | Client |  |  |
| 453 | ↳ [File Search (R)](files.md#m453) | Reply to [452 File Search](files.md#m452) |  | Server |  |  |

### [Accounts](accounts.md), 502 to 551

| Type | Message | Role | Client↔Client (DCC) | Client↔Server | Client↔Tracker | Client↔GConsole |
|---:|---|---|:---:|:---:|:---:|:---:|
| 502 | [Get Account Class](accounts.md#m502) | Request, reply [503 Account Class Info](accounts.md#m503) |  | Client |  |  |
| 503 | ↳ [Account Class Info](accounts.md#m503) | Reply to [502 Get Account Class](accounts.md#m502) |  | Server |  |  |
| 504 | [Set Account Class](accounts.md#m504) | Request |  | Client |  |  |
| 505 | [Get Account Class List](accounts.md#m505) | Request, reply [506 Account Class List](accounts.md#m506) |  | Client |  |  |
| 506 | ↳ [Account Class List](accounts.md#m506) | Reply to [505 Get Account Class List](accounts.md#m505) |  | Server |  |  |
| 507 | [Create Account](accounts.md#m507) | Request |  | Client |  |  |
| 508 | [Delete Account](accounts.md#m508) | Request |  | Client |  |  |
| 509 | [Get Account](accounts.md#m509) | Request, reply [510 Account Info](accounts.md#m510) |  | Client |  |  |
| 510 | ↳ [Account Info](accounts.md#m510) | Reply to [509 Get Account](accounts.md#m509) |  | Server |  |  |
| 511 | [Set Account](accounts.md#m511) | Request |  | Client |  |  |
| 512 | [Get Account List](accounts.md#m512) | Request, reply [513 Account List](accounts.md#m513) |  | Client |  |  |
| 513 | ↳ [Account List](accounts.md#m513) | Reply to [512 Get Account List](accounts.md#m512) |  | Server |  |  |
| 514 | [Account Changed](accounts.md#m514) | Notification, never sent by Server 1.620 |  | Server |  |  |
| 515 | [Account Deleted](accounts.md#m515) | Notification after [508 Delete Account](accounts.md#m508) |  | Server |  |  |
| 550 | [Change Own Password](accounts.md#m550) | Request, reply [551 Change Own Password Done](accounts.md#m551) |  | Client |  |  |
| 551 | ↳ [Change Own Password Done](accounts.md#m551) | Reply to [550 Change Own Password](accounts.md#m550) |  | Server |  |  |

### [News](news.md), 600 to 614

| Type | Message | Role | Client↔Client (DCC) | Client↔Server | Client↔Tracker | Client↔GConsole |
|---:|---|---|:---:|:---:|:---:|:---:|
| 600 | [Create Newsgroup](news.md#m600) | Request |  | Client |  |  |
| 601 | [Delete Newsgroup](news.md#m601) | Request |  | Client |  |  |
| 602 | [Get Newsgroup](news.md#m602) | Request, reply [603 Newsgroup Info](news.md#m603) |  | Client |  |  |
| 603 | ↳ [Newsgroup Info](news.md#m603) | Reply to [602 Get Newsgroup](news.md#m602) |  | Server |  |  |
| 604 | [Set Newsgroup](news.md#m604) | Request |  | Client |  |  |
| 605 | [Get Newsgroup List](news.md#m605) | Request, reply [606 Newsgroup List](news.md#m606) |  | Client |  |  |
| 606 | ↳ [Newsgroup List](news.md#m606) | Reply to [605 Get Newsgroup List](news.md#m605) |  | Server |  |  |
| 607 | [Get News Messages](news.md#m607) | Request, reply [608 News Messages](news.md#m608) |  | Client |  |  |
| 608 | ↳ [News Messages](news.md#m608) | Reply to [607 Get News Messages](news.md#m607) |  | Server |  |  |
| 609 | [Get News Message](news.md#m609) | Request, reply [610 News Message](news.md#m610) |  | Client |  |  |
| 610 | ↳ [News Message](news.md#m610) | Reply to [609 Get News Message](news.md#m609) |  | Server |  |  |
| 611 | [Post News Message](news.md#m611) | Request |  | Client |  |  |
| 612 | [Delete News Message](news.md#m612) | Request |  | Client |  |  |
| 613 | [News Message Posted](news.md#m613) | Notification after [611 Post News Message](news.md#m611) |  | Server |  |  |
| 614 | [News Message Deleted](news.md#m614) | Notification after [612 Delete News Message](news.md#m612) |  | Server |  |  |

### [Tracker](tracker.md), 700 to 721

| Type | Message | Role | Client↔Client (DCC) | Client↔Server | Client↔Tracker | Client↔GConsole |
|---:|---|---|:---:|:---:|:---:|:---:|
| 700 | [Get Tracker Group List](tracker.md#m700) | Request, reply [701 Tracker Group List](tracker.md#m701) |  |  | Client |  |
| 701 | ↳ [Tracker Group List](tracker.md#m701) | Reply to [700 Get Tracker Group List](tracker.md#m700) |  |  | Tracker |  |
| 702 | [Get Tracker Server List](tracker.md#m702) | Request, reply [703 Tracker Server List](tracker.md#m703) |  |  | Client |  |
| 703 | ↳ [Tracker Server List](tracker.md#m703) | Reply to [702 Get Tracker Server List](tracker.md#m702) |  |  | Tracker |  |
| 710 | [Get TrackerServ Settings](tracker.md#m710) | Request, reply [711 TrackerServ Settings Info](tracker.md#m711) |  |  | Client |  |
| 711 | ↳ [TrackerServ Settings Info](tracker.md#m711) | Reply to [710 Get TrackerServ Settings](tracker.md#m710) |  |  | Tracker |  |
| 712 | [Set TrackerServ Settings](tracker.md#m712) | Request, reply [713 Set TrackerServ Settings Done](tracker.md#m713) |  |  | Client |  |
| 713 | ↳ [Set TrackerServ Settings Done](tracker.md#m713) | Reply to [712 Set TrackerServ Settings](tracker.md#m712) |  |  | Tracker |  |
| 720 | [Get TrackerServ Statistics](tracker.md#m720) | Request, reply [721 TrackerServ Statistics](tracker.md#m721) |  |  | Client |  |
| 721 | ↳ [TrackerServ Statistics](tracker.md#m721) | Reply to [720 Get TrackerServ Statistics](tracker.md#m720) |  |  | Tracker |  |

### [Processes](processes.md), 800 to 804

| Type | Message | Role | Client↔Client (DCC) | Client↔Server | Client↔Tracker | Client↔GConsole |
|---:|---|---|:---:|:---:|:---:|:---:|
| 800 | [Open Process Monitor](processes.md#m800) | Request |  | Client |  |  |
| 801 | [Close Process Monitor](processes.md#m801) | Request |  | Client |  |  |
| 802 | [Get Process List](processes.md#m802) | Request, reply [803 Process List](processes.md#m803) |  | Client |  |  |
| 803 | ↳ [Process List](processes.md#m803) | Reply to [802 Get Process List](processes.md#m802) |  | Server |  |  |
| 804 | [Exit/Terminate Process](processes.md#m804) | Request |  | Client |  |  |

### [View display](view-display.md), 900 to 905

| Type | Message | Role | Client↔Client (DCC) | Client↔Server | Client↔Tracker | Client↔GConsole |
|---:|---|---|:---:|:---:|:---:|:---:|
| 900 | [Open View Display](view-display.md#m900) | Request, reply [902 View Display Params](view-display.md#m902) |  | Client |  |  |
| 901 | [Close View Display](view-display.md#m901) | Request |  | Client |  |  |
| 902 | ↳ [View Display Params](view-display.md#m902) | Reply to [900 Open View Display](view-display.md#m900) |  | Server |  |  |
| 903 | [View Display Update](view-display.md#m903) | Screen updates after [902 View Display Params](view-display.md#m902) |  | Server |  |  |
| 904 | [View Display Mouse](view-display.md#m904) | Request |  | Client |  |  |
| 905 | [View Display Keyboard](view-display.md#m905) | Request |  | Client |  |  |

### [Voice](voice.md), 1000 to 1003

| Type | Message | Role | Client↔Client (DCC) | Client↔Server | Client↔Tracker | Client↔GConsole |
|---:|---|---|:---:|:---:|:---:|:---:|
| 1000 | [Start Voice](voice.md#m1000) | Request, reply [1001 Start Voice (R)](voice.md#m1001) | Caller |  |  |  |
| 1001 | ↳ [Start Voice (R)](voice.md#m1001) | Reply to [1000 Start Voice](voice.md#m1000) | Callee |  |  |  |
| 1002 | [Set Voice Format](voice.md#m1002) | Sent by either side | Either |  |  |  |
| 1003 | [Voice Data](voice.md#m1003) | Voice data, sent by either side | Either |  |  |  |

### [Folder access](folder-access.md), 1100 to 1107

| Type | Message | Role | Client↔Client (DCC) | Client↔Server | Client↔Tracker | Client↔GConsole |
|---:|---|---|:---:|:---:|:---:|:---:|
| 1100 | [Create Folder Access](folder-access.md#m1100) | Request |  | Client |  |  |
| 1101 | [Delete Folder Access](folder-access.md#m1101) | Request |  | Client |  |  |
| 1102 | [Folder Access Deleted](folder-access.md#m1102) | Notification after [1101 Delete Folder Access](folder-access.md#m1101) |  | Server |  |  |
| 1103 | [Get Folder Access](folder-access.md#m1103) | Request, reply [1104 Folder Access Info](folder-access.md#m1104) |  | Client |  |  |
| 1104 | ↳ [Folder Access Info](folder-access.md#m1104) | Reply to [1103 Get Folder Access](folder-access.md#m1103) |  | Server |  |  |
| 1105 | [Set Folder Access](folder-access.md#m1105) | Request |  | Client |  |  |
| 1106 | [Get Folder Access List](folder-access.md#m1106) | Request, reply [1107 Folder Access List](folder-access.md#m1107) |  | Client |  |  |
| 1107 | ↳ [Folder Access List](folder-access.md#m1107) | Reply to [1106 Get Folder Access List](folder-access.md#m1106) |  | Server |  |  |

### [GConsole](gconsole.md), 4000 to 4201

| Type | Message | Role | Client↔Client (DCC) | Client↔Server | Client↔Tracker | Client↔GConsole |
|---:|---|---|:---:|:---:|:---:|:---:|
| 4000 | [Open Window](gconsole.md#m4000) | Opens a window |  |  |  | GConsole |
| 4001 | [Draw](gconsole.md#m4001) | Draws in the window |  |  |  | GConsole |
| 4200 | [Key](gconsole.md#m4200) | A key event |  |  |  | Client |
| 4201 | [Mouse](gconsole.md#m4201) | A mouse event |  |  |  | Client |
