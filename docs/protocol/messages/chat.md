# Chat and private messages

Public and private chat rooms, invitations, voice flags, and private messages.

**200 Create Private Chat** → **216 Create Private Chat (R)**
Initiator: Client. Privilege: Create Private Chat (34). **[S
`FUN_08058420`; O `captures/linux-chat-admin.pcapng` frames 84-85]**

A user can have at most 10 private chats. The request is empty. The reply
has the 203 Chat Window Info layout, with the creator as the only member
**[O]**:

| Off | Size | Field |
|---|---|---|
| 0x08 | 4 | chat ID |
| 0x0c | 4 | key |
| 0x14 | 2 | flags; 0 at 85 |
| 0x16 | 2 | member count |
| 0x20 | 1 | name length |
| 0x24 | 4 × count | member user IDs: the creator |
| … | | name, e.g. `Private 1` |

**[O]**: the 49-byte reply at 85 holds time 0, chat ID 0xb06d05e6, key
0x4ca93520, 1 member, user 2, and `Private 1`. The u32 at 0x24 is the member
list of 203, not a separate creator field.
`kdx_proto::msg::chat::ChatWindowInfo` re-encodes it byte for byte (test
`linux_chat_admin`).

**201 Delete Chat**
Initiator: Client. Privilege: Create/Modify Chats (37). **[S
`FUN_08059774`; O]**

The request is a u32 chat ID. Every member, the requester included,
receives **221 Chat Deleted** {u32 chat ID, u32 user ID of the requester}
before the requester's 1 Status success. Users outside the chat receive
nothing.

**[O]** `captures/linux-chat-rooms.pcapng`: deleting `lab-chat` (0xe8df4d29)
with two members sent `e8 df 4d 29 00 00 00 02` to both (frames 125, 127),
then Status at 129. Deleting the empty `lab-room` sent no 221 to a user in
Public Chat (frames 77–83). The client deleted a chat whose window was open
without sending 204 Leave Chat first (frame 122). `ChatId` and `ChatDeleted`
in `kdx_proto::msg::chat` re-encode 201 (frames 77, 122) and 221 (frames
125, 127) byte for byte (test `linux_chat_rooms`).

**202 Join Chat** → **203 Chat Window Info**
Initiator: Client. Privilege: Join Chat (30) and the chat's join classes.
**[S `FUN_08058710`; O]**

Request: {u32 chat ID, u32 key}.

Reply (51 bytes in the lab):

| Off | Size | Field |
|---|---|---|
| 0x00 | 8 | time |
| 0x08 | 4 | chat ID |
| 0x0c | 4 | key |
| 0x14 | 2 | flags |
| 0x16 | 2 | member count |
| 0x1e | 1 | IRC server length | **[S client `FUN_08082b90`]** |
| 0x1f | 1 | IRC channel length | |
| 0x20 | 1 | name length | **[O]** |
| 0x21 | 1 | topic length | |
| 0x22 | 1 | topic setter length | the nick that set the topic |
| 0x24 | 4 × count | member user IDs | |
| | | name, topic, topic setter, IRC server, IRC channel | |

In frame 68 of `captures/probe.pcapng` only the name (`Public Chat`) is
set; the payload is not padded.

**[O]** `captures/linux-chat-rooms.pcapng`: flags is 2 for a voice chat (frames
70, 115) and 0 for Public Chat (frame 176). The time is set for Public Chat
and 0 for chats made with 217 Create Chat.

The other members receive **205 Chat User Joined** {u32 chat ID, u32 user
ID, u32 0} (`captures/kdx-cli.pcapng` frame 417; `linux-chat-rooms.pcapng`
frames 117, 177; `linux-chat-admin.pcapng` frame 68) **[O]**.
`kdx_proto::msg::chat::ChatUserChange` re-encodes it byte for byte.

**204 Leave Chat**
Initiator: Client. **[S `FUN_08058c0c`; O `captures/linux-chat-rooms.pcapng`
frame 75, `captures/kdx-cli.pcapng` frames 423-424]**

