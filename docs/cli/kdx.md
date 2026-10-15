# kdx, the command-line client

`kdx` is a KDX client for the terminal, built on
[kdx-client](../libraries/kdx-client.md). It works against Server 1.620 and
Tracker 1.110.

```sh
cargo run -p kdx-cli --bin kdx -- table fetch    # once, see below
cargo run -p kdx-cli --bin kdx -- ls -s host:10700 -l user 'Uploads [UL]'
```

## Before the first connection

The handshake needs the KDX challenge table, which is not in the repository. Run
`kdx table fetch` once, or `kdx table import FILE` with a KDX program or archive
you already have. See [challenge tables](../setup/challenge-tables.md).

## Commands

| Command | Does |
|---|---|
| `who` | List the users online |
| `chats` | List the chats |
| `ls` | List a folder; `/` separates names and `\xNN` gives a raw byte |
| `get` | Download a file, resuming a partial one |
| `put` | Upload a file into a folder |
| `news` | Read news |
| `post` | Post a news message; the body is read from standard input if omitted |
| `delete-news` | Delete a news message |
| `account` | List, show, create, change, or delete accounts |
| `passwd` | Change your own password |
| `say` | Say something in a chat |
| `msg` | Send a private message |
| `listen` | Join a chat and print what happens until interrupted |
| `tracker` | Browse a tracker, see below |
| `table` | Manage the challenge tables |

Options such as `-s` (server) and `-l` (login) follow the subcommand. The
password comes from `--password` or `KDX_PASSWORD`. `kdx help <command>` lists a
command's options.

## Trackers

```sh
kdx tracker groups HOST
kdx tracker servers HOST GROUP
kdx tracker stats -l admin HOST
```

The port is 10800 unless HOST gives one. `stats` needs the tracker's admin
login. To run a tracker, see [kdx-tracker](../tracker/kdx-tracker.md).

## Related

- [kdx-probe](kdx-probe.md) sends raw requests when a command for them does not
  exist.
- The messages the commands send are in [the protocol
  reference](../protocol/README.md).
