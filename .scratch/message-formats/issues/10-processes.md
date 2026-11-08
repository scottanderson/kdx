# Processes: 800–804

Status: done
Type: research
Targets: 803 Process List, 804 Exit/Terminate Process

Docs: `messages/processes.md`. The Linux server fills 803 from `/bin/ps
auxwww` (`FUN_0805b454`), so each field should map to a `ps` column.

## Gaps

- [x] 803 header: 0x00 and 0x08 are memory used and total
  (`ProcessList::memory_used`, `memory_total`); 0x10, 0xc bytes, are 0.
- [x] 803 process entry: 0x04 is the high word of a Mac process serial
  number; 0x10 to 0x29 are Mac memory, CPU time, background, `%CPU`, and
  `%MEM`; 0x3b is always empty on Linux. Read from Server 1.000 for Mac.
- [x] 804: the u32 at 0x08 is the high word of a Mac process serial
  number; flags bit 2 is restart (`ExitTerminateProcess::RESTART`).
- [x] 800/801: the payload is empty and ignored (`linux-chat-admin`
  frames 153 and 157).
- [x] Audit (01): 803 Process List lost byte 0x1dc, 32
  (`linux-chat-admin` 155). Fixed: `Process::padding` and the header gaps
  keep it.
- [x] Is that byte uninitialized padding after a process entry? No: it is
  `cpu_percent` of `/bin/ps`.
