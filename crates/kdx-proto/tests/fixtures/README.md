# Fixtures

`first-connect.c2s.bin` and `first-connect.s2c.bin` are the two TCP payload
streams of `captures/first-connect.pcapng`, reassembled in order: client to
server (824 bytes) and server to client (3120 bytes). Each starts with the hello
and proof, followed by the session messages. The lab account password is
`lab-only-pw!1`.

`kdx-dissect`'s `captures` test checks every fixture here byte for byte
against the capture it names.

Each proof carries 20 bytes of Haxial's challenge table, which the capture
already holds. The full table is not in the repository.

`probe-<port>.c2s.bin` and `probe-<port>.s2c.bin` are the admin connections of
`captures/probe.pcapng`, named by source port, with the same password. They
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

The transfer connections use the session key from their parent's 105
Server Info:

| Parent | Transfer | Messages |
|---|---|---|
| 33068 | 33082 | 400 Get File List, then a whole download of 70000 bytes |
| 48784 | 48796 | a 100-byte upload, then 400 Get File List |
| 53514 | 53518 | a download resumed at 5000 |
| 56346 | 56352 | a download from offset 70000, past the end |

`probe-dl.bin` and `probe-ul.bin` were generated test data; in
`probe-ul.bin`, byte *i* is `i`.

`kdx-news-<port>.c2s.bin` and `kdx-news-<port>.s2c.bin` come from
`captures/kdx-news.pcapng` the same way. Port 40434 is a guest, so its key is
the empty password's.

| Port | Messages |
|---|---|
| 40426 | 609 Get News Message and its 610 reply |
| 40434 | 611 Post News Message refused with 1 Status 0x10014 |
| 53204 | 611 Post News Message, 613 News Message Posted, 115 Get My Address |
| 53224 | 612 Delete News Message, 614 News Message Deleted, 115 Get My Address |

`kdx-accounts-<port>.c2s.bin` and `kdx-accounts-<port>.s2c.bin` come from
`captures/kdx-accounts.pcapng`. Port 58336 logged in as `rs-test` with
password `rs-test-pw`; the rest are admin.

| Port | Messages |
|---|---|
| 33090 | 508 Delete Account refused with 0x1000c |
| 47988 | 509 Get Account, then 511 Set Account renaming `rs-test` to `rs-test2` |
| 50184 | 510 Account Info for `rs-nopw`, which has no password |
| 50188 | 508 Delete Account and 515 Account Deleted |
| 58284 | 507 Create Account for `rs-test` |
| 58290 | 510 Account Info for `rs-test` |
| 58336 | 550 Change Own Password and 551 |

`linux-files-<port>.c2s.bin` and `linux-files-<port>.s2c.bin` come from
`captures/linux-files.pcapng`, where Client 1.620 for Linux logged in as
admin. The transfer connections use the session key from 59246's 105 Server
Info.

| Port | Messages |
|---|---|
| 59246 | the login connection: 400, 402, 403, 405, 407, 409, 411, 452, 1100, 1103, 1106 |
| 50904 | 415 Upload File of the 6-byte `upload.txt`, then its 417 data |
| 51076 | 412 Download File of the 6-byte `upload2.txt` |
| 54196 | 412 Download File of the 6-byte `lab-folder/upload.txt` |

`linux-catalog-volumes-58160.c2s.bin` and `linux-catalog-volumes-58160.s2c.bin`
come from `captures/linux-catalog-volumes.pcapng`, where Client 1.620 for
Linux logged in as admin. The connection holds the file catalog, search,
launch, volume, account class, and Server Settings messages, and 24 replies
of 143 Connection Monitor Info, 20 of them with a transfer record for a `kdx
get` of `speed.bin`.

`linux-chat-rooms-<port>`, `linux-chat-admin-<port>`, and `kdx-cli-<port>`
come from the captures of the same names, all logged in as admin. In the
first two, Client 1.620 for Linux is the GUI client and the other ports are
`kdx` CLI peers.

