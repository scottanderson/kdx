# kdx

A Rust implementation of the Haxial KDX protocol: a client, a tracker, and later
a server, all compatible with the original KDX programs (Server 1.620, Client
1.600, Tracker 1.110).

**New here? Read [the protocol reference](docs/protocol/KDXProtocol.md).** It
describes how KDX connections work and every message on them, with the evidence
for each claim.

## Tools

| Tool | What it does | Docs |
|---|---|---|
| `kdx` | Command-line client: browse, download, upload, chat, news, accounts | [docs/cli/kdx.md](docs/cli/kdx.md) |
| `kdx-probe` | Send raw requests to a server and dump the replies | [docs/cli/kdx-probe.md](docs/cli/kdx-probe.md) |
| `kdx-tracker` | A tracker: servers register over UDP, clients browse over TCP | [docs/tracker/kdx-tracker.md](docs/tracker/kdx-tracker.md) |
| `kdx-dissect` | Decode KDX connections in a pcap or pcapng capture | [docs/dissect/kdx-dissect.md](docs/dissect/kdx-dissect.md) |

## Libraries

| Crate | What it is | Docs |
|---|---|---|
| `kdx-proto` | Wire format: handshake, encryption, framing, typed payloads. No I/O | [docs/libraries/kdx-proto.md](docs/libraries/kdx-proto.md) |
| `kdx-codec` | `tokio_util::codec` framing for connections | [docs/libraries/kdx-codec.md](docs/libraries/kdx-codec.md) |
| `kdx-client` | Async library for writing KDX clients | [docs/libraries/kdx-client.md](docs/libraries/kdx-client.md) |
| `kdx-test-tables` | Gives the tests the challenge tables | [docs/setup/challenge-tables.md](docs/setup/challenge-tables.md) |
| `kdx-server`, `kdx-daemon` | Empty placeholders for the server | [docs/libraries/placeholders.md](docs/libraries/placeholders.md) |

## Documentation

- [Protocol reference](docs/protocol/KDXProtocol.md): the entry point, with a
  map of every protocol page.
- [Client to server](docs/protocol/server.md),
  [tracker](docs/protocol/tracker.md), [client to client](docs/protocol/dcc.md),
  and [GConsole](docs/protocol/gconsole.md): one page for each kind of
  connection.
- [Plan](docs/protocol/PLAN.md): the phases and what is open.
- [Contributing](CONTRIBUTING.md): the checks, doc conventions, commits, and
  legal rules.
- [Lab](lab/README.md): run the original programs in a sandbox and capture their
  traffic.
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
