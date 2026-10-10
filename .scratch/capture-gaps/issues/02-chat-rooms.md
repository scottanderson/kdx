# Chat rooms

Status: ready-for-human
Scenario: `linux-chat-rooms`
Targets: 201, 213, 217, 218 (GUI), 220, 221, 222, 223

Setup (agent): server detached; `kdx listen` as admin in Public Chat as the
peer that should see 221.

1. Connect as admin.
2. Chat button. Right-click in the Chat List, choose the command for a new
   chat. Name `lab-room`, Description `lab`, leave Access as it is, Save
   (217).
3. Right-click `lab-room`, Edit Chat (218/219). Change Description to
   `lab edited`, Save (220).
4. Edit `lab-room` again, tick Voice Chat, Save. Double-click `lab-room` to
   join it (expect 222/223 [I]). Close the chat window.
5. Right-click `lab-room`, Delete Chat (201; the peer gets 221).
6. In the Edit Chat window of Public Chat, look for an option that forces
   users to join (213 Must Join Chat). If there is one, tick it, Save,
   disconnect and reconnect (expect 213 after login [I]), then untick it.
