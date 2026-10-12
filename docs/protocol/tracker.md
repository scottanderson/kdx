# Tracker

A tracker lists servers in named groups. Servers register over UDP; clients
browse over TCP. `captures/linux-tracker.pcapng` has Server 1.620
registering with Tracker 1.110 and Client 1.620 beta 5 browsing and
administering it, all Linux; **[O]** tags below cite its frames.

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
default port by connection type: 1 → 10700 (Server), 2 → 10600 (Client,
DCC), 3 → 10800 (Tracker), 4 → 10900 (GConsole, a Haxial remote console;
see [dcc.md](dcc.md#gconsole)) **[S client `FUN_0804bac0`]**.

## Registration (UDP)

A server sends one datagram to each tracker in its settings
**[S server `FUN_08060e4c`]**. It registers when its settings are saved or it
starts, then every 400000 ms while its Register with Trackers flag (settings
+0x27c) is on **[S server `FUN_0805f638`, `FUN_08061230`; O frames 23, 78,
146, and 239, 400 s apart, and 255 right after the 172 at 252]**. A tracker
sends
the same datagram to its own upstream trackers, with flags bit 0 set **[S
tracker `FUN_0804df70`]**. Neither waits for a reply, and the tracker sends
none.

### Encryption

The whole datagram is XOR'd word by word with an LCG stream
**[S server `FUN_0804a454`, tracker `FUN_0804a370` / `FUN_0804a3d0`; O
frame 23]**:

```
s = 0xa5a16c4a
for each 32-bit big-endian word w[i]:
    w[i] ^= s
    s = s * 0x41d28485 + 0x322b    (mod 2^32)
```

Only whole words are XOR'd. The sender pads the datagram to a multiple of 4.

Verified: `kdx-proto` decodes registrations 23, 78, and 255 and re-encodes
them byte for byte (`crates/kdx-proto/tests/linux_tracker.rs`) **[O]**.

### Layout

A 64-byte header, then the strings back to back with no length bytes of
their own, then padding to a multiple of 4. Frame 23 is 92 bytes: port
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
| 0x18 | 4 | port | the sender's TCP port (Server 1.620: settings +0x258, default 10700) |
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
| 0x40 | | strings | in field order: len0 bytes, len1 bytes, name, description, len4 bytes, groups 0–5 |

### Checksum

CRC-32 with polynomial 0x04c11db7, MSB first (table `0x04c11db7` at entry 1),
initial value 0xdeadbeef, and no final XOR. The sender sets the checksum field
to the bytes `ca d3 1e cc` (0xcc1ed3ca stored little-endian), takes the CRC
over the padded datagram, then stores the result there big-endian **[S server
`FUN_08060e4c` / `FUN_0806fa28`, tracker `FUN_08067ee4`; O frames 23, 78, and
255, verified by `kdx-proto`]**. With the placeholder stored big-endian, the
checksum of frame 23 doesn't match.

### What the tracker checks

The server's tracker entry is edited in the client's Edit Tracker dialog:
Name, Comments, Address, Login, Password, and up to six Groups **[S client
`FUN_0808b290`, `FUN_0808b4d4`]**. The entry holds Name, Comments, Address,
Login, a 32-byte password key, an unused string, and the groups **[S server
settings]**. The registration sends only the Login and the groups; no
password or key goes on the wire.

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
- Otherwise the record is rebuilt from the datagram, keeping its first-seen
  time **[S `FUN_0804b960`]**.
- A registration with an empty name makes no record, and removes a known one
  **[S `FUN_0804b604` returns none]**.

The name is cut to 64 bytes; the description is kept whole. The record is
listed in each of its groups that the tracker has configured, matched byte for
byte **[S `FUN_0806774c`]**; a group whose flag (+0x0c) is set only takes
records whose flags bit 0 is set. A record with no matching group goes into a
default group **[S `FUN_0804b604`]**, which is General: `Lab, Lab2` was listed
there (frame 37) **[O]**. Once the tracker had a group `Lab`, the
registration with group `Lab` (255) was listed in it (257) **[O]**.

A new group list from 712 drops every record **[S `FUN_0804ab2c`; O frame 244,
an empty General right after the group edit at 240]**. So 257 lists the
arrival time of 255, not of 23 **[O]**.

Records not seen for more than 850 (0x352) are removed. The sweep runs from
the 700 handler, at most every 600 **[S `FUN_0804abf4`]**. Both are probably
seconds **[I]**.

Verified: `kdx-tracker` replays the requests of the `linux-tracker` capture and
its replies match Tracker 1.110's byte for byte, but for the first-seen times:
701 at 34, 234, and 247; 703 at 37, 58, 244, 250, and 257; 711 at 80 and 237;
713 at 83 and 241; and the Status at 61 (`crates/kdx-tracker/src/tests.rs`)
**[O]**. `captures/kdx-tracker.pcapng` has Server 1.620 registering with
`kdx-tracker` (frame 16) and the 703 that lists it (frame 34) **[O]**.

## Browsing (TCP)

### Connection

The handshake and session layer are the same as for a server (`handshake.md`,
`session.md`), with the same challenge table **[S tracker `FUN_0804bd9c`]**.
The tracker sends version 0x30. Like the server, it accepts only service
`%KDX` with version exactly 0x10 (0x40071 for another service, 0x40072 /
0x40073 for a lower / higher version) **[S `FUN_0804bb4c`]**.

Login **[S `FUN_0804bb78`]**:

- An empty login checks the proof against the empty password's key (guest).
- Any other login must be the tracker's admin login, compared ignoring case
  (`FUN_0808244c` with its case flag set), with the admin password. It sets
  admin bit 0 at connection +0x2e, which 710, 712, and 720 require (else
  0x10014; **[O]** frames 61 and 164 for a guest's 710 and 720). A wrong
  password fails the proof with status 1 (frame 104) **[O]**, in the error
  form of `handshake.md`: `auth` is zero but for its key word.

After login the tracker sends 107 Server Greeting, if it has one set
**[S `FUN_0804bbf8`; O frame 117, before the client's 700]**. Each IP may
hold 5 connections; the tracker closes a sixth at once, but counts it in
720's connection counter. A connection idle for more than 1200 (0x4b0,
probably seconds **[I]**) is closed **[S `FUN_0804bd9c`, `FUN_0804bbf8`]**.
The receive cap is 0x10000 bytes **[S `FUN_0804bd9c`]**.

The tracker's dispatcher `FUN_0804b378` handles 115, 700, 702, 710, 712, and
720, and ignores any other type without a reply **[S]**. The client's tracker
connection `FUN_08069578` sends 700 as soon as it logs in and handles 1, 107,
116, 701, 703, 711, 713, and 721 **[S client; O frames 33, 75]**.

### 1 Status

The tracker's error Status is 8 bytes: the error code, then the request type
as a u32, like the first 8 bytes of the server's 24-byte Status
(`KDXProtocol.md`) **[O frames 61, 164; `kdx-cli-tracker` 39, 53]**:

```
00 01 00 14  00 00 02 c6    0x10014 for 710
```

Verified: `kdx-proto` decodes frame 61 as `Status::TrackerError` and
re-encodes it byte for byte **[O]**.

### 115 Get My Address

As on a server: the reply 116 holds the peer's address as a Pascal string
**[S `FUN_0804b378`]**.

### 700 Get Tracker Group List

No payload. Reply 701 **[S `FUN_0804abf4`; O frame 34]**. A fresh Tracker
1.110 has eight groups: Business, Chat, Education, Games, General,
Macintosh, Trackers, and Windows, none with a description **[O]**.

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

**[S `FUN_0804adcc`; sender in client `FUN_08069ac4`; O frames 36/37]**

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 2 | 0 | |
| 0x02 | 1 | name_len | |
| 0x03 | 1 | 0 | |
| 0x04 | | group name | |

A payload shorter than 4 bytes plus the name gives 0x10002, and an unknown
group 0x1000c **[O `kdx-cli-tracker` frame 39]**; group names match byte for
byte **[S]**. Reply 703, with an empty payload for an empty group:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 4 | count | the client rejects more than 0xffff |
| 0x04 | | servers | `count` server records |

Server record, padded to a multiple of 4 **[S tracker `FUN_0804adcc`, client
`FUN_08068da0`]**:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | first seen | u64 milliseconds since 1900, as in `KDXProtocol.md`; frame 37 holds the arrival time of registration 23 **[O]** |
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

Verified: `kdx-proto` re-encodes 701, 702, and 703 from frames 34, 36, 37,
57, 58, 247, and 257 byte for byte **[O]**.

### 710 Get TrackerServ Settings

No payload. Reply 711, the tracker's settings in its file format without the
secrets **[S `FUN_0804cb90`, `FUN_0804cd88`; O frame 80]**:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | magic | `"%KDXTSTG"` |
| 0x08 | 2 | format | 1 |
| 0x0c | 8 | section 0 | offset (0x78) and size: general settings |
| 0x14 | 8 | section 1 | offset and size: upstream trackers |
| 0x1c | 8 | section 2 | offset and size: groups |
| 0x24 | 8 | section 3 | 0 |
| 0x2c | 8 | section 4 | offset and size: the greeting (tracker +0x224) |
| 0x34 | 8 | section 5 | offset and size: admin |
| 0x3c | 0x30 | sections 6–11 | 0 |
| 0x6c | 1 | present | general section |
| 0x6d | 1 | present | upstream trackers |
| 0x6e | 1 | present | groups |
| 0x6f | 1 | present | section 3; 0 |
| 0x70 | 1 | present | greeting |
| 0x71 | 1 | present | admin |
| 0x72 | 6 | present | sections 6–11; 0 |

An absent section has offset and size 0. The others follow the header back to
back in the order general, upstream trackers, groups, admin, greeting; a
present but empty section has size 0 and the offset where it would start
**[O frames 80, 82, 237, 240]**. Verified: `kdx-proto` re-encodes all four
byte for byte, and each general, groups, and admin section from its fields
**[O]**.

The parser reads a section only when its present byte is exactly 1 **[S
`FUN_0804d63c`]**. 711 sets them all **[O frame 80]**. Beyond that:

- general: the size must be at least 0x4c and hold the strings;
- upstream trackers: the section replaces the whole list, even when empty;
- groups: the size must not be 0; the section replaces the whole list and
  drops every registration (`FUN_0804ab2c`);
- admin: the size must be at least 0x94 and hold the login; an empty login
  keeps the old one;
- greeting: a section that isn't present, or is shorter than 5 bytes, clears
  the greeting; a longer one is kept up to 0xffff bytes.

General settings at 0x78: four u16 limits at 0x78..0x7f (file only), flags
at 0x98, the port at 0x9c (a u32), the server ID at 0xa4 (file only; 12
bytes as in the registration **[I]**), and the name and description lengths
at 0xbe and 0xbf with the strings from 0xc4, padded to 4. Flags
bit 0 is Register with Trackers (tracker +0x1a4) and bit 1 Enable Greeting
(tracker +0x22c) **[S `FUN_0804d63c`]**. The flags are a u32: Enable Greeting
alone is `00 00 00 02` **[O frame 82]**. The tracker clears bit 0 when the
name is empty, or no upstream tracker has an address, which includes an
upstream section that isn't present. It keeps 63 bytes of the name and 255
of the description. A port of 0 or above 0xfffe means 10800. 711 sends the
limits and server ID as 0 **[O frame 80]**. A fresh tracker is named `New
TrackerServ` **[O frame 80]**.

The greeting section is a u32 0 and then the text. The tracker sends it as
107 Server Greeting after login when Enable Greeting is on and the section
is longer than 4 bytes **[S `FUN_0804bbf8`]**.

An upstream tracker entry has 12 raw bytes at +0x08, 12 string lengths at
+0x14, and the strings from +0x20 (name, comments, address, login, key,
unused, groups 0-5, as in the server's tracker entry). A group has two u16
values at +0x10 and +0x12, which nothing sets, so they are always 0; the
name and description lengths at +0x18 and +0x19; and the strings from +0x1c.
A group record is 0x1c bytes and the strings, padded to a multiple of 4 **[O
frame 80]**; the parser steps over records by the four lengths at +0x18..+0x1b
**[S]**. A group's flag (+0x0c in memory) is set when its name contains
`tracker` in any case (`FUN_0804aa30` calls the substring search
`FUN_0808274c` with its case flag set) **[S]**, so `Trackers` takes only
trackers. 711 doesn't send the flag: the `Trackers` group has 0 there **[O
frame 80]**. The tracker keeps its groups sorted by name, byte for byte, so a
new group `Lab` lands between General and Macintosh (frame 247) **[S; O]**.
After parsing, the tracker adds the groups `General` and `Trackers` back if
they are missing. The admin section has a flags byte at +0x07 (bit 0: a key
follows), the 32-byte admin key at +0x08, the admin login length at +0x90,
and the login from +0x94, padded to 4; 711 leaves the key out. The key is the
`AccountKey` of the password, as for a server account: frame 82 carries the
key of `lab-tracker-pw!2` **[O, verified]**.

The client's settings window shows Tracker Name, Description, Port Number,
Admin Login, Change Admin Password (Password, Confirm), a Groups panel
(Group Name, Description), a Trackers panel with Register with Trackers,
and a Greeting panel with Enable Greeting **[S client `FUN_0806b1b0`]**.

### 712 Set TrackerServ Settings

The payload is the 711 format, parsed by `FUN_0804d63c` **[S]**. The limits
and server ID are taken from the file only. A port of 0 or above 0xfffe
means 10800; a new port is applied at once (`FUN_0804ca60`).

Client 1.620 beta 5 builds 712 in `FUN_0806b68c` and sets the present bytes for
general, trackers, groups, and greeting, but not admin (0x71). It still
writes the admin section, with flags bit 0 and the new key when the password
field is filled. The tracker ignores that section, so the admin login and
password can't be changed remotely with this client **[S; O frame 82, and
the new password is refused at 104]**. The client fills the groups section
only when the group list has been edited (bit 12 at list +0x24); otherwise
the section has offset and size 0, and the tracker keeps its groups **[S; O
frames 82 and 240]**. The tracker saves `KDXTrackerServ.stg`, replies 713
with no payload, even to a blob it can't read, restarts its upstream
registration, and logs `TrackerServ settings changed remotely` **[S
`FUN_0804cc10`; O frame 83 and the tracker log]**. The file is XOR'd with a
second LCG (seed 0xacb8fa74, multiplier 0x41a609fd, increment 0x1379); that
is storage, not protocol.

### 720 Get TrackerServ Statistics

No payload. Reply 721, 0x38 bytes **[S `FUN_0804b180`; O frame 86, matching
the client's Tracker Stats window]**:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | start time | u64 milliseconds since 1900 |
| 0x08 | 8 | connection counter | TCP connections accepted (`DAT_08104b00`) |
| 0x10 | 8 | UDP packet counter | UDP datagrams received (`DAT_08104b04`) |
| 0x18 | 8 | 0 | |
| 0x20 | 4 | hours since start | |
| 0x24 | 4 | connections | TCP connections open now (tracker +0x1c) |
| 0x28 | 4 | servers | records listed |
| 0x2c | 12 | 0 | |

Verified: `kdx-proto` re-encodes frame 86 byte for byte **[O]**.

## Open questions

- Resolved: captured in `linux-tracker` (issue 05).
- Resolved: registration offset 0x34 is the tracker entry's Login.
- Resolved: 711 flags bit 1 is Enable Greeting, section 4 is the greeting,
  and 712 uses the 711 format.
- Resolved: connection type 4 is GConsole ([dcc.md](dcc.md#gconsole)).
- The units of the 850 and 600 sweep values and the 1200 idle limit.
