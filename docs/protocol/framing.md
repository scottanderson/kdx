# Framing and encryption over TCP (after the handshake)

Tags, hex, and field notation: see [Conventions](README.md#conventions).

Everything on this page is TCP. The tracker's registration datagrams are UDP, a
separate protocol with its own cipher and checksum; see [udp.md](udp.md).

Evidence tags as in [handshake.md](handshake.md). Captures:
`captures/first-connect.pcapng` (session data starts at C→S frame 22 and S→C
frame 23). The whole session was decrypted and parsed with nothing left over
(824 bytes C→S and 3120 bytes S→C, handshakes included) **[O]**.

## Encryption

- **Cipher:** Twofish-256 in **ECB** mode. The key is the same 32-byte
  `account_key` that authenticated the handshake (the fixed constant for an
  empty password). Both directions use the same key. **[S `FUN_080863b4`,
  `FUN_080879c4`; O]**
- **Message layout:** every message is a 16-byte header block followed by
  `ceil(len/16)` payload blocks, and each block is encrypted on its own.
- **Padding:** the unused tail of the last payload block is filled with
  `p = txid * 0x41c64e6d + 0x3039` (the 32-bit value repeated as memory words)
  before the payload bytes are copied over it **[S; O]**. Receivers ignore it.
- **Transfer data padding:** Client 1.620 for Linux pads the data after a
  [417 Upload File (D)](messages/files.md#m417) header the same way, with that
  header's txid (`captures/linux-files.pcapng` frame 47) **[O]**. Server 1.620
  pads the data after a [414 Download File (D)](messages/files.md#m414) header
  with a repeated 32-bit value that is not this fill (frames 69 and 113)
  **[O]**.

## Header scramble

Before the header block is encrypted, it goes through an extra keyless
scramble that the server installs as the connection's header hook (`+0x08`,
set in `FUN_08052ad0` to `FUN_0804a510`; the transport's default is a no-op).
The client does the same, because its messages decode with the same inverse
**[S; O]**.

Let `h0..h3` be the header's four big-endian 32-bit words, and `w0..w3` the
words after scrambling (the input to Twofish):

```
encode (send):                                decode (after Twofish decrypt):
w0 = rotl(h1, 13) ^ 0x6e7dfd34                h0 = rotr(w2 ^ 0xa95a759b, 7)
w1 = rotr(h2, 6)  ^ 0xe152da04                h1 = rotr(w0 ^ 0x6e7dfd34, 13)
w2 = rotl(h0, 7)  ^ 0xa95a759b                h2 = rotl(w1 ^ 0xe152da04, 6)
w3 = rotr(h3, 3)  ^ 0x06992e25                h3 = rotl(w3 ^ 0x06992e25, 3)
```

Both sides are verified: `kdx-proto` re-seals every decrypted message in the
capture and gets the captured bytes back exactly, including the padding
**[O]**.

## Header

Struct `Header` (16 bytes):

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | u32 | txid | nonzero. Each side counts up from a random start, per connection. A reply reuses the request's txid **[S; O]** |
| 0x04 | 4 | u32 | kind | nonzero, see [messages/README.md](messages/README.md) |
| 0x08 | 1 | u8 | reserved | 0 as sent. The original server does not check it, but `kdx-proto` rejects a nonzero byte **[S `FUN_08086108`]** |
| 0x09 | 1 | u8 | flags | bit 0 = this is a reply **[S; O]** |
| 0x0a | 6 | u48 | len | payload length in bytes, before padding. See [Length limits](#length-limits) **[S `FUN_080868b4`, `FUN_08086108`]** |

The wire field is one 48 bit number, a `u48`. Every limit on it is enforced in
software, and the limits differ by connection type; none is part of the wire
format **[S `FUN_080868b4`, `FUN_08086108`]**.

### Length limits

| Situation | Limit | Failure | Source |
|---|---|---|---|
| Sender, any connection | 48 bits, so at most 0xffffffffffff, written from a 64-bit length | fails with 0x40075 past 48 bits | **[S `FUN_080868b4`]** |
| Server 1.620 receiving from a client | 0x20000 | fails with 0x40075 past the cap | **[S `FUN_080856cc`, `FUN_08052ad0`]** |
| Client 1.600 receiving from a server | 0x100000 | fails with 0x40075 past the cap | **[S client `FUN_0046d430`, client `FUN_00402310`, client `FUN_0046e860`]** |
| Tracker receiving on a TCP browse connection | 0x10000 | not stated | **[S `FUN_0804bd9c`]**, see [tracker.md](tracker.md#connection) |
| Receive path, bits 32 to 47 of the length | must be zero | fails with 0x40075, as for any length past the cap | **[S `FUN_08086108`]** |
| Transfer data header, bits 32 to 47 of the length | carried when a fork has more than 4 GiB left: a [414 Download File (D)](messages/files.md#m414) came with byte 0x0b = 0x01 from `kdx.erb.pw` (not captured) | not stated | **[O]** |
| Transfer data (414, 417) | one header covers the whole fork, past the normal caps | none seen | **[O `captures/kdx-cli.pcapng` frames 133, 190]**, see [files.md](messages/files.md#m414) |
| Hello in the handshake | 0x800 | fails with 0x40075 | **[S `FUN_080876e8`]**, see [handshake.md](handshake.md#hello) |
| Chat text in 207 Send Chat Text | 0x1000 | the server caps the message, which can split a multi-byte character | **[S `FUN_08058d74`]**, see [encodings.md](encodings.md#strings) |

The sender also has a header-only path (`FUN_080868b4`) followed by
separately encrypted chunks (`FUN_08086a9c`). The server sends each 414
Download File (D) through it, with the fork's remaining length as a 64-bit
value (`FUN_08050984` at 0x0805101f) **[S `FUN_08050984`, `FUN_080512b4`]**.

## Across versions

The header has the same layout and checks in every build with `%KDX`, from
Server 0.921 to Server 1.620. Each x86 build below has one receive function
that raises both 0x40074 and 0x40075, and they agree **[S]**:

- 0x40075 when the length is past the cap or bytes 0x0a–0x0b are nonzero.
- 0x40074 when the txid or the type is 0.
- Flags bit 0 marks a reply. Byte 0x08 is never read.

The only difference is how they raise the error. Server 1.620 and Client
betas 4 and 5 log the TRN module and line; the older builds raise the bare
code **[S]**.

Every 1.x build has the header scramble's four constants: Windows, Linux, Mac
PEF, and Mach-O, from Client 1.110 and Server 1.300 on. The `%HGC` constants
first appear in Client 1.320 **[S]** (`re/scan_consts.py`). The x86 builds
use them in the same scramble; the Mac builds aren't decompiled **[I]**.

Server 0.921 (Mac PEF) has the same 16-byte header with no scramble hook. Its
receive function `FUN_10035790` makes the same checks, and its header-only
send `FUN_10035b20` the same 48-bit check. It encrypts with a cipher at
connection offset 0x12a that is not yet identified **[S]**.

The header-only send, with the 48-bit length check, is in every build but
Tracker 1.110 for Windows **[S]**.

Receive caps, from the calls to each build's cap setter **[S]**:

| Role | Cap | Where |
|---|---|---|
| Server, 1.300–1.620 | 0x20000 | each accepted connection |
| Tracker 1.110 | 0x10000 | each accepted connection |
| Client, 1.110–beta 5 | 0x100000 | connection to a server |
| Client, 1.110–beta 5 | 0x8000 | transfer connection (sends 402, 412, 415, or 417) |
| Client, 1.110–beta 5 | 0x80000 | DCC listener (see [dcc.md](dcc.md)) |
| Client 1.600, beta 5 | 0x1000000 | one caller, not identified (`FUN_0041bde0`, `FUN_0806f61c`) |

Function addresses by Ghidra project (`re/README.md`):

| Project | Receive header | Send header only | Set receive cap |
|---|---|---|---|
| `KDXClient1110-Win` | `FUN_00460320` | `FUN_00460850` | `FUN_0045ef70` |
| `KDXClient1320-Win` | `FUN_00464cf0` | `FUN_00465220` | `FUN_00463870` |
| `KDXClient1400-Win` | `FUN_004677f0` | `FUN_00467d00` | `FUN_004663c0` |
| `KDXClient1500-Win` | `FUN_0046d530` | `FUN_0046da40` | `FUN_0046c100` |
| `KDXClient1520-Win` | `FUN_0046d7f0` | `FUN_0046dd00` | `FUN_0046c3c0` |
| `KDXClient1600-Win` | `FUN_0046e860` | `FUN_0046ed70` | `FUN_0046d430` |
| `KDXClient-Lnx-beta4` | `FUN_080e94a4` | `FUN_080e9c50` | `FUN_080e8a68` |
| `KDXClient-Lnx-beta5` | `FUN_080eae6c` | `FUN_080eb618` | `FUN_080ea430` |
| `KDXServer1300-Win` | `FUN_0042cf20` | `FUN_0042d450` | `FUN_0042ba90` |
| `KDXServer1500-Win` | `FUN_004300e0` | `FUN_004305f0` | `FUN_0042ec90` |
| `KDXServer1520-Win` | `FUN_00430240` | `FUN_00430750` | `FUN_0042edf0` |
| `KDXServer1520-Lnx` | `FUN_0808ac04` | `FUN_0808b108` | `FUN_0808a248` |
| `KDXServer1600-Win` | `FUN_004317a0` | `FUN_00431cb0` | `FUN_00430350` |
| `KDXServer1600-Lnx` | `FUN_0808b71c` | `FUN_0808bc20` | `FUN_0808ad60` |
| `KDXServer1620-Lnx` | `FUN_08086108` | `FUN_080868b4` | `FUN_080856cc` |
| `KDXTrackerServ1110-Win` | `FUN_00414d00` | none | `FUN_00413990` |
| `KDXTrackerServ1110-Lnx` | `FUN_080885dc` | `FUN_08088ae0` | `FUN_08087c20` |

Client 1.110 is `KDXClient-Win.zip` from a local download dated 2003-12-15.
It isn't in `vendor/MANIFEST.md` because no Wayback capture of it was found.

## Messages

The message types and payload layouts are in
[messages/README.md](messages/README.md), and the header fields are summarised
in [the protocol overview](README.md#header).
