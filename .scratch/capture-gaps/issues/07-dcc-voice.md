# Client-to-client (DCC) and voice

Status: needs-info
Scenario: `linux-dcc-voice`
Targets: 1000–1003, and what the clients send each other on TCP 10600

Two copies of Client 1.620 beta 5 (Linux) run at once: one listens on TCP
10600, and the other connects to it. The lab must run the second copy with
its own state dir and a different DCC port, which `kdx-lab` cannot do yet.
The sandboxes have no sound devices, so voice data (1003) may never flow.
Everything is TCP with the normal `%KDX` handshake; kdx-dissect needs
`--port 10600` and the empty password.

1. Agent starts `client linux`; operator leaves it idle.
2. In the second client: Connect button, Type "Client", Address
   `127.0.0.1:10600`, Connect.
3. Commands → the client connection → look at the commands it offers and
   try each (private message, files, Voice Chat).
4. Voice Controls (button bar or Commands menu), Start Microphone (1000–1002
   [I]).
5. Both clients log in to the server, create a chat with Voice Chat ticked,
   and join it. Open Voice Controls in both before joining, then once more
   with it closed in one of them (1001 with 0x00640007 and the "does not
   have his/her Voice Controls window open" message).

## Comments

2026-10-11: Read statically; see `docs/protocol/dcc.md`. Voice is TCP from
client to client, not UDP, after 222/223 give each member's DCC address. A
voice connection is a secondary login (`00` + the connection ID from the
peer's 117 SKPK). The capture should confirm 117, 1000-1003, and which side
opens the primary connection for a private DCC chat.
