# Run the lab client outside WSLg

Status: needs-triage

`kdx-lab client` requires `/tmp/.X11-unix/X0` and sets `DISPLAY=:0`. That
works under WSLg, which needs no X authentication. On a regular Linux
desktop the display number can differ, the X server needs the
`XAUTHORITY` cookie, and Wayland sessions offer X only through Xwayland.

Derive the socket from `$DISPLAY`, bind the Xauthority file read-only, and
test on a native X11 and a Wayland desktop.

## Comments

2026-10-11: `kdx-lab client` now derives the socket from `$DISPLAY` (`:N`,
`:N.S`, `unix:N`; `:0` when unset) and refuses TCP displays. It copies the
display's cookie from `$XAUTHORITY` or `~/.Xauthority` with `xauth`, as a
wildcard-host entry since the sandbox has its own hostname, and binds it at
`/tmp/.Xauthority`. Tested only statically: `bash -n` and a scratch test of
the parsing and bind arguments. Testing on a native X11 and a Wayland
desktop remains.
