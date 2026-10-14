# kdx-probe, a raw request tool

`kdx-probe` logs in to a server and sends the requests you give it, then dumps
the replies. It is how undocumented messages are tried against a lab server.

```sh
cargo run -p kdx-cli --bin kdx-probe -- --login admin --password secret 142 400:00000000
```

Each request is `TYPE` for an empty payload or `TYPE:HEX` for a payload. After
each one it waits for more messages (`--wait`, 700 ms by default).

| Option | Meaning |
|---|---|
| `--addr` | Server address, 127.0.0.1:10700 by default |
| `--login`, `--password` | The account; guest if the login is empty |
| `--nick` | The nickname sent in the login message |
| `--no-login` | Skip the login message (type 100) |
| `--attach` | Send the requests on a second, attached connection, as file transfers do |
| `--table` | The challenge table, or any KDX program file or archive that holds it; defaults to the cache |
| `--listen ADDR` | Wait for a client and play a GConsole peer, with GConsole's table |

Run it only against servers you control, ideally the lab
([lab/README.md](../../lab/README.md)). Message types are listed in [the
protocol reference](../protocol/KDXProtocol.md).
