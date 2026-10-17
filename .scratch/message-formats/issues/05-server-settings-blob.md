# Server settings blob: 170–173

Status: ready-for-agent
Type: research
Targets: 171 Server Settings Info and 172 Set Server Settings

Docs: `messages/administration.md`, 170 Get Server Settings. Captures:
`linux-server-settings`, `linux-catalog-volumes`, `linux-tracker`, `probe`.

The blob is the server's settings file (`%KDXSSTG`). Only a few fields of
the main section are named, and most sections are unexplained.
`ServerSettings` keeps the unnamed bytes as sent.

The tracker's settings blob (`%KDXTSTG`, `tracker.md` 711) has the same
shape and is fully read: a section table of 11 entries, then one "present"
byte per section at 0x6c. The server's 12 bytes at 0x64 may be the same
kind of table. <!-- [I] -->

## Gaps

- [ ] 0x0c, the section table: "11 entries of u32 offset, u32 length".
  <!-- [I] --> Confirm from the parser.
- [ ] Name every section. Entry 0 is the main section, 3 the greeting, 5
  the icon; entries 1, 2, 4, 6, and 8–10 are empty in every capture; entry
  7 points at the end of the blob with length 0 in a 171 and is 0 in a 172.
- [ ] 0x64, 12 bytes: `01 01 01 01 00 01 00 02 00 00 00 00` in a 171, and 0
  at 0x69 and 0x6b in some 172s.
- [ ] The main section from 0x70 to its end. Named so far: 0x93 bit 1
  (greeting enabled), 0xa8 (total outgoing speed limit), 0xc2 (a length,),
<!-- [I] --> and the server name at 0xcc. Map the server's in-memory settings
  (port at +0x258, File Transfer port at +0x260, Register with Trackers at
  +0x27c, …) to blob offsets through the parser and writer, and match each
  to a control in the client's Server Settings window.
- [ ] The tracker entries (Name, Comments, Address, Login, password key, an
  unused string, six groups; `tracker.md`): which section holds them, and
  their layout. Already captured: `linux-tracker` frame 20 adds the entry
  `lab` / `labcomments` / `127.0.0.1` / `trk-login` / `Lab, Lab2`, and
  frame 252 changes its groups to `Lab`.
- [ ] Anything else the window edits (ports, limits, bans, base folders).
- [ ] 173: confirm the empty payload.

Once the parser is read, a Linux capture that changes one setting at a time
in the Server Settings window confirms the map. Write it as a capture-gaps
issue. These changes are already captured, so the sweep can skip them:

| Setting | Capture and frames |
|---|---|
| server name `Lab Server` | `linux-server-settings` 40 |
| greeting text, and Enable on the Greeting panel | `linux-server-settings` 40, 56 |
| server icon | `linux-server-settings` 40 (the 764-byte 172) |
| Total Outgoing Speed Limit on and off | `linux-server-settings` 40, 225; `linux-catalog-volumes` 69, 174 |
| a tracker entry added, then its groups changed | `linux-tracker` 20, 252 |
