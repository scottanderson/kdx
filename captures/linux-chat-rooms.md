# Scenario: linux-chat-rooms

- Date: 2026-10-10T05:05Z
- Server: KDXServer 1.620 Linux (sha256 ad6fba4bdd01dc23)
- Client: linux 1.620 beta 5
- Server state: the server from linux-server-settings (admin /
  lab-only-pw!1, server name "Lab Server", greeting enabled)
- Peers: `kdx -l admin listen` as `cli-peer` in Public Chat (port 39318) and
  `cli-member` in `lab-chat` (port 55916)

## Steps

The agent drove the lab and watched the capture; the operator clicked. The
steps were known from an earlier recording of this scenario, which this one
replaces.

1. The peer `cli-peer` logged in and joined Public Chat (frames 10–17).
2. The client connected as admin (port 43718, frames 28–32); the peer saw
   103 at 33.
3. Chat button: 214/215 at 36/37. Right-click in the Chat List, new chat:
   505/506 at 39/40 when the window opened. Name `lab-room`, Description
   `lab`, Access left alone, Save: 217 at 42, 1 Status 0xd9 at 45, and the
   refreshed 215 at 47 (chat ID 0xc217598c).
4. Edit Chat on `lab-room`: 218/219 at 49/50. Description `lab edited`,
   Save: 220 at 52, Status 0xdc at 55, 215 at 57.
5. Edit Chat again (218/219 at 59/60), ticked Voice Chat, Save: 220 at 62,
   Status at 65. Joined `lab-room`: 202/203 at 69/70, then 222/223 at 72/73
   (empty). Closed the window: 204 at 75.
6. Delete Chat on `lab-room`: 201 at 77, Status 0xc9 at 81, 215 at 83. No
   221 reached `cli-peer`, which was not a member.
7. Created `lab-chat` / `lab chat` with Voice Chat ticked (217 at 85, Status
   at 88, 215 at 90; chat ID 0xe8df4d29). The agent started `cli-member` in
   it (frames 101–112).
8. Joined `lab-chat`: 202/203 at 114/115, 205 to `cli-member` at 117,
   222/223 at 119/120 (empty).
9. Delete Chat on `lab-chat` with its window still open: 201 at 122, 221 to
   `cli-member` at 125 and to the client at 127, Status at 129, 215 at 131.
10. Edit Chat on Public Chat (218/219 at 136/137), then Cancel.
11. Commands → 127.0.0.1 → Account Classes (505/506 at 139/140),
    Administrator (502/503 at 142/143): ticked Join Chat On Connect and
    saved: 504 at 145, Status 0x1f8 at 148.
12. Connected again (port 60398) without closing the first connection:
    101, 105, 107, 106 at 162–166, then 213 at 167. The client sent 202 for
    chat 1 at 175 (203 at 176, 205 to `cli-peer` at 177).
13. Cleanup: unticked Join Chat On Connect from the first connection (502/503
    at 180/181, 504 at 183, Status at 186), then closed the client (104 at
    193–204).

## Observations

- 213 Must Join Chat is empty (frame 167). The client always joins chat 1;
  its 213 handler hard-codes it (Client 1.620 beta 5 `FUN_08084020`). Join
  Chat On Connect is a class privilege, bit 36: byte 4 went from `f4` in the
  503 to `fc` in the 504 (frames 143, 145), and back in the 504 at 183.
- 217, 219, and 220 share one 0x30-byte chat record header followed by the
  name and description; 217 has chat ID 0. Voice is flags 2 at 0x28. The
  three u32s at 0x14–0x1f are `80 00 00 00` for a new chat and
  `ff ff ff ff` for Public Chat (frames 42, 62, 137).
- Chats made with 217 get random IDs (0xc217598c, 0xe8df4d29).
- 221 Chat Deleted is {chat ID, deleter's user ID}, sent to each member
  including the deleter, before the Status (frames 125, 127, 129).
  Non-members get nothing.
- 222 Get Chat DCC Addresses is the chat ID. 223 stayed empty, alone and
  with a CLI peer in the chat (frames 73, 120).
- The 1 Status success value is the request type: 0xc9, 0xd9, 0xdc, 0x1f8.
- The Public Chat Edit Chat window has Interview Mode; it was not ticked.
- The client can hold two connections to one server; the second login is
  user 4, and the 101 at 162 lists four users.
- First captures of 201, 213, 217, 220, 221, 222, and 223, and the first
  GUI capture of 218.
