# kdx

A Rust implementation of the Haxial KDX protocol: first a client, then a
server and a tracker, all compatible with the original KDX programs (Server
1.620, Client 1.600, Tracker 1.110).

- `docs/protocol/` holds the protocol spec in progress:
  `docs/protocol/KDXProtocol.md` is the reference, and `docs/protocol/PLAN.md`
  is the plan.
- `crates/kdx-proto` handles the wire format: handshake, session encryption,
  message framing, and typed payloads (`msg`). It does no I/O.
- `crates/kdx-dissect` decodes KDX connections in a pcap or pcapng capture.
  It prints each message as a hex dump, with a `=` summary line for the
  payloads `kdx_proto::msg` knows. It decrypts transfer connections with the
  session keys from the capture's 105 Server Info messages, and `--export
  DIR` writes each connection's streams for use as fixtures:

  ```sh
  cargo run -p kdx-dissect -- captures/first-connect.pcapng --password 'lab-only-pw!1'
  ```

- `crates/kdx-client` is a blocking client library (`Client`) and two
  programs. `kdx` is a command-line client:

  ```sh
  cargo run -p kdx-client --bin kdx -- table fetch    # once; see below
  cargo run -p kdx-client --bin kdx -- -s host:10700 -l user ls 'Uploads [UL]'
  ```

  Its subcommands are `who`, `chats`, `ls`, `get`, `put`, `news`, `say`,
  `msg`, and `listen`, and the password comes from `--password` or
  `KDX_PASSWORD`. `kdx-probe` sends raw requests and dumps the replies.
- `lab/` runs the original programs in a sandbox and captures their traffic.
- `re/` holds the headless Ghidra tooling used for static analysis.

The original Haxial binaries and the challenge table they embed are Haxial's
copyrighted material. They are not in this repository; see
`vendor/MANIFEST.md` for where they come from. The handshake needs the
table, so `kdx` keeps a copy in `~/.cache/kdx/`. `kdx table fetch` downloads
the smallest Haxial archive (248 KB) from the Internet Archive, checks its
SHA-256, and extracts the table. `kdx table import FILE` takes it from any
KDX program, zip archive of one, or saved table instead.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT), at your option.
