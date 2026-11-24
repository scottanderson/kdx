# Scenario: linux-chat-admin

- Date: 2026-10-09T19:42Z to 19:49Z
- Server: Server 1.620 for Linux, sha256
  `ad6fba4bdd01dc237abde2b76054e3bbbb43c6d0c79a28d4e0de3b23e546f943`
- Client: Client 1.620 beta 5 for Linux, sha256
  `e7189c55b67a89f35edfdf09570a1d55a096b198579b8633f3adfa50f9d13961`
- Tools: `kdx`, run as `lab/kdx-lab run target/debug/kdx` with the arguments in
  the steps
- Server state: the server from linux-first-connect, with admin /
  `lab-only-pw!1` and guest access on
- Decode: `kdx-dissect captures/linux-chat-admin.pcapng --password
  'lab-only-pw!1'`

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
5. The GUI got its own info (user 2), sent a broadcast, opened Server
   History and the connection and process monitors, and disconnected itself
   (150 for user 2 at frame 159).

## Observations

- Every message decoded with the existing codecs; first captures of 140–142,
  150/151, 154/155, 180–182, 200/216, 211/212, and 800/801/803.
- Server Settings (170) was not opened.
- `cli-peer` received the topic change, the chat invitation, the broadcast,
  and 104 User Departed for the GUI (frame 164).
