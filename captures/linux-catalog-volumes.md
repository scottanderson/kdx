# Scenario: linux-catalog-volumes

- Date: 2026-10-12T03:53Z
- Server: KDXServer 1.620 Linux (sha256 ad6fba4bdd01dc23)
- Client: linux 1.620 beta 5, and the `kdx` CLI for the download
- Server state: the server from linux-server-settings (admin / lab-only-pw!1,
  server name "Lab Server"), plus `Bases/Default/lab-files/` with `a.txt`
  and `b.txt` (6 bytes each) and `hello.sh` (30 bytes, prints a line), and
  `speed.bin` (2 MiB) in the base. Administrator starts without See All
  Volumes (`e8` in byte 2 of its privileges) and with no speed limit.

## Steps

The capture follows `.scratch/capture-gaps/issues/03-files.md` in one pass.
The client's connection is port 58160, user 1. The download's transfer
connection (port 53692) was trimmed out: it only carried 412/413/414 and the
file's 2 MiB, which `linux-server-settings` already has.

1. Connected as admin (frames 4–15), listed the root (400/401 at 17/18) and
   `lab-files` (20/21).
2. Commands → 127.0.0.1 → Generate File Catalog: 450 at 23, then two 451 at
   24 and 26.
3. Search Files... for `a.txt`: 452/453 at 28/29, one result.
4. Right-clicked `hello.sh`, Launch Program, and confirmed: 420 at 31,
   1 Status success at 32. The server ran the script; `hello from lab` went
   to the server log.
5. Account Classes → Administrator: ticked See All Volumes and saved (502/503
   at 37/38, 504 at 40, Status at 43, 506 at 45). The root listed nine
   volumes (400/401 at 47/48). Get Info on `/tmp` (409 → 422 at 50/51).
6. In the Volume Info window, renamed `/tmp` to `/tmp-renamed` and saved: 423
   at 53, 1 Status error at 54. Closed the window.
7. Account Classes → Administrator: unticked See All Volumes and saved
   (502/503 at 56/57, 504 at 59, Status at 62, 506 at 64).
8. Server Settings: ticked Total Outgoing Speed Limit, 20 KB/s, and saved
   (170/171 at 66/67, 172/173 at 69/70).
9. Connection Monitor: 140 at 75, then 142/143 every few seconds from 76.
   The CLI ran `kdx get speed.bin --nick kdx-get` (session port 53680, user
   2, frames 86–98). 419 reached its session at 100. The download took
   1 m 40 s and matched the file; 104 User Departed followed at 164. Closed
   the monitor with its close button (141 at 172).
10. Server Settings: unticked the speed limit and saved (172/173 at 174/176).
    Quit the client.

## Observations

- 450 gets no reply with its txid. The server sends two 451: `00000000
  00000002` (24), then `00000001 0000000b` (26).
- 453 holds a count (1), then an entry like 401's whose name is a path
  with two parts, `lab-files` and `a.txt` (29).
- 420's success Status carries 0x1a4, which is 420 (32).
- 504 at 40 changes byte 2 of the privileges from `e8` to `ea`: bit 22, See
  All Volumes; 504 at 59 changes it back.
- With See All Volumes, the root lists the server's mount points with size
  0xffffffffffffffff (48).
- 422 marks the name editable and the comments read-only (`00000001` at
  0x44, `00000001` at 0x48), and ends with the file system name, `tmpfs`
  (51).
- 423 sets flag bit 0 only, with the current name `/tmp` (length 4 at 0x4e)
  and the new name `/tmp-renamed` (length 0xc at 0x4f) (53). The server
  replies with error 0x10005 at module 0x46, line 2175 (54); nothing on disk
  changes.
- Right after the 172/173 at 69/70, the client reads Server Settings again
  (170/171 at 72/73). It sent no 170 before the 172 at 174.
- The speed limit is `50 00` at offset 0xaa in the 172 at 69 and the 171 at
  73, and `00 00` in the 171 at 67 and the 172 at 174.
- 143 holds one record per connection: 148 bytes for the operator alone
  (78), 452 bytes with the `kdx-get` user and its transfer (104 to 161).
  The transfer record (transfer ID 3) holds 0x200000 (the file's size), the
  bytes done and rate, 0x5000 (the speed limit) at 0x38, 0x19c (412) at
  0x3c, and the path `speed.bin`.
- No 142 follows the 141 at 172.
