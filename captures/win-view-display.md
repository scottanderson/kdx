# Scenario: win-view-display

- Date: 2026-10-13T03:23Z
- Server: KDXServer 1.600 Windows under Wine (sha256 bc121d47c7f5e408)
- Client: linux 1.620 beta 5, plus the `kdx` CLI for the checks in step 2
- Server state: fresh; first-run dialog filled in with admin / lab-only-pw!1,
  guest access allowed

## Steps

1. `lab/kdx-lab server win -d`; the first-run "creating a new Accounts
   database" dialog was filled in with xdotool and saved.
2. `kdx who --login admin` (lists `kdx-rs`), `kdx chats` as guest (Public
   Chat, Voice Chat), and `kdx who --login admin` with a wrong password
   (handshake rejected with `Other(1)`). The setup dialog stayed on screen;
   saving it again made the server show "There was an error while trying
   to listen on the port", but the first listener kept running. One more
   `kdx who --login admin` (port 48116) confirmed it.
3. The GUI client connected to 127.0.0.1 as admin (port 52144, user 5).
4. Commands → 127.0.0.1 → View/Control Display. The client opened a
   transfer connection (port 36564) and sent 900 (frame 105).
5. Moved the mouse over the view, clicked once (904 kind 1 and 2, frames
   308, 310), and typed `abc` (905, frames 312–322).
6. Closed the View/Control Display window (FIN at frame 420).

## Observations

- No 901 went on the wire; closing the window closed the transfer
  connection.
- 902 flags were 1: admin may control the display.
- Wine's screen is black, 3840×2160. The only 903 (frame 108) is the full
  first frame: 253 runs of 0x7fff black pixels and one of 0x10fd. The
  screen never changed, so no later 903 came, even after the click and
  keys.
- 904 sent button 1 for down and up, 0 for moves; the field at 0x08 was
  always 0.
- 905 key down carries the character at 0x04 (0x61) and a key code at
  0x06 (0x04 for `a`); key up is all zero but the kind.
