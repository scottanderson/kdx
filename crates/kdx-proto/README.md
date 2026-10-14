# kdx-proto

The wire format of the Haxial KDX protocol: the connection handshake, session
encryption, message framing, and a typed codec for each message payload. It does
no I/O and has no async runtime, so it fits under any transport.

Codecs decode a payload and encode it again byte for byte, keeping the bytes
they do not understand. `Decoded` also keeps bytes that follow a message.

```rust
use kdx_proto::msg::chat::ChatId;

let payload = ChatId(1).encode();
assert_eq!(ChatId::decode(&payload).unwrap(), ChatId(1));
```

The handshake needs Haxial's challenge tables, which are not distributed with
this crate; see [challenge
tables](https://github.com/scottanderson/kdx/blob/main/docs/setup/challenge-tables.md).

For the layout of each message, see the [protocol
reference](https://github.com/scottanderson/kdx/blob/main/docs/protocol/KDXProtocol.md).
Types and functions are documented on [docs.rs](https://docs.rs/kdx-proto).

## License

Licensed under either of
[Apache License, Version 2.0](https://github.com/scottanderson/kdx/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/scottanderson/kdx/blob/main/LICENSE-MIT),
at your option.
