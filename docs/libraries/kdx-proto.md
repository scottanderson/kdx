# kdx-proto

The wire format of the protocol, with no I/O and no async: the handshake,
session encryption, message framing, and typed payloads.

- `msg` has a type for each message payload. Each decodes a captured payload and
  encodes it again byte for byte, storing the bytes it does not model.
  `Decoded<T>` holds a value and any bytes that follow it, so a payload with
  a tail still round trips.
- `session` has the header, the cipher, and `MessageType`, a `u32` newtype with
  named constants.
- `handshake` has the hello and proof.

Every layout is documented in [the protocol
reference](../protocol/KDXProtocol.md) and tested against capture fixtures in
`crates/kdx-proto/tests/fixtures/` (see its README).
[kdx-dissect](../dissect/kdx-dissect.md)'s `--audit` checks the codecs against
every capture.
