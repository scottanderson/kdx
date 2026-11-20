# kdx-legacy

Readers and writers for the files the original KDX programs keep on disk,
byte for byte, so that an existing data directory can be served by a new
server and handed back to `KDXServer.lexe` at any time. No network or UI
dependency: only `kdx-proto` (whose wire records double as the on-disk
records) and `thiserror`. The formats are documented in
[docs/formats](../../docs/formats/README.md).

| Module | File | Status |
|---|---|---|
| `server_stg` | `KDXServer.stg` | verified (lab): Server 1.620 starts on our rewrite |
| `accounts` | `Accounts.dat` | verified (lab): an account we wrote logs in; the server lists a post we appended |
| `news` | `News.dat` | read/write, byte-exact, layout matches the server's |
| `history` | `History/*.txt` | read/write (append-only) |
| `news_archive` | `News Archives/<group>/*.txt` | read/write (append-only) |
| `catalog` | `Catalog.dat` | read/write, byte-exact |
| `tracker_stg` | `KDXTrackerServ.stg` | read/write; `TrackerSettings` types the sections on request |
| `client_stg` | `KDX.stg` | verified (lab) for identity and address book: Client 1.600 starts on our file |
| `hap` | `Appearances/*.hap` | read/write, byte-exact over Haxial's 111-file pack (lab-only) |
| `scramble` | the LCG XOR stream over the three scrambled files | |
| `store` | `LegacyDir`: `.orig` backups once, atomic saves | |
| `bases` | `Bases/` folder-name tags | |
| `wire`, `time`, `civil` | the cursor, `Time1900` and the civil calendar the files use | |

Sound lists are not here yet; the formats README says why.

## The live store

`LegacyDir::open(dir)` copies each known file to `<name>.orig` once (never
again, even if the live file changes) and `save_*` writes `<name>.tmp` and
renames it over the live file with the original's permissions. Readers keep
every byte they do not understand (`ignored_*`, padding, gaps) so that
`encode(decode(file)) == file` for every file in `tests/fixtures/`
(`tests/fixtures.rs` walks them all), and OpenKDX's lab check
(`../OpenKDX/lab/legacy.sh`) feeds what we write back to the originals:
`tests/lab.rs` runs when `KDX_LAB_LEGACY_DIR` names the snapshots,
`tests/hap.rs` when `KDX_LAB_HAP_DIR` names the appearance pack.

## Samples

Everything under `tests/fixtures/` was written by the original programs in
the lab from inputs of ours;
[tests/fixtures/README.md](tests/fixtures/README.md) lists the step behind
every file. Nothing of Haxial's is committed: the server's stock welcome post
is deleted before the first `News.dat` sample, and the appearance pack, when
used, stays on the lab machine.

`examples/legacytool.rs` dumps a file, unscrambles one, adds an account,
renames a server or a client identity, or posts a news message.

Part of [kdx](../../README.md). Licensed under MIT OR Apache-2.0.
