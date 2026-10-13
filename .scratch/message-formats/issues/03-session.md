# Session: 104–118

Status: ready-for-agent
Type: research
Targets: 104, 105, 107, 108/109, 110/111, 112, 113/114, 118

Docs: `KDXProtocol.md`, Session. 100–103 and the icon are in issue 02, 117
in issue 14.

## Gaps

- [ ] 104 User Departed: byte 0x04 is 7 in every capture; 0x05..0x07 are 0.
  `UserDeparted::value_4` holds it.
- [ ] 105 Server Info 0x00, 32 bytes: the table says "probably a session ID
  for transfer connections" [I], while Transfer connections says it is the
  account key for an attached login. Reconcile them.
- [ ] 105 0x20: the table calls it the File Transfer port [S], but
  `ServerInfo::server_value` says "unknown meaning". Rename the field, and
  confirm with a capture whose server has a File Transfer port set
  (Linux). Every 105 in the 22 captures has 0 there, so none covers it.
- [ ] 105 0x2c..0x45 are missing from the table.
- [ ] 105 0x46, u16 2: name it from the client's version check.
- [ ] 105 third string (≤ 0x7f): what fills it, and does the client show
  it?
- [ ] 107 Server Greeting 0x00, u32 0.
- [ ] 108 Ping / 109 Pong: only `kdx-probe` has sent them. Does Client
  1.620 beta 5 send 108, and when? Static first; a capture only if it
  does. No GUI capture has one, though the GUI session on port 42252 of
  `linux-tracker` stays open 725 s (frames 1–93).
- [ ] 110 Set Mini User Icon: the status byte has values 0 and 1 [O]
  (`linux-server-settings` frames 210, 215, `/away` and `/back`); any
  others the client sends? 0x01..0x03 are 0.
- [ ] 112: a provisional name. No Linux build names it; check the name
  tables of the Windows client 1.600 and Windows server 1.600, and settle
  on a name or keep it provisional with the evidence.
- [ ] 113/114 Extra Server Info: 0x00, 0x10 bytes; the "OS values" at 0x24
  and 0x28 (`ExtraServerInfo::os_value_24`, `os_value_28`); 0x2d..0x2f.
  Read how the Linux server fills them (`FUN_080619d8`).
- [ ] 118: "reports whether the client is registered" [I], and the name is
  provisional. The server's use is read [S]; settle the name from the
  client's sender (`FUN_0804c01c`, `FUN_08107e04`) or keep it provisional.
