# Tracker

A tracker lists servers in named groups. Servers register over UDP; clients
browse over TCP. Nothing here has been captured yet (issue
`.scratch/capture-gaps/issues/05-tracker.md`), so every fact is static.

Evidence tags as in `handshake.md`. Static sources, by Ghidra project
(`re/README.md`):

- `kdxtracker`: `KDXTrackerServ.lexe`, Tracker 1.110 Linux.
- `kdx`: `KDXServer.lexe`, Server 1.620 Linux.
- `kdxlnxclient`: `KDXClient.lexe`, Client 1.620 beta 5 Linux.

All multi-byte integers are big-endian.

## Ports

The tracker opens TCP and UDP on the same port, 10800 (0x2a30) by default
**[S tracker `FUN_0804c540`, default in `FUN_0804c7a8`]**.

When a tracker address has no `:port`, the server appends `:10800` before
sending a registration **[S server `FUN_08060e4c`]**. The client picks the
default port by connection type: 1 → 10700, 2 → 10600, 3 → 10800 (Tracker),
4 → 10900 **[S client `FUN_0804bac0`]**.

## Registration (UDP)

A server sends one datagram to each tracker in its settings
**[S server `FUN_08060e4c`]**. A tracker sends the same datagram to its own
upstream trackers, with flags bit 0 set **[S tracker `FUN_0804df70`]**.
Neither waits for a reply, and the tracker sends none.

### Encryption

The whole datagram is XOR'd word by word with an LCG stream
**[S server `FUN_0804a454`, tracker `FUN_0804a370` / `FUN_0804a3d0`]**:

```
s = 0xa5a16c4a
for each 32-bit big-endian word w[i]:
    w[i] ^= s
    s = s * 0x41d28485 + 0x322b    (mod 2^32)
```

Only whole words are XOR'd. The sender pads the datagram to a multiple of 4.

### Layout

A 64-byte header, then the strings back to back with no length bytes of
their own, then zero padding to a multiple of 4.

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | magic | `"%KDX"` (0x254b4458) |
| 0x04 | 2 | version | 0x0010. The tracker drops anything else |
| 0x06 | 2 | flags | bit 0: the sender is a tracker. Server 1.620 sends 0; Tracker 1.110 sends 1 |
| 0x08 | 4 | checksum | see below |
| 0x0c | 12 | server ID | Server 1.620 fills it with three random words the first time it runs and keeps them in its settings **[S server `FUN_080535b4`]**. Tracker 1.110 ignores it |
| 0x18 | 4 | port | the sender's TCP port (Server 1.620: settings +0x258, default 10700) |
| 0x1c | 8 | 0 | |
| 0x24 | 2 | users | number of users logged in. The server keeps it XOR'd with 0x2ba9 in memory and sends it plain **[S server `FUN_080528c8`, `FUN_0804a930`]** |
| 0x26 | 13 | 0 | |
| 0x33 | 1 | len0 | 0 |
| 0x34 | 1 | len1 | length of a string from the tracker entry in the server's settings (entry +0x18; meaning not known). The tracker skips it |
| 0x35 | 1 | name_len | server name |
| 0x36 | 1 | desc_len | server description |
| 0x37 | 1 | len4 | 0 |
| 0x38 | 6 | group_len[6] | up to six group names, from the tracker entry |
| 0x3e | 2 | 0 | |
| 0x40 | | strings | in field order: len0 bytes, len1 bytes, name, description, len4 bytes, groups 0–5 |

### Checksum

CRC-32 with polynomial 0x04c11db7, MSB first (table `0x04c11db7` at entry 1),
initial value 0xdeadbeef, and no final XOR. The sender sets the checksum field
to 0xcc1ed3ca, takes the CRC over the padded datagram, then stores the result
there **[S server `FUN_08060e4c` / `FUN_0806fa28`, tracker `FUN_08067ee4`]**.

### What the tracker checks

The tracker decrypts the datagram and keeps it only if all of these hold
**[S tracker `FUN_0804c048`]**:

- the length is more than 0x40;
- the magic is `%KDX` and the version is 0x0010;
- the 13 length bytes at 0x33..0x3f are not all 0, and 0x40 plus their sum
  fits in the datagram;
- the checksum matches (computed over the received length).

### Tracker side

The tracker keys each server by its source IP and the port field
**[S tracker `FUN_0804b960`]**:

- A new key gets a new record, but only while the IP has fewer than 5 records
  **[S `FUN_0804b494`, `FUN_0804b604`]**.
- A known key whose name, description, and group names are unchanged gets its
  users, flags, and last-seen time updated.
- Otherwise the record is replaced. It keeps its first-seen time.

The name is cut to 64 bytes. The record is listed in each of its groups that
the tracker has configured; a group whose flag (+0x0c) is set only takes
records whose flags bit 0 is set. A record with no matching group goes into a
default group **[S `FUN_0804b604`]**.

Records not seen for more than 850 (0x352) are removed. The sweep runs from
the 700 handler, at most every 600 **[S `FUN_0804abf4`]**. Both are probably
seconds **[I]**.

## Browsing (TCP)

### Connection

The handshake and session layer are the same as for a server (`handshake.md`,
`session.md`), with the same challenge table **[S tracker `FUN_0804bd9c`]**.
The tracker sends version 0x30. Like the server, it accepts only service
`%KDX` with version exactly 0x10 (0x40071 for another service, 0x40072 /
0x40073 for a lower / higher version) **[S `FUN_0804bb4c`]**.

