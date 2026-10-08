# Session layer (after the handshake)

Evidence tags as in `handshake.md`. Captures: `captures/first-connect.pcapng`
(session data starts at C→S frame 22 and S→C frame 23). The whole session was
decrypted and parsed with nothing left over (824 bytes C→S and 3120 bytes S→C,
handshakes included) **[O]**.

## Encryption

- **Cipher:** Twofish-256 in **ECB** mode. The key is the same 32-byte
  `account_key` that authenticated the handshake (the fixed constant for an
  empty password). Both directions use the same key. **[S `FUN_080863b4`,
  `FUN_080879c4`; O]**
- **Message layout:** every message is a 16-byte header block followed by
  `ceil(len/16)` payload blocks, and each block is encrypted on its own.
- **Padding:** the unused tail of the last payload block is filled with
  `p = txid * 0x41c64e6d + 0x3039` (the 32-bit value repeated as memory words)
  before the payload bytes are copied over it **[S, O]**. Receivers ignore it.

## Header scramble

Before the header block is encrypted, it goes through an extra keyless
scramble that the server installs as the connection's header hook (`+0x08`,
set in `FUN_08052ad0` to `FUN_0804a510`; the transport's default is a no-op).
The client does the same, because its messages decode with the same inverse
**[S, O]**.

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
| 0x00 | 4 | txid | nonzero. Each side counts up from a random start, per connection. A reply reuses the request's txid **[S, O]** |
| 0x04 | 4 | type | nonzero, see below |
| 0x08 | 1 | 0 | |
| 0x09 | 1 | flags | bit 0 = this is a reply **[S, O]** |
| 0x0a | 2 | 0 | the receiver rejects nonzero values with 0x40075 **[S `FUN_08086108`]** |
| 0x0c | 4 | len | payload length. The server caps it at 0x20000 **[S `FUN_080856cc` call in `FUN_08052ad0`]**, with error 0x40075 if exceeded |

The sender also has a header-only path (`FUN_080868b4`) followed by
separately encrypted chunks (`FUN_08086a9c`). It's probably used for large or
streamed payloads such as file transfers **[I]**.

## Message types seen so far

Types come in request/notification pairs that differ by one: N from the
client, N+1 from the server **[O]**.

| Type | Dir | Seen as | Payload notes |
|---|---|---|---|
| 100 (0x64) | C→S | login / user info | 565 bytes, mostly a 2-bit-per-pixel icon bitmap **[O]** |
| 101 (0x65) | S→C | user info (this user) | 568 bytes, same layout as 100 with a 4-byte user ID (1) in front **[O]** |
| 105 (0x69) | S→C | server info | 32-byte value, IDs, server name `New Server` **[O]** |
| 300 (0x12c) | C→S | send private message | `u32 target user id` (1 = self here), 12 zero bytes, then the text (`test`, `hi`) **[O]** |
| 301 (0x12d) | S→C | private message delivered | same layout. The ID is the sender **[I]** **[O]** |
| 605 (0x25d) | C→S | list newsgroups | empty **[O]** |
| 606 (0x25e) | S→C reply | newsgroup list | count, group ID, timestamp-like value, name `General News` **[O]** |
| 607 (0x25f) | C→S | get newsgroup messages | `u32 group id` (1) **[O]** |
| 608 (0x260) | S→C reply | newsgroup messages | message records ("Welcome to KDX! …") **[O]** |

Field-level layouts of these payloads come next. The plan is to locate the
server's dispatch table (by message type) and read each handler, using the
capture to check each field.
