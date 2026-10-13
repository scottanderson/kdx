# News: 600–614

Status: ready-for-agent
Type: research
Targets: 603 (600/604), 606, 608, 610, 611

Docs: `KDXProtocol.md`, News. Captures: `linux-accounts-news`,
`linux-first-connect`, `kdx-news`, `linux-text-encoding`.

## Gaps

- [ ] 606 Newsgroup List entry: 0x0c (4 bytes) and 0x12 (4 bytes).
- [ ] 606 0x10, "message count, 1" [I], and 0x17, "probably the
  description length" [I]: existing captures settle both, so update the
  doc. 0x10 is 3 for General News in `linux-accounts-news` frames 56 and
  105, whose 608 at 108 lists three messages with IDs 1, 6, and 7: a
  count, not the highest ID. It moves between 1 and 2 as `kdx-news` posts
  and deletes (frames 59, 321, 460). 0x17 is 4 for `desc`
  (`linux-accounts-news` frame 63) and 0x12 for `edited description`
  (frame 76).
- [ ] 603 Newsgroup Info: 0x04 (0xc bytes of 0); 0x20, "a third class
  bitmap", `01 00 00 00` from the server and 0 from the client
  (`NewsgroupInfo::unknown_20`); 0x24 (0xc bytes); 0x34 (8 bytes); 0x3e
  (2 bytes). The struct still calls the archive size "probably the size
  limit"; the doc reads it [S].
- [ ] 608 News Messages: the u32 at 0x08.
- [ ] 610 News Message record: 0x10 (8 bytes of 0), 0x1a (2 bytes), 0x1f.
- [ ] 611 Post News Message: 0x04 (8 bytes) and 0x0f are ignored by the
  server [S]. Read what the client writes there.
