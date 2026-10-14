# Session messages

Setup, identity, user list, server information, greeting, ping, and the generic Status reply.

**100 Set Identity**
Initiator: Client. **[S `FUN_0804b304`; O]**

The client's first message after the handshake. It is also sent again
whenever the user changes nick, icon, or info. The payload is a user record
with user ID 0.

- The nick takes effect only with the Use Any Name privilege; otherwise the
  account name is used. Changes are logged as `Set Name: <…> (…)`.
- No direct reply. On the first 100 Set Identity, the server sends, in
  order:
  - 101 User List;
  - 105 Server Info;
  - 107 Server Greeting, unless the account has Don't Show Greeting;
  - 106 Server Icon, if one is set;
  - 213 Must Join Chat, when Join Chat On Connect and Join Chat are both
    set.

  It also sends 103 User Joined to everyone else and writes a 184 Login history
  record. **[O]**: 105, 107, 106 in that order
  (`captures/linux-server-settings.pcapng` frames 80, 82, 83), and 213
  after 106 (`captures/linux-chat-rooms.pcapng` frame 167).
- A later 100 Set Identity broadcasts 102 User Changed to everyone, the
  sender included (`captures/linux-server-settings.pcapng` frames 196, 199,
  201) **[O]**.

**101 User List**
Initiator: Server. **[S `FUN_0804af04`; O]**

A sequence of user records, one per user who is logged in (all of them on
login).

**102 User Changed** / **103 User Joined**
Initiator: Server. **[S; O]**

One user record.

**104 User Departed**
Initiator: Server. **[O `captures/kdx-cli.pcapng` frames 398, 429;
`captures/linux-chat-admin.pcapng` frames 59, 116]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | user ID | |
| 0x04 | 1 | | 7 in every capture |
| 0x05 | 3 | 0 | |

`kdx_proto::msg::user::UserDeparted` re-encodes it byte for byte (test
`linux_chat_admin`).

**105 Server Info**
Initiator: Server. **[S `FUN_0804b304`; O 86 bytes]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 0x20 | | 32 bytes from the user object (+0x74), probably a session ID for transfer connections **[I]** |
| 0x20 | 4 | file transfer port | the server's optional File Transfer port (settings +0x260); 0 if none. The client opens transfer connections there when it is nonzero **[S client `FUN_08052cc0`]** |
| 0x24 | 4 | your user ID | **[O]** 7 and 8 in successive probes |
| 0x28 | 2 | server version | 0x0654 = 1620 |
| 0x2a | 2 | can modify settings | 1 with Modify Server Settings |
| 0x46 | 2 | | 2. Unless it is 2, the client compares the version and warns about an older or newer server **[S client]**; Server 1.620 always sends 2, so the warning never shows |
| 0x48 | 1 | name length | ≤ 0x3f |
| 0x49 | 1 | description length | ≤ 0xff |
| 0x4a | 1 | third string length | ≤ 0x7f |
| 0x4c | | name, description, third string | "New Server" **[O]** |

The whole message is at most 0x400 bytes.

**106 Server Icon**
Initiator: Server. **[O]**

The server's icon bitmap, the same bytes as the icon section of the settings
blob (see 171 Server Settings Info). **[O]**: 524 bytes for a 32×32 icon pasted
from the user icon, starting `01 20 20 04 00 01 00 00 00 ff cc 00`
(`captures/linux-server-settings.pcapng` frame 83), equal to the icon
section of the 171 at frame 221. `kdx_proto::msg::server_info::ServerIcon`
holds the bytes.

**107 Server Greeting**
Initiator: Server. **[S `FUN_0804b738`; O]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | 0 |
| 0x04 | | greeting text, to the end of the payload |

When no greeting is set, the server sends `(No greeting.)`
(`captures/probe.pcapng` frame 163) **[O]**. When a greeting is set but not
enabled, it sends `(Greeting disabled.)`
(`captures/linux-server-settings.pcapng` frame 46) **[O]**. The enable
option is bit 1 of byte 0x93 of the settings blob.
`kdx_proto::msg::server_info::ServerGreeting` re-encodes frames 46, 62, and
82 byte for byte (test `linux_server_settings`).

**108 Ping** → **109 Pong**
Initiator: Client. **[S]**

The request is empty, and so is the reply (109 Pong). Ping does not reset
the idle timer.

**110 Set Mini User Icon**
Initiator: Client. **[S `FUN_0804b6d4`; O]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 1 | status | 1 after `/away`, 0 after `/back` **[O]** |
| 0x01 | 3 | 0 | the payload must be at least 4 bytes |

There is no reply. If the value changed, the server broadcasts **111 Mini
User Icon Changed** to everyone, the sender included:

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | user ID |
| 0x04 | 4 | the 110 payload |

**[O]**: `/away` sent `01 00 00 00`, and the peer received `00 00 00 03 01 00 00
00` for user 3 (`captures/linux-server-settings.pcapng` frames 210, 211,
213). `kdx_proto::msg::user::MiniUserIcon` and `MiniUserIconChanged`
re-encode frames 210 to 218 byte for byte.

**112 (unnamed)**
Initiator: Client. **[S `FUN_0804b738`; O]**

The request is empty. The server answers with 107 Server Greeting as a new
message (its own txid), not as a reply. The client uses this to re-read the
greeting.

**113 Get Extra Server Info** → **114 Extra Server Info**
Initiator: Client. **[S `FUN_080619d8`; O]**

The request is empty. Reply:

| Off | Size | Field |
|---|---|---|
| 0x00 | 0x10 | |
| 0x10 | 8 | server start time (ms) |
| 0x18 | 8 | current server time (ms) |
| 0x20 | 4 | uptime in seconds |
| 0x24 | 4 | OS value |
| 0x28 | 4 | OS value |
| 0x2c | 1 | OS string length |
| 0x30 | | OS string |

**115 Get My Address** → **116 Your Address**
Initiator: Client. **[S; O]**

The request is empty. The reply is a u8 length followed by `"a.b.c.d:port"`:
the client's address as the server sees it.

**118 (unnamed)**
Initiator: Client. **[S `FUN_0804fdac`; O `captures/probe.pcapng` frame
165]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | value |

No reply. The server stores 1 if the value is nonzero and 2 if it is zero
(user +0x3b). The probe sent `00 00 00 01` and got no reply **[O]**.
`kdx_proto::msg::user::RegistrationNotice` re-encodes it.

The client sends it on a timer, 9000000 ms plus a random 0..0x4fffff ms
after login **[S client `FUN_0804c01c`]**. The value comes from the
client's stored purchase certificate (its length; 0 without one) **[S
client `FUN_08107e04`]**. So it reports whether the client is registered
**[I]**.

The server acts on it **[S `FUN_0804b7c8`]**: every 900000 ms plus a random
0..0x8ffff ms, it sends 155 Rcv Broadcast Message "Please help fund the further
development of KDX by purchasing a license." to each user whose stored value is
2 and who has been connected less than 2 hours (7200000 ms). The first round
runs 217500000 ms (about 60 hours) after the server starts **[S
`FUN_080531f0`]**, so a short lab run never shows it. The text is obfuscated in
the binary at 0x080adc00 (each byte is ~b + 0xbe). The 155 payload is 16 zero
bytes then the text.