The request is a u32 chat ID. The other members receive **206 Chat User
Departed** {u32 chat ID, u32 user ID, u8 1, 3 zero bytes}
(`captures/kdx-cli.pcapng` frame 424) **[O]**. The meaning of the 1 is
unknown; 205 has 0 there. A private chat that becomes empty is deleted.
`ChatId` and `ChatUserChange` in `kdx_proto::msg::chat` re-encode 204 and
206 byte for byte (tests `linux_chat_rooms`, `kdx_cli`).

**207 Send Chat Text** → **208 Receive Chat Text**
Initiator: Client. Privilege: Send Chat (31) and the chat's send classes.
**[S `FUN_08058d74`; O]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | chat ID |
| 0x04 | 4 | user ID (the server fills in the sender) |
| 0x08 | 4 | |
| 0x0c | | text, ≤ 0x1000 |

The server forwards the same layout as 208 Receive Chat Text to every
member, the sender included.

**209 Set Chat Topic** → **210 Chat Topic Changed**
Initiator: Client. Privilege: Set Chat Topic (35) and the chat's topic
classes. **[S `FUN_08058eac`; O]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | chat ID |
| 0x0a | 1 | topic length |
| 0x0c | | topic |

**[O]**: a probe that put `Hello` at 0x10 with length 5 got the topic
`00 00 00 00 48`, so the server reads from 0x0c (`captures/probe.pcapng`
frames 73–74).

Members receive 210 Chat Topic Changed:

| Off | Size | Field |
|---|---|---|
| 0x00 | 8 | time |
| 0x08 | 4 | chat ID |
| 0x0c | 4 | setter user ID |
| 0x10 | 8 | 0 |
| 0x18 | 1 | topic length |
| 0x19 | 1 | setter name length |
| 0x1a | 2 | 0 |
| 0x1c | | topic, setter name |

**211 Chat Invite** → **212 Chat Invitation**
Initiator: Client. **[S `FUN_080591ac`; O `captures/linux-chat-admin.pcapng`
frames 87-88, 118-120]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | chat ID |
| 0x08 | 2 | invitee count (≤ 20) |
| 0x0f | 1 | message length |
| 0x10 | 4 × count | invitee user IDs |
| … | | message |

Each invitee receives 212 Chat Invitation, not padded **[O]**:

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | chat ID |
| 0x04 | 4 | key |
| 0x08 | 4 | inviter user ID |
| 0x0c | 4 | 1 at 88, 2 at 120 |
| 0x10 | 2 | 1 |
| 0x18 | 1 | probably the message length **[I]**; 0 |
| 0x19 | 1 | chat name length |
| 0x1c | | message **[I]**, chat name |

**[O]**: the client invited user 1 to `Private 1` (frame 87, 20 bytes), then
users 1 and 4 (frame 118, 24 bytes), with no message. User 4 had left, so
only user 1 got a 212 each time (frames 88 and 120, 37 bytes each). The u32
at 0x0c may be the request's invitee count or a running count **[I]**. The u16
at 0x10 is 1 although 216 gave the chat flags 0. `ChatInvite` and
`ChatInvitation` in `kdx_proto::msg::chat` re-encode both byte for byte
(test `linux_chat_admin`).

**213 Must Join Chat**
Initiator: Server. **[S `FUN_0804b114`; O]**

Sent after login when the account has Join Chat (30) and Join Chat On
Connect (36). The payload is empty, so it names no chat. The client always
joins chat 1, Public Chat: its 213 handler ignores the payload and calls the
join helper with chat ID 1 and key 0, which opens the chat window and sends
202 Join Chat {1, 0} **[S Client 1.620 beta 5 `FUN_08084020`,
`FUN_08080734`]**. If the window for chat 1 is already open, the helper only
brings it to the front and sends nothing.

**[O]** `captures/linux-chat-rooms.pcapng`: with bit 36 set in the
Administrator class (504 Set Account Class at frame 145), 213 arrived after
106 Server Icon (frame 167), and the client sent 202 for chat 1 (frame 175).

**214 Get Chat List** → **215 Chat List**
Initiator: Client. **[S `FUN_080594dc`; O]**

The request is empty. Reply: a u32 count, then one entry per chat, each
padded to 4:

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | chat ID |
| 0x04 | 4 | key |
| 0x0e | 2 | flags (bit 1 voice) |
| 0x10 | 2 | member count |
| 0x12 | 1 | name length |
| 0x13 | 1 | description length |
| 0x14 | | name, description |

