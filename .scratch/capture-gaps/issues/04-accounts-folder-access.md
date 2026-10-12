# Account changes and folder access edits

Status: done
Scenario: `linux-accounts-folder-access`
Targets: 504, 1101, 1102, 1105, and the flood warnings (155)

Setup (agent): server detached; account `peer` (class Users) created with
`kdx account add`; `kdx listen` logged in as `peer`.

1. Connect as admin.
2. Commands → 127.0.0.1 → Account Classes. Open Users, set Flood
   Protection to Very Strict, Save (504; expect 1 Status 0x1f8).
3. Commands → 127.0.0.1 → Folder Access. Create an item named `lab-item`,
   Save (1100).
4. Open `lab-item`, change one tick for the Guest class, Save (1105).
5. Right-click `lab-item`, Delete (1101/1102).
6. Agent only, after the GUI steps: as `peer` (class Users, now Very
   Strict = 20 requests per 2 minutes), send 30 pings within a minute with
   the CLI. Expect 155 warnings at the 20th and 24th request, then a ban
   and disconnect (0x640004) on the 29th. Restart the server afterwards
   to clear the 2-hour ban.

## Comments

2026-10-11: Read statically; see `docs/protocol/KDXProtocol.md` (504,
folder access, Flood ban). 514 Account Changed is never sent by the server
and not handled by the client, so it is dropped. 1101 gets no Status: the
server sends 1102 with the name. 1105 picks the item by the current name at
0xb1. Flood Protection is the class byte 0x25 (Very Strict 20, Strict 40,
Average 60, Relaxed 80, Very Relaxed 120, Total Slacker 200, Off 0).

2026-10-12: Captured as `captures/linux-accounts-folder-access.pcapng`; see
`captures/linux-accounts-folder-access.md`. The static reading held: 504
Status 0x1f8, 1100 Status 0x44c, 1105 Status 0x451, and 1102 instead of a
Status for 1101. 1102 carries the server's own txid, not 0. The flood count
includes the login's 100, so the warnings came on the 19th and 23rd
request and the disconnect on the 28th (messages 20, 24, and 29). The ban
did not block 127.0.0.1, so the server needed no restart. Step 4 set bit 0
of the Guest rule at 0x14.

2026-10-12: `captures/linux-folder-access-guest.pcapng` confirms the column:
a Guest tick alone sets 0x14 and a User tick alone 0x10, so the class rules
start at 0x0c + 4 × class index.
