# Register a Rust server with trackers

Status: needs-triage
Blocked by: 01, kdx-server (not yet tracked)

When `kdx-server` exists, send the registration datagram to each configured
tracker on the interval capture-gaps 05 finds, with a server ID generated once and
kept in the settings. Done when Tracker 1.110 lists the Rust server.
