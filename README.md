# kdx

A Rust implementation of the Haxial KDX protocol: first a client, then a
server and a tracker, all compatible with the original KDX programs (Server
1.620, Client 1.600, Tracker 1.110).

- `docs/protocol/` is the protocol spec in progress. Start with `PLAN.md`.
- `crates/kdx-proto` handles the wire format: handshake, session encryption,
  and message framing. It does no I/O.
- `crates/kdx-dissect` decodes KDX connections in a pcap or pcapng capture:

  ```sh
  cargo run -p kdx-dissect -- captures/first-connect.pcapng --password 'lab-only-pw!1'
  ```

- `lab/` runs the original programs in a sandbox and captures their traffic.
- `re/` holds the headless Ghidra tooling used for static analysis.

The original Haxial binaries and the challenge table they embed are Haxial's
copyrighted material. They are not in this repository; see
`vendor/MANIFEST.md` for where they come from.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT), at your option.
