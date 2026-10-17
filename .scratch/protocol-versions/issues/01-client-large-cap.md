# Identify the client's 0x1000000 receive cap

Status: ready-for-agent

Clients set four receive caps (`docs/protocol/framing.md`, "Across
versions"). Three have known callers. The fourth, 0x1000000, is set by one
caller in Client 1.600 (`FUN_0041bde0`, project `KDXClient1600-Win`) and beta 5
(`FUN_0806f61c`, `KDXClientBETA5-Lnx`). <!-- [S] --> Done when framing.md names
the connection it applies to, with the caller's evidence.
