# Processes: 800–804

Status: ready-for-agent
Type: research
Targets: 803 Process List, 804 Exit/Terminate Process

Docs: `KDXProtocol.md`, Processes. The Linux server fills 803 from `/bin/ps
auxwww` (`FUN_0805b454`), so each field should map to a `ps` column.

## Gaps

- [ ] 803 header: 0x00 and 0x08, "about 33 GB; host memory" [I]
  (`ProcessList::memory_00`, `memory_08`); 0x10, 0xc bytes.
- [ ] 803 process entry: 0x04 (4 bytes); 0x10 (0x29 bytes, perhaps CPU,
  memory, and state [I]); 0x3b.
- [ ] 804: the u32 at 0x08 "must be 0, or nothing happens"
  (`ExitTerminateProcess::value_08`); flags bit 2, the "extra flag",
  which Linux rejects with 0x10005. Read the Windows server 1.600 only if
  the Linux code gives no meaning.
- [ ] 800/801: confirm the payload is empty or ignored.
