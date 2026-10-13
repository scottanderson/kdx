# View display: 900–905

Status: ready-for-agent
Type: research
Targets: 901, 902, 903, 904, 905

Docs: `KDXProtocol.md`, View display. Capture: `win-view-display` (Server
1.600 for Windows under Wine, Client 1.620 beta 5). Server 1.620 for Linux
can't capture its screen, so this area may use the Windows server for
captures.

## Gaps

- [ ] 901 Close View Display: the Linux client sends none; it closes the
  connection [O]. Check the Windows client 1.600 for a sender.
- [ ] 902 View Display Params: 0x04..0x0f, 0 in the capture.
- [ ] 903: only one full frame of a black screen has been seen, so only
  runs. The two-colour codes, single pixels, partial rectangles, the
  unchanged-pixel marker 0x6c59, and later frames need a screen that
  changes. That needs a way to show a window on the Wine server's screen
  (a second Wine program in its prefix, through `kdx-lab`), then a `win-`
  capture.
- [ ] 904 View Display Mouse: modifiers at 0x00 [I]; 0x08, "wheel or click
  count" [I], 0 so far; kind 4 (wheel) [I]; buttons other than 1.
- [ ] 905 View Display Keyboard: modifiers at 0x00 [I]; kind 3 (repeat)
  [I]; the key code table (`a` = 4, `b` = 5, `c` = 6 look like USB HID
  usages [I]) from the client's `FUN_0806f1f4`.
- [ ] 904/905 inject events through the Windows server's handlers: read how
  it maps the modifiers, buttons, and key codes, to name them from both
  ends.
- [ ] The 0x40076 drop of a slow viewer: fine as [S]; no capture needed.

One `win-` capture can cover 903, 904, and 905 together: shift-click,
right-click, the wheel, holding a key, and a window moving on the server's
screen.
