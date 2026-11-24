# Scenario: win-guest-login

- Date: 2026-11-10
- Server: KDX Server 1.620 for Linux
- Programs: KDX Server 1.620 (Linux), KDX Client 1.600 (Windows under Wine 6.0.3)
- Server state: fresh first-run setup; admin `admin`, password `openkdx-lab` (published), guest access on
- Frames: 14


## Steps

1. Start KDX Server 1.620 (Linux) in the sandbox; first-run setup creates admin 'admin', guest access enabled.
1. Start KDX Client 1.600 (Windows, under Wine); accept the license; set user name 'guestuser'.
1. Start capturing loopback.
1. Connect → address '127.0.0.1', login '<empty>', password <empty>; press Connect.
1. Wait 5 s connected (idle), then stop capturing.

## Observations

Server log:

```
26/11/10 12:04:00 AM [] KDX Server started o
26/11/10 12:04:20 AM [1 unnamed] Incoming connection from 127.0.0.1:50750
26/11/10 12:04:22 AM [1 unnamed] Set Name: <guestuser> (guest)
```

## Messages

Decoded by `kdx-dissect captures/win-guest-login.pcapng --inventory --password openkdx-lab`.

```
4  C->S  127.0.0.1:50750 -> 127.0.0.1:10700  hello
6  S->C  127.0.0.1:50750 -> 127.0.0.1:10700  hello
8  C->S  127.0.0.1:50750 -> 127.0.0.1:10700  proof
9  S->C  127.0.0.1:50750 -> 127.0.0.1:10700  proof
10  C->S  127.0.0.1:50750 -> 127.0.0.1:10700  100 Set Identity
12  S->C  127.0.0.1:50750 -> 127.0.0.1:10700  101 User List
13  S->C  127.0.0.1:50750 -> 127.0.0.1:10700  105 Server Info
```
