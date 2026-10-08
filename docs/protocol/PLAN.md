# Plan: documenting the KDX protocol

Goal: a written spec complete enough that a Rust client works with the original
KDX Server 1.620, the original KDX Client 1.600 works with a Rust server, and
the original programs work with a Rust tracker in place of Tracker 1.110.
Every documented fact must point to evidence (a capture or a binary address).

## What we start with

- Reference builds of the client, server and tracker on three platforms
  (`vendor/MANIFEST.md`). The same Haxial cross-platform codebase produced every
  build, so a fact found in one build should hold in the others.
- Public knowledge is thin. The only source is the hlwiki page (UDP tracker
  LCG constants; MD5/CRC32/Twofish in use; story text mixed into packets as
  anti-piracy). No one has published the message set, handshake or file
  transfer format.
- Known so far: framing is `key(4) | header(32) | payload padded to 4`, all
  big-endian. Default server port is probably 10700. There is a separate
  "File Transfer Port" and an "Alternate Port". The protocol has explicit
  version checks ("remote host is too OLD / more recent").

## Approach

Combine dynamic and static analysis. Packet captures show *what* gets sent.
Disassembly explains *why* and finds messages the GUI rarely triggers.

### Phase 0: Isolated lab (done; see `lab/README.md`)
- `lab/kdx-lab` runs Server 1.620 and Tracker 1.110 (Linux builds) and Client
  1.600 (under Wine) or the Linux beta 5 client. Each runs in a `bwrap` sandbox
  inside a private network namespace that only has loopback. Docker isn't used,
  because WSL integration is off and bwrap needs no root.
- `dumpcap` records `lo` to `captures/<scenario>.pcapng`. Each capture has a
  `captures/<scenario>.md` listing the GUI steps performed.
- Server state lives in `lab/run/server/`. The lab admin login is in
  `lab/README.md`.
- Verified: the server listens on TCP 10700 only (no UDP listener) and waits
  for the client to send first. On a bad first packet it logs
  `The network protocol is unknown. (ID 4/113 @ TRN941)`.
- Still open: whether an unregistered server or client limits any features.

### Phase 1: Transport and crypto
1. Confirm the framing and XOR stream against real captures.
   Decide whether the 4-byte key is random per packet, and whether the key
   stream continues across the header and payload.
2. Identify what happens during connection setup before the XOR framing:
   magic bytes, version exchange, any Twofish or MD5 step.
3. Locate the "story text" and its role. The prose is embedded in both client
   and server (for example "Contestant number 3 interrupted. \"My buzzer's not
   working!!\" ..."). Find the code that mixes it into packets.
4. Ship `kdx-dissect`, a Rust CLI in the workspace that reads a pcap and
   prints decrypted, decoded messages. It becomes the main tool for every
   later phase and the first consumer of the `kdx-proto` crate.

### Phase 2: Static mapping (Ghidra)
- Ghidra 12.1.4 runs headless in Docker (`re/ghidra`; see `re/README.md`).
  Server 1.620 is imported and analyzed. The handshake and login are done
  and verified against traffic: `docs/protocol/handshake.md`.
- Start with the Linux ELF builds, which are the easiest to load. Also load the
  Win32 build; the PPC/PEF Mac builds may keep traceback-table function names,
  which are worth checking for free symbols.
- Find anchors using constants: `0x4878` (the XOR key step), the LCG constants,
  MD5 init constants, Twofish q-tables, the CRC32 table. Follow cross-references
  from `send`/`recv`/`sendto` to the framing and then to the message dispatch.
- Recover the dispatch table that maps message type to handler. That gives the
  full list of message types, including admin-only and rare ones.
- Use the length-prefixed UI and error strings to name handlers ("Get Tracker
  Server List", "protocol breach", "limit in the net protocol was exceeded").
- Keep the analysis scripts and a table of function addresses
  (`re/README.md`) in the repo so the work is reviewable. The Ghidra project
  and decompiled output stay local, because they're derived from Haxial's
  code.

### Phase 3: Message coverage by feature
Work through features in roughly the order a client needs them. For each
message, document direction, type ID, field layout, encodings (text encoding,
integer widths, how lists and nested records are encoded), error replies, and
which build introduced it:

1. Connect, version negotiation, login and authentication (including password
   hashing), and the session keepalive.
2. User list, user info, custom icons, and status changes.
3. Public chat rooms (list, join, leave, message) and private messages.
4. News and boards.
5. Files: browse, info, search, plus download and upload over the file
   transfer port, including resume, folders, and the Mac data/resource forks.
6. Accounts, user classes and permissions, and admin and moderation actions.
7. Tracker: server registration (UDP plus LCG) and client listing.
8. Last: voice chat, remote desktop administration, and the IRC bridge. The
   bridge is server-side only and uses standard IRC, so it needs little
   protocol work.

### Phase 4: Spec format
- `docs/protocol/` gets one Markdown file per area (`transport.md`, `crypto.md`,
  `login.md`, `chat.md`, `files.md`, `tracker.md`, ...) and a `messages.md`
  index of every type ID.
- Each field or message gets an evidence tag. **Observed** means a capture file
  and frame number. **Static** means a binary, build, and address.
  **Inferred** means a guess not yet verified. The goal is to turn every
  Inferred tag into one of the other two.
- Byte layouts use tables with offset, size, type, and meaning.

### Validation (connects the spec to the Rust code)
- Capture fixtures become `kdx-proto` tests: decode, re-encode, and compare
  byte-for-byte.
- Differential testing in the lab: the Rust client against the original
  server, and the original client against the Rust server. A feature counts as
  "documented" once both directions work.

## Licensing
The project is dual-licensed MIT OR Apache-2.0.

## Suggested Rust layout (once Phase 1 starts)
Cargo workspace with `kdx-proto` (framing, crypto, message types; no I/O),
`kdx-dissect` (pcap to text), `kdx-client`, and later `kdx-server`.

## Open decisions
- None currently.
