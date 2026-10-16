# Check the string order of every message that has several strings

Status: ready-for-agent
Type: task
Targets: every struct with two or more `str` rows

The wire order of the strings can differ from the order of their length
bytes. 203 Chat Window Info, 402 Create Folder, 610 News Message, 611 Post News
Message, and 803 Process List do it, and 803 has a third string that a capture
had hidden. A table that lists the strings in the length order would be wrong
for a non-empty case that no capture shows.

The scan on 2026-10-16 compared each table's string rows with its length rows.
That finds only the mismatches the tables already show. It does not read the
code. For every struct that has two or more `str` rows, compare the string
order with the decompiled server writer or reader, and the client's.

## Gaps

- [ ] List the structs with two or more `str` rows, and for each the function
  that reads or writes the strings.
- [ ] For each, record that the order matches the code, or fix the table, the
  `kdx-proto` codec, and the note on the page.
- [ ] Check that no length byte goes unread, as 0x3b did in 803. A length row
  that the docs call ignored may hold a string length.
- [ ] Record in each struct the evidence tag for the order. Captures with
  empty strings do not count.

Done when every multi-string struct has its order checked against the code.
