# UDP: tracker registration

Tags, hex, and field notation: see [Conventions](README.md#conventions).

A server tells a tracker it exists with one UDP datagram. UDP is a separate
protocol from the TCP connections in [framing.md](framing.md): the datagram
has no handshake, no message header and no reply, and a different cipher and
checksum. The tracker's TCP side, where clients browse, is described in
[tracker.md](tracker.md). Evidence tags as in [handshake.md](handshake.md); all
multi-byte integers are big-endian. **[O]** tags that give only a frame
number cite `captures/linux-tracker.pcapng`.

A server sends one datagram to each tracker in its settings **[S server
`FUN_08060e4c`]**. It registers when its settings are saved or it starts, then
every 400000 ms while its Register with Trackers flag (settings +0x27c) is on
(frames 23, 78, 146, and 239 are 400 s apart, and 255 comes right after the
[172 Set Server Settings](messages/administration.md#m172) at 252) **[S server
`FUN_0805f638`, `FUN_08061230`; O frames 23, 78, 146, 239, 255]**. A tracker
sends the same datagram to its own upstream trackers, with flags bit 0 set **[S
tracker `FUN_0804df70`]**. Neither waits for a reply, and the tracker sends
none.

## Encryption

The whole datagram is XOR'd word by word with an LCG stream **[S server
`FUN_0804a454`, tracker `FUN_0804a370`, tracker `FUN_0804a3d0`; O frame 23]**:

```
s = 0xa5a16c4a
for each 32-bit big-endian word w[i]:
    w[i] ^= s
    s = s * 0x41d28485 + 0x322b    (mod 2^32)
```

Only whole words are XOR'd. The sender pads the datagram to a multiple of 4.

Verified: `kdx-proto` decodes registrations 23, 78, and 255 and re-encodes
them byte for byte (`crates/kdx-proto/tests/linux_tracker.rs`) **[O]**.

## Layout

A 64-byte header, then the strings back to back with no length bytes of
their own, then padding to a multiple of 4. Frame 23 is 92 bytes: server port
0x29cc, users 1, the Login `trk-login`, the name `Lab Server`, and one group
`Lab, Lab2` **[O]**. The client's Groups field is one text field, sent as
group 0 as typed; Comments, Address, and the password are not sent **[O]**.
Server 1.620 doesn't clear the padding: frame 255 is 86 bytes of header and
strings, then `12 22` **[O]**.

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | magic | `"%KDX"` (0x254b4458) |
| 0x04 | 2 | version | 0x0010. The tracker drops anything else |
| 0x06 | 2 | flags | bit 0: the sender is a tracker. Server 1.620 sends 0; Tracker 1.110 sends 1 |
| 0x08 | 4 | checksum | see below |
| 0x0c | 12 | server ID | Server 1.620 fills it with three random words the first time it runs and keeps them in its settings **[S server `FUN_080535b4`]**. Tracker 1.110 ignores it |
| 0x18 | 4 | server port | the server's TCP port, which a tracker forwards unchanged (Server 1.620: settings +0x258, default 10700) |
| 0x1c | 8 | 0 | |
| 0x24 | 2 | users | number of users logged in. The server keeps it XOR'd with 0x2ba9 in memory and sends it plain **[S server `FUN_080528c8`, `FUN_0804a930`]** |
| 0x26 | 13 | 0 | |
| 0x33 | 1 | len0 | 0 |
| 0x34 | 1 | len1 | the Login of the tracker entry in the server's settings. The tracker skips it |
| 0x35 | 1 | name_len | server name |
| 0x36 | 1 | desc_len | server description |
| 0x37 | 1 | len4 | 0 |
| 0x38 | 6 | group_len[6] | up to six group names, from the tracker entry |
| 0x3e | 2 | 0 | |
| 0x40 | | strings | in field order: len0 bytes, len1 bytes, name, description, len4 bytes, groups 0-5 |

## Checksum

CRC-32 with polynomial 0x04c11db7, MSB first (table `0x04c11db7` at entry 1),
initial value 0xdeadbeef, and no final XOR. The sender sets the checksum field
to the bytes `ca d3 1e cc` (0xcc1ed3ca stored little-endian), takes the CRC over
the padded datagram, then stores the result there big-endian (verified by
`kdx-proto`) **[S server `FUN_08060e4c`, server `FUN_0806fa28`, tracker
`FUN_08067ee4`; O frames 23, 78, 255]**. With the placeholder stored big-endian,
the checksum of frame 23 doesn't match.

## What the tracker checks

The server's tracker entry is edited in the client's Edit Tracker dialog: Name,
Comments, Address, Login, Password, and up to six Groups **[S client
`FUN_0808b290`, `FUN_0808b4d4`]**. The entry holds Name, Comments, Address,
Login, a 32-byte password key, an unused string, and the groups, as the server's
settings store them **[S]**. The registration sends only the Login and the
groups; no password or key goes on the wire.

The tracker decrypts the datagram and keeps it only if all of these hold
**[S tracker `FUN_0804c048`]**:

- the length is more than 0x40;
- the magic is `%KDX` and the version is 0x0010;
- the 13 length bytes at 0x33..0x3f are not all 0, and 0x40 plus their sum
  fits in the datagram;
- the checksum matches (computed over the received length).

## Tracker side

The tracker keys each server by its source IP and the port field
**[S tracker `FUN_0804b960`]**:

- A new key gets a new record, but only while the IP has fewer than 5 records
  **[S `FUN_0804b494`, `FUN_0804b604`]**.
- A known key whose name, description, and group names are unchanged gets its
  users, flags, and last-seen time updated.
- Otherwise the record is rebuilt from the datagram, keeping its first-seen
  time **[S `FUN_0804b960`]**.
- A registration with an empty name makes no record, and removes a known one
  (`FUN_0804b604` returns none) **[S `FUN_0804b604`]**.

The name is cut to 64 bytes; the description is kept whole. The record is
listed in each of its groups that the tracker has configured, matched byte for
byte **[S `FUN_0806774c`]**; a group whose flag (+0x0c) is set only takes
records whose flags bit 0 is set. A record with no matching group goes into a
default group **[S `FUN_0804b604`]**, which is General: `Lab, Lab2` was listed
there (frame 37) **[O]**. Once the tracker had a group `Lab`, the
registration with group `Lab` (255) was listed in it (257) **[O]**.

A new group list from [712 Set TrackerServ Settings](messages/tracker.md#m712)
drops every record (frame 244 is an empty General right after the group edit at
240) **[S `FUN_0804ab2c`; O frame 244]**. So 257 lists the arrival time of 255,
not of 23 **[O]**.

Records not seen for more than 850 (0x352) are removed. The sweep runs from
the [700 Get Tracker Group List](messages/tracker.md#m700) handler, at most
every 600 **[S `FUN_0804abf4`]**. Both are probably seconds **[I]**.
