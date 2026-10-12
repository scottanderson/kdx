# Files: catalog, search, launch, delete, rename, volumes

Status: done
Scenario: `linux-catalog-volumes`
Targets: 420, 422, 423, 450, 451, 453, and 143 transfer records

Setup (agent): server detached; `Bases/Default/lab-files/` with `a.txt`,
`b.txt`, and `hello.sh` (prints a line); `speed.bin` (2 MiB) in the base.
Check the starting state from the first 506 and 171 the client gets: See
All Volumes clear for Administrator (byte 2 of its privileges is `e8`), and
no speed limit (`00 00` at 0xaa).

1. Connect as admin; Files button; open `lab-files` (400/401).
2. Commands → 127.0.0.1 → Generate File Catalog (450, then two 451).
3. Commands → 127.0.0.1 → Search Files..., search `a.txt` (452/453).
4. Right-click `hello.sh`, Launch Program, confirm (420, 1 Status 0x1a4).
5. Account Classes → Administrator: tick See All Volumes, Save (504 with
   `ea` in byte 2). Files button, go to the top level; the volumes are
   listed (400/401). Right-click `/tmp`, Get Info (409, 422 Volume Info).
6. In the Volume Info window, change the Name to `tmp-renamed` and Save
   (423 with flag bit 0, then a 1 Status error 0x10005). The rename is a
   stub on Linux, so nothing on disk changes. Comments stays read-only
   on this server. Close the window.
7. Account Classes → Administrator: untick See All Volumes, Save. The agent
   checks that the 504 has `e8` in byte 2 before the next step.
8. Server Settings: tick Total Outgoing Speed Limit, 20 KB/s, Save (172/173).
9. Commands → 127.0.0.1 → Connection Monitor (140, then 142/143). Leave it
   open. The agent runs, inside the lab:
   `lab/kdx-lab run target/debug/kdx get speed.bin --login admin --password 'lab-only-pw!1' --nick kdx-get -o /tmp/speed.bin`
   It takes about 100 s; `kdx-dissect` shows the 143 transfer records. After
   it ends, close the monitor with its close button and wait about 10 s
   (141).
10. Server Settings: untick the speed limit, Save (172 with `00 00` at 0xaa).

## Comments

2026-10-11: Read statically; see `docs/protocol/KDXProtocol.md` (Files and
142). 404 and 406 have no handler in Server 1.620 and no sender in Client
1.620 beta 5; the client renames with 411 and deletes with 405, both already
in `captures/linux-files.pcapng`. So they are dropped from the targets. 422
is the reply to 409 on a volume, which needs See All Volumes (22). The
capture should confirm the 422 field offsets, 423's lack of a reply, the 451
progress records, the 453 entry layout, and the 143 transfer record fields
(0x20 bytes done, 0x28 total, 0x30 rate, 0x34 seconds left, 0x38 and 0x3c
unnamed).

2026-10-12: A dry run (not kept) got 420, 450, 451, 452, 453, 422, and 143
transfer records, but no 423: the client left the volume's Comments box
read-only. Next: find what makes it editable, then recapture in one clean
pass.

2026-10-12: Comments can't be edited on Server 1.620 for Linux. The 422 it
sends marks only the name editable (`FUN_0804ef00`, `FUN_08096780`), and the
comments setter is empty. So the steps now rename a volume instead, which
still sends 423. The dry run lost the client's X connection after the
volume step; if that happens again, reconnect and carry on, and note it in
the capture's steps.

2026-10-12: Done. `captures/linux-catalog-volumes.pcapng` holds all ten
steps from one pass, without the download's transfer connection. 423
renaming `/tmp` gets error 0x10005 (frames 53/54), and the monitor's 141
follows the transfer (172).

