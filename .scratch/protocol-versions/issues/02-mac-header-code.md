# Confirm the Mac builds' header code

Status: ready-for-agent

The 1.x Mac builds (Client 1.320–1.600 and Server 1.300–1.600 PEF, Server
1.520 and 1.600 Mach-O, Tracker 1.110 PEF and Mach-O) contain the scramble
constants (`re/scan_consts.py`), but none of them is decompiled, so
framing.md marks their header handling **[I]**. Done when the receive and
header-only send functions of at least one PEF and one Mach-O build are found
and compared with the x86 ones, and the tags are updated.
