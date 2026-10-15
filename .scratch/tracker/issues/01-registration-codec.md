# Registration datagram codec

Status: done
Blocked by: capture-gaps 05

Add the UDP registration datagram to `kdx-proto` (`docs/protocol/udp.md`,
"Registration (UDP)"): the LCG XOR, the CRC-32 (poly 0x04c11db7, MSB first,
init 0xdeadbeef, checksum field set to 0xcc1ed3ca while hashing), and the
64-byte header with its strings.

- A `Registration` type with `pub const HEADER_LEN: usize = 0x40`.
- `decode` applies the tracker's checks in `FUN_0804c048` and returns an
  error for each failure.
- Fixture: the server's datagram from the capture. Decode it, re-encode it,
  and compare byte for byte.
- Update the evidence tags in `tracker.md` that the fixture confirms.
