# Scenario: cli-connection-monitor

- Date: 2026-11-13
- Server: KDX Server 1.620 for Linux
- Programs: KDX Server 1.620 (Linux), the command-line client of the lab
- Server state: fresh first-run setup; admin `admin`, password `openkdx-lab` (published), guest access on
- Frames: 376


## Steps

1. Start KDX Server 1.620 (Linux) in the sandbox; first-run setup creates admin 'admin', guest access enabled.
1. Seed the base folder with four.bin (4 MiB).
1. Start capturing loopback.
1. A second admin session downloads four.bin on a transfer connection.
1. Admin: server monitor --for 6 --every 2 (140, then 142 → 143 four times: the sessions and the transfer with its byte counts, then 141).
1. The download completes.

## Observations

Names in the log and transcript are replaced: `lab` stands for the command-line client's command name and nick.

Server log:

```
26/11/13 02:56:08 PM [] KDX Server started o
26/11/13 02:56:08 PM [1 unnamed] Incoming connection from 127.0.0.1:46104
26/11/13 02:56:08 PM [1 unnamed] Set Name: <downloader> (admin)
26/11/13 02:56:08 PM [2 unnamed] Incoming connection from 127.0.0.1:46110
26/11/13 02:56:08 PM [2 downloader] Download "four.bin".
26/11/13 02:56:09 PM [3 unnamed] Incoming connection from 127.0.0.1:44682
26/11/13 02:56:09 PM [3 unnamed] Set Name: <monitor> (admin)
26/11/13 02:56:15 PM [3 monitor] Disconnected
26/11/13 02:56:20 PM [2 downloader] Transfer Disconnect
26/11/13 02:56:20 PM [1 downloader] Disconnected
```

## Client transcript

```
$ lab -l 'admin' --nick monitor server monitor --for 6 --every 2
user_id  transfer_id  nick        login  address          login_time           bytes_done  bytes_total  rate    path
1        0            downloader  admin  127.0.0.1:46104  2026-11-13 14:56:08  0           0            0
1        2            downloader  admin  127.0.0.1:46110  2026-11-13 14:56:08  524288      4194304      524288  four.bin
3        0            monitor     admin  127.0.0.1:44682  2026-11-13 14:56:09  0           0            0

user_id  transfer_id  nick        login  address          login_time           bytes_done  bytes_total  rate    path
1        0            downloader  admin  127.0.0.1:46104  2026-11-13 14:56:08  0           0            0
1        2            downloader  admin  127.0.0.1:46110  2026-11-13 14:56:08  1114112     4194304      371370  four.bin
3        0            monitor     admin  127.0.0.1:44682  2026-11-13 14:56:09  0           0            0

user_id  transfer_id  nick        login  address          login_time           bytes_done  bytes_total  rate    path
1        0            downloader  admin  127.0.0.1:46104  2026-11-13 14:56:08  0           0            0
1        2            downloader  admin  127.0.0.1:46110  2026-11-13 14:56:08  1900544     4194304      380108  four.bin
3        0            monitor     admin  127.0.0.1:44682  2026-11-13 14:56:09  0           0            0

user_id  transfer_id  nick        login  address          login_time           bytes_done  bytes_total  rate    path
1        0            downloader  admin  127.0.0.1:46104  2026-11-13 14:56:08  0           0            0
1        2            downloader  admin  127.0.0.1:46110  2026-11-13 14:56:08  2686976     4194304      383853  four.bin
3        0            monitor     admin  127.0.0.1:44682  2026-11-13 14:56:09  0           0            0
$ lab -l 'admin' --nick downloader files get four.bin --to /home/lab/lab/run/four.down
4194304 bytes
path: four.bin
to: /home/lab/lab/run/four.down
bytes: 4194304
```

## Messages

Decoded by `kdx-dissect captures/cli-connection-monitor.pcapng --inventory --password openkdx-lab`.

```
4  C->S  127.0.0.1:46104 -> 127.0.0.1:10700  hello
6  S->C  127.0.0.1:46104 -> 127.0.0.1:10700  hello
8  C->S  127.0.0.1:46104 -> 127.0.0.1:10700  proof
9  S->C  127.0.0.1:46104 -> 127.0.0.1:10700  proof
10  C->S  127.0.0.1:46104 -> 127.0.0.1:10700  100 Set Identity
11  S->C  127.0.0.1:46104 -> 127.0.0.1:10700  101 User List
12  S->C  127.0.0.1:46104 -> 127.0.0.1:10700  105 Server Info
13  C->S  127.0.0.1:46104 -> 127.0.0.1:10700  118 (unnamed; license report)
17  C->S  127.0.0.1:46110 -> 127.0.0.1:10700  hello
19  S->C  127.0.0.1:46110 -> 127.0.0.1:10700  hello
21  C->S  127.0.0.1:46110 -> 127.0.0.1:10700  proof
22  S->C  127.0.0.1:46110 -> 127.0.0.1:10700  proof
23  C->S  127.0.0.1:46110 -> 127.0.0.1:10700  412 Download File
24  S->C  127.0.0.1:46110 -> 127.0.0.1:10700  413 Download File (R) (reply)
25  S->C  127.0.0.1:46110 -> 127.0.0.1:10700  414 Download File (D) data header, 4194304 bytes
27  S->C  127.0.0.1:46110 -> 127.0.0.1:10700  transfer data, 4194304 bytes
54  C->S  127.0.0.1:44682 -> 127.0.0.1:10700  hello
56  S->C  127.0.0.1:44682 -> 127.0.0.1:10700  hello
58  C->S  127.0.0.1:44682 -> 127.0.0.1:10700  proof
59  S->C  127.0.0.1:44682 -> 127.0.0.1:10700  proof
60  C->S  127.0.0.1:44682 -> 127.0.0.1:10700  100 Set Identity
61  S->C  127.0.0.1:44682 -> 127.0.0.1:10700  101 User List
62  S->C  127.0.0.1:44682 -> 127.0.0.1:10700  105 Server Info
63  S->C  127.0.0.1:46104 -> 127.0.0.1:10700  103 User Joined
64  C->S  127.0.0.1:44682 -> 127.0.0.1:10700  118 (unnamed; license report)
68  C->S  127.0.0.1:44682 -> 127.0.0.1:10700  140 Open Connection Monitor
68  C->S  127.0.0.1:44682 -> 127.0.0.1:10700  142 Get Connection Monitor Info
70  S->C  127.0.0.1:44682 -> 127.0.0.1:10700  143 Connection Monitor Info (reply)
116  C->S  127.0.0.1:44682 -> 127.0.0.1:10700  142 Get Connection Monitor Info
117  S->C  127.0.0.1:44682 -> 127.0.0.1:10700  143 Connection Monitor Info (reply)
175  C->S  127.0.0.1:44682 -> 127.0.0.1:10700  142 Get Connection Monitor Info
176  S->C  127.0.0.1:44682 -> 127.0.0.1:10700  143 Connection Monitor Info (reply)
234  C->S  127.0.0.1:44682 -> 127.0.0.1:10700  142 Get Connection Monitor Info
235  S->C  127.0.0.1:44682 -> 127.0.0.1:10700  143 Connection Monitor Info (reply)
237  C->S  127.0.0.1:44682 -> 127.0.0.1:10700  141 Close Connection Monitor
241  S->C  127.0.0.1:46104 -> 127.0.0.1:10700  104 User Departed
368  S->C  127.0.0.1:46110 -> 127.0.0.1:10700  414 Download File (D) data header, 0 bytes
```
