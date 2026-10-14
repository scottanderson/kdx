# kdx-tracker, a tracker

`kdx-tracker` is a drop-in for Tracker 1.110. Servers register with it over UDP
and clients browse it over TCP, both on one port (10800 by default). It does not
register with other trackers.

```sh
cargo run -p kdx-tracker -- --admin-login admin --admin-password secret
```

The first run needs `--admin-login` and `--admin-password` (or
`KDX_TRACKER_PASSWORD`). They are saved with the other settings in
`kdx-tracker.stg`, which later runs read.

| Option | Meaning |
|---|---|
| `--settings` | The settings file, created on the first run (`kdx-tracker.stg`) |
| `--address` | The address to listen on (0.0.0.0) |
| `--tracker-port` | The port for both TCP and UDP; saved in the settings |
| `--admin-login`, `--admin-password` | The admin account; saved in the settings |
| `--table` | The challenge table, or a KDX program file or archive that holds it; defaults to the cache |

Browse a running tracker with `kdx tracker` ([kdx](../cli/kdx.md#trackers)). The
registration and browse protocols are in [tracker.md](../protocol/tracker.md).
