# Scenario: linux-chat-admin

- Date: 2026-10-09T19:41Z
- Server: KDXServer 1.620 Linux (sha256 ad6fba4bdd01dc23)
- Client: linux 1.620 beta 5
- Server state: the server from linux-first-connect (admin / lab-only-pw!1,
  guest enabled)

## Steps

The operator followed a script; the steps below are what the capture shows.
`cli-peer` and `cli-peer2` are the `kdx` CLI logged in as admin.

1. `cli-peer` ran `kdx listen` in Public Chat (port 37478); the GUI connected
   as admin, nick `kdx-lab-linux` (port 37484).
2. The GUI sent `cli-peer` a private message, driven by xdotool; `cli-peer2`
   sent one back with `kdx msg` (port 47748), and the GUI replied.
3. The GUI joined Public Chat, said "hello chat", and set the topic to
   "lab topic".
4. The GUI created a private chat and invited `cli-peer`.
5. The GUI got `cli-peer`'s info, sent a broadcast, opened Server History and
   the connection and process monitors, and disconnected `cli-peer`.

## Observations

- Every message decoded with the existing codecs; first captures of 140–142,
  150/151, 154/155, 180–182, 200/216, 211/212, and 800/801/803.
- Server Settings (170) was not opened.
- `cli-peer` received the topic change, the chat invitation, and the
  broadcast before the server disconnected it.
