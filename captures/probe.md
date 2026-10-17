# Scenario: probe

- Date: 2026-10-08T08:35Z to 18:49Z
- Server: KDXServer 1.620 Linux (sha256 ad6fba4bdd01dc23)
- Client: none; `kdx-probe` (this repository) only, no GUI
- Server state: the first-connect server (admin / lab-only-pw!1)

## Steps

Each step was one `kdx-probe` run that logged in as admin, sent the listed
requests, and disconnected. There were 18 connections in all.

1. Read-only requests, a few per connection: 108, 112, 113, 115, 118, 140–142,
   152, 170, 214, 400, 409, 502, 505, 509, 512, 602, 605, 607, 609, 802,
   1106. Some were sent malformed on purpose (for example 152 for a missing
   user and 400 with a bad path) to get error replies.
2. Chat: 202 Join Chat, 207 Send Chat Text, 209 Set Chat Topic, 218 Get Chat,
   and 204 Leave Chat.
3. Transfers: five logins paired with an attached connection
   (`kdx-probe --attach`, login name `00` + user ID). These downloaded the
   70000-byte `probe-dl.bin` (whole, then resumed from 5000) and uploaded the
   100-byte `probe-ul.bin` with 412 Download File and 415 Upload File. Both
   files were removed from the server afterwards.

## Observations

- `kdx-dissect` decodes the admin connections. It can't decode the attached
  ones, which use the 105 Server Info session key as the account key.
- The 803 Process List reply (frame 208) lists the lab's `bwrap` command
  line, `./KDXServer.lexe`, and `/bin/ps auxwww`, run as the host user.
- The facts drawn from this capture are tagged `[O]` in the pages under
  `docs/protocol/`.
