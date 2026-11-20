# Captures

Each capture is a pcapng file with a log of the same name that lists the steps
performed. State lineage names the capture whose server state this one
started from; see [the lab](../lab/README.md) for how they are recorded.

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
| [openkdx-first-connect.md](openkdx-first-connect.md) | Windows client 1.600 logs in as the administrator | Handshake and login baseline from the OpenKDX lab | OpenKDX lab, fresh server |
| [openkdx-guest-login.md](openkdx-guest-login.md) | Windows client logs in with empty login and password | Guest login and its session key | OpenKDX lab, fresh server |
| [openkdx-chat-basic.md](openkdx-chat-basic.md) | Windows client joins Public Chat, sends a line, a `/me` action, sets the topic | The action flag of 207 and 208 | OpenKDX lab, fresh server |
| [openkdx-pm-basic.md](openkdx-pm-basic.md) | Two clients exchange private messages | 300 and 301 each way | OpenKDX lab, fresh server |
| [openkdx-news-basic.md](openkdx-news-basic.md) | Windows client lists, reads, and posts news | 605 to 611 | OpenKDX lab, fresh server |
| [openkdx-files-basic.md](openkdx-files-basic.md) | Windows client lists the base folder and downloads a 3-byte file | File list, a transfer connection, two 414 headers | OpenKDX lab, fresh server |
| [openkdx-tracker-basic.md](openkdx-tracker-basic.md) | Server registers with Tracker 1.110; Windows client browses | UDP registration and 700 to 703 with the original tracker | OpenKDX lab, fresh server |
| [openkdx-upload-resume.md](openkdx-upload-resume.md) | OpenKDX client cuts a 256 KiB upload at 150000 bytes and resumes it | The `.prt` partial and the 416 resume offset | OpenKDX lab, fresh server |
| [openkdx-accounts-classes.md](openkdx-accounts-classes.md) | OpenKDX client lists classes and creates, shows, sets, deletes an account | 502 to 515 | OpenKDX lab, fresh server |
| [openkdx-folder-access.md](openkdx-folder-access.md) | OpenKDX client lists, creates, binds and deletes a folder access item | 1100 to 1107 and 402 with `access` | OpenKDX lab, fresh server |
| [openkdx-server-settings.md](openkdx-server-settings.md) | OpenKDX client reads settings and renames the server | 170 to 173, the name length byte, a fresh 105 | OpenKDX lab, fresh server |
| [openkdx-broadcast.md](openkdx-broadcast.md) | OpenKDX client broadcasts to a guest, another admin and itself | 154 and 155 at every session | OpenKDX lab, fresh server |
| [openkdx-disconnect-ban.md](openkdx-disconnect-ban.md) | OpenKDX client fails a login, then disconnects a user with a 10 s ban | Proof status 1, 151 with flag 2, loopback not banned | OpenKDX lab, fresh server |
| [openkdx-server-history.md](openkdx-server-history.md) | OpenKDX client joins the server history while a guest logs in and fails to post | 180 to 183 | OpenKDX lab, fresh server |
| [openkdx-connection-monitor.md](openkdx-connection-monitor.md) | OpenKDX client polls the connection monitor during a 4 MiB download | 143 with a transfer record | OpenKDX lab, fresh server |

The `openkdx-*` captures come from the OpenKDX lab
(https://github.com/spitfire915/OpenKDX, its lab scenario scripts), where the
administrator password is the published `openkdx-lab`.
