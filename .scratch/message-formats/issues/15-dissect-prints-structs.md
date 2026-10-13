# kdx-dissect prints each payload as its struct

Status: ready-for-agent
Type: task
Blocked by: 01, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 12, 13, 14

Today `kdx-dissect` prints a one-line summary for some types, which drops
fields and reorders them, then always hexdumps the payload up to 0x400
bytes (`crates/kdx-dissect/src/main.rs`, `summary` and the loop that calls
it).

Print every field of the decoded struct instead, one per line, in wire
order, under the names the docs give:

```
  [32] S->C txid 2fd80c58 105 Server Info len 86
      session key          e277f120…
      file transfer port   0
      user id              1
      version              1600
      …
```

- Byte strings quoted, keys and blobs as hex, lists as indented records.
- Hexdump only when the payload doesn't fit: no codec for the type, a
  decode error, or a re-encode that doesn't give the payload back (the test
  from issue 01). Then print the fields that did decode, and the dump.
- Keep the problem lines (`!`, `stopped:`, `DOES NOT match`) as they are,
  so `crates/kdx-dissect/tests/captures.rs` still works.
- The same for tracker messages and UDP registrations.

Done when every captured payload prints without a hexdump, or with one that
an issue here explains.
