# Framing and encryption over TCP

For hex and field notation, see [Conventions](README.md#conventions).

This page describes the messages that follow the handshake on a TCP connection.
The registration datagrams that a server sends to a tracker use UDP, a separate
protocol with its own cipher and checksum; see [udp.md](udp.md).

The examples come from `captures/first-connect.pcapng`. Session data starts at
frame 22 from the client and frame 23 from the server. The whole session
decrypts and parses with nothing left over: 824 bytes from the client and 3120
bytes from the server, with the handshakes. <!-- [O] -->

## Encryption

- **Cipher:** Twofish-256 in **ECB** mode. The key is the same 32-byte
  `account_key` that the handshake used, which for an empty password is the
  fixed constant. Both directions use the same key. <!-- [S `FUN_080863b4`,
  `FUN_080879c4`; O] -->
- **Message layout:** each message is a 16-byte header block and then
  `ceil(len/16)` payload blocks. Each block is encrypted separately.
- **Padding:** before the payload bytes are copied into the last payload block,
  its unused end is filled with `p = txid * 0x41c64e6d + 0x3039`, as repeated
  32-bit memory words. <!-- [S; O] --> Receivers ignore it.
- **Transfer data padding:** Client 1.620 for Linux pads the data after a [417
  Upload File (D)](messages/files.md#m417) header in the same way, with the txid
  of that header. <!-- [O `captures/linux-files.pcapng` frame 47] --> Server
  1.620 pads the data after a [414 Download File (D)](messages/files.md#m414)
  header with a different repeated 32-bit value. <!-- [O
  `captures/linux-files.pcapng` frames 69, 113] -->

## Header scramble

Before the header block is encrypted, it goes through a scramble with no key.
The server installs the scramble as the connection's header hook. <!-- [S; O]
The hook is at +0x08; FUN_08052ad0 sets it to FUN_0804a510, and the transport's
default does nothing. --> The client uses the same scramble, because its
messages decode with the same inverse.

Let `h0..h3` be the four 32-bit words of the header, and `w0..w3` the words
after the scramble, which go into Twofish:

```
encode (send):                                decode (after Twofish decrypt):
w0 = rotl(h1, 13) ^ 0x6e7dfd34                h0 = rotr(w2 ^ 0xa95a759b, 7)
w1 = rotr(h2, 6)  ^ 0xe152da04                h1 = rotr(w0 ^ 0x6e7dfd34, 13)
w2 = rotl(h0, 7)  ^ 0xa95a759b                h2 = rotl(w1 ^ 0xe152da04, 6)
w3 = rotr(h3, 3)  ^ 0x06992e25                h3 = rotl(w3 ^ 0x06992e25, 3)
```

This is verified in both directions: `kdx-proto` seals each decrypted message in
the capture again, and gets exactly the captured bytes back, with the
padding. <!-- [O] -->

## Header

Struct `Header` (16 bytes):<a id="sHeader"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | `u32` | `txid` | Not zero. Each side counts up from a random start on each connection. A reply has the txid of its request. <!-- [S; O] --> |
| 0x04 | 4 | `u32` | `kind` | Not zero. See [messages/README.md](messages/README.md). |
| 0x08 | 1 | `u8` | `reserved` | 0 as sent. The original server does not check it, but `kdx-proto` rejects a byte that is not zero. <!-- [S `FUN_08086108`] --> |
| 0x09 | 1 | `u8` | `flags` | Bit 0 is set in a reply. <!-- [S; O] --> |
| 0x0a | 6 | `u48` | `len` | The payload length in bytes, before padding. See [Length limits](#length-limits). <!-- [S `FUN_080868b4`, `FUN_08086108`] --> |

The length is one 48-bit number, a `u48`. The software applies all limits on it,
and the limits are different for each type of connection; none of them is part
of the wire format. <!-- [S `FUN_080868b4`, `FUN_08086108`] -->

### Length limits

| Situation | Limit | Failure | See |
|---|---|---|---|
| Sender, any connection | 48 bits, so at most 0xffffffffffff, written from a 64-bit length <!-- [S `FUN_080868b4`] --> | 0x40075 past 48 bits |  |
| Server 1.620 receiving from a client | 0x20000 <!-- [S `FUN_080856cc`, `FUN_08052ad0`] --> | 0x40075 past the cap |  |
| Client 1.600 receiving from a server | 0x100000 <!-- [S client `FUN_0046d430`, client `FUN_00402310`, client `FUN_0046e860`] --> | 0x40075 past the cap |  |
| Tracker receiving on a TCP browse connection | 0x10000 <!-- [S `FUN_0804bd9c`] --> | unknown | [tracker.md](tracker.md#connection) |
| Receive path, bits 32 to 47 of the length | must be zero <!-- [S `FUN_08086108`] --> | 0x40075, as for any length past the cap |  |
| Transfer data header, bits 32 to 47 of the length | set when a fork has more than 4 GiB left. A [414 Download File (D)](messages/files.md#m414) from `kdx.erb.pw` had byte 0x0b = 0x01. <!-- [O] not captured --> | unknown |  |
| Transfer data ([414 Download File (D)](messages/files.md#m414), [417 Upload File (D)](messages/files.md#m417)) | one header covers the whole fork, past the normal caps <!-- [O `captures/kdx-cli.pcapng` frames 133, 190] --> | none seen | [files.md](messages/files.md#m414) |
| Hello in the handshake | 0x800 <!-- [S `FUN_080876e8`] --> | 0x40075 | [handshake.md](handshake.md#hello) |
| Chat text in [207 Send Chat Text](messages/chat.md#m207) | 0x1000 <!-- [S `FUN_08058d74`] --> | The server cuts the message, which can split a multi-byte character. | [encodings.md](encodings.md#strings) |

The sender also has a path that sends only a header, and then chunks that are
encrypted separately. <!-- [S `FUN_080868b4`, `FUN_08086a9c`] --> The server
sends each [414 Download File (D)](messages/files.md#m414) through this path,
with the fork's remaining length as a 64-bit value. <!-- [S `FUN_08050984`,
`FUN_080512b4`] FUN_08050984 at 0x0805101f -->

## Across versions

The header has the same layout and checks in all builds with `%KDX`, from Server
0.921 to Server 1.620. Each x86 build has one receive function that gives both
0x40074 and 0x40075, and they all agree: <!-- [S] re/README.md lists the
function in each build -->

- 0x40075 if the length is past the cap, or bytes 0x0a and 0x0b are not zero.
- 0x40074 if the txid or the type is 0.
- Bit 0 of the flags marks a reply. Byte 0x08 is never read.

They differ only in how they give the error. Server 1.620 and Client betas 4 and
5 log the TRN module and the line, and older builds give only the code. <!-- [S]
-->

All 1.x builds have the four constants of the header scramble: Windows, Linux,
Mac PEF, and Mach-O, from Client 1.110 and Server 1.300 on. The `%HGC` constants
first appear in Client 1.320. <!-- [S] re/scan_consts.py --> The x86 builds use
the constants in the same scramble. The Mac builds are not decompiled, but
probably do the same. <!-- [I] -->

Server 0.921 (Mac PEF) has the same 16-byte header, with no scramble hook. Its
receive function makes the same checks, and its function that sends only a
header makes the same 48-bit check. <!-- [S `FUN_10035790`, `FUN_10035b20`] -->
It encrypts with a cipher at connection offset 0x12a that is not yet
identified. <!-- [S] -->

All builds except Tracker 1.110 for Windows can send a header alone, with the
48-bit length check. <!-- [S] -->

These are the receive caps, from the calls to the cap setter of each
build: <!-- [S] -->

| Role | Cap | Where |
|---|---|---|
| Server, 1.300 to 1.620 | 0x20000 | each accepted connection |
| Tracker 1.110 | 0x10000 | each accepted connection |
| Client, 1.110 to beta 5 | 0x100000 | connection to a server |
| Client, 1.110 to beta 5 | 0x8000 | transfer connection (sends [402 Create Folder](messages/files.md#m402), [412 Download File](messages/files.md#m412), [415 Upload File](messages/files.md#m415), or [417 Upload File (D)](messages/files.md#m417)) |
| Client, 1.110 to beta 5 | 0x80000 | DCC listener (see [dcc.md](dcc.md)) |
| Client 1.600, beta 5 | 0x1000000 | one caller, not identified <!-- [S `FUN_0041bde0`, `FUN_0806f61c`] --> |

Client 1.110 is `KDXClient-Win.zip`, from a Wayback capture of Haxial's
downloadstatic folder on 2003-12-15; see `vendor/MANIFEST.md`.

## Messages

[messages/README.md](messages/README.md) gives the message types and payload
layouts, and [the protocol overview](README.md#header) sums up the header
fields.
