# Scenario: linux-shutdown

- Date: 2026-10-12T18:33Z to 18:36Z
- Server: Server 1.620 for Linux, sha256
  `ad6fba4bdd01dc237abde2b76054e3bbbb43c6d0c79a28d4e0de3b23e546f943`
- Client: Client 1.620 beta 5 for Linux, sha256
  `e7189c55b67a89f35edfdf09570a1d55a096b198579b8633f3adfa50f9d13961`
- Server state: the server from linux-text-encoding, with admin /
  `lab-only-pw!1` and peer / `lab-peer-pw!1`
- Decode: `kdx-dissect captures/linux-shutdown.pcapng --password
  'lab-only-pw!1'`

Targets: 175 (upgrade), 804 (exit process), 174 (shutdown).

## Steps

1. The client connected as admin, nick `Zoë-日本` (port 49124, user 1).
2. Commands → 127.0.0.1 → Upgrade Server Software…, left the restart KDX
   and restart computer options unticked, Upgrade (175 at 19, Status at
   20).
3. Commands → 127.0.0.1 → Process Monitor (800 at 22, 803 at 23), then
   right-click `./KDXServer.lexe` (PID 2) → Exit Process (804 at 25). The
   server exited and the connection dropped.
4. Server restarted; the client reconnected as admin (port 38848, user 1).
5. Commands → 127.0.0.1 → Shutdown Server…, message `lab shutdown`, exit
   the program only, confirm (174 at 47, 151 at 48). The server exited.

## Observations

- With both restart options unticked, the 175 body is empty (frame 19). It
  gets 1 Status 0x30067 (file missing), module 0x46, line 0x198 (frame
  20), not 0x1000c: the lab has no KDXUpgradeHelper.
- The 803 lists three processes in the sandbox: bwrap (PID 1),
  `./KDXServer.lexe` (PID 2), and the `/bin/ps auxwww` that built the list
  (PID 3).
- The 804 body is `00 00 00 02 00 00 00 02 00 00 00 00`: u32 2 (exit,
  SIGTERM), PID 2, then 0. The flags are a u32, so bit 1 is in byte 3, not
  byte 0. The server logs `Terminate Process` and sends no reply.
- The 174 body is `00 01 00 00` then `lab shutdown` with no length prefix.
  "Exit the program only" sends 1 in byte 1. The 151 at 48 has flags 4,
  user 1, and the message. The server logs neither 175 nor 174.
