# Multi-byte text in names, chat, messages, and news

Status: done
Scenario: `linux-text-encoding`
Targets: how the Linux client and the server carry non-ASCII text: Mac OS
Roman, UTF-8, or raw bytes, in 401, 412, 415, 402, 207/208, 300/301, 611,
and 100.

When this was written, the spec said text is Mac OS Roman and that the server
rejects a path that spells a Mac Roman name in UTF-8 **[I]**. The Linux client
reads keys with `Xutf8LookupString` and pastes `UTF8_STRING`, so it may send
UTF-8. The `kdx` CLI sends all text as UTF-8.

Setup (agent): server detached; `Bases/Default/lab-utf8/` holds `café.txt`
(UTF-8 name), `caf\x8e.txt` and `it\xd5s.txt` (Mac Roman é and ’),
`日本.txt`, and `emoji-😀.txt`. The client folder holds `naïve-ü.txt`. `kdx
listen` logged in as `peer`. Test text: `café ’ — 日本 😀`.

1. Connect as admin.
2. Files → `lab-utf8`: note how each name shows (401). Download `café.txt`
   and `caf\x8e.txt` as shown (412); the agent checks the saved names.
3. Upload `naïve-ü.txt` into `lab-utf8` (415); the agent checks the name on
   the server's disk.
4. New Folder in `lab-utf8` named `dossier-é-日本` (402).
5. Public chat: send the test text (207); `peer` prints what it gets. The
   agent then sends the text as UTF-8 (`kdx say`) and as Mac Roman bytes
   (`kdx-probe` 207); note how each shows.
6. Private message to `peer` with the test text (300). The agent replies as
   UTF-8 and as Mac Roman; note how each shows.
7. News: post with the test text in the subject and body (611).
8. Change the nick to `Zoë-日本` (100).

## Comments

2026-10-12: Captured as `captures/linux-text-encoding.pcapng`. All text is
bytes end to end: the Linux client sends UTF-8 and shows UTF-8 (Mac Roman
bytes as boxes, 0xd5 swallowing the next byte as the decoder in the spec
predicts; 日本 and 😀 as boxes, likely missing glyphs), and the server
stores, relays, and lists the bytes unchanged. `caf\xc3\xa9.txt` and
`caf\x8e.txt` are two files, and 412 fetches each by its own bytes. No new
types. Steps 5 and 6 sent the Mac Roman lines with `kdx-probe` (207, 300);
`kdx say` ran twice by mistake (users 6 and 8).
