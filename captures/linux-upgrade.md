# Scenario: linux-upgrade

- Date: 2026-10-13T00:16Z to 00:17Z
- Server: Server 1.620 for Linux, sha256
  `ad6fba4bdd01dc237abde2b76054e3bbbb43c6d0c79a28d4e0de3b23e546f943`, then
  Server 1.600 for Linux, sha256
  `f57467a724087a578d940511d3a5642119fb5cac3fd82354c4ee5f9d76f3075e` after the
  upgrade
- Client: Client 1.620 beta 5 for Linux, sha256
  `e7189c55b67a89f35edfdf09570a1d55a096b198579b8633f3adfa50f9d13961`
- Server state: the server from linux-shutdown, with admin / `lab-only-pw!1`,
  backed up to `lab/run/server-backup-1620/`. `Bases/KDXServer.lexe` is Server
  1.600 for Linux, and `KDXUpgradeHelper.lexe` sits next to the server.
- Decode: `kdx-dissect captures/linux-upgrade.pcapng --password 'lab-only-pw!1'`

## Steps

1. The client connected as admin, nick `Zoë-日本` (port 38242, user 1). 105
   at 14 reports version 1620.
2. Commands → 127.0.0.1 → Upgrade Server Software…, ticked Restart KDX
   only, Upgrade (175 at 19, 151 at 20). The client disconnected at
   00:16:49 (server log).
3. The server came back at 00:16:54. The client reconnected as admin (port
   52706, user 1; 105 at 37), then Get Server Info (113 at 42, 114 at 43).

## Observations

- With Restart KDX ticked the 175 body is still empty (frame 19), as with
  both options unticked in linux-shutdown. The option doesn't reach the
  server.
- No 1 Status answers 175. The 151 at 20 has user 1, flags 4, and
  "Upgrading server software, be back in 5 seconds!".
- The 105 at 37 reports version 1600, and the 114 at 43 an uptime of 17 s.
  Login, greeting, and icon are unchanged: 1.600 reads 1.620's state.
- On disk, `KDXServer.lexe` became Server 1.600 for Linux, sha256
  `f57467a724087a578d940511d3a5642119fb5cac3fd82354c4ee5f9d76f3075e`, and
  `Bases/KDXServer.lexe` is gone: the helper moved it. 1.620 rewrote
  `Accounts.dat` and `KDXServer.stg` as it exited.
- The server log has no line for 175, only `Disconnected` and the new
  server's `KDX Server started`.
