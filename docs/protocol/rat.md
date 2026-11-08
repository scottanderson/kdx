# RemoteAdminTool 1.080

For hex and field notation, see [Conventions](README.md#conventions).

RemoteAdminTool (RAT) was a separate Haxial program, sold in 2002 and 2003, that
controls a computer from another one. KDX later took over its features. The only
known copy is RAT 1.080 for Mac, in `KDX___Haxial_Bundle.sit`; see
`vendor/MANIFEST.md`. This page comes from its two programs, RATClient and
RATServer, read in the Ghidra projects `RATClient1080-Mac` and
`RATServer1080-Mac`. <!-- [S] --> No capture of RAT exists, so nothing here is
verified.

## Connection
<!-- [S client `FUN_10004670`, `FUN_10052c70`] -->

RAT uses the [0.9x handshake](handshake.md#version-09x) with sub-protocol
`%RAT` and sub-version 1. The client sends an empty login name, and its key is
the [`account_key`](handshake.md#account_keypassword) of a password, so the
server probably has one password and no accounts. After the proof, the session
is Twofish-256 ECB with the 16-byte header of 1.x and no header scramble; see
[framing.md](framing.md#across-versions).

## Message types
<!-- [S client `FUN_10002d50`, server `FUN_10001380`] -->

The names come from the client's table of message names, and the server logs the
same names for the requests. The last column gives the KDX 1.x message that does
the same work.

| Type | Name | Initiator | In KDX 1.x |
|---:|---|---|---|
| 50 | Get Time Info | Client | none |
| 51 | Time Info | Server, reply to 50 | none |
| 52 | OS Info | Server | none |
| 60 | Empty Trash | Client | [421 Empty Trash](messages/files.md#m421) |
| 70 | Restart Computer | Client | [174 Shutdown Server](messages/administration.md#m174), mode bit 1 |
| 71 | Shutdown Computer | Client | [174 Shutdown Server](messages/administration.md#m174), mode bit 2 |
| 100 | Get Process List | Client | [802 Get Process List](messages/processes.md#m802) |
| 101 | Process List | Server, reply to 100 | [803 Process List](messages/processes.md#m803) |
| 102 | Terminate Process | Client | [804 Exit/Terminate Process](messages/processes.md#m804), flags bit 0 |
| 103 | Exit Process | Client | [804 Exit/Terminate Process](messages/processes.md#m804), flags bit 1 |
| 104 | Restart Process | Client | [804 Exit/Terminate Process](messages/processes.md#m804), flags bit 2, probably |
| 200 | Get Launch List | Client | none |
| 201 | Launch List | Server, reply to 200 and 202 | none |
| 202 | Refresh Launch List | Client | none |
| 203 | Launch Item | Client | none |
| 204 | Launched Item | Server, reply to 203 | none |
| 300 | Get Volume List | Client | none |
| 301 | Volume List | Server, reply to 300 | none |
| 400 | View Display Start | Client | [900 Open View Display](messages/view-display.md#m900) |
| 401 | View Display Stop | Client | [901 Close View Display](messages/view-display.md#m901) |
| 402 | View Display Update | Server, reply to 400 and then each change | [903 View Display Update](messages/view-display.md#m903) |
| 403 | View Display Mouse | Client | [904 View Display Mouse](messages/view-display.md#m904) |
| 404 | View Display Keyboard | Client | [905 View Display Keyboard](messages/view-display.md#m905) |

The server's handler for each request: <!-- [S server `FUN_10000730`,
`FUN_100008a0`, `FUN_10000b90`, `FUN_10000d10`, `FUN_10001020`, `FUN_10001130`,
`FUN_10002c50`] -->

- 51 Time Info is 20 bytes. The client shows "Server Started At:", "Local
  Server Time:", and "Duration Of Time:", so it probably holds those three
  times.
- The server sends 52 OS Info without a request. It holds two u32 values and a
  string of at most 0x7f bytes, probably the system version.
- 102 to 104 carry the 8-byte Mac process serial number of the process. KDX
  combined them into the flags of [804 Exit/Terminate
  Process](messages/processes.md#m804), whose bit 2 has no known use on Linux.
- 101 Process List and 301 Volume List have entries of 0x3c bytes and two
  strings, padded to 4. [803 Process List](messages/processes.md#m803) has the
  same 0x3c-byte entries, with a third string.
- 203 Launch Item names an item, probably in the server's `Launch Items`
  folder, and 204 Launched Item echoes the request.

## View display
<!-- [S server `FUN_10002c50`, `FUN_10002d40`, `FUN_10002e50`, `FUN_10002720`,
`FUN_100024f0`] -->

KDX's view display is RAT's, almost unchanged:

- 400 View Display Start gives 0x10003 if the viewer is already open. It then
  sends a full frame as 402 View Display Update, and every 1500 ms after that,
  it sends a 402 if the screen changed.
- 402 has the 12-byte header of [903 View Display
  Update](messages/view-display.md#m903) (width, height, and the changed
  rectangle), then the same pixel stream: RGB555 words, 0x6c59 for an unchanged
  pixel, runs of 10 to 0x7fff pixels, and spans of up to 0xfff pixels in two
  colors, as a mask or a bitmap.
- 403 and 404 have the 12-byte layouts of [904 View Display
  Mouse](messages/view-display.md#m904) and [905 View Display
  Keyboard](messages/view-display.md#m905). Mouse kinds 1 to 4 become input
  events 9, 10, 13, and 14, and key kinds 1 to 3 become events 5, 6, and 7, as
  in KDX.

KDX added [902 View Display Params](messages/view-display.md#m902) before the
first frame, so its update moved from 402 to 903, and mouse and keyboard moved
from 403 and 404 to 904 and 905. RAT has no privileges, so it has no control
flag.
