# Scenario: kdx-cli

- Date: 2026-10-08T23:27Z to 23:31Z
- Server: KDXServer 1.620 Linux (sha256 ad6fba4bdd01dc23)
- Client: none; the `kdx` CLI (this repository) only, no GUI
- Server state: the first-connect server (admin / lab-only-pw!1), plus a
  600000-byte test file `cli-big.bin` in the base folder

## Steps

Each step is one `kdx` run as admin (`--login admin`), nick `kdx-rs` unless
given.

1. `who`, `chats`, `ls` (the root, which lists `cli-big.bin`).
2. `kdx-probe 400` (raw 401 File List with a file entry).
3. `ls 'Uploads [UL]'` (empty).
4. `get cli-big.bin`: the whole 600000 bytes.
5. `get cli-big.bin --resume` into a local copy of its first 300000 bytes.
6. `put cli-up.bin 'Uploads [UL]'`: the first 300000 bytes of the same data.
7. `ls 'Uploads [UL]'`, `news`, `news 1`, `say 'hello from kdx-rs'`,
   `msg 1 hi`.
8. `listen --nick listener` while a second run did `who` and then
   `say --nick talker 'is anyone there?'`.

Afterwards `cli-big.bin` and `Uploads [UL]/cli-up.bin` were deleted from the
server.

## Observations

- Both downloads and the upload were byte-identical to the source. Each fork
  moved under a single 414 or 417 header, above the 0x20000 payload limit.
- The resume from 300000 restarted at 299008, which is 0x49000.
- 401 File List entries for files carry a second string, `exttype/bin`, whose
  length is at 0x36.
- The listener saw 103 User Joined, 205 Chat User Joined, 208 Receive Chat
  Text, 206 Chat User Departed, and 104 User Departed.
