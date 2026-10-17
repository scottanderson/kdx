# Common encodings: Status, user record, icon bitmap, handshake, session

Status: ready-for-agent
Type: research
Targets: 1 Status, the user record (100–103), the icon bitmap (100–103,
106), the hello and proof, the session framing

Docs: `messages/status.md`, `encodings.md` (user record), `handshake.md`,
`session.md`.

## Gaps

1 Status:

- [ ] Error form 0x14, a u32 0: confirm the writer (`FUN_08086640`) and
  whether any build reads it.

User record (`FUN_0804af04`, `FUN_0804b304`; client `FUN_0804d654`):

- [ ] 0x05, 5 bytes listed as 0.
- [ ] 0x0e, 6 bytes: "two 3-byte RGB colors" from the client. Which
  colors, and does anything draw them?
- [ ] 0x14, u8 set by the server, 0xff for admin. Compare with the class
  color in 503 offset 0x31 and 513 offset 0x21 (issue 08).
- [ ] 0x15, u16 set by the server.
- [ ] 0x17 flags: bit 2 from the client, bit 1 the hidden marker; name the
  others or show they are unused.
- [ ] 0x1b, listed as 0.
- [ ] The 0x17-byte info trailer behind flag bit 1 is "probably a
  client-identification trailer". <!-- [I] --> The Linux client never sends it;
  check the Windows client 1.600 (`kdxclient`) for the sender. Its only
  capture, `first-connect` frame 22, has flags 0 and no trailer.

Icon bitmap (also 106 Server Icon and the settings icon section):

- [ ] The format is "2 bits per pixel" with a header such as `01 20 20 04
  00 01 00 00 00 ff cc 00` (`linux-server-settings` frame 83). Name each
  header field from the client's icon decoder. `docs/PLAN.md` area 2 lists the
  icon format as not yet done.

Handshake (`handshake.md`):

- [ ] Hello 0x10, 4 reserved bytes: the reader, or proof that none reads
  them.
- [ ] The nonce "looks time-based": <!-- [O] --> read how each side makes it.
- [ ] "Login names": the `00`+ID login is "probably used for file-transfer
  connections". <!-- [I] --> Transfer connections and view display confirm it
  (`transfers.md`; `win-view-display` frame 99).
  Update the doc.

Session (`session.md`):

- [ ] The header-only send path (`FUN_080868b4`, `FUN_08086a9c`) is
  "probably used for large or streamed payloads". <!-- [I] --> 414 and 417 send
  a whole fork under one header; <!-- [O `kdx-cli` frames 133, 190, 244] -->
  confirm they use it.
- [ ] The server pads the data after a 414 header with "a repeated 32-bit
  value that is not this fill". Find where it comes from.
- [x] Audit (01): 102 and 103 records come with zero bytes after the record
  (`linux-server-settings` 199, 201: 3 bytes; `linux-dcc-voice-failed` 78:
  2; `kdx-cli` 392: 2; `linux-text-encoding` 328: 1). Fixed:
  `UserRecord::padding` keeps them, and the header gaps are stored too.
- [ ] Is the padding after a 102 or 103 record always zero, or
  uninitialized like other records' padding?
