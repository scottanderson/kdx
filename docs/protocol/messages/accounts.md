# Account messages

Accounts, user classes, privileges, and passwords.

**505 Get Account Class List** → **506 Account Class List**
Initiator: Client. Any one of View Folder Access (27), Create/Modify Chats
(37), View Accounts (50), or Create/Modify Newsgroups (73), since every
class-list editor uses it **[I]**. **[S `FUN_0805572c`; O]**

Reply: a u32 count (12 classes in the lab), then one record per class,
padded to 4 with zeros. A record is the 503 Account Class Info record
without the 8 bytes at 0x28 **[O probe.pcapng, port 50770]**:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 0x10 | privileges | 11 bytes used (see Access Privileges) |
| 0x10 | 8 | time | |
| 0x18 | 0x10 | | 0 |
| 0x28 | 1 | class index | 0 to 11, in list order |
| 0x29 | 3 | | `ff 00 00` for Administrator, `00 00 ff` for User, else 0 |
| 0x2c | 1 | name length | |
| 0x30 | | name | |

**[O]** privilege bytes:

- Admin: `28 31 e8 5b f4 d0 3c 0d c3 e0 80`
- User: `28 30 00 03 f0 c0 04 08 03 20`
- Guest: `28 30 00 03 80 00 00 08 02`

**502 Get Account Class** → **503 Account Class Info**
Initiator: Client. Privilege: View Accounts (50). **[S `FUN_080552b8`; O]**

The request is a u8 class index (`AccountClassIndex`). The reply is the stored class record,
padded to 4:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 0x10 | privileges | 11 bytes used |
| 0x10 | 8 | time | |
| 0x18 | 0xd | | |
| 0x25 | 1 | flood protection | commands per 2 minutes; 0x50 (Relaxed) for Administrator, 0x28 (Strict) for Guest; see Flood ban **[S]**; 0x3c (Average) for User, which 504 sets to 0x14 (Very Strict) in `linux-accounts-folder-access` frame 43 **[O]** |
| 0x26 | 2 | | stored as given **[S]** |
| 0x28 | 8 | | |
| 0x30 | 1 | class index | 0, 2 |
| 0x31 | 1 | | 0xff for Administrator, 0 for Guest; probably the colour **[I]** |
| 0x34 | 1 | name length | |
| 0x35 | 1 | second string length | **[S]** |
| 0x36 | 1 | editor login length | set by the server on 504 **[S]** |
| 0x38 | | name, second string, editor login | |

The padding after the name is uninitialised (`@`, `.` in the lab) **[O]**;
`kdx-proto` keeps it on a round trip.

**509 Get Account** → **510 Account Info**
Initiator: Client. Privilege: View Accounts (50). **[S `FUN_08055e98`; O]**

The request is the account's login, with no length byte (`AccountLogin`, as
in 508 and 515). An unknown login
gets 1 Status 0x1000c (`captures/kdx-accounts.pcapng` frame 554). The reply
is the stored account record, with the key replaced by a flag. Its padding is
uninitialised (frame 590) **[O]**; `kdx-proto` keeps it on a round trip.

**Account record.** 507 Create Account, 510 Account Info, and 511 Set
Account share this layout **[S `FUN_08055928`, `FUN_08055ff4`; O]**:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 0x10 | privileges | values of the bits set in the mask; `80 00 00 00 00 00 02 00…` for admin, 0 for new accounts **[S `FUN_080551b0`]** |
| 0x10 | 0x10 | privilege mask | bits the account sets itself; the rest come from its class **[S `FUN_080551b0`]** |
| 0x20 | 0x20 | account key | see below |
| 0x40 | 8 | created | set by the server |
| 0x48 | 8 | modified | set by the server |
| 0x50 | 8 | last login | 0 if never |
| 0x58 | 0x1e | | 0 |
| 0x76 | 1 | class index | must name an existing class, else 0x10002 |
| 0x77 | 3 | | stored as given |
| 0x7a | 1 | login length | |
| 0x7b | 1 | name length | the server filters the name (`FUN_08081ffc`) |
| 0x7c | 1 | third string length | stored as given; 0 in the lab |
| 0x7d | 1 | editor length | see below |
| 0x80 | | login, name, third string, editor | |

The key field:

- 507 Create Account: the new account's key (see
  [handshake.md](../handshake.md)). The CLI sends the empty password's key for an
  account with no password.
- 511 Set Account: the new key. All zero keeps the current key.
- 510 Account Info: byte 0 is 1 if the account has a password, i.e. its key
  is not the empty password's key (`FUN_08066a90`), and 0 otherwise. The
  rest is zero (frames 253 and 590).

