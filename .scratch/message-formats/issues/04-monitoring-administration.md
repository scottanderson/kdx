# Monitoring and administration: 140–155, 174, 175, 180–191

Status: ready-for-agent
Type: research
Targets: 140–143, 150–155, 174, 175, 180–183 and the history record

Docs: `messages/monitoring.md` and `messages/administration.md`. The settings
blob (170-173) is issue 05.

## Gaps

- [ ] 140/141: confirm the payload is empty or ignored.
- [ ] 143 record 0x38: "a per-transfer value", <!-- [S] --> 0x5000 (the speed
  limit) in every record. <!-- [O] --> Name it
  (`MonitoredConnection::value_38`).
- [ ] 143 0x3c state: name the values (0x90 queued, 0x19c for a download).
- [ ] 143 0x40 (u16 0), 0x43..0x5f, and 0x64..0x65 are not in the table.
- [ ] 150 Disconnect User: 0x04..0x07, 0x0c..0x10, 0x12..0x13, and
  0x15..0x17 are not in the table. A Linux capture that bans a user would
  show the ban path. The only 150 (`linux-chat-admin` frame 159) has ban 0.
- [ ] 151 Disconnect Message: 0x04, 8 bytes of 0; 0x0e and 0x10, two u16s
  (`DisconnectMessage::value_0e`, `value_10`); the flags besides 2 and 4.
- [ ] 153 User Info: 0x04 u32; the two times at 0x10 and 0x18; 0x20, 0x1b
  bytes; 0x3c, 0x3e, 0x42, 0x43.
- [ ] 153 0x3f and 0x41 are both "name": "the class or the account name".
<!-- [I] --> A Linux capture of Get User Info on a user whose account name
  differs from its class name settles it. Both 153s on record
  (`linux-chat-admin` frame 124, `probe` frame 230) are for `admin`, whose
  account name and class are both `Administrator` (`kdx-accounts` frame
  672), so neither does.
- [ ] 154 Send Broadcast Message: u16 at 0x00 and the u16 "copied through"
  at 0x02 (`SendBroadcast::value_0`, `value_2`); what the client puts
  there.
- [ ] 155 Rcv Broadcast Message: 0x04..0x0d zeros and the copied u16 at
  0x0e (`RcvBroadcast::value_0e`).
- [ ] 174 Shutdown Server: 0x00 and 0x02..0x03 are 0; mode bits beyond 1,
  2, and 4.
- [ ] 180/182: the request payload, if any.
- [ ] 181: the u32 0 before the count.
- [ ] History record: 0x10, 0x32 bytes of 0 in every capture; 0x44, 0x45,
  0x4a, 0x4b. Find a record kind that writes them.
- [ ] History record kinds: list every kind the server writes (184–191, the
  logged request types, and the numbers such as 0xae, 0xaf, 0x1a7, 0x1c2,
  0x1c4, 0x324, 0x44d) and what its detail and path hold.
- [x] Audit (01): 143 Connection Monitor Info had non-zero bytes in the
  last 1 to 2 bytes of the payload: 0x93 is 02 (`linux-catalog-volumes` 78,
  81), 0x12a..0x12b is `20 43` (`linux-chat-admin` 143, 146, 149), 0x8e..0x8f
  is `6e 88` (`probe` 239). Fixed: `MonitoredConnection::padding` keeps
  them.
- [ ] Are those bytes uninitialised padding? `messages/monitoring.md` says so
  from the captures; <!-- [O] --> confirm in the builder.
