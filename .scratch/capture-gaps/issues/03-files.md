# Files: catalog, search, launch, delete, rename, volumes

Status: ready-for-human
Scenario: `linux-catalog-volumes`
Targets: 420, 422, 423, 450, 451, 453, and 143 transfer records

Setup (agent): server detached; `Bases/Default/lab-files/` with `a.txt`,
`b.txt`, and `hello.sh` (prints a line).

1. Connect as admin; Files button; open `lab-files`.
2. Commands → 127.0.0.1 → Generate File Catalog (450/451).
3. Commands → 127.0.0.1 → Search Files..., search `a.txt` (452/453).
4. Right-click `hello.sh`, Launch Program (420; an error reply is fine).
5. Account Classes → Administrator: tick See All Volumes, Save. Files
   button, go to the top level; the volumes should be listed. Right-click
   one, Get Info (409, reply 422 Volume Info). Change its Comments, Save
   (423; no reply expected). Untick See All Volumes afterwards.
6. Commands → 127.0.0.1 → Connection Monitor (140, then 142/143). Leave it
   open. Agent starts `kdx get` of a large file at a speed limit so the
   transfer lasts several seconds; wait for the monitor to refresh twice
   (143 with a transfer record after the user's record). Close the monitor
   (141).

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
