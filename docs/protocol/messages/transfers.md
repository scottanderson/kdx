# Transfer connections

How a download or upload runs on its own connection: 412, 415, 418 and 419.

Files move on a second connection to the same port, attached to a user who
is already logged in. **[S `FUN_0805270c`, `FUN_080528c8`, `FUN_08052194`;
O kdx-probe `--attach`]**

1. The client connects and runs the normal handshake with a special login:
   - the login name is 5 bytes: `00`, then the u32 user ID from 105 Server
     Info offset 0x24;
   - the account key is the 32 bytes at 105 Server Info offset 0x00, not a
     password hash.
2. The server accepts it only if that user is logged in, is not itself an
   attached connection, and connects from the same IP address. The attached
   connection gets its own user ID, but it acts with the parent's
   privileges and base folder. The server log calls it a transfer, and on
   close it logs `Transfer Disconnect` **[O]**.
3. On this connection, the server handles only 400 Get File List, 402
   Create Folder, 412 Download File, 415 Upload File, and 900, 904, 905
   (view display). It ignores everything else, including 108 Ping **[S, O]**.
4. One transfer runs at a time per connection. A second 412 Download File or
   415 Upload File during a transfer gets 1 Status 0x10003. Afterwards the
   connection takes requests again **[S]**.

The stream is encrypted unless the client asks for plain mode (request
offset 0x17 bit 0, which needs privilege bit 65, probably Disable
Encryption **[I]**). Encrypted data is framed exactly like a message: a header
whose length gives the byte count, followed by that many bytes in 16-byte
Twofish blocks. A client can read it with its normal receive path **[O]**.
Plain mode has not been tried.

**412 Download File**
Initiator: Client, on a transfer connection. Privilege: Download (10).
**[S `FUN_08050984`, `FUN_080512b4`; O]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | data fork offset | where to resume |
| 0x08 | 8 | resource fork offset | |
| 0x10 | 7 | | |
| 0x17 | 1 | options | bit 0: plain (unencrypted) transfer |
| 0x18 | | file path | |

The payload must be at least 0x1a bytes. Each offset is rounded down to a
multiple of 0x1000. An offset at or past the end still gets its 414
Download File (D), with length 0. **[O]**: offset 5000 resumed at 4096, and the
65904 bytes that came back equal the full download from 0x1000; offset
70000 on the 70000-byte file got an empty 414 (`captures/probe.pcapng`
frames 271–281, 383–389, and 348–351; test `downloads`).

Offsets and sizes are 64-bit throughout. The server rounds the offset
down within its 64-bit value, reads with `lseek64` to it, and sends the
remaining length as a 48-bit header length **[S `FUN_08050984`,
`FUN_080945e0`]**. Server 1.600 for Windows seeks with the high half too
**[S `FUN_0042c600`]**. So the data should start exactly at the rounded
offset past 4 GiB as well. Once, though, a download from `kdx.erb.pw` with
an offset above 4 GiB began 0xF000 bytes past the rounded offset **[O, not
captured]**; the server's build is unknown, and the cause is open. A
client should take the start as the data fork size less the 414 length,
not as the rounded offset **[I]**.

A file over 0x40000 bytes is refused with code 0x640003 while the same
user already has a download running, unless the user has privilege bit 25
(probably Allow Simultaneous Downloads **[I]**).

The server answers with **413 Download File (R)**, 99 bytes for a plain
Linux file:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | data fork size | 0x11170 = 70000 |
| 0x08 | 8 | resource fork size | |
| 0x10 | 8 | 0 | |
| 0x18 | 8 | created | |
| 0x20 | 8 | modified | |
| 0x28 | 0xc | | 0, 0, 1 |
| 0x34 | 0x14 | 0 | |
| 0x48 | 1 | name length | |
| 0x49 | 1 | second string length | |
| 0x4a | 1 | third string length | |
| 0x4b | 1 | fourth string length | at most 0x41 |
| 0x4c | | name, strings 2–4 | `probe-dl.bin`, `exttype/bin`, empty, empty **[O]** |

The second string is probably a type that the server derives from the file
extension, since Linux files have no type code **[I]**. Then it streams:

1. **414 Download File (D)** with the request's txid and the reply flag.
   The length is the data-fork bytes that remain. The data follows.
2. **414 Download File (D)** with the server's own txid and no reply flag.
   The length is the resource-fork bytes that remain (0 here). The data
   follows.

One 414 header covers the whole fork, even past the limits on
normal payloads: a 600000-byte file came as a single 414 of length 600000,
byte for byte, and a resume from 300000 as one of 300992 from 299008
(`captures/kdx-cli.pcapng` frames 133 and 190) **[O]**.
`kdx_proto::msg::file::FileData` holds the payload of 414 and 417 unchanged. Data is read in
chunks starting at 0x10000 and growing to 0x80000. With a
speed limit, chunks are multiples of 0x1000. At the end the server writes a
188 Finished File Download history record.

**419 Notify Speed Limit**
Initiator: Server. **[O]**

| Off | Size | Field |
|---|---|---|
| 0x00 | 4 | speed limit, in bytes per second |

It came on the session, not on the transfer connection, when a download
started under the total outgoing speed limit. **[O]**: `00 00 50 00`, the
20480 bytes per second set in the server settings, came right after the 413
(`captures/linux-server-settings.pcapng` frames 106, 110).
`kdx_proto::msg::transfer::NotifySpeedLimit` re-encodes it.

**415 Upload File**
Initiator: Client, on a transfer connection. Privilege: Upload (11).
**[S `FUN_08051750`, `FUN_08051dec`; O]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | data fork size | |
| 0x08 | 8 | resource fork size | |
| 0x10 | 7 | | |
| 0x17 | 1 | options | bit 0: plain (unencrypted) transfer |
| 0x18 | 0x2e | | |
| 0x46 | 2 | path length | of the destination folder path |
| 0x48 | 1 | name length | nonzero |
| 0x49 | 1 | second string length | |
| 0x4a | 1 | third string length | |
| 0x4b | 1 | fourth string length | |
| 0x4c | | name, strings 2–4, folder path | |

The server writes to a temporary name until the upload finishes. If the
name already exists, it fails with 0x30073. An earlier partial upload of
the same name is resumed. At the end, the server applies the second to
fourth strings to the file. They probably carry type and comment
information **[I]**.

The server answers with **416 Upload File (R)**, 24 bytes:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | bytes already on the server | from a partial upload; the server resumes at this value rounded down to 0x1000 |
| 0x08 | 8 | resource fork | always all ones; resource forks don't resume |
| 0x10 | 8 | 0 | |

The client then sends:

1. A **417 Upload File (D)** header. Its length must equal the data-fork
   bytes that remain, or the server fails with 0x40074. The data follows.
2. A **417 Upload File (D)** header for the resource fork, then its data.
   This build expects length 0 here **[S]**.

When both forks are complete, the server renames the file into place,
writes a 189 Finished File Upload history record, and sends an empty
**418 Upload File (C)** with its own txid **[O]**. **[O]**: 100 bytes uploaded
to `Uploads [UL]` arrived byte for byte, and so did 300000 bytes sent under
one 417 header (`captures/kdx-cli.pcapng` frame 244).
