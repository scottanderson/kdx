# kdx-client

An async library, on tokio, for writing clients of the Haxial KDX protocol.

- `Connection` runs the handshake and sends raw session messages.
- `Client` logs in to a server and offers typed requests: users, chats, files,
  news, accounts and classes, folder access, administration, and transfers to
  and from local files with progress and resume.
- `Tracker` does the same for a tracker.

It works against KDX Server 1.620 and Tracker 1.110. The [kdx command-line
client](https://github.com/scottanderson/kdx/blob/main/docs/cli/kdx.md) is built
on it.

The handshake needs Haxial's challenge tables, which are not distributed with
this crate; see [challenge
tables](https://github.com/scottanderson/kdx/blob/main/docs/setup/challenge-tables.md).

## Testing against the mock

The `mock` feature adds `kdx_client::mock`, an in-process server for tests.
It runs the real handshake and framing from `kdx-codec` over a script of
steps: expect a message type from the client, reply to it, push a message,
stream or consume transfer data, close. Both sides use
`ChallengeTable::default()`, so no Haxial table is needed. The mock records
what the client sent and `finish()` panics if the client broke the script.

```toml
[dev-dependencies]
kdx-client = { version = "0.1", features = ["mock"] }
```

## Lab tests

`tests/lab.rs` runs the client against KDX Server 1.620 and Tracker 1.110.
It skips unless `KDX_LAB_SERVER` (`HOST:PORT`) is set; `KDX_LAB_TRACKER`,
`KDX_LAB_PASSWORD` (default `openkdx-lab`) and `KDX_LAB_TRACKER_HAS_SERVER`
tune it. The tests share one server and one address, and the server bans an
address after 30 connections in 300 seconds, so run them with
`--test-threads=1`; the suite opens at most 25. The lab that runs the
original programs runs it through its differential script, which this
repository's lab workflow calls.

`Client::next_notice()` decodes the messages a server pushes ([208 Receive
Chat Text](https://github.com/scottanderson/kdx/blob/main/docs/protocol/messages/chat.md#m208),
[301 Incoming Private Message](https://github.com/scottanderson/kdx/blob/main/docs/protocol/messages/private-messages.md#m301),
[151 Disconnect Message](https://github.com/scottanderson/kdx/blob/main/docs/protocol/messages/administration.md#m151),
...) into the `Notice` enum; a type it does not know comes through as
`Notice::Other`.

See the [message
index](https://github.com/scottanderson/kdx/blob/main/docs/protocol/messages/README.md)
for the messages, and [docs.rs](https://docs.rs/kdx-client) for the API.

## License

Licensed under either of
[Apache License, Version 2.0](https://github.com/scottanderson/kdx/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/scottanderson/kdx/blob/main/LICENSE-MIT),
at your option.
