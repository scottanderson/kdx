# Files and transfers: 400–453

Status: ready-for-agent
Type: research
Targets: 401, 402, 404/406/408, 409/410, 411, 412–418, 420, 422/423, 450,
452/453

Docs: `KDXProtocol.md`, Files and Transfer connections.

## Gaps

- [ ] 401 File List entry: 0x18 (8 bytes), 0x24 (0xa bytes), 0x2e (0xffff
  for folders, `FileListEntry::unknown_2e`), 0x32, 0x34 (4 for the
  Dropbox, 5 for Uploads, 0 for a file, `unknown_34`), and 0x37. Include an
  alias in the check: `linux-files` frame 100 lists `upload2 alias.txt`
  right after the 403 at 95, with attributes 4 at 0x20.
- [ ] 410 File/Folder Info: 0x00 is 0x39, which is also the capability
  constant `FUN_08096780` returns; 0x04 is 0xbf (`FileInfo::unknown_00`,
  `unknown_04`); 0x20, 8 bytes; 0x28, 2 for a folder (`unknown_28`);
  0x2c, 0x11 bytes; 0x3f. Compare a file with a folder.
- [ ] 422 Volume Info: 0x24..0x25, 0x27..0x2b, 0x2c (0x88,
  `VolumeInfo::unknown_2c`), the icon at 0x30, 0x34..0x43, 0x4a..0x4d,
  0x4f (the new-name length in 423), 0x52..0x53.
- [ ] 402 Create Folder: 0x00, 0x13 bytes of 0. What the client writes.
- [ ] 411 Set File/Folder Info: 0x00, 0x28 bytes of 0; 0x2c; 0x30; 0x32,
  "other lengths". Flags bits 0x86 set attributes: which attributes, and
  where their values go. A Linux capture that sets attributes in the File
  Info window (Set File/Folder Attributes, 20) settles it. The only 411
  (`linux-files` frame 89) is a rename, flags 0x1.
- [ ] 412 Download File and 415 Upload File: 0x10, 7 bytes. The options bit
  0 (plain transfer) needs privilege bit 65, which the privilege table now
  names Disable Encryption; and bit 25, now Allow Simultaneous Downloads,
  is still marked **[I]** under 412. Update both, and capture a plain transfer
  on Linux if the client can ask for one. All 16 captured 412s and 415s
  have options 0.
- [ ] 413 Download File (R): 0x10 (8 bytes of 0), 0x28 (0xc bytes: 0, 0,
  1; `DownloadInfo::unknown_30` holds the 1), 0x34 (0x14 bytes of 0).
  Strings 2–4: the second is "probably a type the server derives from the
  extension" **[I]** (`exttype/bin`); the third and fourth are empty. Name
  them (`FileStrings::second`, `third`).
- [ ] 415 Upload File: 0x18, 0x2e bytes; strings 2–4 "probably carry type
  and comment information" **[I]**. Read what the Linux server does with them
  at the end of an upload.
- [ ] 416 Upload File (R): 0x10, 8 bytes of 0.
- [ ] Resource forks: the Linux server always sends size 0 and expects 0.
  Read whether the Windows server 1.600 can send a nonzero resource fork
  (only if the Linux code shows the field is never filled).
- [ ] 420 Launch File: the 8 bytes of 0 before the path.
- [ ] 450 Generate Catalog: flags besides bit 0.
- [ ] 452 File Search: bytes 0x00..0x53 are 0 from the client and ignored
  by the Linux server. Check the client's search window for options it
  could put there. If the Linux client never fills them, read the Windows
  server 1.600 handler.
- [ ] 453 File Search (R) entry: 0x18 is a 4-byte path length in the table
  (check its size), 0x1c..0x1f, 0x24..0x2d, 0x30..0x33, 0x34 (0x10 for a
  file), 0x35, 0x37 (`FileSearchEntry::unknown_2e`, `unknown_34`).
- [ ] 404 Delete File/Folder, 406 Rename File/Folder, 408 Copy File/Folder:
  no handler in Server 1.620, no sender in Client 1.620 beta 5. Check the
  Windows client 1.600 for a sender and the Windows server 1.600 for a
  handler; document the layout if one exists, and hand off a `win-`
  capture only if both do.
- [x] Audit (01): 410 File/Folder Info dropped byte 0x37: 01 in
  `linux-files` 180, 183, 186, 195, 217, and 02 in 204. Fixed:
  `FileInfo::unknown_2c` keeps 0x2c..0x3c.
- [ ] What is 410 byte 0x37?
- [x] Audit (01): 415 Upload File dropped 13 bytes in its unnamed 0x18
  field (`linux-files` 45, `linux-text-encoding` 90). Fixed:
  `UploadFile::unknown_18` and `unknown_30` keep 0x18..0x45.
- [ ] Name the 415 bytes: two u64 times at 0x18 and 0x20 that match the
  capture's date as milliseconds since 1900, and 01 at 0x33.
- [x] Audit (01): 414 Download File (D) and 417 Upload File (D) have no
  codec (`linux-files` 148, 47). Fixed: `FileData` keeps the fork bytes as is.
