# Scenario: cli-upload-resume

- Date: 2026-11-13
- Server: KDX Server 1.620 for Linux
- Programs: KDX Server 1.620 (Linux), the command-line client of the lab
- Server state: fresh first-run setup; admin `admin`, password `openkdx-lab` (published), guest access on
- Frames: 179


## Steps

1. Start KDX Server 1.620 (Linux) in the sandbox; first-run setup creates admin 'admin', guest access enabled.
1. Make big.bin: 256 KiB of random bytes on the client side.
1. Start capturing loopback.
1. The command-line client runs `files put big.bin 'Uploads [UL]' --stop-after 150000`: 415, 416 (offset 0), a 417 header for 262144 bytes, then the client drops the connection after 150000 bytes.
1. The command-line client runs `files ls`: the server keeps the partial as big.bin.prt.
1. The command-line client runs `files put --resume`: 416 offers the offset the server holds (the size of the `.prt`); the client sends the rest under one 417 header, an empty 417 for the resource fork, and gets 418.
1. The command-line client runs `files ls`: big.bin is listed with its full size.
1. The command-line client runs `files get`: the download matches the original byte for byte.

## Observations

Names in the log and transcript are replaced: `lab` stands for the command-line client's command name and nick.

Server log:

```
26/11/13 02:37:22 PM [] KDX Server started o
26/11/13 02:37:23 PM [1 unnamed] Incoming connection from 127.0.0.1:36654
26/11/13 02:37:23 PM [1 unnamed] Set Name: <lab> (admin)
26/11/13 02:37:23 PM [2 unnamed] Incoming connection from 127.0.0.1:36662
26/11/13 02:37:23 PM [2 lab] Upload "big.bin".
26/11/13 02:37:24 PM [2 lab] Transfer Disconnect
26/11/13 02:37:24 PM [1 lab] Disconnected
26/11/13 02:37:25 PM [3 unnamed] Incoming connection from 127.0.0.1:36672
26/11/13 02:37:25 PM [3 unnamed] Set Name: <lab> (admin)
26/11/13 02:37:25 PM [3 lab] Disconnected
26/11/13 02:37:25 PM [4 unnamed] Incoming connection from 127.0.0.1:36678
26/11/13 02:37:25 PM [4 unnamed] Set Name: <lab> (admin)
26/11/13 02:37:25 PM [5 unnamed] Incoming connection from 127.0.0.1:36680
26/11/13 02:37:25 PM [5 lab] Upload "big.bin".
26/11/13 02:37:26 PM [5 lab] Transfer Disconnect
26/11/13 02:37:26 PM [4 lab] Disconnected
26/11/13 02:37:26 PM [6 unnamed] Incoming connection from 127.0.0.1:36696
26/11/13 02:37:26 PM [6 unnamed] Set Name: <lab> (admin)
26/11/13 02:37:26 PM [6 lab] Disconnected
26/11/13 02:37:26 PM [7 unnamed] Incoming connection from 127.0.0.1:36700
26/11/13 02:37:26 PM [7 unnamed] Set Name: <lab> (admin)
26/11/13 02:37:26 PM [8 unnamed] Incoming connection from 127.0.0.1:36712
26/11/13 02:37:26 PM [8 lab] Download "big.bin".
26/11/13 02:37:27 PM [8 lab] Transfer Disconnect
26/11/13 02:37:27 PM [7 lab] Disconnected
```

## Client transcript

```
name         size    modified             type  access
big.bin.prt  131072  2026-11-13 14:37:24
262144 bytes
to: Uploads [UL]/big.bin
bytes: 262144
from: 131072
name     size    modified             type         access
big.bin  262144  2026-11-13 14:37:26  exttype/bin
262144 bytes
path: Uploads [UL]/big.bin
to: /home/lab/lab/run/down.bin
bytes: 262144
```

## Messages

Decoded by `kdx-dissect captures/cli-upload-resume.pcapng --inventory --password openkdx-lab`.

