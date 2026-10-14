# Scenario: kdx-cli-tracker

- Date: 2026-10-12T20:02Z
- Server: KDXServer 1.620 Linux (sha256 ad6fba4bdd01dc23)
- Tracker: KDXTrackerServ 1.110 Linux (sha256 80dbc5658717a0fe), fresh.
  First-run console setup with admin login `admin`, password
  `lab-tracker-pw!1`.
- Client: none; the `kdx` CLI (this repository) only, and `kdx-probe` for
  the server settings
- Server state: fresh (admin / lab-only-pw!1, guest access yes)

Decode the tracker side with
`kdx-dissect captures/kdx-cli-tracker.pcapng --password 'lab-tracker-pw!1'`, and
the server side with `--password 'lab-only-pw!1'`.

## Steps

Each step is one `kdx tracker` run against `127.0.0.1` (port 10800 by
default), as guest unless given.

1. `groups` (port 57336, 700/701 at 10/11): eight groups, each with 0
   servers.
2. `servers General` (57338): 702 at 24, empty 703 at 25. The command prints
   nothing.
3. `servers Nope` (57354): 702 at 38, 1 Status 0x1000c at 39.
4. `stats` (57362): 720 at 52, 1 Status 0x10014 at 53.
5. `stats --login admin` (57378): 720/721 at 66/67.
6. `kdx-probe --login admin 172:…` to the server (port 53770, 74–85) sent
   the 172 of `linux-tracker` frame 252: name `Lab Server`, a tracker entry
   for `127.0.0.1` with Login `trk-login` and Groups `Lab`. 173 at 85, and
   the server registered at once (UDP, 86).
7. `groups` (53546, 100/101): General has 1 server. `servers General`
   (53556, 114/115): `Lab Server`. `stats --login admin` (53572, 128/129).
8. `groups --login admin` with a wrong password (53576): proof status 1 at
   141. `servers 127.0.0.1:10800 General` (53590, 154/155).

## Output

```
$ kdx tracker groups 127.0.0.1          # step 7
    0  Business
    0  Chat
    0  Education
    0  Games
    1  General
    0  Macintosh
    0  Trackers
    0  Windows
$ kdx tracker servers 127.0.0.1 General
127.0.0.1:10700           1 users  Lab Server
$ kdx tracker stats --login admin 127.0.0.1
started:      2026-10-12 20:02:16.921 UTC
hours:        0
connections:  8 accepted, 1 open
datagrams:    1
servers:      1
$ kdx tracker servers 127.0.0.1 Nope    # step 3
Error: server refused 702 Get Tracker Server List: error 0x1000c
$ kdx tracker stats 127.0.0.1           # step 4
Error: server refused 720 Get TrackerServ Statistics: error 0x10014
$ kdx tracker groups --login admin 127.0.0.1   # step 8, wrong password
Error: server proof

Caused by:
    peer rejected the handshake: Other(1)
```

## Observations

- The tracker sends no 107: a fresh tracker has no greeting.
- An unknown group gives 0x1000c in the tracker's 8-byte Status (`00 01 00
  0c 00 00 02 be` at 39).
- `Lab` matches no group on a fresh tracker, so the record went into
  General, as `Lab, Lab2` did in `linux-tracker`.
- The registration at 86 is 88 bytes, 4 fewer than `linux-tracker` frame 23
  for the shorter group string. It went out with the 173, while `kdx-probe`
  was still logged in, so the listing shows 1 user.
- The connection counter counts the stats connection itself: 5 at step 5
  (steps 1–5), 8 at step 7 (plus its three). The UDP counter is 1, the
  registration at 86.
