# Chat and private messages: 200–223, 300, 301, and the chat record

Status: ready-for-agent
Type: research
Targets: 203/216, 205/206, 207/208, 209/210, 211/212, 215, the chat record
(217–220), 223, 300/301

Docs: `KDXProtocol.md`, Chat record, Chat, and Private messages.

## Gaps

Chat record (`FUN_08059944`, `FUN_08059be8`, `FUN_08059e24`):

- [ ] 0x04: the record table says key, then 0x08 for 0xc bytes; the 217
  table says 0x04 for 0x10 bytes of 0. Reconcile them.
- [ ] 0x20, 8 bytes.
- [ ] 0x2c..0x2f: the record table has the IRC server and channel lengths at
  0x2c and 0x2d; the 217 table has a u32 0 there. Reconcile, and find
  0x2e..0x2f.
- [ ] The three class masks at 0x14, 0x18, 0x1c are "probably" join, send,
  and topic classes **[I]**; confirm from the handlers (202, 207, 209) and
  `ChatRecord`.
- [ ] A Linux capture of a chat made with Make IRC Gateway, to see the IRC
  strings on the wire (in the record and in 203). No capture has one.

203 Chat Window Info / 216 Create Private Chat (R):

- [ ] 0x10..0x13, 0x18..0x1d, and 0x23 are not in the tables.
- [ ] The IRC length bytes at 0x1e and 0x1f are read from the client only.

Other messages:

- [ ] 205/206: the third word; 206 has byte 0x08 set to 1, 205 to 0, meaning
  unknown (`ChatUserChange::unknown_08`).
- [ ] 207/208 0x08, a u32.
- [ ] 209 Set Chat Topic: 0x04..0x09 and 0x0b.
- [ ] 210 Chat Topic Changed: 0x10, 8 bytes of 0, and 0x1a..0x1b.
- [ ] 211 Chat Invite: 0x04..0x07 and 0x0a..0x0e.
- [ ] 212 Chat Invitation: the u32 at 0x0c, "the request's invitee count
  or a running count" **[I]** (`ChatInvitation::unknown_0c`); the u16 at 0x10,
  1 although the chat's flags were 0 (`unknown_10`); 0x12..0x17; 0x18,
  "probably the message length" **[I]**; 0x1a..0x1b. A Linux capture of an
  invitation with a message settles 0x18; the two on record have none.
  They don't separate the 0x0c readings either: it is 1, then 2
  (`linux-chat-admin` frames 88, 120), for 211s that invite one user,
  then two (frames 87, 118). Inviting two users first would.
- [ ] 215 Chat List entry: 0x08..0x0d.
- [ ] 223 Chat DCC Addresses: an entry has never been seen, so 223 has no
  codec. Entry bytes 0x04..0x0d and 0x0f are listed as 0 **[S]**. A Linux
  capture is enough: the GUI client with its DCC listener on (port 10600
  in its user record) joins a voice chat after a second member asks with
  222. The second member can be `kdx-probe` or the CLI with a nonzero DCC
  port, so two GUI clients aren't needed (unlike capture-gaps 07). Both
  223s on record are empty (`linux-chat-rooms` frames 73, 120), the
  second with a CLI member whose DCC port is 0.
- [ ] 300/301: 0x04, 0xc bytes. What the client puts there, and whether the
  server reads them.
