# Reference binaries

The original Haxial KDX builds used as the reference for protocol work. Haxial
shut down in 2009. These are the last versions on Haxial's own download server,
fetched from Wayback Machine captures of `haxial.com/download/` (raw `id_`
captures, so the bytes are unmodified). `/downloadstatic/KDX*-<platform>`, the
site's "latest version" links, resolve to the same builds from 2005 on; the
Windows server archive was byte-identical. Their 2003 captures are older
builds: `KDXClient-Win.zip` is Client 1.110, and `NetFone-Mac.sit` is
byte-identical to `NetFone1400-Mac.sit`.

Treat everything here as untrusted, 20-year-old closed-source code. **Never run
it on the host.** Run it only through `lab/kdx-lab` (see `lab/README.md`).

`vendor/haxial/` is git-ignored: KDX was commercial shareware, and the archives
can be fetched again from the URLs below. `vendor/fetch` downloads the four
builds the lab runs (`vendor/fetch all` for every archive), checks each
SHA-256, and unpacks it with `unar` into `vendor/haxial/<name>/extracted/`.
It reads this table, so keep the table's columns as they are.

If the archives are already at hand from some other source, put each one
where `vendor/fetch` would download it, `vendor/haxial/<name>/<archive>`
(for example `vendor/haxial/KDXServer1620-Lnx/KDXServer1620-Lnx.zip`), then
run `vendor/fetch`. It skips the download when the checksum matches, and
still verifies and unpacks the archive.

