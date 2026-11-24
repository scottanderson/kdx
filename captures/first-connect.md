# Scenario: first-connect

- Date: 2026-10-08T07:17Z to 07:19Z
- Server: Server 1.620 for Linux, sha256
  `ad6fba4bdd01dc237abde2b76054e3bbbb43c6d0c79a28d4e0de3b23e546f943`
- Client: Client 1.600 for Windows, sha256
  `99d4a8f45c61fca1ee5753028de28479e040f43a08c507ccfcc23be92c20b983`, under Wine
- Server state: fresh, set up on first run with admin / `lab-only-pw!1` and
  guest access allowed
- Decode: `kdx-dissect captures/first-connect.pcapng --password 'lab-only-pw!1'`

## Steps
1. Connected to 127.0.0.1 as admin (password lab-only-pw!1).
2. Sent several private messages to self.
3. Read news.
4. Disconnected.

(Recorded from the operator's description; the exact order and count of PMs
weren't given.)

## Observations
