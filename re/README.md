# Static analysis

Ghidra 12.1.4 runs headless in Docker, with no network, as your UID. Build the
image with the script in the Ghidra 12.1.4 release from GitHub:

```sh
ghidra_12.1.4_PUBLIC/docker/build-docker-image.sh   # -> ghidra/ghidra:12.1.4_PUBLIC
```

`re/ghidra` wraps `docker run` and takes the usual `analyzeHeadless` arguments.
Paths inside the container are:
`/work/bin` (vendor/haxial, read-only), `/work/scripts` (re/scripts, read-only),
`/work/projects` and `/work/out` (both git-ignored).

```sh
# import + auto-analyze (done once per binary; ~1 min)
re/ghidra /work/projects kdx -import /work/bin/KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe
# run scripts against the analyzed program without re-analysis
re/ghidra /work/projects kdx -process KDXServer.lexe -noanalysis -readOnly -scriptPath /work/scripts \
  -postScript decompile.py 0x08085a04 -postScript callers.py 0x08085328
```

Scripts (PyGhidra / Python 3):
- `overview.py` exports functions, network imports, constant sites, and
  strings (including Haxial's length-prefixed strings) with the functions that
  reference them. It then decompiles every "anchor" function to
  `re/out/<binary>/`.
- `decompile.py <addr|name>...` decompiles functions to
  `re/out/<binary>/decomp/`.
- `find_scalars.py <hex>...` lists instructions that use the given constants.
- `callers.py <addr>...` lists call sites.
- `fnptr_stores.py <offset>` finds code that stores a function pointer at
  `[reg + offset]`, which is how callbacks and hooks get installed.
- `disasm.py <addr> [count]` prints instructions with their references, for
  code the decompiler mangles (regparm calls, tail jumps).
- `xrefs.py <text>...` finds a byte string and what references it, two levels
  deep.

`re/out/` holds decompiled Haxial code, so it stays local and is never
committed. Write findings into `docs/protocol/` instead, citing function
addresses.

## Server 1.620 landmarks

| Address | What |
|---|---|
| `0x08085328` | create connection object (magic `0x2aa24aec`, service, version) |
| `0x0808529c` | set challenge table (default "All your base…") |
| `0x08085500` | set login name + 32-byte account key (also runs the Twofish key schedule) |
| `0x08085a04` | handshake state machine |
| `0x080872f4` / `0x0808751c` | send HELLO / send PROOF |
| `0x080876e8` | validate HELLO header and return its length |
| `0x0808584c` | verify the peer's PROOF against a candidate key |
| `0x0805270c` | server login callback (guest, session-attach, account lookup via `0x08054900`) |
| `0x08052ad0` | server accept: creates the connection with `%KDX`, version 0x20, story table |
| `0x080863b4` / `0x080868b4` / `0x08086a9c` | session: send message / send header only / send encrypted chunk |
| `0x08086108` / `0x080879c4` | session: receive and validate header / ECB-decrypt buffered data |
| `0x0804a510` | header scramble hook, installed by `0x08052ad0` |
| `0x08066024` | Twofish decrypt block |
| `0x080669cc` | account_key(password): SHA-1 + FNV-1 |
| `0x0807bbfc` / `0x0807bc64` | SHA-1 (one-shot / init) |
| `0x0806565c` / `0x08062840` | Twofish encrypt block / key schedule |
| `0x0806fcbc` | FNV-1 variant (seeded) |
| `0x0807aac0` | RNG |
| `0x0804a454` / `0x0804a4b4` | LCG XOR (`0xa5a16c4a`/`0x41d28485`/`0x322b`; tracker UDP according to hlwiki, call sites not yet read) |
| `0x08066fd8` / `0x08066c14` | raise error (code, module, line) / format "ID x/y @ TRNnnn" |
| `0x080528c8` | session read loop: drops replies, updates idle time, calls the dispatcher |
| `0x0804fdac` | message dispatcher (type → handler); catch at `0x0805090c` sends Status (1) |
| `0x08086640` / `0x080865c0` | send Status (1): error form / success form |
| `0x08049d14` | message type → name (jump table at `0x080abc6c`) |
| `0x08066d04` | error code → text (codes at `0x080b1dc0`, text at `0x080b1f00`) |
| `0x0804ad64` | broadcast a message to all users |
| `0x0804af04` | encode a user record |
| `0x0804b114` | post-login: sends 105, 107, 106, 213 |
| `0x08061d80` | write a server history record and push 183 |
| `0x080969f8` / `0x08069880` / `0x08096b4c` | file path: decode / validate / join to a host path |
| `0x08052194` / `0x0805234c` | transfer connection: dispatcher / read loop that switches to streaming |
| `0x08050984` / `0x080512b4` | 412 Download File: request / data stream |
| `0x08051750` / `0x08051dec` | 415 Upload File: request / data stream |
