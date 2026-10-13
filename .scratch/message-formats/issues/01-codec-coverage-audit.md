# Audit which captured payloads the codecs reproduce

Status: ready-for-agent
Type: task
Targets: every message type in `captures/*.pcapng`

The doc tables can look complete while a `kdx-proto` decoder drops bytes.
Find those cases mechanically, so issues 02–14 can't miss them.

Write a test (or a `kdx-dissect` mode) that, for every message in every
capture:

- decodes the payload with the `kdx-proto` type for its message type;
- re-encodes it and compares with the payload, ignoring padding the docs
  call uninitialised;
- reports the types it has no codec for, and the frames whose bytes don't
  come back.

As of 2026-10-13, `kdx-dissect` prints no decoded form for these captured
types, though some have empty payloads: 108, 109, 112, 113, 115, 140, 141,
142, 170, 173, 175, 180, 182, 200, 213, 214, 223, 414, 417, 418, 421, 505,
512, 551, 605, 700, 710, 713, 720, 800, 801, 802, and 900–905. Some of them
have a codec in `kdx-proto` that `kdx-dissect` doesn't call.

## Gaps

- [ ] The list of types with no codec, by issue number of this effort.
- [ ] The list of lossy decoders, with the bytes they drop.
- [ ] Add each finding to the issue for its area as a new gap.

Done when the test runs in `cargo test` and every lossy decoder or missing
codec is recorded in an issue here.
