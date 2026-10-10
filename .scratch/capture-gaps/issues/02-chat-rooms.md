# Chat rooms

Status: done
Scenario: `linux-chat-rooms`
Targets: 201, 213, 217, 218 (GUI), 220, 221, 222, 223

Setup (agent): server detached; `kdx listen` as admin in Public Chat, outside
the chats that get deleted.

1. Connect as admin.
2. Chat button. Right-click in the Chat List, choose the command for a new
   chat. Name `lab-room`, Description `lab`, leave Access as it is, Save
   (217).
3. Right-click `lab-room`, Edit Chat (218/219). Change Description to
   `lab edited`, Save (220).
4. Edit `lab-room` again, tick Voice Chat, Save. Double-click `lab-room` to
   join it (222/223). Close the chat window.
5. Right-click `lab-room`, Delete Chat (201; the peer outside gets no 221).
6. Create `lab-chat` / `lab chat` with Voice Chat ticked. The agent reads its
   chat ID from the 215 and starts a second peer with `listen --chat <id>`.
   Join `lab-chat`, then Delete Chat with its window open (221 to both
   members).
7. Edit Chat on Public Chat, then Cancel (219 with all-class masks).
8. Account Classes → Administrator: tick Join Chat On Connect, Save, and
   connect again (213 after 106, then 202 for chat 1). Untick it afterwards.

## Comments

2026-10-10: Captured in `captures/linux-chat-rooms.pcapng`; the steps and
frames are in `captures/linux-chat-rooms.md`. Every target appeared. The
first recording corrected the original steps, which are rewritten above. 221
goes only to members of the deleted chat, so a second peer has to sit inside
it. 213 comes from the class privilege Join Chat On Connect (36), not from a
chat option, and names no chat. 223 was always empty, even with a CLI peer in
the voice chat; issue 07 may show an entry. The capture was then re-recorded
with the corrected steps, so it holds no detours.
