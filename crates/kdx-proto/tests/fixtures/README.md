# Fixtures

`first-connect.c2s.bin` and `first-connect.s2c.bin` are the two TCP payload
streams of `captures/first-connect.pcapng`, reassembled in order: client to
server (824 bytes) and server to client (3120 bytes). Each starts with the hello
and proof, followed by the session messages. The lab account password is
`lab-only-pw!1`.

Each proof carries 20 bytes of Haxial's challenge table, which the capture
already holds. The full table is not in the repository.
