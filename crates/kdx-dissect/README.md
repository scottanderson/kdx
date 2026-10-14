# kdx-dissect

Decodes KDX connections in a pcap or pcapng capture. It prints the handshakes
and every session message as a hex dump, with a summary line for the payloads it
understands. It decrypts file transfer connections with the session keys found
in the capture, and also reads the tracker's UDP registrations.

```sh
kdx-dissect capture.pcapng --password 'account-password'
```

`--audit` checks that each payload encodes back to the same bytes with its
[kdx-proto](https://crates.io/crates/kdx-proto) codec, and `--export DIR` writes
each connection's streams for use as test fixtures.

See the [kdx-dissect
guide](https://github.com/scottanderson/kdx/blob/main/docs/dissect/kdx-dissect.md).
The handshake needs Haxial's challenge tables, which are not distributed with
this crate; see [challenge
tables](https://github.com/scottanderson/kdx/blob/main/docs/setup/challenge-tables.md).

## License

Licensed under either of
[Apache License, Version 2.0](https://github.com/scottanderson/kdx/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/scottanderson/kdx/blob/main/LICENSE-MIT),
at your option.