**[O]**: Public Chat (1), Voice Chat (2, flags 2), Admin Lounge (3). Chats
made with 217 Create Chat follow, with a random chat ID (0xc217598c and
0xe8df4d29 in `captures/linux-chat-rooms.pcapng` frames 47, 90).

**217 Create Chat** / **218 Get Chat** → **219 Chat Info** / **220 Set Chat**
Initiator: Client. Privilege: Create/Modify Chats (37) for 217 Create Chat
and 220. **[S; O `captures/linux-chat-rooms.pcapng` frames 42-62, 85,
136-137]**

217 Create Chat and 220 Set Chat carry a chat record. 218 Get Chat is a u32
chat ID, and 219 Chat Info is a chat record. 220 Set Chat refuses private
chats. 217 and 220 get 1 Status success.

Chat record, not padded:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | chat ID | 0 in 217 Create Chat |
| 0x04 | 0x10 | | 0 |
| 0x14 | 4 | class mask | |
| 0x18 | 4 | class mask | |
| 0x1c | 4 | class mask | |
| 0x20 | 8 | | 0 |
| 0x28 | 2 | flags | bit 1 voice |
| 0x2a | 1 | name length | |
| 0x2b | 1 | description length | |
| 0x2c | 4 | | 0 |
| 0x30 | | name, description | |

**[O]** `captures/linux-chat-rooms.pcapng`: 217 for `lab-room` / `lab` (frame
42), 219 and 220 for `lab edited` (frames 50, 52), 220 with Voice Chat ticked
(flags 2, frame 62), and 219 for Public Chat (frame 137). The three class
masks are `ff ff ff ff` for Public Chat and `80 00 00 00` for a chat whose
Access was left at the default. Their bits are probably account classes,
MSB first (Administrator is class 0), for join, send, and topic **[I]**. The
client sends 505 Get Account Class List before it opens the new-chat window
(frame 39). 218 is the chat ID alone (frames 49, 59, 136). 217 gets 1 Status
success 0xd9 and 220 gets 0xdc (frames 45, 55). `ChatRecord` and `ChatId` in
`kdx_proto::msg::chat` re-encode 217 to 220 byte for byte (test
`linux_chat_rooms`).

**222 Get Chat DCC Addresses** → **223 Chat DCC Addresses**
Initiator: Client. **[S `FUN_0805a110`; O]**

The request is a u32 chat ID. The chat must have the voice flag (bit 1)
and the sender must be a member; otherwise 1 Status 0x10014. An unknown
chat gets 0x1000c **[S]**. The reply has one entry for each other member
whose user record has a nonzero DCC port (0x0a), each padded to 4 **[S]**:

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | user ID |
| 0x04 | 0xa | 0 |
| 0x0e | 1 | address length |
| 0x0f | 1 | 0 |
| 0x10 | | `"ip:port"`: the member's address and DCC port |

The client then opens a TCP connection to each address for voice; see
[dcc.md](../dcc.md).

**[O]** `captures/linux-chat-rooms.pcapng`: the client sends 222 right after
joining a voice chat. The reply was empty (len 0) when the client was alone
(frames 72–73) and when a `kdx` CLI peer was also a member (frames
119-120). The CLI sends DCC port 0, so the server leaves it out **[S]**.

**[O]** `captures/linux-dcc-voice-failed.pcapng`: one entry, user 1 with length
0xf and `127.0.0.1:10600`, padded with 0x22 rather than 0 (frames 109 and
155). The padding byte is probably uninitialised **[I]**;
`DccAddress::padding` keeps it. `ChatId` re-encodes 222 and `DccAddress` re-encodes 223
byte for byte (tests `linux_chat_rooms`, `linux_dcc_voice_failed`).


## Private messages

**300 Send Private Message** → **301 Incoming Private Message**
Initiator: Client. Privilege: Send Messages (32). **[S `FUN_0804d2d0`; O]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | target user ID |
| 0x04 | 0xc | |
| 0x10 | | text |

The payload must be at least 0x11 bytes. The server forwards the same layout
to the target as 301 Incoming Private Message, with the sender's ID in place
of the target's.
