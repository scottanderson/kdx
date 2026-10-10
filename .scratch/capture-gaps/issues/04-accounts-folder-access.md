# Account changes and folder access edits

Status: ready-for-human
Scenario: `linux-accounts-folder-access`
Targets: 514, 1101, 1102, 1105

Setup (agent): server detached; account `peer` (class Users) created with
`kdx account add`; `kdx listen` logged in as `peer`.

1. Connect as admin.
2. Commands → 127.0.0.1 → Accounts. Open `peer`, change its Name to
   `peer edited`, Save (511; expect 514 Account Changed [I]).
3. Commands → 127.0.0.1 → Folder Access. Create an item named `lab-item`,
   Save (1100).
4. Open `lab-item`, change one tick for the Guest class, Save (1105).
5. Right-click `lab-item`, Delete (1101/1102).
