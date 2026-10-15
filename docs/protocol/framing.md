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

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | txid | nonzero. Each side counts up from a random start, per connection. A reply reuses the request's txid **[S; O]** |
| 0x04 | 4 | type | nonzero, see [messages/README.md](messages/README.md) |
| 0x08 | 1 | 0 | the server does not check it **[S `FUN_08086108`]** |
| 0x09 | 1 | flags | bit 0 = this is a reply **[S; O]** |
| 0x0a | 2 | len high | bits 32-47 of the length, so the length is 48 bits. The sender writes them from a 64-bit length and fails with 0x40075 past 48 bits **[S `FUN_080868b4`]**. The receive path fails with 0x40075 when they are nonzero, as for any length past the cap **[S `FUN_08086108`]**. Only a transfer data header carries them: a [414 Download File (D)](messages/files.md#m414) with more than 4 GiB left came with byte 0x0b = 0x01 from `kdx.erb.pw` (not captured) **[O]** |
| 0x0c | 4 | len | payload length. Each side caps what it receives and fails with 0x40075 past the cap. Server 1.620 caps messages from a client at 0x20000 (`FUN_080856cc` called from `FUN_08052ad0`) **[S `FUN_080856cc`, `FUN_08052ad0`]**. Client 1.600 caps messages from a server at 0x100000 (`FUN_0046d430` called from `FUN_00402310`, checked in `FUN_0046e860`) **[S client `FUN_0046d430`, client `FUN_00402310`, client `FUN_0046e860`]** |

The sender also has a header-only path (`FUN_080868b4`) followed by
separately encrypted chunks (`FUN_08086a9c`). The server sends each 414
Download File (D) through it, with the fork's remaining length as a 64-bit
value (`FUN_08050984` at 0x0805101f) **[S `FUN_08050984`, `FUN_080512b4`]**.

## Messages

The message types and payload layouts are in
[messages/README.md](messages/README.md), and the header fields are summarised
in [the protocol overview](README.md#header).
