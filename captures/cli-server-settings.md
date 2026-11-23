# Scenario: cli-server-settings

- Date: 2026-11-13
- Server: KDX Server 1.620 for Linux
- Programs: KDX Server 1.620 (Linux), the command-line client of the lab
- Server state: fresh first-run setup; admin `admin`, password `openkdx-lab` (published), guest access on
- Frames: 64


## Steps

1. Start KDX Server 1.620 (Linux) in the sandbox; first-run setup creates admin 'admin', guest access enabled.
1. Start capturing loopback.
1. Admin: server settings (170/171), server set-name a new name (170/171 then 172 with section 0's name and length byte changed → Status), server settings again.
1. A second login: 105 Server Info now names the server a new name.
1. Restore the name.

## Observations

Names in the log and transcript are replaced: `lab` stands for the command-line client's command name and nick.

Server log:

```
26/11/13 02:54:16 PM [] KDX Server started o
26/11/13 02:54:17 PM [1 unnamed] Incoming connection from 127.0.0.1:43622
26/11/13 02:54:17 PM [1 unnamed] Set Name: <lab> (admin)
26/11/13 02:54:17 PM [1 lab] Disconnected
26/11/13 02:54:17 PM [2 unnamed] Incoming connection from 127.0.0.1:43628
26/11/13 02:54:17 PM [2 unnamed] Set Name: <lab> (admin)
26/11/13 02:54:17 PM [2 lab] Disconnected
26/11/13 02:54:17 PM [3 unnamed] Incoming connection from 127.0.0.1:43636
26/11/13 02:54:17 PM [3 unnamed] Set Name: <lab> (admin)
26/11/13 02:54:17 PM [3 lab] Disconnected
```

## Client transcript

```
$ lab -l 'admin' --nick lab run --keep-going
> server settings
version: 1
name: New Server
greeting_enabled: false
total_outgoing_speed_limit: 0
section_lengths: [104,0,0,0,0,0,0,0,0,0,0]
> server set-name "<new name>"
> server settings
version: 1
name: <new name>
greeting_enabled: false
total_outgoing_speed_limit: 0
section_lengths: [104,0,0,0,0,0,0,0,0,0,0]
$ lab -l 'admin' --nick lab login
name: <new name>
description: 
version: 1.620
address: 
user_id: 2
users: 1
transfer_port: 0
can_modify_settings: 1
$ lab -l 'admin' --nick lab server set-name New Server
```

## Messages

Decoded by `kdx-dissect captures/cli-server-settings.pcapng --inventory --password openkdx-lab`.

```
4  C->S  127.0.0.1:43622 -> 127.0.0.1:10700  hello
6  S->C  127.0.0.1:43622 -> 127.0.0.1:10700  hello
8  C->S  127.0.0.1:43622 -> 127.0.0.1:10700  proof
9  S->C  127.0.0.1:43622 -> 127.0.0.1:10700  proof
10  C->S  127.0.0.1:43622 -> 127.0.0.1:10700  100 Set Identity
11  S->C  127.0.0.1:43622 -> 127.0.0.1:10700  101 User List
12  S->C  127.0.0.1:43622 -> 127.0.0.1:10700  105 Server Info
13  C->S  127.0.0.1:43622 -> 127.0.0.1:10700  118
15  C->S  127.0.0.1:43622 -> 127.0.0.1:10700  170 Get Server Settings
17  S->C  127.0.0.1:43622 -> 127.0.0.1:10700  171 Server Settings Info (reply)
18  C->S  127.0.0.1:43622 -> 127.0.0.1:10700  170 Get Server Settings
19  S->C  127.0.0.1:43622 -> 127.0.0.1:10700  171 Server Settings Info (reply)
20  C->S  127.0.0.1:43622 -> 127.0.0.1:10700  172 Set Server Settings
21  S->C  127.0.0.1:43622 -> 127.0.0.1:10700  173 Set Server Settings Done (reply)
22  C->S  127.0.0.1:43622 -> 127.0.0.1:10700  170 Get Server Settings
23  S->C  127.0.0.1:43622 -> 127.0.0.1:10700  171 Server Settings Info (reply)
30  C->S  127.0.0.1:43628 -> 127.0.0.1:10700  hello
32  S->C  127.0.0.1:43628 -> 127.0.0.1:10700  hello
34  C->S  127.0.0.1:43628 -> 127.0.0.1:10700  proof
35  S->C  127.0.0.1:43628 -> 127.0.0.1:10700  proof
36  C->S  127.0.0.1:43628 -> 127.0.0.1:10700  100 Set Identity
37  S->C  127.0.0.1:43628 -> 127.0.0.1:10700  101 User List
38  S->C  127.0.0.1:43628 -> 127.0.0.1:10700  105 Server Info
39  C->S  127.0.0.1:43628 -> 127.0.0.1:10700  118
46  C->S  127.0.0.1:43636 -> 127.0.0.1:10700  hello
48  S->C  127.0.0.1:43636 -> 127.0.0.1:10700  hello
50  C->S  127.0.0.1:43636 -> 127.0.0.1:10700  proof
51  S->C  127.0.0.1:43636 -> 127.0.0.1:10700  proof
52  C->S  127.0.0.1:43636 -> 127.0.0.1:10700  100 Set Identity
53  S->C  127.0.0.1:43636 -> 127.0.0.1:10700  101 User List
54  S->C  127.0.0.1:43636 -> 127.0.0.1:10700  105 Server Info
55  C->S  127.0.0.1:43636 -> 127.0.0.1:10700  118
57  C->S  127.0.0.1:43636 -> 127.0.0.1:10700  170 Get Server Settings
59  S->C  127.0.0.1:43636 -> 127.0.0.1:10700  171 Server Settings Info (reply)
60  C->S  127.0.0.1:43636 -> 127.0.0.1:10700  172 Set Server Settings
61  S->C  127.0.0.1:43636 -> 127.0.0.1:10700  173 Set Server Settings Done (reply)
```
