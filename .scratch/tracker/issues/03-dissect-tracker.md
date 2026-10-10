# Dissect tracker traffic

Status: ready-for-agent
Blocked by: 01, 02

Teach `kdx-dissect` to:

- decode UDP datagrams to port 10800 as registrations (decrypt, check the
  CRC, print the fields);
- treat a TCP connection to 10800 as a tracker session and print 700–721
  with the new codecs.

Run it on the capture-gaps 05 capture and add that run to the checks in
`AGENTS.md` if it is fast.
