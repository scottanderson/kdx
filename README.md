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

- `crates/kdx-client` is a blocking library for writing KDX clients.
  `Connection` runs the handshake and sends raw session messages, and
  `Client` logs in and offers typed requests on top of it.
- `crates/kdx-cli` holds two programs built on `kdx-client`. `kdx` is a
  command-line client:

  ```sh
  cargo run -p kdx-cli --bin kdx -- table fetch    # once; see below
  cargo run -p kdx-cli --bin kdx -- -s host:10700 -l user ls 'Uploads [UL]'
  ```

  Its subcommands are `who`, `chats`, `ls`, `get`, `put`, `news`, `post`,
  `delete-news`, `account`, `passwd`, `say`, `msg`, and `listen`, and the
  password comes from `--password` or `KDX_PASSWORD`. `kdx help` lists them.
  `kdx-probe` sends raw requests and dumps the replies.
- The other crates are empty placeholders for the work in
  `docs/protocol/PLAN.md`. `crates/kdx-codec` will frame sessions over async
  I/O. `crates/kdx-server` will be a library for writing KDX servers, and
  `crates/kdx-daemon` the server built on it. `crates/kdx-tracker` will be
  the tracker.
- `lab/` runs the original programs in a sandbox and captures their traffic.
- `re/` holds the headless Ghidra tooling used for static analysis.

The original Haxial binaries and the challenge table they embed are Haxial's
copyrighted material. They are not in this repository; see
`vendor/MANIFEST.md` for where they come from. The handshake needs the
table, so `kdx` keeps a copy in `~/.cache/kdx/`. `kdx table fetch` downloads
the smallest Haxial archive (248 KB) from the Internet Archive, checks its
SHA-256, and extracts the table. `kdx table import FILE` takes it from any
KDX program, zip archive of one, or saved table instead.

## Getting started

The crates need Rust 1.96 or newer. What else you need depends on the work:

- **The client, the dissector, and the tests** need only `cargo`. Run `kdx
  table fetch` once before connecting to a server; the tests that need the
  table read it from `vendor/haxial/` or that cache, and skip without either.
- **Captures in the lab** need the Haxial binaries (`vendor/fetch`) and the
  packages in `lab/README.md`.
- **Static analysis** needs the binaries, Docker, and Ghidra (`re/README.md`).

Running the lab means using Haxial's software, so the Windows client shows
its license agreement. Whether to accept it is your decision.

See `CONTRIBUTING.md` for the checks, the doc conventions, and the legal
rules.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT), at your option.
