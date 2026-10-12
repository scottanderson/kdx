# Connection handshake and login

Every KDX TCP connection starts with a symmetric two-message handshake. It
also serves as login: the client proves it knows the account's
password-derived key, and the server proves it knows the same key. The
password itself never crosses the wire.

Evidence tags: **[O]** means observed in a capture, **[S]** means found
statically in a binary, **[I]** means inferred.
- Capture: `captures/first-connect.pcapng`. The Win32 Client 1.600 logs in as
  `admin` to Server 1.620. Frames 16 (C→S hello), 18 (S→C hello), 20 (C→S
  proof), 21 (S→C proof).
- Static: `KDXServer.lexe` 1.620 Linux. Addresses are virtual addresses in the
  Ghidra project (see `re/README.md`).

All multi-byte integers are big-endian.

## XOR layer

Both messages are sent as `key(4) | body`. `body` is a whole number of 32-bit
words, each XORed with a key stream **[S `FUN_080872f4`, `FUN_0808751c`; O]**:

```
k = key
for each 32-bit word w in body:  k = (k << 1) + 0x4878 (mod 2^32);  w ^= k
```

The stream runs continuously over the hello header and its body. The key is
random for the hello, and for the proof it equals the proof's first 4 bytes
(see below).

## Sequence

```
client                                server
  | -- HELLO (login name) ----------->  |   the client speaks first [O]
  | <------------------------ HELLO --  |
  | -- PROOF ------------------------>  |   server verifies against the account's stored key
  | <------------------------ PROOF --  |   client verifies the server; then the session begins
```

**[O]** The server's hello waits for the client's hello. **[S]** The server
also sends its own hello before reporting an error, if it hasn't sent one yet.

## HELLO

