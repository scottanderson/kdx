# kdx-codec

`tokio_util::codec` framing for KDX connections, built on
[kdx-proto](https://crates.io/crates/kdx-proto). `HandshakeCodec` frames the
hello and proof each side sends first, and `SessionCodec` frames the encrypted
messages after them. `SessionCodec` streams the data of file downloads and
uploads in chunks, so a whole file is never held in memory.

Most programs want [kdx-client](https://crates.io/crates/kdx-client), which runs
the handshake and offers typed requests on top of these codecs. Use this crate
directly to build a server, a proxy, or a test peer.

The handshake needs Haxial's challenge tables, which are not distributed with
this crate; see [challenge
tables](https://github.com/scottanderson/kdx/blob/main/docs/setup/challenge-tables.md).

See the [protocol
reference](https://github.com/scottanderson/kdx/blob/main/docs/protocol/KDXProtocol.md)
for the protocol, and [docs.rs](https://docs.rs/kdx-codec) for the API.

## License

Licensed under either of
[Apache License, Version 2.0](https://github.com/scottanderson/kdx/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/scottanderson/kdx/blob/main/LICENSE-MIT),
at your option.
