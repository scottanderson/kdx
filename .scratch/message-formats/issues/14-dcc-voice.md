# Client-to-client (DCC) and voice: 117, 100/300 on DCC, 1000–1003

Status: ready-for-agent
Type: research
Targets: 117 SKPK, 300 on a DCC connection, 1000–1003

Docs: `dcc.md`. Nothing here has been captured. The capture is
`.scratch/capture-gaps/issues/07-dcc-voice.md` (needs-info: `kdx-lab` can't
run a second Linux client yet). Do the static gaps here, and add what the
capture must show to issue 07 rather than a new capture-gaps issue.

## Gaps

- [ ] Which side opens the primary connection, and from which UI action
  (`dcc.md`, Open questions).
- [ ] 117 SKPK: 0x2a (2 bytes) and 0x2c (4 bytes), listed as 0. Does the
  server name 117 at all? The server's name table has it; its dispatcher
  ignores it.
- [ ] 300 on a DCC connection: "u32, 12 bytes, then the text". Name the
  u32 and the 12 bytes, as for 300 on a server (issue 06).
- [ ] 1001 Start Voice (R): values besides 0 and 0x00640007.
- [ ] 1002 Set Voice Format: 0x00, 0xc bytes of 0; the codecs besides
  `ADPC`.
- [ ] 1003 Voice Data: confirm nothing precedes the ADPCM bytes.