| Fixture | Messages |
|---|---|
| `linux-chat-rooms-43718` | the GUI: 201, 204, 217, 218, 219, 220, 221, 222, and empty 223s |
| `linux-chat-rooms-55916` | `cli-member` in `lab-chat`: 205 and 221 |
| `linux-chat-admin-37478` | `cli-peer` in Public Chat: 205 and two 212 Chat Invitations |
| `linux-chat-admin-37484` | the GUI: 200/216 Create Private Chat, two 211 Chat Invites, 104 User Departed, 150/151 Disconnect, 152/153 User Info, 154/155 Broadcast, and 181 |
| `kdx-cli-51784` | `listener` in Public Chat: 205 and 206 for user 0x14 |

`linux-accounts-news-51382.c2s.bin` and `linux-accounts-news-51382.s2c.bin`
come from `captures/linux-accounts-news.pcapng`, where Client 1.620 for Linux
logged in as admin. The connection holds 502 and 504 for the Guest class,
the `gui-test` account messages, and 600, 602, 604, 607, and 601 for a new
newsgroup. Port 36078 is the same client logged in as guest, so its key is
the empty password's.

`linux-accounts-folder-access-55746.c2s.bin` and `.s2c.bin` come from
`captures/linux-accounts-folder-access.pcapng`, where Client 1.620 for Linux
logged in as admin. The connection holds 1100, 1103, 1104, 1105, 1106, and
1107 for `lab-item`, then 1101 and 1102.

`linux-server-settings-<port>.c2s.bin` and `.s2c.bin` come from
`captures/linux-server-settings.pcapng`, where Client 1.620 for Linux logged
in as admin.

| Port | Messages |
|---|---|
| 45150 | 171/172 Server Settings, 107 Server Greeting, 181 and 183 Server History |
| 40560 | 106 Server Icon, 107, 110/111 Mini User Icon, 171/172, 181, 183, 419 Notify Speed Limit |

`linux-shutdown-<port>.c2s.bin` and `.s2c.bin` come from
`captures/linux-shutdown.pcapng`, where Client 1.620 for Linux logged in as
admin.

| Port | Messages |
|---|---|
| 49124 | 175 Upgrade Server Software and its 1 Status, 800, 803 Process List, 804 Exit/Terminate Process |
| 38848 | 174 Shutdown Server and 151 Disconnect Message |

`linux-tracker-<port>.c2s.bin` and `.s2c.bin` are tracker connections from
`captures/linux-tracker.pcapng`, exported with `--password 'lab-tracker-pw!1'`.
Port 60698 is a guest, so its key is the empty password's.
`linux-tracker-reg-<frame>.bin` are the server's UDP registrations, as sent; 146
and 239 match 23 byte for byte.

| Fixture | Messages |
|---|---|
| `linux-tracker-60698` | 700/701, 702/703 for General and Chat, 710 refused with an 8-byte 1 Status |
| `linux-tracker-41900` | admin: 700/701, 710/711 of a fresh tracker, 712/713, 720/721 |
| `linux-tracker-34704` | admin: 107, 711, 712 with the new group `Lab`, 701 with nine groups, 703 for `Lab` |
| `linux-tracker-reg-23` | the first registration: users 1, group `Lab, Lab2` |
| `linux-tracker-reg-78` | users 2 |
| `linux-tracker-reg-255` | group `Lab`, with two uncleared padding bytes |

`linux-gconsole-51924.c2s.bin` and `.s2c.bin` come from
`captures/linux-gconsole.pcapng`, where Client 1.620 for Linux logged in as
admin to `kdx-probe --listen` as a GConsole peer. Its headers use GConsole's
scramble.

| Port | Messages |
|---|---|
| 51924 | 4000 Open Window, 4001 Draw with every record kind, 64 4200 Key, 14 4201 Mouse |

`linux-dcc-voice-failed-<port>.c2s.bin` and `.s2c.bin` come from
`captures/linux-dcc-voice-failed.pcapng`, where client B connected to client
A's DCC listener. Both are Client 1.620 for Linux.

| Port | Messages |
|---|---|
| 45702 | the primary connection, with no password: both 117 SKPKs, 100, and 300 |

`kdx-dissect --audit` re-encodes each payload it decodes and flags the
types with no codec (`no codec`) or whose bytes don't come back byte for
byte (`mismatch`, a codec bug). The codecs keep padding and unknown bytes,
uninitialised ones included. The `captures` test runs it over every capture
and checks its findings against `NO_CODEC`, which lists only `no
codec` types. `tests/round_trip.rs` re-encodes the fixtures' payloads of
the types that store padding or unknown bytes.
