# Tracker

Goal: the Rust code speaks the tracker protocol in all three directions. A
Rust server registers with Tracker 1.110, the `kdx` CLI browses Tracker
1.110, and Client 1.620 beta 5 browses `kdx-tracker` (`docs/PLAN.md`, area 7).

## Where it stands

`docs/protocol/udp.md` covers UDP registration and
`docs/protocol/messages/tracker.md` the TCP messages
115, 700–703, 710–713, and 720/721 from the binaries, checked against
`captures/linux-tracker.pcapng` (capture-gaps 05) for everything but 115.

## Order

1. Capture and the remaining unknowns (capture-gaps 05). Codecs are tested
   against capture fixtures, so it comes first.
2. Codecs in `kdx-proto` (01, 02), then `kdx-dissect` (03) and the CLI (04).
3. `kdx-tracker` (05) and server registration in `kdx-server` (06) last.

## Notes

- The TCP side reuses the handshake and session layer as they are. Only the
  version check and the login rules differ.
- The UDP datagram has no reply, so a lab capture of the server's sends
  plus the tracker's 703 listing is enough to prove the registration codec.
