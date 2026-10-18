# Chat messages

For hex and field notation, see [Conventions](../README.md#conventions).

Message types 200 to 223 cover public and private chat rooms, invitations, and
voice flags. Private messages have [their own page](private-messages.md).

## <a id="m200"></a>200 Create Private Chat
Initiator: Client. Privilege: Create Private Chat (34). <!-- [S `FUN_08058420`; O `captures/linux-chat-admin.pcapng` frames 84 to 85] -->

A new private chat is one request and one reply:

```mermaid
%%{init: {"sequence": {"mirrorActors": false}}}%%
sequenceDiagram
    participant C as Client
    participant S as Server
    C->>S: 200 Create Private Chat
    S->>C: 216 Create Private Chat (R)
```

No payload.

A user can have at most 10 private chats. The server replies with [216 Create
Private Chat (R)](#m216).

## <a id="m201"></a>201 Delete Chat
Initiator: Client. Privilege: Create/Modify Chats (37). <!-- [S `FUN_08059774`; O] -->

Every member gets [221 Chat Deleted](#m221) before the requester gets [1
Status](status.md#m1):

```mermaid
%%{init: {"sequence": {"mirrorActors": false}}}%%
sequenceDiagram
    participant C as Client
    participant S as Server
    participant O as Client
    C->>S: 201 Delete Chat
    S-->>C: 221 Chat Deleted
    S-->>O: 221 Chat Deleted
    S->>C: 1 Status (success)
```

Struct `ChatId` (4 bytes):<a id="sChatId"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `u32` | `chat_id` |  |

Every member, the requester too, receives [221 Chat Deleted](#m221) before the
requester's [1 Status](status.md#m1) success. Users outside the chat receive
nothing.

In `captures/linux-chat-rooms.pcapng`, deleting `lab-chat` (0xe8df4d29) with two
members sent [221 Chat Deleted](#m221) to both (frames 125 and 127), then the
Status at frame 129. Deleting the empty `lab-room` sent no [221 Chat
Deleted](#m221) to a user in Public Chat (frames 77 to 83). The client deleted a
chat whose window was open without sending [204 Leave Chat](#m204) first (frame
122). <!-- [O] --> `ChatId` in `kdx_proto::msg::chat` re-encodes [201 Delete
Chat](#m201) (frames 77 and 122) byte for byte (test `linux_chat_rooms`).

## <a id="m202"></a>202 Join Chat
Initiator: Client. Privilege: Join Chat (30) and the chat's join classes.
<!-- [S `FUN_08058710`; O] -->

When a user joins a chat, the server tells the other members:

```mermaid
%%{init: {"sequence": {"mirrorActors": false}}}%%
sequenceDiagram
    participant C as Client
    participant S as Server
    participant O as Client
    C->>S: 202 Join Chat
    S->>C: 203 Chat Window Info
    S-->>O: 205 Chat User Joined
```

Struct `JoinChat` (8 bytes):<a id="sJoinChat"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `u32` | `chat_id` |  |
| 0x04 | 4 | `u32` | `key` | Needed for private chats. |

The server replies with [203 Chat Window Info](#m203), and the other members
receive [205 Chat User Joined](#m205).

## <a id="m203"></a>203 Chat Window Info
Initiator: Server. Reply to [202 Join Chat](#m202).

The reply is 51 bytes in the lab. The five strings follow the member IDs in this
order: name, topic, topic setter, IRC server, IRC channel. The length fields
before them are in a different order: IRC server, IRC channel, name, topic,
topic setter. <!-- [S server `FUN_08058710`, client `FUN_08082b90`] --> In every
capture, the IRC lengths are 0 and the byte at 0x20 is the name's length, so no
capture shows the order of the strings. <!-- [O] -->

Struct `ChatWindowInfo` (dynamic):<a id="sChatWindowInfo"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 8 | `u64` | `time` |  |
| 0x08 | 4 | `u32` | `chat_id` |  |
| 0x0c | 4 | `u32` | `key` |  |
| 0x10 | 4 | `u8[4]` | `ignored_10` |  |
| 0x14 | 2 | `u16` | `flags` | Bit 0: private. Bit 1: voice. |
| 0x16 | 2 | `u16` | `members_count` |  |
| 0x18 | 6 | `u8[6]` | `ignored_18` |  |
| 0x1e | 1 | `u8` | `irc_server_len` |  |
| 0x1f | 1 | `u8` | `irc_channel_len` |  |
| 0x20 | 1 | `u8` | `name_len` |  |
| 0x21 | 1 | `u8` | `topic_len` |  |
| 0x22 | 1 | `u8` | `topic_setter_len` |  |
| 0x23 | 1 | `u8` | `ignored_23` |  |
| 0x24 | `members_count` | `u32[]` | `members` |  |
|  | `name_len` | `str` | `name` | <!-- [O] --> |
|  | `topic_len` | `str` | `topic` |  |
|  | `topic_setter_len` | `str` | `topic_setter` | The nick that set the topic. |
|  | `irc_server_len` | `str` | `irc_server` |  |
|  | `irc_channel_len` | `str` | `irc_channel` |  |

In frame 68 of `captures/probe.pcapng`, only the name (`Public Chat`) is set,
and the payload is not padded.

In `captures/linux-chat-rooms.pcapng`, the flags are 2 for a voice chat (frames
70 and 115) and 0 for Public Chat (frame 176). The time is set for Public Chat,
and 0 for chats made with [217 Create Chat](#m217). <!-- [O] -->

## <a id="m204"></a>204 Leave Chat
Initiator: Client. <!-- [S `FUN_08058c0c`; O `captures/linux-chat-rooms.pcapng` frame 75; O `captures/kdx-cli.pcapng` frames 423 to 424] -->

When a user leaves a chat, the server tells the other members:

```mermaid
%%{init: {"sequence": {"mirrorActors": false}}}%%
sequenceDiagram
    participant C as Client
    participant S as Server
    participant O as Client
    C->>S: 204 Leave Chat
    S-->>O: 206 Chat User Departed
```

Struct `LeaveChat` (4 bytes):<a id="sLeaveChat"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | [`ChatId`](#sChatId) | `chat` |  |

The other members receive [206 Chat User Departed](#m206). The server deletes a
private chat when it becomes empty. `ChatId` in `kdx_proto::msg::chat`
re-encodes [204 Leave Chat](#m204) byte for byte (tests `linux_chat_rooms` and
`kdx_cli`).

## <a id="m205"></a>205 Chat User Joined
Initiator: Server. The server sends this to the other members after
[202 Join Chat](#m202).

Struct `ChatUserChange` (12 bytes):<a id="sChatUserChange"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `u32` | `chat_id` |  |
| 0x04 | 4 | `u32` | `user_id` | The user who joined or left. |
| 0x08 | 1 | `u8` | `ignored_08` | 0 in [205 Chat User Joined](#m205), and 1 in [206 Chat User Departed](#m206). |
| 0x09 | 3 | `u8[3]` | `ignored_09` |  |

In [205 Chat User Joined](#m205), the last four bytes are 0
(`captures/kdx-cli.pcapng` frame 417, `captures/linux-chat-rooms.pcapng` frames
117 and 177, and `captures/linux-chat-admin.pcapng` frame 68). <!-- [O] -->
`kdx_proto::msg::chat::ChatUserChange` re-encodes it byte for byte.

## <a id="m206"></a>206 Chat User Departed
Initiator: Server. The server sends this to the other members after
[204 Leave Chat](#m204).

Struct `ChatUserDeparted` (12 bytes):<a id="sChatUserDeparted"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 12 | [`ChatUserChange`](#sChatUserChange) | `change` | The byte at 0x08 is 1, and 3 zero bytes follow. |

The payload is a u32 chat ID, a u32 user ID, a byte 1, and 3 zero bytes
(`captures/kdx-cli.pcapng` frame 424). <!-- [O] --> The meaning of the 1 is
unknown; [205 Chat User Joined](#m205) has 0 there. `ChatUserChange` in
`kdx_proto::msg::chat` re-encodes [206 Chat User Departed](#m206) byte for byte
(tests `linux_chat_rooms` and `kdx_cli`).

## <a id="m207"></a>207 Send Chat Text
Initiator: Client. Privilege: Send Chat (31) and the chat's send classes.
<!-- [S `FUN_08058d74`; O] -->

The server forwards the text to every member, the sender too:

```mermaid
%%{init: {"sequence": {"mirrorActors": false}}}%%
sequenceDiagram
    participant C as Client
    participant S as Server
    participant O as Client
    C->>S: 207 Send Chat Text
    S-->>C: 208 Receive Chat Text
    S-->>O: 208 Receive Chat Text
```

Struct `ChatText` (dynamic):<a id="sChatText"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `u32` | `chat_id` |  |
| 0x04 | 4 | `u32` | `user_id` | The server fills in the sender. |
| 0x08 | 4 | `u8[4]` | `ignored` |  |
| 0x0c | rest | `str` | `text` | At most 0x1000 bytes. |

The server forwards it as [208 Receive Chat Text](#m208).

## <a id="m208"></a>208 Receive Chat Text
Initiator: Server. Forwarded from [207 Send Chat Text](#m207).

The server forwards the [207 Send Chat Text](#m207) layout to every member, the
sender too.

Struct `ReceiveChatText` (dynamic):<a id="sReceiveChatText"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 |  | [`ChatText`](#sChatText) | `text` | `user_id` is the sender's. |

## <a id="m209"></a>209 Set Chat Topic
Initiator: Client. Privilege: Set Chat Topic (35) and the chat's topic
classes. <!-- [S `FUN_08058eac`; O] -->

The members receive the new topic:

```mermaid
%%{init: {"sequence": {"mirrorActors": false}}}%%
sequenceDiagram
    participant C as Client
    participant S as Server
    participant O as Client
    C->>S: 209 Set Chat Topic
    S-->>C: 210 Chat Topic Changed
    S-->>O: 210 Chat Topic Changed
```

Struct `SetChatTopic` (dynamic):<a id="sSetChatTopic"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `u32` | `chat_id` |  |
| 0x04 | 6 | `u8[6]` | `ignored_04` | 0 from the client. |
| 0x0a | 1 | `u8` | `topic_len` |  |
| 0x0b | 1 | `u8` | `ignored_0b` | 0 from the client. |
| 0x0c | `topic_len` | `str` | `topic` |  |
|  | rest | `u8[]` | `rest` | The server ignores it. Empty from the client. |

A probe that put `Hello` at 0x10 with length 5 got the topic `00 00 00 00 48`,
so the server reads from 0x0c (`captures/probe.pcapng` frames 73 and
74). <!-- [O] -->

The members receive [210 Chat Topic Changed](#m210).

## <a id="m210"></a>210 Chat Topic Changed
Initiator: Server. The server sends this to the members after
[209 Set Chat Topic](#m209).

Struct `ChatTopicChanged` (dynamic):<a id="sChatTopicChanged"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 8 | `u64` | `time` |  |
| 0x08 | 4 | `u32` | `chat_id` |  |
| 0x0c | 4 | `u32` | `user_id` |  |
| 0x10 | 8 | `u8[8]` | `ignored_10` | 0 as sent. |
| 0x18 | 1 | `u8` | `topic_len` |  |
| 0x19 | 1 | `u8` | `nick_len` |  |
| 0x1a | 2 | `u8[2]` | `ignored_1a` | 0 as sent. |
| 0x1c | `topic_len` | `str` | `topic` |  |
|  | `nick_len` | `str` | `nick` |  |

## <a id="m211"></a>211 Chat Invite
Initiator: Client. <!-- [S `FUN_080591ac`; O `captures/linux-chat-admin.pcapng` frames 87 to 88, 118 to 120] -->

Each invitee receives an invitation:

```mermaid
%%{init: {"sequence": {"mirrorActors": false}}}%%
sequenceDiagram
    participant C as Client
    participant S as Server
    participant O as Client
    C->>S: 211 Chat Invite
    S-->>O: 212 Chat Invitation
```

Struct `ChatInvite` (dynamic):<a id="sChatInvite"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `u32` | `chat_id` |  |
| 0x04 | 4 | `u8[4]` | `ignored_04` |  |
| 0x08 | 2 | `u16` | `user_ids_count` |  |
| 0x0a | 5 | `u8[5]` | `ignored_0a` |  |
| 0x0f | 1 | `u8` | `message_len` |  |
| 0x10 | `user_ids_count` | `u32[]` | `user_ids` | At most 20. |
|  | `message_len` | `str` | `message` |  |

Each invitee receives [212 Chat Invitation](#m212). The client invited user 1 to
`Private 1` (frame 87, 20 bytes), then users 1 and 4 (frame 118, 24 bytes), with
no message. User 4 had left, so only user 1 got a [212 Chat Invitation](#m212)
each time (frames 88 and 120). <!-- [O] --> `ChatInvite` in
`kdx_proto::msg::chat` re-encodes it byte for byte (test `linux_chat_admin`).

## <a id="m212"></a>212 Chat Invitation
Initiator: Server. The server sends this to each invitee of
[211 Chat Invite](#m211).

The payload is not padded: <!-- [O] -->

Struct `ChatInvitation` (dynamic):<a id="sChatInvitation"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `u32` | `chat_id` |  |
| 0x04 | 4 | `u32` | `key` | For [202 Join Chat](#m202). |
| 0x08 | 4 | `u32` | `user_id` |  |
| 0x0c | 4 | `u32` | `ignored_0c` | 1 at frame 88, and 2 at frame 120. |
| 0x10 | 2 | `u16` | `ignored_10` | 1 |
| 0x12 | 6 | `u8[6]` | `ignored_12` |  |
| 0x18 | 1 | `u8` | `message_len` |  |
| 0x19 | 1 | `u8` | `name_len` |  |
| 0x1a | 2 | `u8[2]` | `ignored_1a` |  |
| 0x1c | `message_len` | `str` | `message` | Probably the message. 0 in the captures. <!-- [I] --> |
|  | `name_len` | `str` | `name` |  |

Frames 88 and 120 are 37 bytes each. <!-- [O] --> The u32 at 0x0c may be the
request's invitee count, or a running count. <!-- [I] --> The u16 at 0x10 is 1,
although [216 Create Private Chat (R)](#m216) gave the chat flags 0.
`ChatInvitation` in `kdx_proto::msg::chat` re-encodes it byte for byte (test
`linux_chat_admin`).

## <a id="m213"></a>213 Must Join Chat
Initiator: Server. <!-- [S `FUN_0804b114`; O] --> Part of the [login
sequence](../sequences.md#login).

After login, the server asks the client to join, and the client joins chat 1:

```mermaid
%%{init: {"sequence": {"mirrorActors": false}}}%%
sequenceDiagram
    participant C as Client
    participant S as Server
    S-->>C: 213 Must Join Chat
    C->>S: 202 Join Chat
    S->>C: 203 Chat Window Info
```

No payload.

The payload is empty. <!-- [O `captures/linux-chat-rooms.pcapng` frame 167] -->
The server sends it after login when the account has Join Chat (30) and Join
Chat On Connect (36). It names no chat. The client always joins chat 1, Public
Chat. In Client 1.620 beta 5, its [213 Must Join Chat](#m213) handler ignores
the payload and calls the join helper with chat ID 1 and key 0. The helper opens
the chat window and sends [202 Join Chat](#m202) with chat 1 and key 0. <!-- [S
`FUN_08084020`, `FUN_08080734`] --> If the window for chat 1 is already open,
the helper only brings it to the front and sends nothing.

In `captures/linux-chat-rooms.pcapng`, bit 36 was set in the Administrator class
([504 Set Account Class](accounts.md#m504) at frame 145). The [213 Must Join
Chat](#m213) arrived after [106 Server Icon](session.md#m106) (frame 167), and
the client sent [202 Join Chat](#m202) for chat 1 (frame 175). <!-- [O] -->

## <a id="m214"></a>214 Get Chat List
Initiator: Client. <!-- [S `FUN_080594dc`; O] -->

The chat list is one request and one reply:

```mermaid
%%{init: {"sequence": {"mirrorActors": false}}}%%
sequenceDiagram
    participant C as Client
    participant S as Server
    C->>S: 214 Get Chat List
    S->>C: 215 Chat List
```

No payload.

The server replies with [215 Chat List](#m215).

## <a id="m215"></a>215 Chat List
Initiator: Server. Reply to [214 Get Chat List](#m214).

The payload is a u32 count, then one entry for each chat, each padded to a
multiple of 4:

Struct `ChatList` (dynamic):<a id="sChatList"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `u32` | `chats_count` |  |
| 0x04 | `chats_count` | [`ChatListEntry`](#sChatListEntry)`[]` | `chats` |  |

Struct `ChatListEntry` (dynamic):<a id="sChatListEntry"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `u32` | `chat_id` |  |
| 0x04 | 4 | `u32` | `key` |  |
| 0x08 | 6 | `u8[6]` | `ignored` |  |
| 0x0e | 2 | `u16` | `flags` | Bit 1: voice. |
| 0x10 | 2 | `u16` | `member_count` |  |
| 0x12 | 1 | `u8` | `name_len` |  |
| 0x13 | 1 | `u8` | `description_len` |  |
| 0x14 | `name_len` | `str` | `name` |  |
|  | `description_len` | `str` | `description` |  |
|  | pad4 | `u8[]` | `padding` |  |

The lab has Public Chat (1), Voice Chat (2, flags 2), and Admin Lounge (3).
Chats made with [217 Create Chat](#m217) follow, with random chat IDs
(0xc217598c and 0xe8df4d29 in `captures/linux-chat-rooms.pcapng` frames 47 and
90). <!-- [O] -->

## <a id="m216"></a>216 Create Private Chat (R)
Initiator: Server. Reply to [200 Create Private Chat](#m200).

The reply has the layout of [203 Chat Window Info](#m203), with the creator as
the only member. <!-- [O] --> The flags are 0 at frame 85, and the name is like
`Private 1`.

Struct `CreatePrivateChatReply` (dynamic):<a id="sCreatePrivateChatReply"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 |  | [`ChatWindowInfo`](#sChatWindowInfo) | `info` | One member, the creator. |

The 49-byte reply at frame 85 holds time 0, chat ID 0xb06d05e6, key 0x4ca93520,
1 member, user 2, and `Private 1`. <!-- [O] --> The u32 at 0x24 is the member
list of [203 Chat Window Info](#m203), not a separate creator field.
`kdx_proto::msg::chat::ChatWindowInfo` re-encodes it byte for byte (test
`linux_chat_admin`).

## <a id="m217"></a>217 Create Chat
Initiator: Client. Privilege: Create/Modify Chats (37), for [217 Create
Chat](#m217) and [220 Set Chat](#m220). <!-- [S `FUN_08059944`; O
`captures/linux-chat-rooms.pcapng` frames 42 to 62, 85, 136 to 137] -->

A client creates, reads, and changes a chat record:

```mermaid
%%{init: {"sequence": {"mirrorActors": false}}}%%
sequenceDiagram
    participant C as Client
    participant S as Server
    C->>S: 217 Create Chat
    S->>C: 1 Status (success 0xd9)
    C->>S: 218 Get Chat
    S->>C: 219 Chat Info
    C->>S: 220 Set Chat
    S->>C: 1 Status (success 0xdc)
```

[217 Create Chat](#m217) and [220 Set Chat](#m220) hold a chat record. [218 Get
Chat](#m218) is a u32 chat ID, and [219 Chat Info](#m219) is a chat record. [220
Set Chat](#m220) refuses private chats. [217 Create Chat](#m217) and [220 Set
Chat](#m220) get [1 Status](status.md#m1) success.

The chat record is not padded. It is `ChatRecord` in
[encodings.md](../encodings.md#chat-record).

Struct `CreateChat` (dynamic):<a id="sCreateChat"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 |  | [`ChatRecord`](../encodings.md#sChatRecord) | `chat` | `chat_id` is 0. |

In `captures/linux-chat-rooms.pcapng`: <!-- [O] -->

- [217 Create Chat](#m217) creates `lab-room` with the description `lab` (frame
  42).
- [219 Chat Info](#m219) and [220 Set Chat](#m220) show the edit to `lab edited`
  (frames 50 and 52).
- [220 Set Chat](#m220) ticks Voice Chat, which sets flags 2 (frame 62).
- [219 Chat Info](#m219) reads Public Chat (frame 137).

The three class masks are `ff ff ff ff` for Public Chat, and `80 00 00 00` for a
chat whose Access was left at the default. Their bits are probably account
classes, MSB first (Administrator is class 0), for join, send, and topic. <!--
[I] --> The client sends [505 Get Account Class List](accounts.md#m505) before
it opens the new-chat window (frame 39). [217 Create Chat](#m217) gets [1
Status](status.md#m1) success 0xd9, and [220 Set Chat](#m220) gets 0xdc (frames
45 and 55). `ChatRecord` and `ChatId` in `kdx_proto::msg::chat` re-encode 217 to
220 byte for byte (test `linux_chat_rooms`).

## <a id="m218"></a>218 Get Chat
Initiator: Client. <!-- [S `FUN_08059be8`; O] -->

Struct `GetChat` (4 bytes):<a id="sGetChat"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | [`ChatId`](#sChatId) | `chat` |  |

The request is the chat ID alone (frames 49, 59, and 136). The server replies
with [219 Chat Info](#m219). [217 Create Chat](#m217) has the captures.

## <a id="m219"></a>219 Chat Info
Initiator: Server. Reply to [218 Get Chat](#m218).

The payload is a chat record; see [217 Create Chat](#m217).

Struct `ChatInfo` (dynamic):<a id="sChatInfo"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 |  | [`ChatRecord`](../encodings.md#sChatRecord) | `chat` |  |

## <a id="m220"></a>220 Set Chat
Initiator: Client. Privilege: Create/Modify Chats (37). <!-- [S `FUN_08059e24`] -->

The request holds a chat record. The server refuses private chats. [217 Create
Chat](#m217) has the record, the captures, and the status code.

Struct `SetChat` (dynamic):<a id="sSetChat"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 |  | [`ChatRecord`](../encodings.md#sChatRecord) | `chat` |  |

## <a id="m221"></a>221 Chat Deleted
Initiator: Server. The server sends this to every member after
[201 Delete Chat](#m201).

Struct `ChatDeleted` (8 bytes):<a id="sChatDeleted"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `u32` | `chat_id` |  |
| 0x04 | 4 | `u32` | `user_id` | The requester. |

In `captures/linux-chat-rooms.pcapng`, deleting `lab-chat` (0xe8df4d29) sent `e8
df 4d 29 00 00 00 02` to both members (frames 125 and 127). <!-- [O] -->
`ChatDeleted` in `kdx_proto::msg::chat` re-encodes them byte for byte (test
`linux_chat_rooms`).

## <a id="m222"></a>222 Get Chat DCC Addresses
Initiator: Client. <!-- [S `FUN_0805a110`; O] --> Part of the [voice call
sequence](../dcc.md#voice).

A member of a voice chat asks where the others listen:

```mermaid
%%{init: {"sequence": {"mirrorActors": false}}}%%
sequenceDiagram
    participant C as Client
    participant S as Server
    C->>S: 222 Get Chat DCC Addresses
    S->>C: 223 Chat DCC Addresses
```

Struct `GetChatDccAddresses` (4 bytes):<a id="sGetChatDccAddresses"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | [`ChatId`](#sChatId) | `chat` |  |

The chat must have the voice flag (bit 1), and the sender must be a member. If
not, the reply is [1 Status](status.md#m1) 0x10014. An unknown chat gets
0x1000c. <!-- [S] --> The server replies with [223 Chat DCC Addresses](#m223).

The client then opens a TCP connection to each address for voice; see
[dcc.md](../dcc.md).

In `captures/linux-chat-rooms.pcapng`, the client sends [222 Get Chat DCC
Addresses](#m222) right after it joins a voice chat. <!-- [O] --> `ChatId`
re-encodes 222 byte for byte (tests `linux_chat_rooms` and
`linux_dcc_voice_failed`).

## <a id="m223"></a>223 Chat DCC Addresses
Initiator: Server. Reply to [222 Get Chat DCC Addresses](#m222). Part of the
[voice call sequence](../dcc.md#voice).

The reply has one entry for each other member whose user record has a DCC port
(0x0a) that is not 0. Each entry is padded to a multiple of 4: <!-- [S] -->

Struct `ChatDccAddresses` (dynamic):<a id="sChatDccAddresses"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | rest | [`DccAddress`](#sDccAddress)`[]` | `addresses` |  |

Struct `DccAddress` (dynamic):<a id="sDccAddress"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `u32` | `user_id` |  |
| 0x04 | 10 | `u8[10]` | `ignored_04` | 0 as sent. |
| 0x0e | 1 | `u8` | `address_len` |  |
| 0x0f | 1 | `u8` | `ignored_0f` | 0 as sent. |
| 0x10 | `address_len` | `str` | `address` | `ip:port`: the member's address and DCC port. |
|  | pad4 | `u8[]` | `padding` |  |

In `captures/linux-chat-rooms.pcapng`, the reply was empty when the client was
alone (frames 72 and 73), and when a `kdx` CLI peer was also a member (frames
119 and 120). <!-- [O] --> The CLI sends DCC port 0, so the server leaves it
out. <!-- [S] -->

In `captures/linux-dcc-voice-failed.pcapng`, the reply has one entry: user 1,
length 0xf, and `127.0.0.1:10600`, padded with 0x22 rather than 0 (frames 109
and 155). <!-- [O] --> The padding byte is probably uninitialized. <!-- [I] -->
`DccAddress::padding` keeps it. `DccAddress` re-encodes [223 Chat DCC
Addresses](#m223) byte for byte (tests `linux_chat_rooms` and
`linux_dcc_voice_failed`).
