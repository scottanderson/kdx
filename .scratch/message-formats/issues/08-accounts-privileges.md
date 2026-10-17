# Accounts and privileges: 502–515, 550, 551

Status: ready-for-agent
Type: research
Targets: 503/504/506, 507/510/511, 513, 514, the privilege bits

Docs: `messages/accounts.md` and `privileges.md`.

Several unnamed bytes may be colours: 506 offset 0x29 is `ff 00 00` for
Administrator and `00 00 ff` for User, and privilege bit 3 is Color
Overrides Class. <!-- [I] --> Check them together with the user record's 0x14
(issue 02).

## Gaps

- [ ] 506 Account Class List entry: 0x18, 0x10 bytes of 0; 0x29, 3 bytes.
- [ ] 503 Account Class Info: 0x18 (0xd bytes); 0x26 (2 bytes, "stored as
  given"); 0x28 (8 bytes, left out of 506); 0x31, "probably the colour"
  <!-- [I] --> (`AccountClass::unknown_after_index`); 0x32..0x33; the second
  string. `AccountClass::unknown_25` is the flood protection byte the doc
  names; rename it.
- [ ] Account record (507, 510, 511): 0x58, 0x1e bytes of 0; 0x77, 3 bytes
  "stored as given" (`Account::unknown_77`); the third string
  (`Account::third`); 0x7e..0x7f.
- [ ] 513 Account List entry: 0x10 (0xb bytes); 0x1b, 1 for admin and 0 for
  guest (`AccountListEntry::unknown_1b`); 0x1c (4 bytes); 0x21, 0xff for
  admin, like 503 0x31 (`unknown_21`); 0x22..0x23; 0x26..0x27.
- [ ] A Linux capture that gives a class and an account a colour, if the
  static reading leaves it open. No capture changes a colour. Accounts
  with and without a password are already captured: `kdx-accounts` step 3
  (`rs-test`) and step 8 (`rs-nopw`); 510 byte 0x20 is 1 and 0.
- [ ] 514 Account Changed: Server 1.620 never sends it and Client 1.620 beta
  5 doesn't handle it. Check the Windows server 1.600 and client 1.600 for
  a sender and handler.
- [ ] Privilege bit 2 is set in every class and has no name. Bits with no
  client label: confirm no Linux server handler tests them, or name them.
- [x] Audit (01): `AccountClass` dropped the editor login length at 0x36 and
  the editor login, so 503 came back 4 or 8 bytes short
  (`linux-accounts-news` 28; `linux-catalog-volumes` 38, 57;
  `linux-chat-rooms` 143, 181; `linux-accounts-folder-access` 41). Fixed:
  `AccountClass` decodes the second string and the editor login, and
  `padding` keeps the bytes after them.
- [ ] Other 503s have one non-zero byte in their last 3
  (`linux-accounts-news` 18 and `probe` 196 at 0x3d, `probe` 193 at 0x45),
  now kept in `AccountClass::padding`; confirm it is uninitialised.
- [x] Audit (01): 513 Account List dropped byte 0x97, ff (`kdx-accounts`
  271). Fixed: `AccountListEntry` keeps every header gap and its padding.
- [ ] Which 513 field holds that byte, and what is it?
- [x] Audit (01): no codec for 502 Get Account Class, 508 Delete Account,
  509 Get Account, 515 Account Deleted, and 550 Change Own Password
  (`linux-accounts-news` 17, 52, 46, 53; `kdx-accounts` 323). Fixed:
  `AccountClassIndex`, `AccountLogin`, `ChangeOwnPassword`.
