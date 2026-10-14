# kdx-dissect, a capture decoder

`kdx-dissect` reads a pcap or pcapng capture and prints every KDX connection in
it: the handshake, then each message as a hex dump, with a `=` summary line for
payloads that [kdx-proto](../libraries/kdx-proto.md) decodes.

```sh
cargo run -p kdx-dissect -- captures/first-connect.pcapng --password 'lab-only-pw!1'
```

## What it handles

- Server connections, and the transfer connections that attach to them, using
  the session keys from the capture's 105 Server Info messages.
- Tracker sessions and the UDP registrations sent to a tracker.
- Client to client (DCC) connections and GConsole.

A session is encrypted under the account key, so the dissector needs the
password: give each candidate with `--password` (repeatable) and it picks the
one whose proof matches. The empty password is always tried.

## Options

| Option | Meaning |
|---|---|
| `--password`, `-p` | A candidate account password |
| `--export DIR` | Write each connection's reassembled streams to DIR, named by source port, for use as test fixtures |
| `--audit` | After each payload, say whether its codec re-encodes it byte for byte |
| `--client-port`, `--server-port`, `--tracker-port`, `--gconsole-port` | Where each kind of service listens, if not on the default |

## The audit

`--audit` decodes each payload with its `kdx-proto` type, encodes it again, and
compares. A payload is the same, has `no codec`, has `trailing` bytes after the
message, or is a `mismatch`, which is a codec bug. `crates/kdx-dissect/tests/captures.rs` runs the audit over every
capture and lists the types with no codec in `NO_CODEC`.

The captures themselves, and how they are made, are in
[lab/README.md](../../lab/README.md). The message layouts are in [the protocol
reference](../protocol/KDXProtocol.md).
