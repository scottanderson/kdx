# kdx-cli

Two command-line programs for the Haxial KDX protocol.

- `kdx` is a client: list users and chats, browse, download and upload files,
  read and post news, manage accounts, chat, and browse a tracker.
- `kdx-probe` sends raw requests to a server and dumps the replies.

```sh
cargo install --path crates/kdx-cli
kdx table fetch
kdx ls -s host:10700 -l user 'Uploads [UL]'
```

`kdx table fetch` downloads the challenge tables the handshake needs and caches
them. See the [kdx
guide](https://github.com/scottanderson/kdx/blob/main/docs/cli/kdx.md), the
[kdx-probe
guide](https://github.com/scottanderson/kdx/blob/main/docs/cli/kdx-probe.md),
and [challenge
tables](https://github.com/scottanderson/kdx/blob/main/docs/setup/challenge-tables.md).

## License

Licensed under either of
[Apache License, Version 2.0](https://github.com/scottanderson/kdx/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/scottanderson/kdx/blob/main/LICENSE-MIT),
at your option.
