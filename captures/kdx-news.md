# Scenario: kdx-news

- Date: 2026-10-08T23:41Z to 23:46Z
- Server: KDXServer 1.620 Linux (sha256 ad6fba4bdd01dc23)
- Client: none; the `kdx` CLI (this repository) only, no GUI
- Server state: the first-connect server (admin / lab-only-pw!1, guest
  enabled), with only the default "Welcome" message in group 1

## Steps

Each step is one `kdx` run as admin (`--login admin`), nick `kdx-rs`, unless
it says guest (no login, nick `guest-rs`). A `listen` run sat in Public Chat
during steps 1–2 (nick `listener`) and 9–12 (nick `listener3`).

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
