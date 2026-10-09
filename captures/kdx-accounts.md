# Scenario: kdx-accounts

- Date: 2026-10-08T23:53Z to 2026-10-09T00:00Z
- Server: KDXServer 1.620 Linux (sha256 ad6fba4bdd01dc23)
- Client: none; the `kdx` CLI (this repository) only, no GUI
- Server state: the first-connect server (admin / lab-only-pw!1, guest
  enabled), with only the `admin` and `guest` accounts

## Steps

Each step is one `kdx` run as admin unless it names another login.

1. `account classes`, `account list`.
2. `account add rs-test --name 'RS Test' --class 1 --new-password …`, then
   `account show rs-test` and five `rs-test` logins. All seven failed the
   handshake: in this build the positional `rs-test` replaced the global
   `--login`, so admin's password was tried for `rs-test`. The CLI was fixed
   and the steps repeated.
3. `account add rs-test` (password `rs-test-pw`, class 1), `account show`,
   `account list`, and `account add rs-test` again (0x1000d).
4. As `rs-test`: `who`, `passwd rs-test-pw2`, `who` with the old password
   (handshake refused), `who` with the new one, `account list` (0x10014),
   and `account rm guest` (0x10014).
5. `account set rs-test --rename rs-test2 --name 'RS Two' --class 2`,
   `account show rs-test2`, and a login as `rs-test2` with the same password.
6. `account set rs-test2 --new-password rs-test-pw3` and a login with it.
7. The next connection got no hello: the server had banned 127.0.0.1 for
   flooding, about 30 connections into the session. The server was stopped
   with SIGTERM and started again with `kdx-lab server -d`, which ended the
   ban.
8. `account set nobody` (509 refused with 0x1000c), `account add rs-nopw
   --class 2` with no password, `account show rs-nopw`, `account rm
   rs-test2`, `account rm rs-test2` again (0x1000c), `account rm rs-nopw`,
   `account list`.

The accounts ended as they started.

## Observations

- 507 and 511 answer with 1 Status success whose value is the request type.
- 508 has no reply; the requester gets 515 Account Deleted as a
  notification.
- 510 never holds the key; byte 0x20 is 1 with a password and 0 without.
- A flood ban closes connections before the hello, with no message.
