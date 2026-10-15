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
| 32980 | [1 Status](../../../../docs/protocol/messages/status.md#m1), [171 Server Settings Info](../../../../docs/protocol/messages/administration.md#m171), [401 File List](../../../../docs/protocol/messages/files.md#m401), [603 Newsgroup Info](../../../../docs/protocol/messages/news.md#m603), [610 News Message](../../../../docs/protocol/messages/news.md#m610) |
| 35844 | 1, [203 Chat Window Info](../../../../docs/protocol/messages/chat.md#m203), [208 Receive Chat Text](../../../../docs/protocol/messages/chat.md#m208), [210 Chat Topic Changed](../../../../docs/protocol/messages/chat.md#m210), [219 Chat Info](../../../../docs/protocol/messages/chat.md#m219) |
| 45802 | [107 Server Greeting](../../../../docs/protocol/messages/session.md#m107) |
| 50770 | [109 Pong](../../../../docs/protocol/messages/session.md#m109), [114 Extra Server Info](../../../../docs/protocol/messages/session.md#m114), [116 Your Address](../../../../docs/protocol/messages/session.md#m116), [215 Chat List](../../../../docs/protocol/messages/chat.md#m215), [506 Account Class List](../../../../docs/protocol/messages/accounts.md#m506), [513 Account List](../../../../docs/protocol/messages/accounts.md#m513) |
| 50984 | 1, [143 Connection Monitor Info](../../../../docs/protocol/messages/monitoring.md#m143), [153 User Info](../../../../docs/protocol/messages/administration.md#m153) |
| 54328 | 401, [410 File/Folder Info](../../../../docs/protocol/messages/files.md#m410), [606 Newsgroup List](../../../../docs/protocol/messages/news.md#m606), [608 News Messages](../../../../docs/protocol/messages/news.md#m608) |
| 59006 | 1, [503 Account Class Info](../../../../docs/protocol/messages/accounts.md#m503), [510 Account Info](../../../../docs/protocol/messages/accounts.md#m510), 513, [803 Process List](../../../../docs/protocol/messages/processes.md#m803), [1107 Folder Access List](../../../../docs/protocol/messages/folder-access.md#m1107) |

The transfer connections use the session key from their parent's
[105 Server Info](../../../../docs/protocol/messages/session.md#m105):

| Parent | Transfer | Messages |
|---|---|---|
| 33068 | 33082 | [400 Get File List](../../../../docs/protocol/messages/files.md#m400), then a whole download of 70000 bytes |
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
| 40426 | [609 Get News Message](../../../../docs/protocol/messages/news.md#m609) and its 610 reply |
| 40434 | [611 Post News Message](../../../../docs/protocol/messages/news.md#m611) refused with 1 Status 0x10014 |
| 53204 | 611 Post News Message, [613 News Message Posted](../../../../docs/protocol/messages/news.md#m613), [115 Get My Address](../../../../docs/protocol/messages/session.md#m115) |
| 53224 | [612 Delete News Message](../../../../docs/protocol/messages/news.md#m612), [614 News Message Deleted](../../../../docs/protocol/messages/news.md#m614), 115 Get My Address |

`kdx-accounts-<port>.c2s.bin` and `kdx-accounts-<port>.s2c.bin` come from
`captures/kdx-accounts.pcapng`. Port 58336 logged in as `rs-test` with
password `rs-test-pw`; the rest are admin.

| Port | Messages |
|---|---|
| 33090 | [508 Delete Account](../../../../docs/protocol/messages/accounts.md#m508) refused with 0x1000c |
| 47988 | [509 Get Account](../../../../docs/protocol/messages/accounts.md#m509), then [511 Set Account](../../../../docs/protocol/messages/accounts.md#m511) renaming `rs-test` to `rs-test2` |
| 50184 | 510 Account Info for `rs-nopw`, which has no password |
| 50188 | 508 Delete Account and [515 Account Deleted](../../../../docs/protocol/messages/accounts.md#m515) |
| 58284 | [507 Create Account](../../../../docs/protocol/messages/accounts.md#m507) for `rs-test` |
| 58290 | 510 Account Info for `rs-test` |
| 58336 | [550 Change Own Password](../../../../docs/protocol/messages/accounts.md#m550) and [551 Change Own Password Done](../../../../docs/protocol/messages/accounts.md#m551) |

`linux-files-<port>.c2s.bin` and `linux-files-<port>.s2c.bin` come from
`captures/linux-files.pcapng`, where Client 1.620 for Linux logged in as
admin. The transfer connections use the session key from 59246's 105 Server
Info.

| Port | Messages |
|---|---|
| 59246 | the login connection: 400, [402 Create Folder](../../../../docs/protocol/messages/files.md#m402), [403 Create Alias](../../../../docs/protocol/messages/files.md#m403), [405 Move To Trash](../../../../docs/protocol/messages/files.md#m405), [407 Move File/Folder](../../../../docs/protocol/messages/files.md#m407), [409 Get File/Folder Info](../../../../docs/protocol/messages/files.md#m409), [411 Set File/Folder Info](../../../../docs/protocol/messages/files.md#m411), [452 File Search](../../../../docs/protocol/messages/files.md#m452), [1100 Create Folder Access](../../../../docs/protocol/messages/folder-access.md#m1100), [1103 Get Folder Access](../../../../docs/protocol/messages/folder-access.md#m1103), [1106 Get Folder Access List](../../../../docs/protocol/messages/folder-access.md#m1106) |
| 50904 | [415 Upload File](../../../../docs/protocol/messages/files.md#m415) of the 6-byte `upload.txt`, then its [417 Upload File (D)](../../../../docs/protocol/messages/files.md#m417) data |
| 51076 | [412 Download File](../../../../docs/protocol/messages/files.md#m412) of the 6-byte `upload2.txt` |
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
| `linux-chat-rooms-43718` | the GUI: [201 Delete Chat](../../../../docs/protocol/messages/chat.md#m201), [204 Leave Chat](../../../../docs/protocol/messages/chat.md#m204), [217 Create Chat](../../../../docs/protocol/messages/chat.md#m217), [218 Get Chat](../../../../docs/protocol/messages/chat.md#m218), 219, [220 Set Chat](../../../../docs/protocol/messages/chat.md#m220), [221 Chat Deleted](../../../../docs/protocol/messages/chat.md#m221), [222 Get Chat DCC Addresses](../../../../docs/protocol/messages/chat.md#m222), and empty [223 Chat DCC Addresses](../../../../docs/protocol/messages/chat.md#m223) |
| `linux-chat-rooms-55916` | `cli-member` in `lab-chat`: [205 Chat User Joined](../../../../docs/protocol/messages/chat.md#m205) and 221 |
| `linux-chat-admin-37478` | `cli-peer` in Public Chat: 205 and two [212 Chat Invitation](../../../../docs/protocol/messages/chat.md#m212)s |
| `linux-chat-admin-37484` | the GUI: [200 Create Private Chat](../../../../docs/protocol/messages/chat.md#m200)/[216 Create Private Chat (R)](../../../../docs/protocol/messages/chat.md#m216), two [211 Chat Invite](../../../../docs/protocol/messages/chat.md#m211)s, [104 User Departed](../../../../docs/protocol/messages/session.md#m104), [150 Disconnect User](../../../../docs/protocol/messages/administration.md#m150)/[151 Disconnect Message](../../../../docs/protocol/messages/administration.md#m151), [152 Get User Info](../../../../docs/protocol/messages/administration.md#m152)/153 User Info, [154 Send Broadcast Message](../../../../docs/protocol/messages/administration.md#m154)/[155 Rcv Broadcast Message](../../../../docs/protocol/messages/administration.md#m155), and [181 Server History Join (R)](../../../../docs/protocol/messages/administration.md#m181) |
| `kdx-cli-51784` | `listener` in Public Chat: 205 and [206 Chat User Departed](../../../../docs/protocol/messages/chat.md#m206) for user 0x14 |

`linux-accounts-news-51382.c2s.bin` and `linux-accounts-news-51382.s2c.bin`
come from `captures/linux-accounts-news.pcapng`, where Client 1.620 for Linux
logged in as admin. The connection holds
[502 Get Account Class](../../../../docs/protocol/messages/accounts.md#m502) and
[504 Set Account Class](../../../../docs/protocol/messages/accounts.md#m504) for
the Guest class, the `gui-test` account messages, and
[600 Create Newsgroup](../../../../docs/protocol/messages/news.md#m600),
[602 Get Newsgroup](../../../../docs/protocol/messages/news.md#m602),
[604 Set Newsgroup](../../../../docs/protocol/messages/news.md#m604),
[607 Get News Messages](../../../../docs/protocol/messages/news.md#m607), and
[601 Delete Newsgroup](../../../../docs/protocol/messages/news.md#m601) for a
new newsgroup. Port 36078 is the same client logged in as guest, so its key is
the empty password's.

`linux-accounts-folder-access-55746.c2s.bin` and `.s2c.bin` come from
`captures/linux-accounts-folder-access.pcapng`, where Client 1.620 for Linux
logged in as admin. The connection holds 1100, 1103,
[1104 Folder Access Info](../../../../docs/protocol/messages/folder-access.md#m1104),
[1105 Set Folder Access](../../../../docs/protocol/messages/folder-access.md#m1105),
1106, and 1107 for `lab-item`, then
[1101 Delete Folder Access](../../../../docs/protocol/messages/folder-access.md#m1101)
and
[1102 Folder Access Deleted](../../../../docs/protocol/messages/folder-access.md#m1102).

`linux-server-settings-<port>.c2s.bin` and `.s2c.bin` come from
`captures/linux-server-settings.pcapng`, where Client 1.620 for Linux logged
in as admin.

| Port | Messages |
|---|---|
| 45150 | 171/[172 Set Server Settings](../../../../docs/protocol/messages/administration.md#m172), 107 Server Greeting, 181 and [183 Server History Added](../../../../docs/protocol/messages/administration.md#m183) |
| 40560 | [106 Server Icon](../../../../docs/protocol/messages/session.md#m106), 107, [110 Set Mini User Icon](../../../../docs/protocol/messages/session.md#m110)/[111 Mini User Icon Changed](../../../../docs/protocol/messages/session.md#m111), 171/172, 181, 183, [419 Notify Speed Limit](../../../../docs/protocol/messages/files.md#m419) |

`linux-shutdown-<port>.c2s.bin` and `.s2c.bin` come from
`captures/linux-shutdown.pcapng`, where Client 1.620 for Linux logged in as
admin.

| Port | Messages |
|---|---|
| 49124 | [175 Upgrade Server Software](../../../../docs/protocol/messages/administration.md#m175) and its 1 Status, [800 Open Process Monitor](../../../../docs/protocol/messages/processes.md#m800), 803 Process List, [804 Exit/Terminate Process](../../../../docs/protocol/messages/processes.md#m804) |
| 38848 | [174 Shutdown Server](../../../../docs/protocol/messages/administration.md#m174) and 151 Disconnect Message |

`linux-tracker-<port>.c2s.bin` and `.s2c.bin` are tracker connections from
`captures/linux-tracker.pcapng`, exported with `--password 'lab-tracker-pw!1'`.
Port 60698 is a guest, so its key is the empty password's.
`linux-tracker-reg-<frame>.bin` are the server's UDP registrations, as sent; 146
and 239 match 23 byte for byte.

| Fixture | Messages |
|---|---|
| `linux-tracker-60698` | [700 Get Tracker Group List](../../../../docs/protocol/messages/tracker.md#m700)/[701 Tracker Group List](../../../../docs/protocol/messages/tracker.md#m701), [702 Get Tracker Server List](../../../../docs/protocol/messages/tracker.md#m702)/[703 Tracker Server List](../../../../docs/protocol/messages/tracker.md#m703) for General and Chat, [710 Get TrackerServ Settings](../../../../docs/protocol/messages/tracker.md#m710) refused with an 8-byte 1 Status |
| `linux-tracker-41900` | admin: 700/701, 710/[711 TrackerServ Settings Info](../../../../docs/protocol/messages/tracker.md#m711) of a fresh tracker, [712 Set TrackerServ Settings](../../../../docs/protocol/messages/tracker.md#m712)/[713 Set TrackerServ Settings Done](../../../../docs/protocol/messages/tracker.md#m713), [720 Get TrackerServ Statistics](../../../../docs/protocol/messages/tracker.md#m720)/[721 TrackerServ Statistics](../../../../docs/protocol/messages/tracker.md#m721) |
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
| 51924 | [4000 Open Window](../../../../docs/protocol/messages/gconsole.md#m4000), [4001 Draw](../../../../docs/protocol/messages/gconsole.md#m4001) with every record kind, 64 [4200 Key](../../../../docs/protocol/messages/gconsole.md#m4200), 14 [4201 Mouse](../../../../docs/protocol/messages/gconsole.md#m4201) |

`linux-dcc-voice-failed-<port>.c2s.bin` and `.s2c.bin` come from
`captures/linux-dcc-voice-failed.pcapng`, where client B connected to client
A's DCC listener. Both are Client 1.620 for Linux.

| Port | Messages |
|---|---|
| 45702 | the primary connection, with no password: both [117 SKPK](../../../../docs/protocol/messages/session.md#m117)s, [100 Set Identity](../../../../docs/protocol/messages/session.md#m100), and [300 Send Private Message](../../../../docs/protocol/messages/private-messages.md#m300) |
| 43114 | B's server connection, logged in as guest with no password: both [223 Chat DCC Addresses](../../../../docs/protocol/messages/chat.md#m223) |

`kdx-dissect --audit` re-encodes each payload it decodes and flags the
types with no codec (`no codec`) or whose bytes don't come back byte for
byte (`mismatch`, a codec bug). The codecs keep padding and unknown bytes,
uninitialised ones included. The `captures` test runs it over every capture
and checks its findings against `NO_CODEC`, which lists only `no
codec` types. `tests/round_trip.rs` re-encodes the fixtures' payloads of
the types that store padding or unknown bytes.
