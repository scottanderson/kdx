# Scenario: linux-dcc-voice-failed

- Date: 2026-10-13T09:12Z
- Server: KDXServer 1.620 Linux (sha256 ad6fba4bdd01dc23)
- Client: two linux 1.620 beta 5: A (`client linux`, DCC listener on TCP 10600)
  and B (`client linux2`, its own state in `lab/run/client-linux2/`)
- Server state: existing `lab/run/server/` (admin `lab-only-pw!1`, guest access
  on, a "Voice Chat" chat)

Decode it with
`kdx-dissect captures/linux-dcc-voice-failed.pcapng --password 'lab-only-pw!1'`.

The capture follows `.scratch/capture-gaps/issues/07-dcc-voice.md`. The operator
drove both clients; the steps below are read back from the traffic. The
sandboxes have no sound devices, and in them each client answered the other's
1002 with "Operation is unimplemented. (ID 1/4 @ S101)", so no 1003 Voice Data
flows. The voice call failed; a capture in which voice works should be named
`linux-dcc-voice`.

## Steps

1. Frames 1-11: in B, Connect, Type "Client", `127.0.0.1:10600`, login `admin`
   with `lab-only-pw!1`.
2. Frames 13-28, 44-45: the same with an empty login. B sends a private message
   "dcc message body" (frame 28).
3. Frames 30-42: in B, Voice Chat on the DCC connection, while A's Voice
   Controls window is closed.
4. Frames 47-80: A logs in to the server as `admin` and joins "Voice Chat".
5. B restarts with its DCC port set to 10601. That wasn't needed: B calls A, so
   B's own listener is never used.
6. Frames 82-141: B logs in to the server as guest and joins "Voice Chat" with
   Voice Controls open in both; B calls A.
7. Frames 144-158: B leaves and joins again, then leaves.

## Observations

- Frames 1-11: A rejects a DCC login with a name: its proof (frame 9) has status
  1, and its auth is 4 bytes followed by zeros.
- Frames 13-28: the empty login opens a primary connection. Each side sends 117
  SKPK then 100 Set Identity. The SKPKs carry DCC port 10600 (0x2968) and
  version 0x0654; A's connection ID is 2, B's is 4. Each client numbers its own
  connections.
- Frames 30-42: B logs in as `00 00000002` with A's SKPK key K, and A's proof
  matches with K. B sends 1000; A answers 1001 with 0x00640007, and B closes.
- Frames 71-72: A, alone in the chat, gets an empty 223.
- Frames 108-109, 153-155: B gets a 223 with one entry, A's `127.0.0.1:10600`.
- Frames 110-123: B opens a new primary to A right after the 223. B's SKPK
  (frame 122) carries port 10601 (0x2969).
- Frames 125-141: B logs in as `00 00000004` with A's key K, sends 1000; A
  answers 1001 with 0 and both send 1002: 11025 Hz (0x2b11), `ADPC`, buffer 500
  ms (0x01f4), 1 channel, 16 bits. Each answers the other's 1002 with status
  0x10004 (unimplemented).
- Frames 148-155: the second join makes no new DCC connection; the first ones
  are still open.
