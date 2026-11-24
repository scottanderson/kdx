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

## Commands

These commands send the requests in the capture. The requests are synthetic;
only the server's replies are evidence.

```sh
# The first eight runs, one per connection, ports 59490 to 50984.

lab/kdx-lab run target/debug/kdx-probe --login admin --password 'lab-only-pw!1' \
  --table vendor/haxial/KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe \
  605

lab/kdx-lab run target/debug/kdx-probe --login admin --password 'lab-only-pw!1' \
  --table vendor/haxial/KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe \
  108 113 115 214 505 512

lab/kdx-lab run target/debug/kdx-probe --login admin --password 'lab-only-pw!1' \
  --table vendor/haxial/KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe \
  202:0000000100000000 207:00000001000000000000000048656c6c6f \
  209:0000000100000000000005000000000048656c6c6f 218:00000001 152:00000001 \
  204:00000001

lab/kdx-lab run target/debug/kdx-probe --login admin --password 'lab-only-pw!1' \
  --table vendor/haxial/KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe \
  152:00000003 400 400:0000 602:00000001 609:0000000100000001 170

lab/kdx-lab run target/debug/kdx-probe --login admin --password 'lab-only-pw!1' \
  --table vendor/haxial/KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe \
  400:000100000c55706c6f616473205b554c5d \
  409:000100000c55706c6f616473205b554c5d 605 607:00000001

lab/kdx-lab run target/debug/kdx-probe --login admin --password 'lab-only-pw!1' \
  --table vendor/haxial/KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe \
  112 118:00000001

lab/kdx-lab run target/debug/kdx-probe --login admin --password 'lab-only-pw!1' \
  --table vendor/haxial/KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe \
  152:00000001 152:00000002 152:00000004 502:00 502:02 509:61646d696e 512 1106 \
  802 142

lab/kdx-lab run target/debug/kdx-probe --login admin --password 'lab-only-pw!1' \
  --table vendor/haxial/KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe \
  152:00000008 152:00000009 140 142 141

# The five attached runs, ports 33068/33082 to 48784/48796. Each logs in,
# then sends on an attached connection. --wait 1500 matches the 1.5 s
# between their requests.

lab/kdx-lab run target/debug/kdx-probe --login admin --password 'lab-only-pw!1' \
  --table vendor/haxial/KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe \
  --attach --wait 1500 \
  400 \
  412:000000000000000000000000000000000000000000000000000100000c70726f62652d646c2e62696e \
  108

lab/kdx-lab run target/debug/kdx-probe --login admin --password 'lab-only-pw!1' \
  --table vendor/haxial/KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe \
  --attach --wait 1500 \
  412:000000000000000000000000000000000000000000000000000100000c70726f62652d646c2e62696e

lab/kdx-lab run target/debug/kdx-probe --login admin --password 'lab-only-pw!1' \
  --table vendor/haxial/KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe \
  --attach --wait 1500 \
  412:000000000001117000000000000000000000000000000000000100000c70726f62652d646c2e62696e \
  108

lab/kdx-lab run target/debug/kdx-probe --login admin --password 'lab-only-pw!1' \
  --table vendor/haxial/KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe \
  --attach --wait 1500 \
  412:000000000000138800000000000000000000000000000000000100000c70726f62652d646c2e62696e \
  108

lab/kdx-lab run target/debug/kdx-probe --login admin --password 'lab-only-pw!1' \
  --table vendor/haxial/KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe \
  --attach --wait 1500 \
  415:0000000000000064000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000110c00000070726f62652d756c2e62696e000100000c55706c6f616473205b554c5d \
  417:000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60616263 \
  417 400:000100000c55706c6f616473205b554c5d
```

## Observations

- `kdx-dissect` decodes the admin connections. It can't decode the attached
  ones, which use the 105 Server Info session key as the account key.
- The 803 Process List reply (frame 208) lists the lab's `bwrap` command
  line, `./KDXServer.lexe`, and `/bin/ps auxwww`, run as the host user.
- The facts drawn from this capture are tagged `[O]` in the pages under
  `docs/protocol/`.
