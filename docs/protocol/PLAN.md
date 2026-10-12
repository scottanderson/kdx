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
  1.600 (under Wine) or Client 1.620 beta 5 (Linux). Each runs in a `bwrap`
  sandbox inside a private network namespace that only has loopback. Docker
  isn't used, because WSL integration is off and bwrap needs no root.
- `dumpcap` records `lo` to `captures/<scenario>.pcapng`. Each capture has a
  `captures/<scenario>.md` listing the GUI steps performed.
- Server state lives in `lab/run/server/`. The lab admin login is in
  `lab/README.md`.
- Verified: the server listens on TCP 10700 only (no UDP listener) and waits
  for the client to send first. On a bad first packet it logs
  `The network protocol is unknown. (ID 4/113 @ TRN941)`.
- An unregistered client reports itself with 118, and the server then sends
  it a licence reminder every 15 minutes or so during its first two hours
  (`KDXProtocol.md`, 118). No other limit has been found.

### Phase 1: Transport and crypto (done; see `handshake.md`, `session.md`)
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
- Recover the dispatch table that maps message type to handler. Done: the
  dispatcher is `FUN_0804fdac`, and the name table at `FUN_08049d14` names
  every type. Both are listed in `KDXProtocol.md`.
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

Status (2026-10-12). "Client works" means the `kdx` CLI did it against Server
1.620 in the lab (`captures/kdx-cli.md`):

| Area | Documented | Codec tested on a capture | Client works |
|---|---|---|---|
| 1. Login | yes; guest login not tried against traffic | yes | yes (admin) |
| 2. Users | user record, 101 to 107, 110, 111, 118, 152, 153; the icon bitmap format not yet | yes | user list |
| 3. Chat, private messages | yes; 200-223 captured (`linux-chat-admin`, `linux-chat-rooms`, `kdx-cli`); 223 only empty | yes | join, say, listen, msg |
| 4. News | yes; 600, 601, 604 captured (`linux-accounts-news`) | yes | read, post, delete |
| 5. Files | yes; 420, 422, 423, 450–453 captured (`linux-catalog-volumes`) | yes | ls, get, put |
| 6. Accounts, admin | yes, with every privilege bit named; 804, 1101, 1105 read but not probed | yes | account list, show, add, set, rm; passwd |
| 7. Tracker | yes, statically (`tracker.md`); not captured | no | no |
| 8. Voice, view display | yes, statically (`dcc.md`; view display in `KDXProtocol.md`); not captured | no | no |

### Phase 4: Spec format
- `KDXProtocol.md` is the single reference: the message catalog, a
  description of each message, error codes, privileges, and sequences. It
  grows as each area is read and probed.
- `handshake.md`, `session.md`, and `tracker.md` hold the long write-ups
  that are too detailed for the reference. Add another such file only when an
  area needs one (file transfers).
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

## Rust layout
Cargo workspace. Every crate builds on `kdx-proto`, and on `kdx-codec` where
it does async I/O.

| Crate | Kind | Role |
|---|---|---|
| `kdx-proto` | library | framing, crypto, message types; no I/O |
| `kdx-codec` | library | tokio codecs for the handshake and session |
| `kdx-client` | library | for writing KDX clients (`Connection`, `Client`) |
| `kdx-cli` | programs | `kdx`, the command-line client, and `kdx-probe` |
| `kdx-server` | library | for writing KDX servers (placeholder) |
| `kdx-daemon` | program | the server built on `kdx-server` (placeholder) |
| `kdx-tracker` | program | the tracker (placeholder) |
| `kdx-dissect` | program | pcap to text |

`kdx-proto::msg` has codecs for the payloads in areas 1 to 5, and in area 6
for the account messages, 114, 116, 143, 150 to 152, 154, 155, 171 and 172
(the named fields, with the rest kept as raw bytes), 181, 183, 803, 1107, and
the folder access record of 1100, 1104, and 1105.

## Open decisions
- None currently.