```
4  C->S  127.0.0.1:36654 -> 127.0.0.1:10700  hello
6  S->C  127.0.0.1:36654 -> 127.0.0.1:10700  hello
8  C->S  127.0.0.1:36654 -> 127.0.0.1:10700  proof
9  S->C  127.0.0.1:36654 -> 127.0.0.1:10700  proof
10  C->S  127.0.0.1:36654 -> 127.0.0.1:10700  100 Set Identity
11  S->C  127.0.0.1:36654 -> 127.0.0.1:10700  101 User List
12  S->C  127.0.0.1:36654 -> 127.0.0.1:10700  105 Server Info
14  C->S  127.0.0.1:36654 -> 127.0.0.1:10700  118 (unnamed; license report)
18  C->S  127.0.0.1:36662 -> 127.0.0.1:10700  hello
20  S->C  127.0.0.1:36662 -> 127.0.0.1:10700  hello
22  C->S  127.0.0.1:36662 -> 127.0.0.1:10700  proof
23  S->C  127.0.0.1:36662 -> 127.0.0.1:10700  proof
24  C->S  127.0.0.1:36662 -> 127.0.0.1:10700  415 Upload File
25  S->C  127.0.0.1:36662 -> 127.0.0.1:10700  416 Upload File (R) (reply)
26  C->S  127.0.0.1:36662 -> 127.0.0.1:10700  417 Upload File (D) data header, 262144 bytes
29  C->S  127.0.0.1:36662 -> 127.0.0.1:10700  transfer data, 150000 bytes
37  C->S  127.0.0.1:36662 -> 127.0.0.1:10700  transfer data cut, 150000 of 262144 bytes
48  C->S  127.0.0.1:36672 -> 127.0.0.1:10700  hello
50  S->C  127.0.0.1:36672 -> 127.0.0.1:10700  hello
52  C->S  127.0.0.1:36672 -> 127.0.0.1:10700  proof
53  S->C  127.0.0.1:36672 -> 127.0.0.1:10700  proof
54  C->S  127.0.0.1:36672 -> 127.0.0.1:10700  100 Set Identity
55  S->C  127.0.0.1:36672 -> 127.0.0.1:10700  101 User List
56  S->C  127.0.0.1:36672 -> 127.0.0.1:10700  105 Server Info
57  C->S  127.0.0.1:36672 -> 127.0.0.1:10700  118 (unnamed; license report)
59  C->S  127.0.0.1:36672 -> 127.0.0.1:10700  400 Get File List
61  S->C  127.0.0.1:36672 -> 127.0.0.1:10700  401 File List (reply)
68  C->S  127.0.0.1:36678 -> 127.0.0.1:10700  hello
70  S->C  127.0.0.1:36678 -> 127.0.0.1:10700  hello
72  C->S  127.0.0.1:36678 -> 127.0.0.1:10700  proof
73  S->C  127.0.0.1:36678 -> 127.0.0.1:10700  proof
74  C->S  127.0.0.1:36678 -> 127.0.0.1:10700  100 Set Identity
75  S->C  127.0.0.1:36678 -> 127.0.0.1:10700  101 User List
76  S->C  127.0.0.1:36678 -> 127.0.0.1:10700  105 Server Info
77  C->S  127.0.0.1:36678 -> 127.0.0.1:10700  118 (unnamed; license report)
81  C->S  127.0.0.1:36680 -> 127.0.0.1:10700  hello
83  S->C  127.0.0.1:36680 -> 127.0.0.1:10700  hello
85  C->S  127.0.0.1:36680 -> 127.0.0.1:10700  proof
86  S->C  127.0.0.1:36680 -> 127.0.0.1:10700  proof
87  C->S  127.0.0.1:36680 -> 127.0.0.1:10700  415 Upload File
88  S->C  127.0.0.1:36680 -> 127.0.0.1:10700  416 Upload File (R) (reply)
89  C->S  127.0.0.1:36680 -> 127.0.0.1:10700  417 Upload File (D) data header, 131072 bytes
92  C->S  127.0.0.1:36680 -> 127.0.0.1:10700  transfer data, 131072 bytes
100  C->S  127.0.0.1:36680 -> 127.0.0.1:10700  417 Upload File (D) data header, 0 bytes
102  S->C  127.0.0.1:36680 -> 127.0.0.1:10700  418 Upload File (C)
112  C->S  127.0.0.1:36696 -> 127.0.0.1:10700  hello
114  S->C  127.0.0.1:36696 -> 127.0.0.1:10700  hello
116  C->S  127.0.0.1:36696 -> 127.0.0.1:10700  proof
117  S->C  127.0.0.1:36696 -> 127.0.0.1:10700  proof
118  C->S  127.0.0.1:36696 -> 127.0.0.1:10700  100 Set Identity
119  S->C  127.0.0.1:36696 -> 127.0.0.1:10700  101 User List
120  S->C  127.0.0.1:36696 -> 127.0.0.1:10700  105 Server Info
121  C->S  127.0.0.1:36696 -> 127.0.0.1:10700  118 (unnamed; license report)
123  C->S  127.0.0.1:36696 -> 127.0.0.1:10700  400 Get File List
125  S->C  127.0.0.1:36696 -> 127.0.0.1:10700  401 File List (reply)
132  C->S  127.0.0.1:36700 -> 127.0.0.1:10700  hello
134  S->C  127.0.0.1:36700 -> 127.0.0.1:10700  hello
136  C->S  127.0.0.1:36700 -> 127.0.0.1:10700  proof
137  S->C  127.0.0.1:36700 -> 127.0.0.1:10700  proof
138  C->S  127.0.0.1:36700 -> 127.0.0.1:10700  100 Set Identity
139  S->C  127.0.0.1:36700 -> 127.0.0.1:10700  101 User List
140  S->C  127.0.0.1:36700 -> 127.0.0.1:10700  105 Server Info
142  C->S  127.0.0.1:36700 -> 127.0.0.1:10700  118 (unnamed; license report)
146  C->S  127.0.0.1:36712 -> 127.0.0.1:10700  hello
148  S->C  127.0.0.1:36712 -> 127.0.0.1:10700  hello
150  C->S  127.0.0.1:36712 -> 127.0.0.1:10700  proof
151  S->C  127.0.0.1:36712 -> 127.0.0.1:10700  proof
152  C->S  127.0.0.1:36712 -> 127.0.0.1:10700  412 Download File
153  S->C  127.0.0.1:36712 -> 127.0.0.1:10700  413 Download File (R) (reply)
154  S->C  127.0.0.1:36712 -> 127.0.0.1:10700  414 Download File (D) data header, 262144 bytes
156  S->C  127.0.0.1:36712 -> 127.0.0.1:10700  transfer data, 262144 bytes
177  S->C  127.0.0.1:36712 -> 127.0.0.1:10700  414 Download File (D) data header, 0 bytes
```
