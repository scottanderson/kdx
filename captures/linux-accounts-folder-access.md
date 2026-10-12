# Scenario: linux-accounts-folder-access

- Date: 2026-10-12T06:06Z
- Server: KDXServer 1.620 Linux (sha256 ad6fba4bdd01dc23)
- Client: linux 1.620 beta 5, the `kdx` CLI, and `kdx-probe`
- Server state: the server from linux-catalog-volumes (admin / lab-only-pw!1),
  plus account `peer` (class User, password `lab-peer-pw!1`). Class User
  starts with Flood Protection Average (`3c`). Folder access items
  `[Default]`, `DB`, `test`, and `UL`.

## Steps

The capture follows `.scratch/capture-gaps/issues/04-accounts-folder-access.md`
in one pass.

1. `kdx listen` logged in as `peer` (port 55730, user 1) and joined the
   public chat (frames 4–17). The client connected as admin `lab-linux`
   (port 55746, user 2, frames 22–33).
2. Account Classes → User: set Flood Protection to Very Strict and saved
   (505/506 at 37/38, 502/503 at 40/41, 504 at 43, Status at 46, 506 at 48).
3. Folder Access: created `lab-item` and saved (1106/1107 at 50/51, 1100 at
   53, Status at 56, 1107 at 58).
4. Opened `lab-item` (1103/1104 at 60/61), changed one tick, and saved (1105
   at 63, Status at 66, 1107 at 68).
5. Right-clicked `lab-item`, Delete: 1101 at 70, 1102 at 71.
6. `kdx-probe` logged in as `peer` with nick `peer-flood` (port 52182, user
   3, frames 76–86) and sent thirty 115 Get My Address, 300 ms apart, from
   frame 92. The server disconnected it at 178.
7. `kdx who` as `peer` with nick `peer-retry` (port 40734, user 4, frames
   188–199) logged in at once and left. Quit the client.

## Observations

- 504 at 43 sets the flood protection byte at 0x25 to `14` (Very Strict),
  where the 503 at 41 had `3c` (Average); the privileges are unchanged. The
  Status is success 0x1f8 (46).
- 1100 at 53 is a record of zeros with the name `lab-item` (length 8 at
  0xad); the Status is success 0x44c (56). 1104 at 61 returns the same 192
  bytes.
- 1105 at 63 is 200 bytes: the record with byte 0x14 set to `80`, the
  current name length 8 at 0xb1, and the name twice. 0x14 is the rule for
  class 1 (User), and `80` is bit 0, Can See Folder. The Status is success
  0x451 (66).
- 1101 at 70 is the name `lab-item`. The server replies with 1102 (71),
  payload the name, under its own txid (`f2cfecc3`, next after the login's
  `f2cfecc2`), not the request's. No Status follows, and the client does not
  ask for the list again.
- The flood count includes the login's 100: `peer-flood`'s 19th request
  (146, its 20th message) gets the WARNING 155 at 147 before its 116 reply;
  the 23rd (160, its 24th message) gets "Final warning" at 161. The 28th
  (177, 29th message) gets no reply; the server sends FIN at 178.
- Each 155 is 16 zero bytes and the text, with no length byte.
- The server log reads "Auto-banned for flooding!", then "Banned for
  flooding!", then "Disconnected" for user 3. The ban does not block
  127.0.0.1: `peer-retry` logs in 12 seconds later (188–199).
