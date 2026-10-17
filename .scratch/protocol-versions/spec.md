# Protocol versions

Goal: know how each Haxial build with `%KDX` differs on the wire, so the
Rust client, server, and tracker can say which versions they talk to. The
handshake's 0.9x section in `docs/protocol/handshake.md` and "Across
versions" in `docs/protocol/framing.md` hold what is known. The x86 1.x
builds agree on the header; the issues here are the gaps that survey left.

Builds come from `vendor/MANIFEST.md`. The Ghidra projects are named in
`re/README.md`.
