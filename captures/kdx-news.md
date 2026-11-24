# Scenario: kdx-news

- Date: 2026-10-08T23:41Z to 23:46Z
- Server: Server 1.620 for Linux, sha256
  `ad6fba4bdd01dc237abde2b76054e3bbbb43c6d0c79a28d4e0de3b23e546f943`
- Client: none
- Tools: `kdx`. Steps 1 to 8 ran an earlier build, as step 9 says.
- Server state: the server from first-connect, with admin / `lab-only-pw!1`,
  guest access on, and only the default "Welcome" message in group 1
- Decode: `kdx-dissect captures/kdx-news.pcapng --password 'lab-only-pw!1'`

## Steps

Each step is one run of `lab/kdx-lab run target/debug/kdx --login admin
--password 'lab-only-pw!1'` with the step's command, as nick `kdx-rs`. A guest
step runs `lab/kdx-lab run target/debug/kdx --nick guest-rs` instead. A `listen`
run sat in Public Chat during steps 1–2 (nick `listener`) and 9–12 (nick
`listener3`).

1. `post 1 'Hello from kdx-rs'` with the body on standard input. This build
   waited for a reply that never came and was killed; the post was stored.
2. `news`, `news 1`, then a second listener (`listener2`) for steps 3–8.
3. `post 1 'Second post' 'One line, given as an argument.'` (hung the same
   way, killed by a timeout).
4. `news 1 2`.
5. `post 9 'No such group' x`: 1 Status 0x1000c.
6. Guest: `post 1 'Guest post' 'from guest'`: 1 Status 0x10014.
7. Guest: `news 1`, then `delete-news 1 2`: 1 Status 0x10014.
8. `delete-news 1 3`, `delete-news 1 2`, `delete-news 1 2` again (all with the
   old build, so each hung after the request), `news`.
9. With the rebuilt CLI, which follows each post or delete with 115 Get My
   Address: `post 1 'Third post'` from standard input.
10. `news 1`.
11. `delete-news 1 4`.
12. `delete-news 1 4` again, `news`.

The group ended as it started, with only message 1; the server's next
message ID is now 5.

## Observations

- 611 Post News Message and 612 Delete News Message have no reply on
  success, and deleting a missing message has no reply at all.
- 613 News Message Posted reached the poster and the listener; 614 News
  Message Deleted reached everyone, as notifications.
- 608 News Messages permissions were 7 for admin and 4 for guest.
