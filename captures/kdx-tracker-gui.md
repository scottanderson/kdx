# Scenario: kdx-tracker-gui

- Date: 2026-10-13T03:03Z
- Server: KDXServer 1.620 Linux (sha256 ad6fba4bdd01dc23), state from
  `kdx-cli-tracker`: name `Lab Server`, a tracker entry for `127.0.0.1`
  with Login `trk-login` and Groups `Lab`.
- Tracker: `kdx-tracker` from this repository, run in the lab network with
  `lab/kdx-lab run`, settings from `kdx-tracker`, admin / `lab-tracker-pw!1`.
- Client: linux 1.620 beta 5, driven by hand.

Decode the tracker side with
`kdx-dissect captures/kdx-tracker-gui.pcapng --password 'lab-tracker-pw!1'`.

## Steps

1. Started the server, then the tracker:

   ```sh
   KDX_TRACKER_PASSWORD='lab-tracker-pw!1' lab/kdx-lab run \
     target/debug/kdx-tracker --settings lab/run/kdx-tracker/kdx-tracker.stg \
     --admin-login admin --table lab/run/server/KDXServer.lexe
   ```

   The server's start-up registration went out before the tracker was
   listening, so the server was restarted. It registered at frame 1, and
   the tracker logged `registered "Lab Server" port 10700`.
2. Started `lab/kdx-lab client linux`.
3. Browsed the tracker at `127.0.0.1` as guest (port 38066): 700/701 at
   11/12, 702/703 for General at 14/15. The client listed `Lab Server` at
   `127.0.0.1:10700`.
4. Logged in to the tracker as `admin` (port 39886): 700/701 at 26/27,
   702/703 at 32/33, and 710/711 at 35/36.

## Observations

- Client 1.620 beta 5 lists a server through `kdx-tracker`, as guest and as
  admin.
- The client's hello has sub-version 0x10, and its name is empty as guest and
  `admin` as admin (frames 5 and 20).
- The registration at 1 is 88 bytes with users 0, as no one was logged in
  to the server. `Lab` matches no group, so the record is in General.
- 703 at 15 and 33 lists `Lab Server` with users 0 and first seen
  03:04:21.974 UTC, the arrival time of frame 1.
