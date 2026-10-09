# Scenario: linux-accounts-news

- Date: 2026-10-09T20:04Z
- Server: KDXServer 1.620 Linux (sha256 ad6fba4bdd01dc23)
- Client: linux 1.620 beta 5
- Server state: the server from linux-first-connect (admin / lab-only-pw!1,
  guest enabled)

## Steps

The operator followed a script; the steps below are what the capture shows.

1. Connected as admin (port 51382) and opened the account classes.
2. Edited the Guest class twice with 504 Set Account Class: bit 11 (Upload)
   went off (`28300003…` to `28200003…`) and back on.
3. Created the account `gui-test` ("GUI Test", class User), changed it, and
   deleted it.
4. Created the newsgroup "Lab Group", changed it to "test newsgroup" with a
   420 KiB limit, and deleted it.
5. Connected as guest with no login (port 36078) and read General News.

## Observations

- 504 Set Account Class replies with 1 Status success 0x1f8. This is the
  first capture of 504, and of 600, 601, and 604.
- Guest's 608 News Messages permissions are 0x4, as with the CLI.
- Every message decoded with the existing codecs.
