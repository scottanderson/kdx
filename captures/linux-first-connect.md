# Scenario: linux-first-connect

- Date: 2026-10-09T19:30Z to 19:36Z
- Server: Server 1.620 for Linux, sha256
  `ad6fba4bdd01dc237abde2b76054e3bbbb43c6d0c79a28d4e0de3b23e546f943`
- Client: Client 1.620 beta 5 for Linux, sha256
  `e7189c55b67a89f35edfdf09570a1d55a096b198579b8633f3adfa50f9d13961`, on its
  first run in the lab
- Server state: the server from the earlier captures, with admin /
  `lab-only-pw!1`, guest access on, `cli-big.bin` in the root folder, and only
  the Welcome message in General News
- Decode: `kdx-dissect captures/linux-first-connect.pcapng --password
  'lab-only-pw!1'`

## Steps

The operator explored freely; the steps are reconstructed from the capture
and confirmed by the operator.

1. Launched the client and accepted Haxial's license agreement. The client
   looked up `kdxserver.haxial.com` (frames 1–4, no DNS in the lab), printed
   `Invalid network address. (ID 4/103 @ N986)`, and was closed by the
   operator.
2. Relaunched the client and connected to 127.0.0.1 as admin with a wrong
   password (port 35552, frames 5–16).
3. Connected again as admin, nick `kdx-lab-linux` (port 48736), and opened a
   second connection in the same window as admin, nick `kdx-lab-linux2`
   (port 50104).
4. Both joined Public Chat; the first said "hi", the second "a".
5. Both listed the root folder, `Uploads [UL]`, and `Admin Dropbox [DB]`.
6. The second posted "test", read it, and deleted it; then posted "a", read
   it, posted the reply "Re: a", and read that.
7. The first listed newsgroups and messages, then opened the account class
   list (505) and the newsgroup's info (602).
8. Closed the client. No private messages were sent.

## Observations

- The hello carries sub-version 0x10 (frames 8, 20), as Client 1.600's does; the
  server answers 0x20.
- A wrong password gets a server proof with status 1 and the connection is
  closed (frames 13–14).
- kdx-dissect decoded every message with the codecs written for 1.600 and
  the CLI. Nothing new appeared.
- General News now holds messages 1, 6 ("a"), and 7 ("Re: a").
