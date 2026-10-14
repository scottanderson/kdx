# Audit which captured payloads the codecs reproduce

Status: done
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

- [x] The list of types with no codec, by issue number of this effort.
- [x] The list of lossy decoders, with the bytes they drop.
- [x] Add each finding to the issue for its area as a new gap.
- [x] The type catalog in `KDXProtocol.md` has no rows for 4000 Open
  Window, 4001 Draw, 4200 Key, and 4201 Mouse. Their layouts are in
  `gconsole.md`.

Done when the test runs in `cargo test` and every lossy decoder or missing
codec is recorded in an issue here.

## Comments

2026-10-14: `kdx-dissect --audit` decodes each payload with its
`kdx-proto` type, encodes it again, and prints `~ no codec` or `~ lossy`
with the first differing bytes. Empty payloads with no codec are not
flagged. `crates/kdx-dissect/tests/captures.rs` runs it over every capture
and pins the findings in `EXPECTED_AUDIT`, so a fixed or new gap fails the
test until the list changes. `audit::without_padding` zeroes the padding
`KDXProtocol.md` calls uninitialised before comparing: after each 401 File
List entry and at the end of 510 Account Info. With it, 401 and 510 come
back byte for byte. Other findings below may be padding too, which the
Strings section says is uninitialised; each issue's gap asks for that.

No codec, by issue:

| Issue | Types |
|---|---|
| 06 | 223 |
| 07 | 414, 417 |
| 08 | 502, 508, 509, 515, 550 |
| 09 | 601, 602, 607, 609, 612 |
| 11 | 1101, 1102, 1103, 1106 |
| 12 | 902, 903, 904, 905 |
| 14 | 1001, 1002 |

Lossy, by issue (each issue gives the frames):

| Issue | Type | Bytes |
|---|---|---|
| 02 | 102, 103 | 1 to 3 trailing zero bytes after the record |
| 04 | 143 | non-zero bytes in the last 2 |
| 06 | 209 | 4 bytes after the topic (`probe` 73) |
| 07 | 410 | 0x37 |
| 07 | 415 | two times at 0x18 and 0x20, and 0x33 |
| 08 | 503 | the editor login at 0x36; non-zero bytes in the last 3 |
| 08 | 513 | 0x97 |
| 09 | 608, 610 | non-zero bytes after strings |
| 09 | 611 | the encoder pads to 4; the client doesn't |
| 10 | 803 | 0x1dc |
| 11 | 1100, 1104 | 0xc1, in the last 3 |

The spec's never-captured list had 117, 1000 to 1002, and 108/109 from a
GUI client, all in `linux-dcc-voice-failed`, and said 223 was captured only
empty, but frames 109 and 155 there hold addresses. The list is fixed.
