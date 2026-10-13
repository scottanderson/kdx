# Server upgrade

Status: ready-for-human
Scenario: `linux-upgrade`
Targets: 175, the 151 it broadcasts, and the relaunched server's 105

Upgrade the lab server from 1.620 to 1.600 so the version in 105 Server Info
proves that KDXUpgradeHelper swapped the binary. `kdx-lab server` already
installs the helper and keeps the sandbox up for it (`lab/README.md`, Server
upgrade).

Setup (agent):

1. Find the 1.600 Linux server under `haxial.com/download/old/` in the
   Wayback Machine, fetch it, and add it to `vendor/MANIFEST.md` with its
   checksum. `vendor/fetch` builds URLs under `download/` only.
2. Back up `lab/run/server/`; 1.600 may rewrite 1.620's files.
3. Put the 1.600 binary at `lab/run/server/Bases/KDXServer.lexe`.
4. `kdx-lab up linux-upgrade`, `server -d`, `client linux`.

Steps (operator):

1. Connect as admin.
2. Commands → 127.0.0.1 → Upgrade Server Software…, tick Restart KDX only,
   Upgrade (175; expect 151 "Upgrading server software, be back in 5
   seconds!").
3. Wait 5 seconds, reconnect, Get Server Info (expect version 1600).

Don't tick Restart Computer, and don't send 174 with the restart or
shut-down choices: they run `/sbin/shutdown` (`FUN_080916e4`).

## Comments

2026-10-12: Tabled until the Internet Archive is back; its CDX API returned
504 and 503. Shows whether a ticked Restart KDX adds bytes to the 175 body,
which the server doesn't read (`captures/linux-shutdown.md`).

2026-10-12: The archive is back. `vendor/fetch old/KDXServer1600-Lnx.zip`
fetches the 1.600 server (Wayback 20060515145257, the only capture besides
an identical one from 2007). Its KDXUpgradeHelper.lexe is byte-identical to
1.620's.
