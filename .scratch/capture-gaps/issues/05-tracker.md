# Tracker

Status: ready-for-human
Scenario: `linux-tracker`
Targets: 700–703, 710–713, 720, 721, and the server's UDP registration

Setup (agent): server detached; tracker started in the foreground first to
see whether it runs a console setup like the server, then detached.

1. Connect to the server as admin. Commands → 127.0.0.1 → Server Settings,
   Trackers panel, Add Tracker...: Name `lab`, Address `127.0.0.1`, Groups
   `Lab`. Save, then Save the settings (UDP registration).
2. Connect button: Type "Tracker", Address `127.0.0.1`, no login, Connect
   (700–703). Open the `Lab` group; the server should be listed.
3. Commands → 127.0.0.1 (the tracker) → Settings/Admin (710/711). Change a
   setting, Save (712/713). This may need a tracker login; if it is
   refused, note the error.
4. Commands → the tracker → Statistics (720/721).
