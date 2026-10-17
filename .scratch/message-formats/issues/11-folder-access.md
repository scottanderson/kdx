# Folder access: 1100–1107

Status: ready-for-agent
Type: research
Targets: 1106/1107, the folder access record (1100, 1104, 1105)

Docs: `messages/folder-access.md`. Captures: `linux-files`,
`linux-accounts-folder-access`, `linux-folder-access-guest`.

## Gaps

- [ ] 1106 request: the optional u32, of which only the top byte's bit 0 is
  read. <!-- [S] --> Confirm no other bit matters.
- [x] 1107 entry: 0x00 is named flags in the doc <!-- [S] --> but
  `FolderAccessItem::unknown_00` in code, and 0x04 is the icon in the doc
  but `item_id` in code; rename them. Fixed: `flags` and `icon`.
- [ ] 1107 entry: 0x08 (8 bytes, "not written by the server"); 0x10
  (`unknown_10`); the second string (`second`).
- [ ] Record 0x00, a u32 (`FolderAccess::unknown_00`).
- [ ] Record 0x0c: the first class rule, 0xfffe0000 in both lab items, is
  "probably Administrator's"; <!-- [I] --> 0x8c is "probably the owner's rule"
  <!-- [I] --> (`FolderAccess::rule`). Confirm both from the access check
  `FUN_08057e5c`.
- [ ] The low u16 of each rule, and permission bit 15.
- [ ] Record 0x90 (0x1d bytes of 0), 0xb0 (`unknown_b0`, perhaps the
  second string's length), <!-- [I] --> 0xb2 (6 bytes).
- [x] Audit (01): 1100 and 1104 lost byte 0xc1, 01
  (`linux-folder-access-guest` 21, 28). Fixed: `FolderAccess::padding`
  keeps the bytes after the strings.
- [ ] Is that byte uninitialized padding, as the client sends it?
- [x] Audit (01): no codec for 1101, 1102, 1103 (a name), or 1106 with its
  optional u32 (`linux-accounts-folder-access` 70, 71, 60; `linux-files`
  23). Fixed: `FolderAccessName` and `GetFolderAccessList`, which keeps
  the bytes after the u32, or a payload under 4 bytes, in `rest`.
