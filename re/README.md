# Static analysis

Ghidra 12.1.4 runs headless in Docker, with no network, as your UID. It needs
Docker, the binaries from `vendor/fetch`, and an image built with the script
in the Ghidra 12.1.4 release from GitHub:

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
re/ghidra /work/projects kdxclient -import /work/bin/KDXClient1600-Win/extracted/KDXClient1600-Win/KDXClient.exe
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
- `jumptable.py <table> <count>` prints each jump table entry with the string
  its target loads, such as the message name tables.
- `import_callers.py <name>...` lists the call sites of imported functions,
  such as `send` or `connect`.
- `decompile_all.py` decompiles every function into one file,
  `re/out/<binary>/all.c`, with a `//// <address> <name>` line before each,
  for grepping.

`re/out/` holds decompiled Haxial code, so it stays local and is never
committed. Write findings into `docs/protocol/` instead, citing function
addresses.

## Without Docker

Cloud sessions may have no Docker daemon. If GitHub release downloads work,
the official release is simplest: it ships a glibc decompiler, so only
PyGhidra needs installing.

```sh
curl -fsSLO https://github.com/NationalSecurityAgency/ghidra/releases/download/Ghidra_12.1.4_build/ghidra_12.1.4_PUBLIC_20260921.zip
unzip -q ghidra_12.1.4_PUBLIC_20260921.zip && mv ghidra_12.1.4_PUBLIC /opt/ghidra
python3 -m venv /opt/pyghidra-venv
/opt/pyghidra-venv/bin/pip install --no-index -f /opt/ghidra/Ghidra/Features/PyGhidra/pypkg/dist pyghidra
```

If GitHub downloads are blocked, Alpine's package works instead: Ghidra 12.1.2
in edge/testing, a near match for 12.1.4. Its native decompiler links against
musl, so rebuild it from the source the package ships. This needs a JDK 21,
Python 3, `make`, and `g++`.

```sh
# unpack the package (an .apk is a gzipped tar)
curl -fsSO https://dl-cdn.alpinelinux.org/alpine/edge/testing/x86_64/ghidra-12.1.2-r1.apk
mkdir pkg && tar -xzf ghidra-12.1.2-r1.apk -C pkg && mv pkg/usr/share/ghidra /opt/ghidra
# rebuild the decompiler for glibc; the x86 .sla files ship compiled, so skip sleigh
cp -r /opt/ghidra/Ghidra/Features/Decompiler/src/decompile/cpp /tmp/decomp
make -C /tmp/decomp -j4 ghidra_opt   # one goal per make run, or it builds every tool
cp /tmp/decomp/ghidra_opt /opt/ghidra/Ghidra/Features/Decompiler/os/linux_x86_64/decompile
# PyGhidra from the bundled wheels
python3 -m venv /opt/pyghidra-venv
/opt/pyghidra-venv/bin/pip install --no-index -f /opt/ghidra/Ghidra/Features/PyGhidra/pypkg/dist pyghidra
```

Then use `re/ghidra-native` wherever the examples above say `re/ghidra`. It
takes the same arguments, links the same `/work` paths, and runs Ghidra
under `unshare -n` so it has no network. `GHIDRA_HOME` and
`PYGHIDRA_PYTHON` override the `/opt` paths above.

```sh
re/ghidra-native /work/projects kdx -import /work/bin/KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe
```

