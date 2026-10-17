# View display with a changing screen

Status: needs-triage
Scenario: `win-view-display-content`
Targets: 903 two-color and bitmap codes, a later 903, 904 and 905 reaching
the server's screen

`captures/win-view-display` (issue 06) only exercised 903 run codes: Wine
gave Server 1.600 a black screen that never changed. The rest of View display
in `docs/protocol/messages/view-display.md` is: <!-- [S] -->

- the two-color codes (inline mask, 4-12 pixels) and the bitmap code (13 or
  more pixels);
- a later 903 that marks unchanged pixels 0x6c59 and gives the changed
  rectangle;
- a 903 after a screen-size change, whose bottom is 0xffff;
- how the server plays back 904 (`mouse_event`) and 905 (`keybd_event`).

Client 1.620 beta 5 sends USB HID usages as 905 key codes, but Server 1.600
for Windows expects Haxial key codes (`A` 0x41), so it probably turns the
Linux client's `a` (4) into virtual key 0. <!-- [I] --> Client 1.600 for Windows
sends the codes the server expects. <!-- [S] -->

Wine's screen is black because the server grabs the desktop window with
`BitBlt`. <!-- [S `FUN_0041ffd0`] --> A Wine virtual desktop (`wine explorer
/desktop=kdx,1024x768`) with a program such as Notepad open on it may give
the grab real pixels; <!-- [I] --> `kdx-lab server win` would need an option for
it.

Steps:

1. Start the server in a virtual desktop with Notepad open.
2. Client 1.620 beta 5: connect as admin, View/Control Display (900, 902,
   the first 903 with two-color and bitmap codes).
3. Click in Notepad's text area (904; the cursor should move), type `abc`
   (905; check whether Notepad shows it), wait for the next 903.
4. Resize the virtual desktop if Wine allows it (a 903 with bottom 0xffff).
5. Repeat 2-3 with Client 1.600 for Windows under Wine, to compare its 905
   key codes and whether its keys arrive.

## Comments
