# Scenario: linux-text-encoding

- Date: 2026-10-12T06:25Z
- Server: KDXServer 1.620 Linux (sha256 ad6fba4bdd01dc23)
- Client: linux 1.620 beta 5, LANG C.UTF-8
- Server state: the server from linux-accounts-folder-access (admin /
  lab-only-pw!1, peer / lab-peer-pw!1). `Bases/Default/lab-utf8/` holds
  `caf\xc3\xa9.txt` (UTF-8 é), `caf\x8e.txt` and `it\xd5s.txt` (Mac Roman é
  and ’), `\xe6\x97\xa5\xe6\x9c\xac.txt` (日本), and
  `emoji-\xf0\x9f\x98\x80.txt` (😀). The client's folder holds
  `na\xc3\xafve-\xc3\xbc.txt` (naïve-ü).

Test text: `café ’ — 日本 😀`, 25 bytes in UTF-8.

## Steps

1. `kdx listen` as peer, nick `peer` (port 37678, user 1), joined Public
   Chat (202 at 16).
2. The client connected as admin, nick `lab-linux` (port 40504, user 2).
3. Files → `lab-utf8` (401 at 41). Downloaded the two `café` files: the
   UTF-8 one (412 at 52, port 46818) and the Mac Roman one (412 at 71, port
   39198).
4. Uploaded `naïve-ü.txt` into `lab-utf8` (415 at 90, port 57826).
5. New Folder in `lab-utf8` named `dossier-é-日本` (402 at 103, Status at
   106, 401 at 108).
6. Public chat: sent the test text (207 at 118; 208 at 119 to peer, 121 to
   the client).
7. `kdx say` as peer, nick `peer-utf8`, sent `utf8: ` + the test text (user
   6, port 41902; 207 at 148, 208 at 151). `kdx-probe` as peer, nick
   `peer-mac`, sent `mac: caf\x8e \xd5 \xd1` (user 7, port 41904; 207 at
   192, 208 at 195). `kdx say` ran once more by mistake (user 8, port 54588;
   208 at 234).
8. Private message to peer with the test text (300 at 249, 301 at 250).
   Replies: `kdx msg` as `peer-utf8` (user 9, 301 at 274) and `kdx-probe` 300
   as `peer-mac` (user 10, 301 at 302).
9. News: posted the test text as subject and body (611 at 319, 613 at 322,
   610 at 325).
10. Settings → Identity: nick `Zoë-日本` (100 at 327, 102 at 330). Quit.

## Observations

- Every text field is bytes from end to end. The client sends what it
  types as UTF-8 (207 at 118, 300 at 249, 611 at 319, 100 at 327, 402 at
  103, 415 at 90), and the server stores and relays the bytes unchanged
  (208, 301, 610, 102, and the names on disk). Lengths count bytes: the
  test text has length 0x19 in 611.
- The 401 at 41 sends names as the bytes on disk, in directory order. Both
  `café` files are listed and each 412 names its own file; the client
  saves the downloads under the same bytes (`caf\xc3\xa9.txt`,
  `caf\x8e.txt`).
- The client shows text as UTF-8. `é ’ —` in UTF-8 show right, in the file
  list, chat, and PMs. Invalid UTF-8 shows as boxes: `mac: caf\x8e \xd5
  \xd1` shows as `mac: caf⊠ ⊠⊠` (195, 302), because the 0xd5 lead byte
  uses up the space after it. 日本 and 😀 show as boxes too, one per
  character, likely missing glyphs in the client's font.
- The server log writes names raw: `Download "caf\x8e.txt"`, `Upload
  "naïve-ü.txt"`, `Set Name: <Zoë-日本>`.
