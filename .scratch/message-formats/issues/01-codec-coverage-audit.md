# Audit which captured payloads the codecs reproduce

Status: done
Type: task
Targets: every message type in `captures/*.pcapng`

The doc tables can look complete while a `kdx-proto` decoder drops bytes.
Find those cases mechanically, so issues 02–14 can't miss them.

Write a test (or a `kdx-dissect` mode) that, for every message in every
capture:

- decodes the payload with the `kdx-proto` type for its message type;
- re-encodes it and compares with the payload byte for byte;
- reports the types it has no codec for, and the frames whose bytes don't
  come back.

As of 2026-10-13, `kdx-dissect` prints no decoded form for these captured
types, though some have empty payloads: 108, 109, 112, 113, 115, 140, 141,
142, 170, 173, 175, 180, 182, 200, 213, 214, 223, 414, 417, 418, 421, 505,
512, 551, 605, 700, 710, 713, 720, 800, 801, 802, and 900–905. Some of them
have a codec in `kdx-proto` that `kdx-dissect` doesn't call.

## Gaps

- [x] The list of types with no codec, by issue number of this effort.
- [x] The list of decoders that dropped bytes, with the bytes they drop.
- [x] Keep every dropped byte in the decoded type, so each payload comes
  back byte for byte.
- [x] Store the header gaps that every other codec wrote as zeros, and the
  padding of list records, so a nonzero byte there comes back too.
- [ ] Bytes after the last string or record of a payload are still dropped
  by most single-message decoders and by the counted lists. Store them, or
  reject them, once a capture or a handler shows any.
- [x] Add each finding to the issue for its area as a new gap.
- [x] The type catalog in `KDXProtocol.md` has no rows for 4000 Open
  Window, 4001 Draw, 4200 Key, and 4201 Mouse. Their layouts are in
  `gconsole.md`.

Done when the test runs in `cargo test`, no payload comes back different,
and every missing codec is recorded in an issue here.

## Comments

2026-10-14: `kdx-dissect --audit` decodes each payload with its `kdx-proto`
type, encodes it again, and prints `~ no codec`, or `~ mismatch` with the first
differing offset and a note when the lengths differ. Empty payloads with no
codec are not flagged. `crates/kdx-dissect/tests/captures.rs` runs it over every
capture and pins the findings in `NO_CODEC`, so a fixed or new gap fails
the test until the list changes.

No codec, by issue:

| Issue | Types |
|---|---|
| 08 | 502, 508, 509, 515, 550 |
| 12 | 902, 903, 904, 905 |
| 14 | 1001, 1002 |

Mismatches found on 2026-10-14, by issue (each issue gives the frames), all
fixed the same day:

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

2026-10-14: Data must round trip losslessly, so a difference is a codec bug,
never an accepted category, and the audit no longer masks padding before
comparing. Each type in the table above now stores its padding and unknown bytes
and writes them back, `NO_CODEC` lists only `no codec` types, and
`crates/kdx-proto/tests/round_trip.rs` re-encodes the fixtures' payloads of
those types. 401 and 510, whose padding the audit used to mask, keep it too.
What the stored bytes mean stays open in each issue.

2026-10-14: The audit only sees the bytes the captures hold, so it missed
header gaps that happen to be zero. Every codec in `kdx-proto/src/msg` now
stores those gaps as `unknown_*` fields and each list record's padding, with
zero defaults so new messages encode as before. 181 has a `HistoryList` type
for its leading u32. A 4001 Draw record whose length isn't its kind's decodes
as `Other`, and `Draw::padding` keeps each record's padding. 105's u16 at
0x2a and 0x46 are stored as read. `round_trip.rs` sets every gap of each
codec to nonzero bytes in a synthetic payload and checks it comes back.
