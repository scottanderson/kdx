# Server settings, greeting, icon, identity, history

Status: done
Scenario: `linux-server-settings`
Targets: 102, 106, 107, 110, 111, 112, 113, 114, 172, 173, 183, 419

Setup (agent): server detached; `kdx listen` as admin in Public Chat as the
peer that watches for 102 and 111; a 2 MB file `speed.bin` in
`Bases/Default/`.

1. Connect: Connect button, Type "Server", Address `127.0.0.1`, Login
   `admin`, Password `lab-only-pw!1`, Connect.
2. Commands → 127.0.0.1 → Server History. Leave the window open for the
   whole session; every later step should push a 183.
3. Commands → 127.0.0.1 → Server Settings (170/171). On General Settings:
   set Server Name to `Lab Server`; copy your own icon from Settings →
   Identity and paste it into the Server Icon box; tick Total Outgoing Speed
   Limit and enter 20 KB/sec. On Greeting: type `Welcome to the lab.` and
   tick the enable option. Click Save (172/173).
4. Commands → 127.0.0.1 → Show Greeting (expect 112 then 107).
5. Commands → 127.0.0.1 → Get Server Info (113/114).
6. Commands → 127.0.0.1 → Disconnect, then connect again as in step 1
   (expect 105, 106 and 107 after login).
7. Files button, right-click `speed.bin`, Download (expect 419 **[I]**).
8. Settings button, Identity panel: change the name to `lab-linux`, click
   Save (100; the peer gets 102).
9. Chat button, join Public Chat, type `/away`, Enter, then `/back`, Enter
   (expect 110/111 **[I]**).
10. Undo step 3's speed limit: Server Settings, untick Total Outgoing Speed
    Limit, Save.

## Comments

2026-10-10: Captured in `captures/linux-server-settings.pcapng`; the steps
and frames are in `captures/linux-server-settings.md`. Every target appeared.
The greeting stayed "(Greeting disabled.)" until the enable option on the
Greeting panel was ticked.
