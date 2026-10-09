# Run the lab client outside WSLg

Status: needs-triage

`kdx-lab client` requires `/tmp/.X11-unix/X0` and sets `DISPLAY=:0`. That
works under WSLg, which needs no X authentication. On a regular Linux
desktop the display number can differ, the X server needs the
`XAUTHORITY` cookie, and Wayland sessions offer X only through Xwayland.

Derive the socket from `$DISPLAY`, bind the Xauthority file read-only, and
test on a native X11 and a Wayland desktop.
