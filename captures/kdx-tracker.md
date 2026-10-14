# Scenario: kdx-tracker

- Date: 2026-10-12T20:14Z
- Server: KDXServer 1.620 Linux (sha256 ad6fba4bdd01dc23), fresh: console
  setup with admin / `lab-only-pw!1` and guest access on.
- Tracker: `kdx-tracker` from this repository, fresh, run in the lab network
  with `lab/kdx-lab run`, admin / `lab-tracker-pw!1`.
- Client: `kdx-probe`, no GUI client.

Decode the server side with
`kdx-dissect captures/kdx-tracker.pcapng --password 'lab-only-pw!1'`, and the
tracker side with `--password 'lab-tracker-pw!1'`.

## Steps

1. Started the server, then the tracker:

   ```sh
   KDX_TRACKER_PASSWORD='lab-tracker-pw!1' lab/kdx-lab run \
     target/debug/kdx-tracker --settings lab/run/kdx-tracker/kdx-tracker.stg \
     --admin-login admin --table <KDXServer.lexe>
   ```

2. Logged in to the server as admin with `kdx-probe` (port 38842, frames
   1–13) and sent 172 Set Server Settings with the blob of
   `linux-tracker` frame 20: a tracker entry with Address `127.0.0.1`,
   Login `trk-login`, and Groups `Lab, Lab2` (14; 173 at 15). The server
   sent its registration at once (frame 16, UDP to 10800), and the tracker
   logged `registered "Lab Server" port 10700`.
3. Browsed the tracker as guest with `kdx-probe --addr 127.0.0.1:10800
   --no-login 700 702:0000070047656e6572616c` (port 36474, frames 21–38):
   700/701 at 30/31, 702/703 for General at 33/34.

## Observations

- The registration at 16 is 92 bytes, like `linux-tracker` frame 23, with
  users 1 (the probe).
- The tracker's hello at 26 has version 0x30, and its proof at 29 checks
  out against the empty password's key.
- 701 at 31 is the same as `linux-tracker` frame 34: eight groups, with 1
  server in General. 703 at 34 lists `Lab Server` at `127.0.0.1:10700`
  with users 1, the same as `linux-tracker` frame 37 but for the first-seen
  time.
