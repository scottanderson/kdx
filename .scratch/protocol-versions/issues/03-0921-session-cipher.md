# Identify Server 0.921's session cipher

Status: done

Server 0.921's header-only send `FUN_10035b20` encrypts through
`FUN_1000eff0` with state at connection offset 0x12a, and the key schedule
is probably `FUN_1000c1f0` <!-- [S] --> (project `KDXServer0921-Mac`). Neither
the cipher nor whether its key is 1.x's `account_key` is known. Done when
handshake.md or framing.md names the cipher and its key, with evidence.

## Comments

Done: Twofish-256 ECB keyed with `account_key`; framing.md, "Across versions",
and handshake.md, "Version 0.9x".
