# kdx-client

An async library, on tokio, for writing clients of the Haxial KDX protocol.

- `Connection` runs the handshake and sends raw session messages.
- `Client` logs in to a server and offers typed requests: users, chats, files,
  news, and accounts.
- `Tracker` does the same for a tracker.

It works against KDX Server 1.620 and Tracker 1.110. The [kdx command-line
client](https://github.com/scottanderson/kdx/blob/main/docs/cli/kdx.md) is built
on it.

The handshake needs Haxial's challenge tables, which are not distributed with
this crate; see [challenge
tables](https://github.com/scottanderson/kdx/blob/main/docs/setup/challenge-tables.md).

See the [protocol
reference](https://github.com/scottanderson/kdx/blob/main/docs/protocol/KDXProtocol.md)
for the messages, and [docs.rs](https://docs.rs/kdx-client) for the API.

## License

Licensed under either of
[Apache License, Version 2.0](https://github.com/scottanderson/kdx/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/scottanderson/kdx/blob/main/LICENSE-MIT),
at your option.
