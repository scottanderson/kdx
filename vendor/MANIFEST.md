# Reference binaries

The original Haxial KDX builds used as the reference for protocol work. Haxial
shut down in 2009. These are the last versions on Haxial's own download server,
fetched from Wayback Machine captures of `haxial.com/download/` (raw `id_`
captures, so the bytes are unmodified). `/downloadstatic/KDX*-<platform>`, the
site's "latest version" links, resolve to the same builds; the Windows server
archive was byte-identical.

Treat everything here as untrusted, 20-year-old closed-source code. **Never run
it on the host.** Run it only inside the lab container or VM described in
`docs/protocol/PLAN.md`.

`vendor/haxial/` is git-ignored: KDX was commercial shareware, and the archives
can be fetched again from the URLs below. Each archive is unpacked into its own
`extracted/` directory.

| Archive | Version | Wayback timestamp | SHA-256 |
|---|---|---|---|
| KDXServer1620-Lnx.zip | Server 1.620, Linux i386 (newest server) | 20051014063537 | a270a27180b9551b31e3d09fe99d863e88e93926fc7595fb62e975681ce12e74 |
| KDXServer1600-Win.zip | Server 1.600, Win32 | 20051014063500 | 1fca4b462ccefb70025c5353e7c9da1d7346cc49860cfebf9a6b8296569d67eb |
| KDXServer1600-Mac.sit | Server 1.600, Mac PEF/Carbon | 20051014063432 | 8d76c97b199c10ffc889643ee3e7c041b09a66bc806ca0aa0df65234d2013845 |
| KDXServer1600-MacUnix.zip | Server 1.600, Mach-O ppc | 20051014065217 | d01478822c4b68b2ed0d13126c96ac511d3630392ebd035b5bfdba40a5cd936b |
| KDXClient1600-Win.zip | Client 1.600, Win32 | 20051014063319 | cc6ec09bad1969e354ed5778ad594a0f8639ae8e3aa05e169916cc4b8d4f038b |
| KDXClient1600-Mac.sit | Client 1.600, Mac PEF/Carbon | 20051014063241 | cfdd587f9d73580868d66065ebbde5a166aa2b5e25a8a0ef3d8c3b27136ec282 |
| KDXClient-Lnx-beta5.zip | Client Linux beta 5, i386/X11 | 20051014064546 | efe6d9c3a96784129dc8e49e41863487d441fe9db89ab1ee7a4ad12de6972fd9 |
| KDXTrackerServ1110-Lnx.zip | Tracker 1.110, Linux i386 | 20051014065532 | 1410b1dfc60094ef3ddb408dcd0077db065bf49125d9518d718c5578a3ba49cd |
| KDXTrackerServ1110-Win.zip | Tracker 1.110, Win32 | 20051014065628 | 8f50ef0cd4f82fc5b071340932c498a9df3556ed6cff6f23ecd6c36c43683583 |
| KDXTrackerServ1110-Mac.sit | Tracker 1.110, Mac PEF | 20051014065559 | 4ccf77333baf50d9f9e66dc59797d0c716451aaef2ee58bb5beefc47ab369d09 |
| KDXTrackerServ1110-MacUnix.zip | Tracker 1.110, Mach-O ppc | 20051014070016 | 06437455a9b33ff4e0891380855c3552d6d562bb05c49b74c47ac9c44e81afa7 |

URL pattern:
`https://web.archive.org/web/<timestamp>id_/http://www.haxial.com:80/download/<archive>`

Older builds (client 1.320–1.520, server 1.300–1.600 Linux) are in Wayback under
`haxial.com/download/old/`. Use them to diff protocol versions if needed.

Version notes: some download sites list a "1.65 (2008)", but no such build was
on Haxial's server, so treat it as unverified. The Linux binaries are stripped,
dynamically linked against `libstdc++.so.5`, and store UI text as Pascal
(length-prefixed) strings. The server's default port appears to be 10700
(`setport 10700` usage text).

## refs/

- `hlwiki-KDX_Protocol.html` (local snapshot only, not committed because its
  license is unknown): https://hlwiki.com/index.php/KDX_Protocol (UDP tracker
  LCG constants: init `0xA5A16C4A`, mul `0x41D28485`, add `12843`; notes on
  MD5/CRC32/Twofish and story text mixed into packets).
