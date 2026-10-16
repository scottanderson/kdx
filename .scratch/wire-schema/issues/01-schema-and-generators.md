# Describe each message in one schema and generate the docs and codecs

Status: needs-triage
Type: task
Targets: `docs/protocol/messages/*.md`, `crates/kdx-proto/src/msg/`,
`ci/check-docs.py`

The wire format is written three times: the struct tables in the docs, the
Rust codecs in `kdx-proto`, and the rules in `ci/check-docs.py` that compare
the two. This issue is a plan to keep one description and generate the rest.
It needs more thought before work starts.

## Why

- A change to a field means editing a table, a struct, an encoder, a decoder,
  and sometimes the checker. The checker catches a mismatch, but only after it
  is made.
- The tables carry facts that Rust cannot hold: ignored rows, offsets, and
  evidence tags with function addresses and frames. The tables must stay the
  place for these.

## Plan

One schema file per message, holding what a table holds now: offset, size,
type, field, notes with their tags. A generator reads the schemas. Do it in two
stages, so the first one is useful on its own:

1. Generate the Markdown struct tables from the schemas, and test the
   handwritten codecs against them (field names, types, sizes, offsets). The
   codecs stay handwritten, and the checker's table-to-Rust comparison becomes
   a schema-to-Rust test.
2. Generate the Rust types and codecs. Keep a hook for the few fields that need
   code, such as the cipher, the tracker settings sections, and records that
   depend on their kind.

## Notes

- Kaitai Struct was set aside. It describes parsing well, but its
  serialization is weak, and it has no place for evidence tags.
- A Rust derive macro was set aside. It cannot hold ignored rows, offsets,
  or notes.
- The schema format is open: TOML, YAML, or a Markdown table kept as the
  source. Decide with a prototype on two or three messages that differ, such
  as 203 Chat Window Info, 413 Download Info, and the Registration datagram.
- Rows that mix two wire fields in one Rust field (the tracker settings
  `sections`, folder-access `flags` and `rest`, the gconsole `records` padding)
  need a structure decision first. The schema cannot generate them until the
  table and the Rust agree. `ServerListEntry` is done: its address and port are
  two fields.
- A string's length byte can come in a different order from the string. 203
  Chat Window Info, 402 Create Folder, 610 News Message, 611 Post News Message,
  and 803 Process List do it. The schema must record the order of the strings
  apart from the order of the lengths. A generator that paired each length with
  the next string would get these wrong, and would also drop a length that a
  table calls ignored, as 803's third string was. Issue
  `message-formats/16-string-order-audit` checks the rest by hand; the schema
  should make that check mechanical.
- The captures stay the test of the result: a generated codec must re-encode
  each decoded capture byte for byte.

## Gaps

- [ ] Choose the schema format with a prototype.
- [ ] Decide the composite rows listed above.
- [ ] Stage 1: generated tables and a schema-to-Rust test.
- [ ] Stage 2: generated Rust.

Done when a change to a field is made in one place and the docs, the codec, and
the checks follow.
