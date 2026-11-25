# Scenario: cli-disconnect-ban

- Date: 2026-11-13
- Server: KDX Server 1.620 for Linux
- Programs: KDX Server 1.620 (Linux), the command-line client of the lab
- Server state: fresh first-run setup; admin `admin`, password `openkdx-lab` (published), guest access on; the first login uses `wrong-password`
- Frames: 90


## Steps

1. Start KDX Server 1.620 (Linux) in the sandbox; first-run setup creates admin 'admin', guest access enabled.
1. Start capturing loopback.
1. A login with a wrong password: the server answers the proof with status 1 and closes (log: 'Login failed').
1. A guest session ('victim') logs in and stays.
1. Admin: users (the victim is listed), then server disconnect victim --ban 10 --message bye (150).
1. The victim prints the 151 Disconnect Message (banned, 'bye') and exits with status 3; the admin gets a success Status for the 150.
1. A guest login right after: the server's own ban list does not apply to 127.0.0.1 (`docs/protocol/sequences.md`), so from the lab's loopback it succeeds; the ban itself is only visible as flag 2 in the 151.

## Observations

Names in the log and transcript are replaced: `lab` stands for the command-line client's command name and nick.

Server log:

```
26/11/13 03:00:20 PM [] KDX Server started o
26/11/13 03:00:21 PM [1 unnamed] Incoming connection from 127.0.0.1:54320
26/11/13 03:00:21 PM [1 unnamed] Login failed
26/11/13 03:00:21 PM [1 unnamed] Disconnected
26/11/13 03:00:21 PM [2 unnamed] Incoming connection from 127.0.0.1:54332
26/11/13 03:00:21 PM [2 unnamed] Set Name: <victim> (guest)
26/11/13 03:00:22 PM [3 unnamed] Incoming connection from 127.0.0.1:54336
26/11/13 03:00:22 PM [3 unnamed] Set Name: <lab> (admin)
26/11/13 03:00:22 PM [3 lab] Disconnected
26/11/13 03:00:22 PM [4 unnamed] Incoming connection from 127.0.0.1:54344
26/11/13 03:00:22 PM [4 unnamed] Set Name: <lab> (admin)
26/11/13 03:00:22 PM [2 victim] Disconnected
26/11/13 03:00:22 PM [4 lab] Disconnected
26/11/13 03:00:22 PM [5 unnamed] Incoming connection from 127.0.0.1:54350
26/11/13 03:00:22 PM [5 unnamed] Set Name: <lab> (guest)
26/11/13 03:00:22 PM [5 lab] Disconnected
```

## Client transcript

```
$ lab -l 'admin' --nick lab login
$ lab -l 'admin' --nick lab users
user_id  nick     status  info
2        victim   0
3        lab  0
$ lab -l 'admin' --nick lab server disconnect victim --ban 10 --message bye
$ lab -l '' --nick victim login --stay 20
name: New Server
description: 
version: 1.620
address: 
user_id: 2
users: 1
transfer_port: 0
can_modify_settings: 0
user_id: 3
nick: lab
user_id: 3
nick: #3
user_id: 4
nick: lab
banned: true
shutdown: false
by: lab
message: bye
message: bye
error: 
$ lab -l '' --nick lab login
name: New Server
description: 
version: 1.620
address: 
user_id: 5
users: 1
transfer_port: 0
can_modify_settings: 0
```

## Messages

Decoded by `kdx-dissect captures/cli-disconnect-ban.pcapng --inventory --password openkdx-lab -p wrong-password`.

```
4  C->S  127.0.0.1:54320 -> 127.0.0.1:10700  hello
6  S->C  127.0.0.1:54320 -> 127.0.0.1:10700  hello
8  C->S  127.0.0.1:54320 -> 127.0.0.1:10700  proof
9  S->C  127.0.0.1:54320 -> 127.0.0.1:10700  proof
16  C->S  127.0.0.1:54332 -> 127.0.0.1:10700  hello
18  S->C  127.0.0.1:54332 -> 127.0.0.1:10700  hello
20  C->S  127.0.0.1:54332 -> 127.0.0.1:10700  proof
21  S->C  127.0.0.1:54332 -> 127.0.0.1:10700  proof
22  C->S  127.0.0.1:54332 -> 127.0.0.1:10700  100 Set Identity
23  S->C  127.0.0.1:54332 -> 127.0.0.1:10700  101 User List
24  S->C  127.0.0.1:54332 -> 127.0.0.1:10700  105 Server Info
25  C->S  127.0.0.1:54332 -> 127.0.0.1:10700  118 (unnamed; license report)
30  C->S  127.0.0.1:54336 -> 127.0.0.1:10700  hello
32  S->C  127.0.0.1:54336 -> 127.0.0.1:10700  hello
34  C->S  127.0.0.1:54336 -> 127.0.0.1:10700  proof
35  S->C  127.0.0.1:54336 -> 127.0.0.1:10700  proof
36  C->S  127.0.0.1:54336 -> 127.0.0.1:10700  100 Set Identity
37  S->C  127.0.0.1:54336 -> 127.0.0.1:10700  101 User List
38  S->C  127.0.0.1:54336 -> 127.0.0.1:10700  105 Server Info
39  S->C  127.0.0.1:54332 -> 127.0.0.1:10700  103 User Joined
40  C->S  127.0.0.1:54336 -> 127.0.0.1:10700  118 (unnamed; license report)
44  S->C  127.0.0.1:54332 -> 127.0.0.1:10700  104 User Departed
49  C->S  127.0.0.1:54344 -> 127.0.0.1:10700  hello
51  S->C  127.0.0.1:54344 -> 127.0.0.1:10700  hello
53  C->S  127.0.0.1:54344 -> 127.0.0.1:10700  proof
54  S->C  127.0.0.1:54344 -> 127.0.0.1:10700  proof
55  C->S  127.0.0.1:54344 -> 127.0.0.1:10700  100 Set Identity
56  S->C  127.0.0.1:54344 -> 127.0.0.1:10700  101 User List
57  S->C  127.0.0.1:54344 -> 127.0.0.1:10700  105 Server Info
58  S->C  127.0.0.1:54332 -> 127.0.0.1:10700  103 User Joined
59  C->S  127.0.0.1:54344 -> 127.0.0.1:10700  118 (unnamed; license report)
62  C->S  127.0.0.1:54344 -> 127.0.0.1:10700  150 Disconnect User
64  S->C  127.0.0.1:54332 -> 127.0.0.1:10700  151 Disconnect Message
67  S->C  127.0.0.1:54344 -> 127.0.0.1:10700  1 Status (reply)
68  S->C  127.0.0.1:54344 -> 127.0.0.1:10700  104 User Departed
78  C->S  127.0.0.1:54350 -> 127.0.0.1:10700  hello
80  S->C  127.0.0.1:54350 -> 127.0.0.1:10700  hello
82  C->S  127.0.0.1:54350 -> 127.0.0.1:10700  proof
83  S->C  127.0.0.1:54350 -> 127.0.0.1:10700  proof
84  C->S  127.0.0.1:54350 -> 127.0.0.1:10700  100 Set Identity
85  S->C  127.0.0.1:54350 -> 127.0.0.1:10700  101 User List
86  S->C  127.0.0.1:54350 -> 127.0.0.1:10700  105 Server Info
87  C->S  127.0.0.1:54350 -> 127.0.0.1:10700  118 (unnamed; license report)
```
