# Capture gaps

Goal: every message type in the catalog of `docs/protocol/KDXProtocol.md` has
at least one capture, made with Client 1.620 beta 5 (Linux). No Wine.

## What the captures hold now

The fifteen captures in `captures/` decode fully (`kdx-dissect` with the lab
passwords). These types appear in at least one of them:

1, 100–116, 118, 140–143, 150–155, 170–173, 180–183, 200–223, 300, 301,
400–403, 405, 407, 409–423, 450–453, 502–513, 515, 550, 551, 600–614,
800–803, 1100–1107.

108 and 118 appear only in `probe` (the kdx-probe tool), so no GUI client has
been seen sending them. 223 Chat DCC Addresses has only been seen empty
(issue 07 may fill it). Client 1.600 appears only in `first-connect`; every
other GUI capture uses Client 1.620 beta 5.

## Missing types

| Types | What | Issue |
|---|---|---|
| 102, 106, 107 (GUI), 110, 111, 112 (GUI), 113/114 (GUI), 172, 173, 183, 419 | session, server settings, history | 01 (done) |
| 201, 213, 217, 220, 221, 222, 223 | chat rooms | 02 (done) |
| 423 | files | 03 (done) |
| 514, 1101, 1102, 1105 | accounts and folder access | 04 (done; 514 is never sent) |
| 700–703, 710–713, 720, 721, tracker UDP registration | tracker | 05 |
| 900–905 | View Display | 06 |
| 1000–1003, client-to-client (DCC) on TCP 10600 | voice and DCC | 07 |
| 174, 175, 804 | shutdown, upgrade, processes | 08 |
| none new | multi-byte text in names, chat, messages, news, nicks | 09 (done) |
| 117, 408 | no known sender | static analysis first (`re/opcodes.md`) |

## How each session runs

The agent drives the lab; the operator clicks.

1. The agent runs `lab/kdx-lab status`, then `up <scenario>`, starts the
   server detached, and starts any `kdx` CLI peers the issue lists.
2. The agent starts the client (`lab/kdx-lab client linux`). If Haxial's
   license agreement appears, the operator decides.
3. The agent gives one step at a time from the issue. After each step it
   runs `kdx-dissect` on the growing capture and confirms the expected types
   before the next step, or adjusts the step.
4. The agent runs `down` only after the operator says the session is over,
   then fills in `captures/<scenario>.md` with the steps and frame numbers.

Lab convention: server at `127.0.0.1`, login `admin`, password
`lab-only-pw!1`. "Commands → 127.0.0.1 → X" means: click the Commands button
in the button bar, choose the connection, then X. The server submenu holds
News, Public Chat, Files, Search Files..., User List, Get Server Info, Show
Greeting, Show My Address, Accounts, Account Classes, Folder Access, Broadcast
Message..., Connection Monitor, Empty Trash, Generate File Catalog, Process
Monitor, Server History, Server Settings, Shutdown Server..., Upgrade Server
Software..., View/Control Display, and Disconnect.

Steps whose result is a guess are marked [I] in the issues; the capture
decides.
