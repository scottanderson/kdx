# Tracker: registration, 115/116, 700–721

Status: ready-for-agent
Type: research
Targets: the UDP registration, 701, 702, 703, 711/712, 721, 115/116 and 107
on a tracker

Docs: `tracker.md`. Captures: `linux-tracker`, `kdx-tracker`,
`kdx-tracker-gui`, `kdx-cli-tracker`. Use Tracker 1.110 for Linux
(`kdxtracker`); the Windows tracker only if the Linux one can't settle a
gap.

## Gaps

UDP registration (server `FUN_08060e4c`, tracker `FUN_0804c048`,
`FUN_0804b604`):

- [ ] 0x1c (8 bytes of 0), 0x26 (13 bytes of 0), 0x3e (2 bytes of 0).
- [ ] len0 at 0x33 and len4 at 0x37 are always 0, and their strings are
  never sent. Find a writer or reader for each.
- [ ] The server ID at 0x0c: Tracker 1.110 ignores it. Check whether
  anything else reads it.
- [ ] A tracker's own registration to its upstream trackers (flags bit 0,
  `FUN_0804df70`) has not been captured. It needs two Linux trackers in
  the lab, or Tracker 1.110 registering with `kdx-tracker`; `kdx-lab`
  runs one tracker today. Hand it off as a capture-gaps issue. Every
  registration on record comes from a server.

TCP:

- [ ] 115/116 on a tracker: handled inline **[S]** but never captured; every
  115 on record goes to a server on port 10700. A Linux capture with
  `kdx-probe` or the CLI is enough.
- [ ] 701 group record: 0x02, 4 bytes "not set by the tracker"
  (`GroupListEntry::value_02`).
- [ ] 702: 0x00 (2 bytes of 0) and 0x03.
- [ ] 703 server record: 0x08 (6 bytes of 0), 0x12 (6 bytes of 0), and the
  extra string (`extra_len`, always 0). Find what fills it.
- [ ] 711/712 general section: the four u16 limits at 0x78..0x7f ("file
  only"), and the bytes not named between 0x80 and 0xc3.
- [ ] 711/712 section 3 and sections 6–11: always absent. Find a reader.
- [ ] 711/712 upstream tracker entry: the 12 raw bytes at +0x08 and the
  "unused" string.
- [ ] 711/712 group: the two u16 at +0x10 and +0x12 "which nothing sets"
  (`SettingsGroup::value_10`, `value_12`).
- [ ] 711/712 admin section: the bytes other than the flags at +0x07, the
  key, the login length, and the login.
- [ ] 712 with the admin section present (byte 0x71 = 1): Client 1.620 beta
  5 never sets it (`linux-tracker` frames 82, 240). Test it with
  `kdx-probe` against Tracker 1.110, so the admin login and password change
  is confirmed.
- [ ] 721: 0x18 (8 bytes of 0) and 0x2c (12 bytes of 0).
- [ ] The units of the 850 and 600 sweep values and the 1200 idle limit,
  "probably seconds" **[I]**. Read the clock they compare against.
