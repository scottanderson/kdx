# Fixtures

`first-connect.c2s.bin` and `first-connect.s2c.bin` are the two TCP payload
streams of `captures/first-connect.pcapng`, reassembled in order: client to
server (824 bytes) and server to client (3120 bytes). Each starts with the hello
and proof, followed by the session messages. The lab account password is
`lab-only-pw!1`.

Each proof carries 20 bytes of Haxial's challenge table, which the capture
already holds. The full table is not in the repository.

`probe-<port>.c2s.bin` and `probe-<port>.s2c.bin` are the admin connections of
`captures/probe.pcapng`, named by client port, with the same password. They
were written by `kdx-dissect --export`:

```sh
cargo run -p kdx-dissect -- captures/probe.pcapng --password 'lab-only-pw!1' --export DIR
```

| Port | Replies |
|---|---|
| 32980 | 1, 171, 401, 603, 610 |
| 35844 | 1, 203, 208, 210, 219 |
| 45802 | 107 |
| 50770 | 109, 114, 116, 215, 506, 513 |
| 50984 | 1, 143, 153 |
| 54328 | 401, 410, 606, 608 |
| 59006 | 1, 503, 510, 513, 803, 1107 |
