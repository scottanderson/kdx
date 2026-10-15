# kdx-tracker

A tracker for the Haxial KDX protocol, compatible with Tracker 1.110. Servers
register with it over UDP and clients browse it over TCP, both on one port
(10800 by default).

```sh
kdx-tracker --admin-login admin --admin-password secret
```

The first run saves its settings, including the admin account, in
`kdx-tracker.stg`. As a library it offers `Tracker::run` to embed the same
tracker in another program.

See the [kdx-tracker
guide](https://github.com/scottanderson/kdx/blob/main/docs/tracker/kdx-tracker.md)
and the [tracker
protocol](https://github.com/scottanderson/kdx/blob/main/docs/protocol/tracker.md) and
[registration datagram](https://github.com/scottanderson/kdx/blob/main/docs/protocol/udp.md).
The handshake needs Haxial's challenge tables, which are not distributed with
this crate; see [challenge
tables](https://github.com/scottanderson/kdx/blob/main/docs/setup/challenge-tables.md).

## License

Licensed under either of
[Apache License, Version 2.0](https://github.com/scottanderson/kdx/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/scottanderson/kdx/blob/main/LICENSE-MIT),
at your option.