| Archive | Version | Wayback timestamp | SHA-256 |
|---|---|---|---|
| KDXServer1620-Lnx.zip | Server 1.620, Linux i386 (newest server) | 20051014063537 | a270a27180b9551b31e3d09fe99d863e88e93926fc7595fb62e975681ce12e74 |
| KDXServer1600-Win.zip | Server 1.600, Win32 | 20051014063500 | 1fca4b462ccefb70025c5353e7c9da1d7346cc49860cfebf9a6b8296569d67eb |
| KDXServer1600-Mac.sit | Server 1.600, Mac PEF/Carbon | 20051014063432 | 8d76c97b199c10ffc889643ee3e7c041b09a66bc806ca0aa0df65234d2013845 |
| KDXServer1600-MacUnix.zip | Server 1.600, Mach-O ppc | 20051014065217 | d01478822c4b68b2ed0d13126c96ac511d3630392ebd035b5bfdba40a5cd936b |
| KDXClient1600-Win.zip | Client 1.600, Win32 | 20051014063319 | cc6ec09bad1969e354ed5778ad594a0f8639ae8e3aa05e169916cc4b8d4f038b |
| KDXClient1600-Mac.sit | Client 1.600, Mac PEF/Carbon | 20051014063241 | cfdd587f9d73580868d66065ebbde5a166aa2b5e25a8a0ef3d8c3b27136ec282 |
| KDXClient-Lnx-beta5.zip | Client 1.620 beta 5, Linux i386/X11 (built 2004-12-19, after 1.600) | 20051014064546 | efe6d9c3a96784129dc8e49e41863487d441fe9db89ab1ee7a4ad12de6972fd9 |
| KDXTrackerServ1110-Lnx.zip | Tracker 1.110, Linux i386 | 20051014065532 | 1410b1dfc60094ef3ddb408dcd0077db065bf49125d9518d718c5578a3ba49cd |
| KDXTrackerServ1110-Win.zip | Tracker 1.110, Win32 | 20051014065628 | 8f50ef0cd4f82fc5b071340932c498a9df3556ed6cff6f23ecd6c36c43683583 |
| KDXTrackerServ1110-Mac.sit | Tracker 1.110, Mac PEF | 20051014065559 | 4ccf77333baf50d9f9e66dc59797d0c716451aaef2ee58bb5beefc47ab369d09 |
| KDXTrackerServ1110-MacUnix.zip | Tracker 1.110, Mach-O ppc | 20051014070016 | 06437455a9b33ff4e0891380855c3552d6d562bb05c49b74c47ac9c44e81afa7 |
| old/KDXServer1600-Lnx.zip | Server 1.600, Linux i386 (installed by 175 in the lab's downgrade test) | 20060515145257 | 75cfe75a26b625ec6ff767491b2e863fc74b0cc5f5a186c483e9d3d22d47b937 |
| old/KDXServer1300-Win.zip | Server 1.300, Win32 | 20060515145128 | 86081323d65c71c3ef9ba4ce0da41531f3880538b4f6d8a44e34293b142f0279 |
| old/KDXServer1500-Win.zip | Server 1.500, Win32 | 20060515145159 | 02d885c29ea686c156f6aeff710f2c8d2bfae1340a77f69bda29646500da9e56 |
| old/KDXServer1520-Win.zip | Server 1.520, Win32 | 20060515145245 | f043b9963669b4a6504bb27c1c7b0facf52be2edb2d887c54deabd29a333df4d |
| old/KDXServer1520-Lnx.zip | Server 1.520, Linux i386 | 20060515145214 | ddf299370288401177fda5debc54d47d29a4c145751631e45541af1480144f2b |
| old/KDXClient1320-Win.zip | Client 1.320, Win32 (first client with `%HGC`) | 20060515144935 | 8a1a4d1af741beec1ae6f099065d099a43d17fc68cc216e090349a1ba60a8154 |
| old/KDXClient1400-Win.zip | Client 1.400, Win32 | 20060515145001 | d2adb07d710773d5308e54d88c5bc017b4563076eeaf2a6355dc5aeea3af830b |
| old/KDXClient1500-Win.zip | Client 1.500, Win32 | 20060515145035 | 8cdd7d1df28730f139acacce9fce3e31a1b2ed4d3a1d6c32b67777e1f0900aa5 |
| old/KDXClient1520-Win.zip | Client 1.520, Win32 | 20060515145103 | 268f75a070dc29b24cab8bb5d0509d623bf0dd8a5dd3edf046508e2bfcc65ba5 |
| old/KDXClient1320-Mac.sit | Client 1.320, Mac PEF/Carbon | 20060321235138 | a56ad6f443d086b33dfc575f7841ee4be9cc7f674c93563d9bb5045adf52c879 |
| old/KDXClient1400-Mac.sit | Client 1.400, Mac PEF/Carbon | 20060321235143 | db12be42d0444ec191bb5d800c8d4ae62e646c844c661656a6d17ea588c22048 |
| old/KDXClient1500-Mac.sit | Client 1.500, Mac PEF/Carbon | 20060321235215 | d7c587134e1ae73543e71e8a8fb26687f2cd099a3fbb7fb8ca59181a223efe44 |
| old/KDXClient1520-Mac.sit | Client 1.520, Mac PEF/Carbon | 20060321235205 | e3aa21e596023352ef4c190cf5cc26faf7b3e75f8290548a7950629e26c0c140 |
| old/KDXServer1300-Mac.sit | Server 1.300, Mac PEF/Carbon | 20060321235150 | 89a437fbfb2deb4a9481f3f2baf138967401f2773b3995c699203cc6f2869f79 |
| old/KDXServer1500-Mac.sit | Server 1.500, Mac PEF/Carbon | 20060321235221 | a4b09767cb7030765b1b0049e3e7a331fa5a5075cac736ae5ba905e8df2d3d75 |
| old/KDXServer1520-Mac.sit | Server 1.520, Mac PEF/Carbon | 20060321235045 | 355d6facc7f347e9403b99d125084076c650e178d375879ed635482eeb72525c |
| old/KDXServer1520-MacUnix.zip | Server 1.520, Mach-O ppc | 20060515144742 | 0269b7933b1570ccb0547957bb2b2f6dcff728c5e630bf0a3cba89f78dd95320 |
| ../downloadstatic/KDXClient-Win.zip | Client 1.110, Win32 (unpacks to `KDXClient1110-Win/`) | 20031215224532 | ca81ce0a8a0cf1e550fb953f676cef3fe34d7acf755fbee0c6ef9fb7901c8d4a |
| KDXClient-Lnx-beta4.zip | Client 1.620 beta 4, Linux i386/X11 | 20051014064518 | 943d7c3c85a5a8af26f33fcbf9de00fd3c43f12edb0869cfcc0ce6fa868ff3f9 |
| KDX0915.sit.bin | Client and Server 0.915, Mac PEF | 20010723053143 | 2c47b13ee4570272132d93f5b3641b683e3b04ec83404494a9651befd2fa2fd7 |
| KDX0916.sit.bin | Client and Server 0.916, Mac PEF | 20010723044626 | d7d6b7bc7c644c678607ba65acccc98288007cdfc619077f7a1a927c15be416a |
| KDX0918.sit.bin | Client and Server 0.918, Mac PEF | 20010723045056 | e08718acd5636017a8cf4638940ba1ee48ad0dde4963be91714a2aaa3e61ebd7 |
| KDX0919.sit.bin | Client and Server 0.919, Mac PEF | 20010723045704 | 1f0c525ee0bd9b04754e388e96aee0e95c7e3e9f0874395390b3487a1805c853 |
| KDX0920.sit.bin | Client and Server 0.920, Mac PEF | 20010810110141 | 560fe2d86718cd8596dafe6b859fc0edd8173d0ed9cbcf6268b36df215af2801 |
| KDX0921.sit.bin | Client and Server 0.921, Mac PEF (2001 handshake, see handshake.md) | 20010810111341 | 3bf2570c208ee0e82f35c2c23aebfea028c68cf4adc57b222fdd58d0b75226f1 |
| KDXTrackerServ0900.sit.bin | Tracker 0.900, Mac PEF | 20010810110838 | e0507fa9f31c518c18be7aabbed1d164ef780a27a79bf49a175ae54031e4e06f |
| KDXTrackerServ0902.sit.bin | Tracker 0.902, Mac PEF (2001 handshake) | 20010810112345 | cc627c5c2be519e9d317d0befdd849cd0c68e96649f82ee3d7ee55bf44ab1283 |
| KDXLogView1120-Win.zip | LogView 1.120, Win32 | 20051014064043 | 698bb5914da524f2b39311206e1cdddcbbe50e09caaf55356a6676f60e24bec6 |
| KDXLogView1120-Mac.sit | LogView 1.120, Mac PEF | 20051014064009 | 4e6938d6dcd76687b66f0b04deeefab2b2275b5da535768b304c36b89d33b6f8 |
| KDXSoundListEdit1000-Win.zip | SoundListEdit 1.000, Win32 | 20051014065758 | 7c03f446e34e7a0af6af5343cebfbdf9c323497523b90a1052ca333c1a3fa26b |
| KDXSoundListEdit1000-Mac.sit | SoundListEdit 1.000, Mac PEF | 20051014065718 | 736c6057f13c2a6f228b9e4873d51f34d196af8c9a40738a86f6b7e8fdddd636 |
| NetFone1400-Mac.sit | NetFone 1.400, Mac PEF (`%NFO` over the 2001 handshake) | 20030403191716 | 1cffb26f1b49d8fcd1ef927e1f509acdc9a35d382946a31ad6111d8c2d7da57c |

URL pattern:
`https://web.archive.org/web/<timestamp>id_/http://www.haxial.com:80/download/<archive>`

Older builds (client 1.320–1.520, server 1.300–1.600) are in Wayback under
`haxial.com/download/old/`. Use them to diff protocol versions if needed. A
row for one names it with the `old/` prefix, and a `/downloadstatic/` row
with `../downloadstatic/`. `vendor/fetch` keeps each in
`vendor/haxial/<name>/` like the others, so Client 1.110 lands in
`vendor/haxial/KDXClient-Win/`.

Haxial's 2002 directory listings also name Client and Server 0.933–0.942,
Tracker 0.904–0.905, and NetFone 1.000–1.221, but Wayback has no successful
capture of any of them.

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
