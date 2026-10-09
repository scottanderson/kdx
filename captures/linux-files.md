# Scenario: linux-files

- Date: 2026-10-09T19:51Z
- Server: KDXServer 1.620 Linux (sha256 ad6fba4bdd01dc23)
- Client: linux 1.620 beta 5
- Server state: the server from linux-first-connect (admin / lab-only-pw!1,
  guest enabled)

## Steps

The operator followed a script; the steps below are what the capture shows.

1. Connected as admin (port 59246) and browsed the root and `Uploads [UL]`.
2. Created `Uploads [UL]/lab-folder`.
3. Uploaded `upload.txt` (6 bytes) into `lab-folder` on a transfer
   connection, renamed it to `upload2.txt`, made an alias, and moved files
   between folders.
4. Downloaded files, including `cli-big.bin` from the root.
5. Searched for "upload".
6. Opened folder access and created an item for `lab-folder`.
7. Moved `lab-folder` to the trash and emptied it.

## Observations

- 452 File Search failed with 1 Status error 0x640001.
- Transfer connections (ports 45422, 50904, 51076, 54196) send a hello whose
  name field is `00 00 00 00 01` instead of a login.
- While the client was running, `kdx-lab status` showed it listening on TCP
  port 10600. No connection to it appears in the capture.
- First captures of 402, 403, 405, 407, 411, 421, 452, 1100, and 1104.
