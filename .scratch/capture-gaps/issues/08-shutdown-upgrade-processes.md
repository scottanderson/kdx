# Upgrade, process exit, shutdown

Status: ready-for-human
Scenario: `linux-shutdown`
Targets: 175, 804, 174

Each step ends the server, so the agent restarts it between steps. Run this
session last.

1. Connect as admin. Commands → 127.0.0.1 → Upgrade Server Software...,
   click Upgrade (175; the lab has no KDXUpgradeHelper, so expect an error).
2. Commands → 127.0.0.1 → Process Monitor. The sandbox shows only the
   server. Right-click it, Exit Process (804). The server exits.
3. Agent restarts the server; operator reconnects.
4. Commands → 127.0.0.1 → Shutdown Server..., message `lab shutdown`,
   choose to exit the program only, confirm (174; expect 151 or a
   disconnect).

## Comments

2026-10-11: Read statically; see `docs/protocol/KDXProtocol.md`. Expected:
175 gets 1 Status 0x1000c in the lab (no KDXUpgradeHelper). 804 has no
reply; flags bit 0 sends SIGKILL and bit 1 SIGTERM, so note which the GUI's
Exit Process sends. 174 broadcasts 151 to everyone and the server exits
5000 ms later; the capture should show which mode byte the "exit the
program only" choice sends.
