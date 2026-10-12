# Scenario: linux-tracker

- Date: 2026-10-12T07:13Z
- Server: KDXServer 1.620 Linux (sha256 ad6fba4bdd01dc23)
- Tracker: KDXTrackerServ 1.110 Linux, fresh. First-run console setup with
  admin login `admin`, password `lab-tracker-pw!1`.
- Client: linux 1.620 beta 5, LANG C.UTF-8
- Server state: the server from linux-text-encoding (admin /
  lab-only-pw!1, name `Lab Server`, greeting `Welcome to the lab.`).

Decode everything, the registrations included, with `kdx-dissect
captures/linux-tracker.pcapng --password 'lab-only-pw!1' --password
'lab-tracker-pw!1'`.

## Steps

The capture follows `.scratch/capture-gaps/issues/05-tracker.md`, plus a
group edit at the end (step 9).

1. Connected to the server as admin (port 42252, user 1; frames 4–14).
   Server Settings → Trackers → Add Tracker: Name `lab`, Comments
   `labcomments`, Address `127.0.0.1`, Login `trk-login`, Groups
   `Lab, Lab2` (one text field; the comma was typed). Saved the entry and
   the settings: 170/171 at 17/18, 172/173 at 20/21. The server sent the
   first registration at 23 (UDP to 10800).
2. A guest connection to the server (port 55444, user 2, 42–53) was opened
   too; it stays idle until step 8.
3. Connected to the tracker as guest (port 60698, 27–32). 700/701 at 33/34
   lists eight groups. Opened General (702/703 at 36/37: `Lab Server`), then
   Chat (57/58: empty).
4. Commands → the tracker → Get TrackerServ Settings as guest: 710 at 60,
   1 Status 0x10014 at 61 ("Access Denied. (ID 1/20)").
5. Connected to the tracker as admin (port 41900, 69–74; 700/701 at 75/76).
   Get TrackerServ Settings: 710/711 at 79/80. Set the name to `Lab
   Tracker`, the description to `Description`, the greeting to `Tracker
   greeting` with Enable Greeting, and the admin password to
   `lab-tracker-pw!2`; Save: 712/713 at 82/83. Statistics: 720/721 at 85/86.
   The second registration came at 78, 400.05 s after the first.
6. Reconnected as admin with `lab-tracker-pw!2`: refused (port 48232, proof
   status 1 at 104). Reconnected with `lab-tracker-pw!1` (port 46846): 107
   `Tracker greeting` at 117 right after login, 700/701 at 119/120, then the
   client fetched every group (702/703 at 122–144; only General at 134/135
   has the server). Third registration at 146.
7. Connected as guest (port 43808; 107 at 159, 700/701 at 160/161).
   Statistics: 720 at 163, 1 Status 0x10014 at 164.
8. Connected as admin (port 37720, 172–218), opened the settings (711 at
   184) and browsed General and Education, but closed the window without
   saving. The guest server connection asked for the server settings: 170
   at 186, 1 Status 0x10014 at 187.
9. Connected to the server as admin again (port 57100, user 3; 195–209).
   Connected to the tracker as admin (port 34704; 107 at 232, 711 at 237),
   added the group `Lab` / `lab group`, Saved: 712/713 at 240/241. 701 at
   247 lists nine groups; 702 `Lab` at 249 is empty (250). Fourth
   registration at 239 (groups unchanged). Changed the server's tracker entry
   to Groups `Lab` and saved the settings (172/173 at 252/253);
   registration at 255. 702 `Lab` at 256 now lists `Lab Server` (257).

## Observations

- Each registration is 92 bytes and decrypts with the LCG. The checksum
  passes when the field holds the bytes `ca d3 1e cc` while the CRC runs,
  not `cc 1e d3 ca`; the result is stored big-endian (`40 17 b3 a7` at 23).
- The registration at 23 carries port 0x29cc, users 1, len1 9 (`trk-login`),
  name `Lab Server`, and group 0 `Lab, Lab2`. Comments, Address, and the
  password do not appear. 78 has users 2 (the guest of step 2); 146 and 239
  are byte for byte the same as 23. 255 has group 0 `Lab`.
- Registrations at 81.39, 481.44, 881.53, and 1281.63 s: every 400 s.
  The save at 252 sent 255 at once.
- `Lab, Lab2` matches no tracker group, so the record went into General. The
  General record in 701 at 34 counts 1 server.
- 703 at 37: first seen `00 00 03 a3 80 f5 a1 44`, which is 07:15:33.828 UTC
  in milliseconds since 1900, the arrival time of 23. Flags 0, users 1,
  address `7f 00 00 01 29 cc`, name `Lab Server`, no description.
- The tracker's 1 Status is 8 bytes: the code, then the request type
  (`00 01 00 14 00 00 02 c6` at 61 for 710; `… 00 00 02 d0` at 164 for
  720). The server's 24-byte Status at 187 starts the same way (`… 00 00 00
  aa` for 170).
- 711 at 80 is 660 bytes and sets every present byte (0x6c–0x71: `01 01 01
  00 01 01`). The four limits at 0x78 are 0. Upstream trackers and the
  greeting are empty sections. The groups are records of 0x1c bytes plus
  the strings, padded to 4. The Trackers group's flag at +0x0c is 0. The
  admin section has flags 0 at +0x07 and login `admin`.
- 712 at 82 sets the present bytes `01 01 01 00 01 00`: admin (0x71) is 0,
  so the tracker ignored the new password key in the admin section (flags
  `01` at +0x07, key at +0x08). The groups section has offset and size 0,
  and the tracker kept its groups. Flags at 0x98 are `00 00 00 02` (Enable
  Greeting).
- 712 at 240 carries the nine groups in full, with `Lab` between General
  and Macintosh, and an admin section with flags 0 (no password typed).
- 721 at 86: start time 07:13:51.362, connection counter 2, UDP counter 2,
  hours 0, open connections 1, servers 1. The client's window shows the same.
