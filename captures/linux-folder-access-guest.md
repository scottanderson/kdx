# Scenario: linux-folder-access-guest

- Date: 2026-10-12T06:19Z
- Server: KDXServer 1.620 Linux (sha256 ad6fba4bdd01dc23)
- Client: linux 1.620 beta 5
- Server state: the server from linux-accounts-folder-access (admin /
  lab-only-pw!1), with folder access items `[Default]`, `DB`, `test`, and
  `UL`.

## Steps

This checks which class rule a column of the Folder Access editor sets.

1. Connected as admin (port 54824, user 1).
2. Folder Access: created `lab-guest` and saved (1106/1107 at 17/19, 1100 at
   21, Status at 24, 1107 at 25).
3. Opened `lab-guest` (1103/1104 at 27/28), ticked Can See Folder for Guest
   only, and saved (1105 at 30, Status at 33).
4. Opened it again (1103/1104 at 37/38), unticked Guest, ticked Can See
   Folder for User only, and saved (1105 at 40, Status at 43).
5. Right-clicked `lab-guest`, Delete: 1101 at 47, 1102 at 48. Quit the
   client.

## Observations

- The Guest tick sets byte 0x14 to `80` (30); the User tick sets byte 0x10
  to `80` and clears 0x14 (40). With User as class 1 and Guest as class 2
  (506 at 18), the rule for a class is at 0x0c + 4 × class index.
- The 1104 at 38 returns the 1105 from step 3 byte for byte, with the
  current name length at 0xb1 and the name twice (204 bytes). The 1104 at
  28, after 1100, has no current name (196 bytes).
