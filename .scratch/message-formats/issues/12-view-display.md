# View display: 900–905

Status: ready-for-agent
Type: research
Targets: 901, 902, 903, 904, 905

Docs: `messages/view-display.md`. Capture: `win-view-display` (Server
1.600 for Windows under Wine, Client 1.620 beta 5). Server 1.620 for Linux
can't capture its screen, so this area may use the Windows server for
captures.

## Gaps

- [ ] 901 Close View Display: the Linux client sends none; it closes the
  connection. <!-- [O] --> Check the Windows client 1.600 for a sender.
- [ ] 902 View Display Params: 0x04..0x0f, 0 in the capture.
- [ ] 903: only one full frame of a black screen has been seen, so only
  runs. The two-color codes, single pixels, partial rectangles, the
  unchanged-pixel marker 0x6c59, and later frames need a screen that
  changes. That needs a way to show a window on the Wine server's screen
  (a second Wine program in its prefix, through `kdx-lab`), then a `win-`
  capture.
- [ ] 904 View Display Mouse: Client 1.620 beta 5's modifier bits; a capture
  of buttons 2 and 3 and of kind 4 (wheel). Buttons 1-3 and the kinds are
  <!-- [S] --> from Server 1.600 for Windows, which ignores the wheel.
- [ ] 905 View Display Keyboard: Client 1.620 beta 5's modifier bits; a
  capture of kind 3 (repeat). The key codes are: <!-- [S] --> Client 1.620 beta
  5 sends USB HID usages (`FUN_0811667c`), Client 1.600 Haxial's own codes.
- [x] 904/905 inject events through the Windows server's handlers: read how
  it maps the modifiers, buttons, and key codes, to name them from both
  ends (`mouse_event` in `FUN_00420270`, `keybd_event` in `FUN_00420380`).
- [ ] The 0x40076 drop of a slow viewer: fine as; <!-- [S] --> no capture
      needed.

One `win-` capture can cover 903, 904, and 905 together: shift-click,
right-click, the wheel, holding a key, and a window moving on the server's
screen.

- [ ] Audit (01): no codec for 902, 903, 904, and 905
  (`win-view-display` 107, 108, 110, 312).

## Comments

2026-10-13: Read statically in the Windows server; see
`messages/view-display.md`, 904 and 905. The 903 encoder is read too, but its
two-color and bitmap codes still need a screen that changes
(`.scratch/capture-gaps/issues/12-view-display-content.md`).