Projects made by 12.1.4 may not open in 12.1.2, so import the binaries again.
The addresses are the same.

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
| `0x0804a454` / `0x0804a4b4` | LCG XOR (`0xa5a16c4a`/`0x41d28485`/`0x322b`) for tracker registration datagrams |
| `0x08060e4c` | send a registration datagram to each tracker in the settings |
| `0x0806fa28` | CRC-32 (poly 0x04c11db7, MSB first, caller's initial value) |
| `0x080535b4` | settings defaults: port 10700, user count 0x2ba9 (kept XOR'd), random 12-byte server ID |
| `0x080526e0` | handshake version callback: `%KDX` with version 0x10 only |
| `0x08066fd8` / `0x08066c14` | raise error (code, module, line) / format "ID x/y @ TRNnnn" |
| `0x080528c8` | session read loop: drops replies, updates idle time, calls the dispatcher |
| `0x0804fdac` | message dispatcher (type → handler); catch at `0x0805090c` sends Status (1) |
| `0x08086640` / `0x080865c0` | send Status (1): error form / success form |
| `0x08049d14` | message type → name (jump table at `0x080abc6c`) |
| `0x08066d04` | error code → text (codes at `0x080b1dc0`, text at `0x080b1f00`) |
| `0x0804ad64` | broadcast a message to all users |
| `0x0805c87c` | send a message to the users whose class is in a class bitmap (613 News Message Posted) |
| `0x0804af04` | encode a user record |
| `0x0804b114` | post-login: sends 105, 107, 106, 213 |
| `0x08061d80` | write a server history record and push 183 |
| `0x080969f8` / `0x08069880` / `0x08096b4c` | file path: decode / validate / join to a host path |
| `0x0804ef00` / `0x0804f8d0` | 422 Volume Info builder (editable bits `caps & 2 \| 1`) / 423 Set Volume Info handler |
| `0x08096780` / `0x08096530` / `0x08094e6c` | Linux volume stubs: capabilities (0x39) / rename (throws 0x10005) / set comments (empty) |
| `0x08052194` / `0x0805234c` | transfer connection: dispatcher (400, 402, 412, 415, 900, 904, 905) / read loop that switches to streaming |
| `0x08050984` / `0x080512b4` | 412 Download File: request / data stream |
| `0x08051750` / `0x08051dec` | 415 Upload File: request / data stream |
| `0x08094558` / `0x080945e0` | file: 64-bit fork size (`stat64`) / read a chunk at a 64-bit offset (`lseek64`) |
| `0x080551b0` | account privilege check for 507/511: effective bits vs the requester's (0x640005) |
| `0x08066a90` | is the key the empty password's key |
| `0x0805c0f0` / `0x0805c344` / `0x0805c498` | 900 Open View Display (replies 902, 903) / 904 mouse / 905 keyboard |
| `0x0805bb54` / `0x0805ba44` / `0x0805c5bc` | view display: grab and diff the screen, send 903 / run encoder / two-colour encoder |
| `0x0804fc40` | per-request flood counter: warnings, then a 2-hour ban |
| `0x0804b7c8` | timer: licence reminder (155) to users whose 118 value was 0 |
| `0x0804b9b0` | 142 Get Connection Monitor Info: connection and transfer records |
| `0x08061598` / `0x0806172c` | 174 Shutdown Server / 175 Upgrade Server Software |
| `0x0806155c` | timer: shut down after 174/175; mode bit 2 halts, bit 1 reboots the host |
| `0x080916e4` | run `/sbin/shutdown -h` or `-r now` |
| `0x08092db8` | open a file by folder and name; 0x30067 when missing |
| `0x0805b8f4` | 804 Exit/Terminate Process |
| `0x08090eb0` | current time: u64 ms since 1900 |
| `0x0805ed88` | move old news into `News Archives` past 1.5 × the archive size |
| `0x0805a110` | 222 Get Chat DCC Addresses |
| `0x0804a310` | "Unknown!" stub of the name table at `0x080abc6c` |

Other binaries, by Ghidra project:

| Binary (project) | Address | What |
|---|---|---|
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x08126aac` | message name jump table, indexed by type − 8 |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x0804d7b4` | message dispatcher (type → handler) |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x08084020` / `0x08080734` | 213 Must Join Chat handler / open a chat window and send 202 Join Chat |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x0804bac0` | connect; default port by type (10700, 10600, 10800, 10900) |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x0812c760` | account editor privilege table: {label, bit} pairs |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x0804c01c` | send 118 (licence report) on a timer |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x0804d654` | build the 100 Set Identity user record |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x0808fae4` / `0x0808f77c` / `0x0808efcc` | DCC listener / accept / login callback |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x0808f3e0` / `0x0808eefc` / `0x0808ff64` | DCC primary connection loop / send 117 SKPK / handle 117 |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x08092240` / `0x08094074` / `0x08094218` | voice: open secondary connection / handle 1000 / handle 1001 |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x08093ae0` / `0x08093670` / `0x0809455c` | voice: send 1002 / send 1003 / receive 1003 (IMA ADPCM) |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x0806f9d8` / `0x0806f36c` | view display: connection loop / 903 pixel decoder |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x080758dc` / `0x08074420` | 422 Volume Info handler / send 423 Set Volume Info with a bit per edited field |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x080a0410` | make a text field read-only when its bit is clear |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x0806fc68` / `0x0806f1f4` | send 904 mouse / 905 keyboard |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x0806b1b0` / `0x0806b68c` | tracker settings: read 711 / build 712 |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x08095c84` | GConsole connection (4000, 4001) |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x0804b824` / `0x0804b938` | Connect dialog: check the peer's hello (`%HGC` switches table and scramble) / pick the connection kind by its version |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x0809aefc` | GConsole header scramble |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x080ea08c` / `0x080ea000` | create connection object (KDX story) / set challenge table at +0xa4, length at +0xa0 |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x080ec058` / `0x080ec280` / `0x080ea768` | challenge table: pick our challenge / answer the peer's / check the peer's response |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x08094be8` / `0x08095ac4` | GConsole: handle 4000 / 4001 draw records |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x0809607c` / `0x08096188` | GConsole: send 4200 Key / 4201 Mouse |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x08069578` | tracker connection: sends 700, dispatches 1, 107, 116, 701, 703, 711, 713, 721 |
| Client 1.620 beta 5 (`kdxlnxclient`) | `0x08068bcc` / `0x08068da0` / `0x08069ac4` | 701 handler / 703 handler / send 702 |
| Client 1.600 Win (`kdxclient`) | `0x0046d0b0` / `0x0046d430` | create connection object / set its payload limit |
| Client 1.600 Win (`kdxclient`) | `0x0046e860` | session: receive and validate header (0x40075 if over the limit) |
| Client 1.600 Win (`kdxclient`) | `0x00402310` | connect to a server; sets the payload limit to 0x100000 |
| Tracker 1.110 (`kdxtracker`) | `0x0804b378` | TCP message dispatcher: 115, 700, 702, 710, 712, 720 |
| Tracker 1.110 (`kdxtracker`) | `0x0804bd9c` | TCP accept: connection with `%KDX`, version 0x30, story table; 5 per IP |
| Tracker 1.110 (`kdxtracker`) | `0x0804bb78` / `0x0804bb4c` | login callback (guest or admin) / version callback |
| Tracker 1.110 (`kdxtracker`) | `0x0804bbf8` | session read loop: sends 107, calls the dispatcher, drops idle connections |
| Tracker 1.110 (`kdxtracker`) | `0x0804c048` | UDP receive: decrypt and validate a registration |
| Tracker 1.110 (`kdxtracker`) | `0x0804b960` / `0x0804b604` | register or refresh a server / build a server record |
| Tracker 1.110 (`kdxtracker`) | `0x0804a370` / `0x0804a3d0` | LCG XOR copy / in place |
| Tracker 1.110 (`kdxtracker`) | `0x08067ee4` | CRC-32 (same as server `0x0806fa28`) |
| Tracker 1.110 (`kdxtracker`) | `0x0804abf4` / `0x0804adcc` | 700 handler (also expires servers) / 702 handler |
| Tracker 1.110 (`kdxtracker`) | `0x0804cb90` / `0x0804cc10` / `0x0804cd88` | 710 handler / 712 handler / serialize settings (`%KDXTSTG`) |
| Tracker 1.110 (`kdxtracker`) | `0x0804b180` | 720 handler |
| Tracker 1.110 (`kdxtracker`) | `0x0804aa30` / `0x0804ab2c` | add a group (sets the tracker-only flag) / drop all groups and server records |
| Tracker 1.110 (`kdxtracker`) | `0x0804e81c` | parse an upstream tracker entry |
| Tracker 1.110 (`kdxtracker`) | `0x0804ca60` | rebind TCP and UDP on a new port |
| Tracker 1.110 (`kdxtracker`) | `0x0804d63c` | parse a `%KDXTSTG` settings blob (712 and the settings file) |
| Tracker 1.110 (`kdxtracker`) | `0x0804df70` | register with upstream trackers (flags bit 0) |
| Tracker 1.110 (`kdxtracker`) | `0x0804c540` | start: opens TCP and UDP on the port (default 10800) |
