# Captures

Captures are the ground truth for the
[protocol docs](../docs/protocol/README.md): they show what the Haxial programs
really do. Only Haxial programs and this repository's own tools may be on the
wire. Never learn anything from a third-party client.

What this repository's tools send to a Haxial program is synthetic, so only the
Haxial program's replies are evidence. Build the tool from this repository as it
stands when you record the capture, with only code that is merged here or part
of the same pull request, and give each command in full in the note.

Each capture is a pcapng file with a note of the same name that lists the steps
performed. See [the lab](../lab/README.md) for how they are recorded.

## Notes

Each note starts with a header that `ci/check-docs.py` checks. `lab/kdx-lab up`
writes it with placeholders. Give the fields in this order:

- **Date:** the UTC minutes of the first and last frames, such as
  `2026-10-08T07:17Z to 07:19Z`. Give the date again if the capture runs past
  midnight, and give one time if both frames fall in the same minute.
- **Server**, **Tracker**, and **Client:** each Haxial program on the wire,
  named as in the Lab binaries table of
  [`vendor/MANIFEST.md`](../vendor/MANIFEST.md), with the full sha256 of its
  file. Write `none` if no such program is on the wire, or name this
  repository's tool that plays the part. Leave Tracker out if no tracker is
  on the wire.
- **Tools:** each of this repository's tools on the wire. Leave it out if no
  tool is on the wire. The note shows each command in full, as
  `lab/kdx-lab run target/debug/kdx-probe …`, here or in the steps.
- **Server state:** the state the server started from, with each login and
  password.
- **Decode:** the `kdx-dissect` command that decodes the capture, with each
  password it needs.

For example:

```markdown
# Scenario: linux-tracker

- Date: 2026-10-12T07:14Z to 07:36Z
- Server: Server 1.620 for Linux, sha256
  `ad6fba4bdd01dc237abde2b76054e3bbbb43c6d0c79a28d4e0de3b23e546f943`
- Tracker: Tracker 1.110 for Linux, sha256
  `80dbc5658717a0feb5e765600253240b0fb42eb973e1b847b0dff50a368004fa`, fresh, set
  up at its console with admin / `lab-tracker-pw!1`
- Client: Client 1.620 beta 5 for Linux, sha256
  `e7189c55b67a89f35edfdf09570a1d55a096b198579b8633f3adfa50f9d13961`, with
  `LANG=C.UTF-8`
- Server state: the server from linux-text-encoding, with admin /
  `lab-only-pw!1`, name `Lab Server`, and greeting `Welcome to the lab.`
- Decode: `kdx-dissect captures/linux-tracker.pcapng --password
  'lab-only-pw!1' --password 'lab-tracker-pw!1'`
```

Then give the steps, with the frames, ports, and user IDs each one shows,
and each command of this repository's tools in full. Write the observations
last.

## Passwords

Use the passwords that the other captures share:

- (empty)
- `lab-only-pw!1`
- `lab-peer-pw!1`
- `lab-tracker-pw!1`
- `lab-tracker-pw!2`

If a capture needs another password, give it the next number, such as
`lab-peer-pw!2` or `lab-tracker-pw!3`. A login that should be refused can use
any wrong password.

Every password a capture uses goes in the list that
`crates/kdx-dissect/tests/captures.rs` tries on every capture, and in each note
and test that decodes it. To keep that list short, use only passwords from this
scheme when possible. The other passwords in the list come from older captures.

## Index

State lineage names the capture whose server state this one started from.