Login **[S `FUN_0804bb78`]**:

- An empty login checks the proof against the empty password's key (guest).
- Any other login must be the tracker's admin login with the admin password.
  It sets admin bit 0 at connection +0x2e, which 710, 712, and 720 require
  (else 0x10014).

After login the tracker sends 107 Server Greeting, if it has one set
**[S `FUN_0804bbf8`]**. Each IP may hold 5 connections, and a connection idle
for more than 1200 (0x4b0, probably seconds **[I]**) is closed **[S
`FUN_0804bd9c`, `FUN_0804bbf8`]**.
The receive cap is 0x10000 bytes **[S `FUN_0804bd9c`]**.

The tracker's dispatcher `FUN_0804b378` handles 115, 700, 702, 710, 712, and
720 **[S]**. The client's tracker connection `FUN_08069578` sends 700 as soon
as it logs in and handles 1, 107, 116, 701, 703, 711, 713, and 721
**[S client]**.

### 115 Get My Address

As on a server: the reply 116 holds the peer's address as a Pascal string
**[S `FUN_0804b378`]**.

### 700 Get Tracker Group List

No payload. Reply 701 **[S `FUN_0804abf4`]**:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | count | |
| 0x04 | | groups | `count` group records |

Group record, padded to a multiple of 4:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 2 | servers | number of servers in the group |
| 0x02 | 4 | | not set by the tracker; the client ignores it **[S client `FUN_08068bcc`]** |
| 0x06 | 1 | name_len | |
| 0x07 | 1 | desc_len | |
| 0x08 | | name, description | |

### 702 Get Tracker Server List

**[S `FUN_0804adcc`; sender in client `FUN_08069ac4`]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 2 | 0 | |
| 0x02 | 1 | name_len | |
| 0x03 | 1 | 0 | |
| 0x04 | | group name | |

An unknown group gives 0x1000c. Reply 703, with an empty payload for an empty
group:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | count | the client rejects more than 0xffff |
| 0x04 | | servers | `count` server records |

Server record, padded to a multiple of 4 **[S tracker `FUN_0804adcc`, client
`FUN_08068da0`]**:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | first seen | u64 time; the client shows it as a date |
| 0x08 | 6 | 0 | |
| 0x0e | 2 | flags | from the registration |
| 0x10 | 2 | users | |
| 0x12 | 6 | 0 | |
| 0x18 | 1 | addr_len | 6 |
| 0x19 | 1 | name_len | |
| 0x1a | 1 | desc_len | |
| 0x1b | 1 | extra_len | 0 |
| 0x1c | | address | IPv4 address (4 bytes) and TCP port (2 bytes) |
| | | name, description, extra | |

### 710 Get TrackerServ Settings

No payload. Reply 711, the tracker's settings in its file format without the
secrets **[S `FUN_0804cb90`, `FUN_0804cd88`]**:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | magic | `"%KDXTSTG"` |
| 0x08 | 2 | format | 1 |
| 0x0c | 8 | section 0 | offset (0x78) and size: general settings |
| 0x14 | 8 | section 1 | offset and size: upstream trackers |
| 0x1c | 8 | section 2 | offset and size: groups |
| 0x2c | 8 | section 4 | offset and size: an opaque blob (tracker +0x224) |
| 0x34 | 8 | section 3 | offset and size: admin |

General settings at 0x78: four u16 limits at 0x78..0x7f (file only), flags
at 0x98 (bit 0: register with upstream trackers; bit 1 not known), the port
at 0x9c, the server ID at 0xa4 (file only), and the name and description
lengths at 0xbe and 0xbf with the strings from 0xc4.

An upstream tracker entry has 12 raw bytes at +0x08, 12 string lengths at
+0x14, and the strings from +0x20. A group has two u16 values at +0x10 and
+0x12, the name and description lengths at +0x18 and +0x19, and the strings
from +0x1c. The admin section has the admin login length at +0x90 and the
login from +0x94; the file adds the 32-byte admin key at +0x08.

### 712 Set TrackerServ Settings

The payload is parsed by `FUN_0804d63c`, probably the same format as 711
**[I]**. The tracker saves `KDXTrackerServ.stg`, replies 713 with no payload,
restarts its upstream registration, and logs `TrackerServ settings changed
remotely` **[S `FUN_0804cc10`]**. The file is XOR'd with a second LCG (seed
0xacb8fa74, multiplier 0x41a609fd, increment 0x1379); that is storage, not
protocol.

### 720 Get TrackerServ Statistics

No payload. Reply 721, 0x38 bytes **[S `FUN_0804b180`]**:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | start time | u64 time |
| 0x08 | 8 | connections | TCP connections accepted (`DAT_08104b00`) |
| 0x10 | 8 | registrations | UDP datagrams received (`DAT_08104b04`) |
| 0x18 | 8 | 0 | |
| 0x20 | 4 | uptime | hours |
| 0x24 | 4 | | tracker +0x1c; meaning not known |
| 0x28 | 4 | servers | records listed |
| 0x2c | 12 | 0 | |

## Open questions

- Capture all of the above (issue 05).
- The string at registration offset 0x34 (server tracker entry +0x18).
- The 711 fields marked "not known", and whether 712 uses the 711 format.
- Connection type 4 (port 10900) in the client.