Total length: `(name_len + pad + ext_len + 0x27) & ~3`. It must be at most
0x800 **[S `FUN_080876e8`]**.

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | key | random XOR key, sent in the clear |
| 0x04 | 4 | magic | `"%TXP"` (0x25545850). Anything else gives error 0x40071 at TRN line 941 **[S, O: lab probe]** |
| 0x08 | 4 | service | `"%KDX"` (0x254b4458) for client/server **[S, O]** |
| 0x0c | 1 | format | `1`. A value of 0 means "peer too old" (0x40072), and >1 means "peer too new" (0x40073) **[S]** |
| 0x0d | 1 | version | Client 1.600 and Client 1.620 beta 5 send `0x10`, Server 1.620 sends `0x20` **[O; `linux-first-connect.pcapng` frames 8, 10]**, Tracker 1.110 sends `0x30` **[S]**. Server 1.620 and Tracker 1.110 accept only service `%KDX` (else 0x40071) with version exactly `0x10` (0x40072 below, 0x40073 above) **[S server `0x080526e0`, tracker `FUN_0804bb4c`]** |
| 0x0e | 1 | pad | bytes skipped before the name, sent as 0 **[S, O]** |
| 0x0f | 1 | name_len | 0..31 |
| 0x10 | 4 | reserved | 0 **[O]** |
| 0x14 | 2 | ext_len | more bytes skipped before the name, sent as 0 **[S, O]** |
| 0x16 | 2 | challenge | random offset into the challenge table, `< table_len - 0x13` **[S, O]** |
| 0x18 | 12 | nonce | per-connection value. It looks time-based (both sides' nonces share bytes) **[O]**. A peer that echoes our nonce is treated as a connection to ourselves and rejected (0x40074) **[S]** |
| 0x24 | name_len | name | login name (client) or empty (server 1.620) **[O]**. See "Login names" |
| … | | padding | zeros up to a 4-byte boundary |

## PROOF (44 bytes)

| Off | Size | Field |
|---|---|---|
| 0x00 | 20 | `auth`: the first 4 bytes double as the XOR key for the rest |
| 0x14 | 20 | the 20 bytes of the challenge table at the offset from the **peer's** hello |
| 0x28 | 1 | status: `0x10` OK, `2` unknown protocol/service, `3` peer too old, `4` peer too new **[S]** |
| 0x29 | 3 | zero |

Bytes 0x04..0x2b are XORed with the stream keyed by bytes 0..3.

When the status isn't OK, `auth` is all zeros except the key word, which is
`peer_nonce[4..8] XOR 0x201f7790` **[S `FUN_0808751c`]**.

Verified on both proofs in the capture **[O, reproduced exactly]**:

```
secret = account_key(password)                 # 32 bytes, below
n      = PEER's hello nonce                    # 12 bytes
input  = s[0:4] n[0:4] s[4:12] n[4:8] s[12:20] n[8:12] s[20:32]     # 44 bytes
d      = SHA-1(input)                          # 20 bytes
auth   = Twofish-256(key = secret).encrypt(d[0:16]) || d[16:20]
```

The receiver recomputes `auth` with its own nonce and compares all 20 bytes,
then compares the 20 challenge bytes with its own table **[S
`FUN_08085a04`, `FUN_0808584c`]**. A failure is error 0x40070.

### account_key(password) [S `FUN_080669cc`; O]

```
password empty: fixed bytes  da39a3ee caaebe6e 5e6b4b0d ba8e29ba 3255bfef 18a2b5c4 95601890 afd80709
otherwise:      d = SHA-1(password)
                d[0:20] || d[0:8] || BE32(fnv1(password, seed = 1))
fnv1(h, b) = (h * 0x01000193) ^ b   for each byte, starting at h = 1
```

The empty-password constant isn't what the formula gives for an empty input.
Words 0, 2, 4, 6 and 7 are the five words of SHA-1(""), and words 1, 3 and 5
are other values. Use the bytes exactly as written above. Guest logins use
it on both sides **[O `captures/kdx-news.pcapng` port 40434, the `kdx` CLI;
`captures/linux-accounts-news.pcapng` frames 94 to 100, Client 1.620 beta
5]**.

The server stores this 32-byte key per account, so the password can't be
recovered from `Accounts.dat` alone **[I]**.

## Login names [S `FUN_0805270c`]

- Empty: the server substitutes `guest`.
- `0x00` followed by a big-endian 32-bit ID (name_len 5): this attaches to the
  existing session with that ID, using that session's key and requiring the
  same peer IP. Probably used for file-transfer connections **[I]**.
- Anything else: an account login. The account's stored key is used to check
  the proof.

## Challenge table

A 1739-byte blob, identical in every client, server and tracker build
(SHA-256 `4fca7636e77e85fdde95862a6be66e12e5e031ad61add65c5e975d3daff4b9e7`).
It is a 1578-byte copyright notice and short story ("Sale of the Cesspool"),
a NUL, and 160 bytes that look like PowerPC code. In the 1.620 Linux server
it's at file offset `0x635a0`. Without a table, the code falls back to
`"All your base are belong to us!"` **[S `FUN_0808529c`]**.

The blob is Haxial's copyrighted text, so **don't commit it**. The Rust
implementation treats it as a required input, obtained once and then cached:

1. Use the cached copy if one exists and its SHA-256 matches.
2. Otherwise, offer to fetch it. Download the smallest known Haxial archive
   that contains it, `KDXTrackerServ1110-Win.zip` (about 249 KB, a plain zip
   with no StuffIt; see `vendor/MANIFEST.md` for the Wayback URL). Extract
   the blob from it.
3. Otherwise, ask the user for either a KDX file to extract it from (any
   client, server or tracker build; zip archives are opened) or the blob
   itself.

Extraction: find `This program including the following story`, take 1739
bytes from there, and check the SHA-256 above.

## Error codes [S `FUN_08066fd8`, `FUN_08066c14`]

Logged as `ID <hi>/<lo> @ <module><line>`. Here `<hi>/<lo>` are the two 16-bit
halves of the code, and the module tag is three ASCII bytes passed as an
integer (`0x54524e` is `TRN`, the transport module). So
`ID 4/113 @ TRN941` means code 0x40071 at TRN line 941 (bad magic).

The full code table is in [KDXProtocol.md](KDXProtocol.md#error-codes).
These are the codes the handshake raises:

| Code | Meaning (from log strings and call sites) |
|---|---|
| 0x40070 | authentication failed (bad proof or challenge) |
| 0x40071 | unknown protocol (magic or service mismatch) |
| 0x40072 | peer's protocol version too old |
| 0x40073 | peer's protocol version too new |
| 0x40074 | invalid protocol data; the handshake raises it for a nonce echo (a connection to ourselves) |
| 0x40075 | protocol limit exceeded; the handshake raises it for a hello over 0x800 bytes |

## Open questions

- Resolved: the `0x00`+ID login opens a transfer connection. See
  [KDXProtocol.md](KDXProtocol.md#transfer-connections).
- Everything after the handshake is covered in `session.md`.
- Resolved: the version-acceptance callback (`+0x88`) is `0x080526e0` in the
  server. See the version field above.
- Resolved: guest logins use the empty-password constant (see
  account_key above).