| File | Scenario | Purpose | State lineage |
|---|---|---|---|
| [first-connect.md](first-connect.md) | Windows client 1.600 logs in, sends private messages, reads news | First baseline of handshake, login, and private message and news traffic | fresh first-run setup |
| [kdx-accounts.md](kdx-accounts.md) | `kdx` CLI account management as admin | Account and class messages sent by this repository's client | [first-connect](first-connect.md) |
| [kdx-cli.md](kdx-cli.md) | `kdx` CLI browse, download, and upload runs | Check the client against a real server, including a 600000-byte transfer | [first-connect](first-connect.md) |
| [kdx-cli-tracker.md](kdx-cli-tracker.md) | `kdx` CLI and `kdx-probe` against a fresh tracker | Tracker registration and browsing from the CLI | fresh server and tracker |
| [kdx-news.md](kdx-news.md) | `kdx` CLI news and chat runs as admin and guest | News messages sent by this repository's client | [first-connect](first-connect.md) |
| [kdx-tracker.md](kdx-tracker.md) | `kdx-probe` against `kdx-tracker` from this repository | Check our tracker against a real server | fresh server and tracker |
| [kdx-tracker-gui.md](kdx-tracker-gui.md) | Linux client 1.620 beta 5 browsing `kdx-tracker` | Check our tracker against the original GUI client | [kdx-cli-tracker](kdx-cli-tracker.md) |
| [linux-accounts-folder-access.md](linux-accounts-folder-access.md) | Linux client, `kdx`, and `kdx-probe` edit accounts and folder access | Account classes, flood protection, and folder access items | [linux-catalog-volumes](linux-catalog-volumes.md) |
| [linux-accounts-news.md](linux-accounts-news.md) | Linux client edits account classes and accounts, and posts news | Account class and news messages from the GUI | [linux-first-connect](linux-first-connect.md) |
| [linux-catalog-volumes.md](linux-catalog-volumes.md) | Linux client lists the catalog and volumes, runs a file, downloads | File catalog, volume, and execute messages | [linux-server-settings](linux-server-settings.md) |
| [linux-chat-admin.md](linux-chat-admin.md) | Linux client chats and sends private messages with CLI peers | Chat administration and private messages | [linux-first-connect](linux-first-connect.md) |
| [linux-chat-rooms.md](linux-chat-rooms.md) | Linux client creates and joins chat rooms with CLI peers | Chat room messages | [linux-server-settings](linux-server-settings.md) |
| [linux-dcc-voice-failed.md](linux-dcc-voice-failed.md) | Two Linux clients try a DCC connection and Voice Chat | The client to client protocol and a failed voice attempt | existing lab server |
| [linux-files.md](linux-files.md) | Linux client browses, uploads, renames, aliases, and moves files | File management messages and transfer connections | [linux-first-connect](linux-first-connect.md) |
| [linux-first-connect.md](linux-first-connect.md) | Linux client 1.620 beta 5 first run | Baseline for the Linux client, including its failed tracker lookup | earlier captures |
| [linux-folder-access-guest.md](linux-folder-access-guest.md) | Linux client edits a Folder Access column | Which class rule each Folder Access column sets | [linux-accounts-folder-access](linux-accounts-folder-access.md) |
| [linux-gconsole.md](linux-gconsole.md) | Linux client connects to `kdx-probe --listen` playing GConsole | The GConsole protocol | - |
| [linux-server-settings.md](linux-server-settings.md) | Linux client reads and changes Server Settings | Server settings messages and file transfer speed | [linux-first-connect](linux-first-connect.md) |
| [linux-shutdown.md](linux-shutdown.md) | Linux client upgrades, then exits the server process | Upgrade, shutdown, process monitor, and exit process messages | [linux-text-encoding](linux-text-encoding.md) |
| [linux-text-encoding.md](linux-text-encoding.md) | Linux client moves non-ASCII names and text | Text encodings on the wire | [linux-accounts-folder-access](linux-accounts-folder-access.md) |
| [linux-tracker.md](linux-tracker.md) | Linux client with a server and the original tracker | Tracker registration and browsing with the original tracker | [linux-text-encoding](linux-text-encoding.md) |
| [linux-upgrade.md](linux-upgrade.md) | Linux client replaces Server 1.620 with the staged 1.600 through Upgrade | Upgrade messages and the server restart | [linux-shutdown](linux-shutdown.md) |
| [probe.md](probe.md) | `kdx-probe` sends raw requests, 18 connections | Which requests a server answers and how | [first-connect](first-connect.md) |
| [win-view-display.md](win-view-display.md) | Linux client and `kdx` against Server 1.600 for Windows under Wine | View display messages | fresh first-run setup |
| [win-first-connect.md](win-first-connect.md) | Windows client 1.600 logs in as the administrator | The handshake and login baseline for the lab server | fresh first-run setup |
| [win-guest-login.md](win-guest-login.md) | Windows client logs in with empty login and password | Guest login and its session key | fresh first-run setup |
| [win-chat-basic.md](win-chat-basic.md) | Windows client joins Public Chat, sends a line, a `/me` action, sets the topic | The action flag of [207 Send Chat Text](../docs/protocol/messages/chat.md#m207) and [208 Receive Chat Text](../docs/protocol/messages/chat.md#m208) | fresh first-run setup |
| [win-pm-basic.md](win-pm-basic.md) | Two clients exchange private messages | [300 Send Private Message](../docs/protocol/messages/private-messages.md#m300) and [301 Incoming Private Message](../docs/protocol/messages/private-messages.md#m301) each way | fresh first-run setup |
| [win-news-basic.md](win-news-basic.md) | Windows client lists, reads, and posts news | The news messages from [605 Get Newsgroup List](../docs/protocol/messages/news.md#m605) to [611 Post News Message](../docs/protocol/messages/news.md#m611) | fresh first-run setup |
| [win-files-basic.md](win-files-basic.md) | Windows client lists the base folder and downloads a 3-byte file | [401 File List](../docs/protocol/messages/files.md#m401), a transfer connection, two [414 Download File (D)](../docs/protocol/messages/files.md#m414) headers | fresh first-run setup |
| [win-tracker-basic.md](win-tracker-basic.md) | Server registers with Tracker 1.110; Windows client browses | UDP registration and the messages from [700 Get Tracker Group List](../docs/protocol/messages/tracker.md#m700) to [703 Tracker Server List](../docs/protocol/messages/tracker.md#m703) with the original tracker | fresh first-run setup |
| [cli-upload-resume.md](cli-upload-resume.md) | The command-line client of the lab cuts a 256 KiB upload at 150000 bytes and resumes it | The `.prt` partial in [401 File List](../docs/protocol/messages/files.md#m401) and the resume offset in [416 Upload File (R)](../docs/protocol/messages/files.md#m416) | fresh first-run setup |
| [cli-accounts-classes.md](cli-accounts-classes.md) | The command-line client lists classes and creates, shows, sets, deletes an account | The account class and account messages from [502 Get Account Class](../docs/protocol/messages/accounts.md#m502) to [515 Account Deleted](../docs/protocol/messages/accounts.md#m515) | fresh first-run setup |
| [cli-folder-access.md](cli-folder-access.md) | The command-line client lists, creates, binds and deletes a folder access item | The folder access messages from [1100 Create Folder Access](../docs/protocol/messages/folder-access.md#m1100) to [1107 Folder Access List](../docs/protocol/messages/folder-access.md#m1107), and [402 Create Folder](../docs/protocol/messages/files.md#m402) with its access name | fresh first-run setup |
| [cli-server-settings.md](cli-server-settings.md) | The command-line client reads the settings and renames the server | [171 Server Settings Info](../docs/protocol/messages/administration.md#m171) and [172 Set Server Settings](../docs/protocol/messages/administration.md#m172), the name length byte, and a fresh [105 Server Info](../docs/protocol/messages/session.md#m105) | fresh first-run setup |
| [cli-broadcast.md](cli-broadcast.md) | The command-line client broadcasts to a guest, another admin and itself | [154 Send Broadcast Message](../docs/protocol/messages/administration.md#m154) and [155 Rcv Broadcast Message](../docs/protocol/messages/administration.md#m155) at every session | fresh first-run setup |
| [cli-disconnect-ban.md](cli-disconnect-ban.md) | The command-line client fails a login, then disconnects a user with a 10 s ban | Proof status 1, [151 Disconnect Message](../docs/protocol/messages/administration.md#m151) with flag 2, loopback not banned | fresh first-run setup |
| [cli-server-history.md](cli-server-history.md) | The command-line client joins the server history while a guest logs in and fails to post | The history messages from [180 Server History Join](../docs/protocol/messages/administration.md#m180) to [183 Server History Added](../docs/protocol/messages/administration.md#m183) | fresh first-run setup |
| [cli-connection-monitor.md](cli-connection-monitor.md) | The command-line client polls the connection monitor during a 4 MiB download | [143 Connection Monitor Info](../docs/protocol/messages/monitoring.md#m143) with a transfer record | fresh first-run setup |
