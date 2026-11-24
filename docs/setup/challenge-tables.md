# Challenge tables

The handshake proves knowledge of a challenge table built into the Haxial
programs: a 1739-byte table for KDX, and a 3193-byte one for GConsole, which
only the clients hold. The tables are Haxial's copyrighted material, so they are
not in the repository.

## For the tools

`kdx` and the other programs keep a copy of each table in `~/.cache/kdx/table/`,
as `kdx.bin` and `hgc.bin`.

```sh
kdx table fetch              # download the smallest Haxial archives that hold each table
kdx table import FILE        # take every table found in a KDX program, zip archive, or saved table
kdx table path               # show where the cached tables are
```

`fetch` downloads archives of 248 KB and 793 KB from the Internet Archive,
checks their SHA-256, and extracts the table. It skips a table the cache already
holds, unless given `--force`.

## For the tests

The tests that need a table read it through `kdx-test-tables`, from
`KDX_CHALLENGE_TABLE` or `HGC_CHALLENGE_TABLE` (base64), from `vendor/haxial/`,
or from the cache above, and fail without all three. CI sets both variables from
repository secrets of the same names. Pull requests from forks get no secrets,
so runs on main also save the tables in the Actions cache, and a fork's run
restores them from there.

Where the Haxial archives come from is in
[vendor/MANIFEST.md](../../vendor/MANIFEST.md). The legal rules for the tables
are in [CONTRIBUTING.md](../../CONTRIBUTING.md#legal-rules).

### CI in a fork

A fork's own runs, such as a push to its main, need the same two secrets.
Fetch the tables, then give each one to `gh` as base64, with your fork's name
in place of `OWNER/kdx`:

```sh
kdx() { cargo run -q --bin kdx -- "$@"; }
kdx table fetch
KDX_TABLE_DIR=$(dirname "$(kdx table path | sed -n 1p)")
base64 -w0 "$KDX_TABLE_DIR/kdx.bin" | gh secret set KDX_CHALLENGE_TABLE --repo OWNER/kdx
base64 -w0 "$KDX_TABLE_DIR/hgc.bin" | gh secret set HGC_CHALLENGE_TABLE --repo OWNER/kdx
```
