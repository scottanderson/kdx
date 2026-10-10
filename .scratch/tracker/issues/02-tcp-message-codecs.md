# Tracker message codecs

Status: ready-for-agent
Blocked by: capture-gaps 05

Add payload codecs to `kdx-proto::msg` for 701, 702, 703, 711, and 721, and
the empty requests 700, 710, 720, and 713 (`docs/protocol/tracker.md`).

- 703 records carry an `addr_len`; accept only 6 (IPv4 and port) and keep
  `extra_len` bytes as opaque.
- 711 is the `%KDXTSTG` settings file. Decode the header and sections that
  `tracker.md` names; keep the rest as raw bytes so it re-encodes exactly.
- Fixtures from the capture; prove each encoder by re-encoding.
- Mark the matching facts verified in `tracker.md`.
