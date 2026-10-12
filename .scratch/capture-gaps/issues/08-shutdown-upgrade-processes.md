# Upgrade, process exit, shutdown

Status: done
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

2026-10-12: Captured as `captures/linux-shutdown.pcapng`. 175 has no body
and gets 0x30067, not 0x1000c: the server opens `KDXServer.lexe` under
`Bases` before the helper. Exit Process sends 804 flags 2 (SIGTERM) in a
u32, not a byte. "Exit the program only" sends 174 mode 1 in byte 1; bits
1 and 2 reboot or halt the host. The doc's 174 and 804 layouts were off by
the byte order of the decompiled reads.
