# kdx

A Rust implementation of the Haxial KDX protocol: a client, a tracker, and later
a server, all compatible with the original KDX programs (Server 1.620, Client
1.600, Tracker 1.110).

**New here? Read [the protocol reference](docs/protocol/README.md).** It
describes how KDX connections work and every message on them, with the evidence
for each claim.

## Tools

| Tool | Crate | What it does | Docs |
|---|---|---|---|
| `kdx` | `kdx-cli` | Command-line client: browse, download, upload, chat, news, accounts | [docs/cli/kdx.md](docs/cli/kdx.md) |
| `kdx-probe` | `kdx-cli` | Send raw requests to a server and dump the replies | [docs/cli/kdx-probe.md](docs/cli/kdx-probe.md) |
| `kdx-tracker` | `kdx-tracker` | A tracker: servers register over UDP, clients browse over TCP | [docs/tracker/kdx-tracker.md](docs/tracker/kdx-tracker.md) |
| `kdx-dissect` | `kdx-dissect` | Decode KDX connections in a pcap or pcapng capture | [docs/dissect/kdx-dissect.md](docs/dissect/kdx-dissect.md) |

## Development tools

| Tool | What it does | Docs |
|---|---|---|
| `lab/kdx-lab` | Run the original Haxial programs in a sandbox and capture their traffic | [lab/README.md](lab/README.md) |
| `vendor/fetch` | Download the Haxial archives listed in the manifest, verify and extract them | [vendor/MANIFEST.md](vendor/MANIFEST.md) |
| `re/ghidra`, `re/ghidra-native` | Headless Ghidra for reading the original binaries | [re/README.md](re/README.md) |

## Crates

| Crate | What it is | README |
|---|---|---|
| `kdx-proto` | Wire format: handshake, encryption, framing, typed payloads. No I/O | [crates/kdx-proto/README.md](crates/kdx-proto/README.md) |
| `kdx-codec` | `tokio_util::codec` framing for connections | [crates/kdx-codec/README.md](crates/kdx-codec/README.md) |
| `kdx-client` | Async library for writing KDX clients | [crates/kdx-client/README.md](crates/kdx-client/README.md) |
| `kdx-cli` | The `kdx` and `kdx-probe` programs | [crates/kdx-cli/README.md](crates/kdx-cli/README.md) |
| `kdx-tracker` | The tracker program and library | [crates/kdx-tracker/README.md](crates/kdx-tracker/README.md) |
| `kdx-dissect` | The capture dissector | [crates/kdx-dissect/README.md](crates/kdx-dissect/README.md) |
| `kdx-test-tables` | Gives the tests the challenge tables | [crates/kdx-test-tables/README.md](crates/kdx-test-tables/README.md) |
| `kdx-server` | Empty placeholder for the server library | [crates/kdx-server/README.md](crates/kdx-server/README.md) |
| `kdx-daemon` | Empty placeholder for the server program | [crates/kdx-daemon/README.md](crates/kdx-daemon/README.md) |

## Documentation

- [Protocol reference](docs/protocol/README.md): the entry point, with a
  map of every protocol page.
- [Client to server](docs/protocol/server.md),
  [tracker](docs/protocol/tracker.md), [client to client](docs/protocol/dcc.md),
  and [GConsole](docs/protocol/gconsole.md): one page for each kind of
  connection.
- [Plan](docs/PLAN.md): the phases and what is open.
- [Contributing](CONTRIBUTING.md): the checks, doc conventions, commits, and
  legal rules.
- [Lab](lab/README.md): run the original programs in a sandbox and capture their
  traffic.
- [Captures](captures/README.md): an index of the capture logs.
- [Static analysis](re/README.md): the headless Ghidra tooling and address
  landmarks.
- [Vendor manifest](vendor/MANIFEST.md): where the Haxial archives come from.

## Getting started

The crates need Rust 1.96 or newer. What else you need depends on the work:

- **The client, the dissector, and the tests** need only `cargo`. Run `cargo run
  --bin kdx -- table fetch` once before connecting to a server or running the
  tests; see [challenge tables](docs/setup/challenge-tables.md).
- **Captures in the lab** need the Haxial binaries (`vendor/fetch`) and the
  packages in [lab/README.md](lab/README.md).
- **Static analysis** needs the binaries, Docker, and Ghidra
  ([re/README.md](re/README.md)).

Running the lab means using Haxial's software, so the Windows client shows its
license agreement. Whether to accept it is your decision.

The original Haxial binaries and the challenge tables they embed are Haxial's
copyrighted material. They are not in this repository.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT
license](LICENSE-MIT), at your option.
