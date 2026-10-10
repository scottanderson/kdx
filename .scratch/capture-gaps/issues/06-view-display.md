# View Display

Status: needs-info
Scenario: `linux-view-display`
Targets: 900–905

Server 1.620 for Linux has no screen support: 900 can only fail with 0x10004 (unimplemented),
and the 904/905 injectors are empty (`re/opcodes.md`). A nested X display does
not help. Options:

- Capture only the failure against the Linux server: 900 and its error.
- Run a Windows or Mac server, which has a real screen grab. That needs
  Wine or another OS, so it is out of scope for the Linux-only captures.

Steps, either way:

1. Connect as admin.
2. Commands → 127.0.0.1 → View/Control Display (900, then 902/903 or an
   error).
3. Click inside the view (904), then type `abc` (905).
4. Close the View/Control Display window (901 [I]).
