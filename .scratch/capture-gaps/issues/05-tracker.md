# Tracker

Status: ready-for-human
Scenario: `linux-tracker`
Targets: 115/116, 700–703, 710–713, 720, 721, 107 from the tracker, and the
server's UDP registration

Setup (agent): server detached; tracker started in the foreground first to
see whether it runs a console setup like the server (it asks for an admin
login and password, per its strings), then detached. Give the tracker a lab
admin login and record it in `lab/README.md`. The tracker listens on TCP and
UDP 10800 by default.

1. Connect to the server as admin. Commands → 127.0.0.1 → Server Settings,
   Trackers panel, Add Tracker...: fill every field with a distinct value
   (Name `lab`, Address `127.0.0.1`, Groups `Lab` and `Lab2`, and anything
   else the dialog offers). Save, then Save the settings (UDP registration).
   The distinct values identify the unknown string at datagram offset 0x34.
2. Connect button: Type "Tracker", Address `127.0.0.1`, no login, Connect
   (guest; the client sends 700 at once). Open the `Lab` group (702/703);
   the server should be listed.
3. Commands → 127.0.0.1 (the tracker) → Settings/Admin (710). As guest it
   should fail with 0x10014. Reconnect with the tracker's admin login and
   repeat (711). Change a setting, Save (712/713).
4. Commands → the tracker → Statistics (720/721).
5. Optional: set a tracker greeting and reconnect (107 after login).

What the capture should confirm, from `docs/protocol/tracker.md`:

- the UDP datagram decrypts with the LCG and passes the CRC check;
- the 701, 703, 711, and 721 layouts;
- guest gets 0x10014 for 710, 712, and 720.

Unknowns the capture or a static read should settle, each recorded in
`tracker.md` with its evidence:

- the string at registration offset 0x34 (the server's tracker entry
  +0x18; step 1's distinct values should show it);
- the server's registration interval (the timer that calls
  `FUN_08060e4c`);
- 711: bit 1 of the flags at 0x98, the four u16 limits, the two u16 values
  per group, the group flag at +0x0c, and the blob at tracker +0x224;
- whether 712 uses the 711 format (`FUN_0804d63c`);
- 721 offset 0x24 (tracker +0x1c);
- the client's connection type 4, default port 10900 (`FUN_0804bac0`).

`kdx-dissect` does not decode the tracker yet (`.scratch/tracker/` issue 03),
so check frames by hand or with a quick script until it does.

## Comments

2026-10-10: Read statically first; see `docs/protocol/tracker.md`. The steps
above now name the admin login, the guest refusal, and the distinct field
values that the static read calls for.

2026-10-11: The static unknowns are settled in `tracker.md`: offset 0x34 is
the tracker entry's Login, the server registers every 400000 ms, flags bit 1
is Enable Greeting, section 4 is the greeting, 712 uses the 711 format,
721 offset 0x24 is the open TCP connections, and connection type 4 is
GConsole. The group u16s are always 0. Two things the capture should check:
the client's 712 omits the admin section (present byte 0x71 is 0), so step
3 should also try changing the admin password and see that it does not
take; and step 5's greeting should arrive as 107 right after login.
