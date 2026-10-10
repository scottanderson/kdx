# Client-to-client (DCC) and voice

Status: needs-info
Scenario: `linux-dcc-voice`
Targets: 1000–1003, and what the clients send each other on TCP 10600

Two copies of Client 1.620 beta 5 (Linux) run at once: one listens on TCP
10600, and the other connects to it. The lab must run the second copy with
its own state dir and port, which `kdx-lab` cannot do yet. The sandboxes have no sound devices, so
voice data (1003, UDP) may never flow. kdx-dissect needs `--port 10600`, and
may need work if the client service differs from `%KDX`.

1. Agent starts `client linux`; operator leaves it idle.
2. In the second client: Connect button, Type "Client", Address
   `127.0.0.1:10600`, Connect.
3. Commands → the client connection → look at the commands it offers and
   try each (private message, files, Voice Chat).
4. Voice Controls (button bar or Commands menu), Start Microphone (1000–1002
   [I]).
