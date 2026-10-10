# Scenario: linux-server-settings

- Date: 2026-10-10T01:20Z
- Server: KDXServer 1.620 Linux (sha256 ad6fba4bdd01dc23)
- Client: linux 1.620 beta 5
- Server state: the server from linux-first-connect (admin / lab-only-pw!1),
  plus a 2 MB random file `Bases/Default/speed.bin` in the file root
- Peer: `kdx -l admin --nick cli-peer listen` (port 52888), joined to Public
  Chat

## Steps

The agent drove the lab and gave one step at a time; the operator clicked.

1. The peer logged in (frames 4–15).
2. The client connected as admin (port 45150, frames 20–28) and joined the
   server history (32/33).
3. Commands → 127.0.0.1 → Server Settings: the client read the settings
   (170/171 at 35/36). The operator set the server name "Lab Server", the
   greeting "Welcome to the lab.", a server icon pasted from Settings →
   Identity, and a Total Outgoing Speed Limit of 20 KB/s, then saved (172 of
   764 bytes / 173 at 40/41).
4. Commands → 127.0.0.1 → Show Greeting: 112 → 107 "(Greeting disabled.)"
   (45/46), then 113/114 (48/49).
5. Server Settings again: ticked the enable option on the Greeting panel and
   saved (171 at 52, 172 of 240 bytes / 173 at 56/57).
6. Show Greeting: 112 → 107 "Welcome to the lab." (61/62).
7. Disconnected (the peer saw 104 at 67) and reconnected (port 40560): 105
   "Lab Server" at 80, 107 at 82, 106 Server Icon at 83, history at 88/89.
8. Downloaded `speed.bin` on a transfer connection (port 36370): 412/413/414
   at 105–107, and 419 on the session at 110. The transfer ended with an
   empty 414 at 189.
9. Settings → Identity: changed the nick to "lab-linux" (100 at 196). The
   server sent 102 to the peer (199) and to the client (201).
10. Joined Public Chat (202/203) and typed `/away` (110/111 at 210–213), then
    `/back` (110/111 at 215–218).
11. Server Settings: unticked Total Outgoing Speed Limit and saved (171 at
    221, 172 of 240 bytes / 173 at 225/226).

## Observations

- 107 Server Greeting is a u32 0 followed by the text, with no length or
  terminator: `00 00 00 00` + "Welcome to the lab." (frame 62).
- With the greeting disabled, the server still answers 112 with 107 and the
  text "(Greeting disabled.)" (frame 46).
- At login the server sends 107 and 106 Server Icon (524 bytes) after 105
  (frames 82, 83).
- 419 Notify Speed Limit is 4 bytes: `00 00 50 00`, 0x5000 = 20480 bytes per
  second, the limit set in step 3 (frame 110). It arrives on the session, not
  on the transfer connection.
- The speed limit sits at offset 0xaa of the settings block: `50 00` in the
  171 at frame 221, `00 00` in the 172 at frame 225.
- The second and third 172 (240 bytes) leave out the icon; the first (764
  bytes) carries it.
- 110 Set Mini User Icon is 4 bytes: `01 00 00 00` for /away, `00 00 00 00`
  for /back. 111 Mini User Icon Changed is the user ID followed by those 4
  bytes, sent to everyone including the sender (frames 211, 213).
- 102 User Changed (40 bytes) goes to every user, including the one who
  changed.
- Each settings save and each connection adds a 183 Server History Added.
- First captures of 102, 106, 110, 111, 172, 173, 183, and 419, and the first
  GUI captures of 107, 112, 113, and 114.
