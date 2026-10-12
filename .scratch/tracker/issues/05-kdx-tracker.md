# A Rust tracker

Status: ready-for-agent
Blocked by: 01, 02

Implement `kdx-tracker` (now a placeholder): accept registrations on UDP
10800 with the same checks, limits, and expiry as Tracker 1.110, and serve
115, 700, 702, 710, 712, and 720 on TCP 10800 with guest and admin logins.
Done when Client 1.620 beta 5 lists a lab server through it, and Server
1.620 registers with it. Settings storage format is open: copying
`KDXTrackerServ.stg` is not needed.