The editor field:

- In a request it is ignored by 507, and in 511 it names the account to
  change.
- The server stores the requester's login there. 510 shows `admin` for the
  accounts admin created (frame 253). The setup-created `admin` account shows
  an empty editor.

**507 Create Account**
Initiator: Client. Privilege: Create Accounts (51). **[S `FUN_08055928`; O]**

The request is an account record. The server sets the times and the
editor, then replies with 1 Status success 0x1fb (frame 235). Errors:

- 0x1000d if the login exists (frame 289);
- 0x10002 for an invalid login (`FUN_08057f88`) or class;
- 0x640005 when the account would hold a privilege the requester lacks
  (`FUN_080551b0`, below).

The privilege check (`FUN_080551b0`) **[S]**: the account's effective
privileges are its class's bits, with the bits in the mask taken from the
record instead. Bits 3 and 36 are ignored, and so is bit 23 if the requester
has bit 24. The request fails if any remaining bit is one the requester lacks.
A requester with bit 0 passes.

**511 Set Account**
Initiator: Client. Privilege: Modify Accounts (52). **[S `FUN_08055ff4`; O]**

The request is an account record whose editor field holds the current login.
If the login field differs, the server renames the account, failing with
0x1000d if the new login exists. It keeps the creation and last-login times,
sets the modified time, and replies with 1 Status success 0x1ff (frames
412–413). Users logged in to the account are updated in place with the new
login and privileges.

Errors:

- 0x1000c for an unknown account;
- 0x10014 if the account has Protected Account (54) set in both the mask
  and the values and the requester lacks privilege 0;
- the same checks as 507 Create Account otherwise.

**508 Delete Account**
Initiator: Client. Privilege: Modify Accounts (52). **[S `FUN_08055d50`; O]**

The request is the login, with no length byte. On success the server sends
**515 Account Deleted**, whose payload is the same login, to the requester
as a notification (its own txid, no reply flag). It sends no reply (frames
607–610).

Errors:

- 0x1000c for an unknown login (frame 630);
- 0x10014 without the privilege (frame 391);
- 0x10014 if the target has privilege 0 set in both its mask and its
  values;
- 0x10014 if the target has Protected Account (54) set in both and the
  requester lacks privilege 0.

**550 Change Own Password** → **551 Change Own Password Done**
Initiator: Client. Privilege: Change Own Password (53). **[S `FUN_08056a04`;
O]**

The request is the 32-byte key of the new password, not the password
(`kdx_proto::msg::admin::ChangeOwnPassword`). The
server stores it as the account's key and replies with an empty 551
(frames 323–324). The old password stops working at once.

**512 Get Account List** → **513 Account List**
Initiator: Client. Privilege: View Accounts (50). **[S `FUN_080566bc`; O]**

The request is empty. Reply: a u32 count, then one entry per account, each
padded to 4:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 8 | created | |
| 0x08 | 8 | last login | 0 if never |
| 0x10 | 0xb | | |
| 0x1b | 1 | | 1 for admin, 0 for guest |
| 0x1c | 4 | | |
| 0x20 | 1 | class index | 0 for admin, 2 for guest |
| 0x21 | 1 | | 0xff for admin; matches 503 Account Class Info offset 0x31 |
| 0x24 | 1 | login length | |
| 0x25 | 1 | name length | |
| 0x28 | | login, name | |

**504 Set Account Class**
Initiator: Client. Privilege: Modify Accounts (52). **[S `FUN_080553c8`; O]**

The request is a 503 Account Class Info record (at least 0x38 bytes) with
the class index at 0x30. Client 1.620 for Linux sends it unpadded, with the
time 0 and the second string and editor login empty (69 bytes for
Administrator, `captures/linux-catalog-volumes.pcapng` frames 40 and 59;
61 bytes for Guest, `captures/linux-accounts-news.pcapng` frames 20 and 30).
The server filters the name (`FUN_08081ffc`), writes the requester's login
as the editor, sets the time at 0x10, and saves the class. Privileges the
requester lacks can't be granted (`FUN_080551b0`; failure 1 Status
0x640005). Success is 1 Status 0x1f8 (frames 43 and 62); the history record
0x1f8 carries the old class name. Logged-in users of that class get the new
privileges at once (`FUN_08054900`).

**514 Account Changed**
Server 1.620 never sends it and Client 1.620 beta 5 doesn't handle it
**[S]**.
