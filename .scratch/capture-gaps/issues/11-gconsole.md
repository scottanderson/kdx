# GConsole

Status: done
Scenario: `linux-gconsole`
Targets: 4000, 4001, 4200, 4201, and the client's login on TCP 10900

GConsole is connection type 4 in the client's Connect dialog, port 10900
(`gconsole.md`). The client is the display: the other end sends 4000
(open a window) and 4001 (draw records), and the client sends 4200 and 4201
back. Only the three clients contain the string "GConsole", and they only
play the display end: none sends a `%HGC` hello or 4000/4001.
<!-- [S `FUN_0804b824`] --> No Haxial program can drive the client, so the lab
needs a peer of our own.

What a capture can prove: the client's hello and login on a GConsole
connection, its 4200/4201 payloads, and whether it accepts and draws the
4000/4001 our peer sends. The 4000/4001 bytes themselves come from our
reading of the client, so they stay `[S]` or `[I]`.

Setup (agent):

1. Read the 4001 record kinds (2, 3, 4, 100–109) in `FUN_08095ac4`, and the
   login the connection makes (`FUN_08094b74` sets the state, then
   `FUN_08095c84` waits for "Logged in.").
2. Write a minimal peer that listens on `127.0.0.1:10900`, runs the server
   side of the handshake (as `kdx-tracker` does), accepts any login, then
   sends 4000 (width 0x200, height 0x180) and one 4001 with a record of
   each kind. Log every message it receives.
3. `kdx-lab up linux-gconsole`, start the peer, `client linux`.

Steps (operator):

1. Connect button, Type "GConsole", Address `127.0.0.1`, login `admin`,
   password `lab-only-pw!1`, Connect (hello and login; expect "Logged
   in. Opening GConsole...").
2. The GConsole window opens at 0x200 × 0x180 (4000) and shows the drawing
   (4001). Take a screenshot.
3. Click in the window, then drag (4200; kinds 1, 2, 3 from events 5, 6, 7).
   <!-- [S `FUN_0809607c`] -->
4. Type `abc`, then Shift+A (4201; kinds 5, 6, 10 from events 9, 10, 14).
   <!-- [S `FUN_08096188`] -->
5. Close the window.

## Comments

2026-10-13: Added at the operator's request. 4200 is 0x1068 and 4201 is
0x1069; each carries 16 bytes. <!-- [S `FUN_0809607c`, `FUN_08096188`] -->
Whether 4200 is mouse and 4201 keyboard is, <!-- [I] --> from the event kinds.

2026-10-13: Done in `captures/linux-gconsole.pcapng`, against `kdx-probe
--listen`. The client accepted the peer and drew every record kind. The
guess was backwards: 4200 is the keyboard and 4201 the mouse. Drags send no
moves, and kind 3 of 4200 never appeared, so step 3's "drag" gave 4201
press and release only. Layouts are in `gconsole.md` and
`kdx_proto::msg::gconsole`.

2026-10-13: Client 1.600 for Windows gives the modifier bits: 0x21 is Ctrl.
<!-- [S `FUN_00463870`] --> It sends Haxial key codes in 4200 where Client 1.620
beta 5 sends USB HID usages, so a GConsole peer sees different codes from
the two clients. <!-- [S] --> `gconsole.md` compares GConsole with view display.
