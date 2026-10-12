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
| 59246 | the login connection: 400, 405, 407, 409, 452, 1100, 1103, 1106 |
| 50904 | 415 Upload File of the 6-byte `upload.txt`, then its 417 data |
| 51076 | 412 Download File of the 6-byte `upload2.txt` |
| 54196 | 412 Download File of the 6-byte `lab-folder/upload.txt` |

`linux-catalog-volumes-58160.c2s.bin` and `linux-catalog-volumes-58160.s2c.bin`
come from `captures/linux-catalog-volumes.pcapng`, where Client 1.620 for
Linux logged in as admin. The connection holds the file catalog, search,
launch, volume, account class, and Server Settings messages, and 24 replies
of 143 Connection Monitor Info, 20 of them with a transfer record for a `kdx
get` of `speed.bin`.
