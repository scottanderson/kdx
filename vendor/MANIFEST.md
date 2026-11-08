# Reference binaries

Haxial's own software, in every build that survives. Haxial shut down in 2009.
The KDX builds are the reference for protocol work; the other programs share
the same cross-platform codebase.

Most archives come from Wayback Machine captures of Haxial's download server,
`haxial.com/download/` (raw `id_` captures, so the bytes are unmodified). These
include the last builds on the server. `/downloadstatic/<Program>-<platform>`,
the site's "latest version" links, resolve to the same builds from 2005 on;
the Windows server archive was byte-identical. Their 2003 captures are older
builds: `KDXClient-Win.zip` is Client 1.110, and `NetFone-Mac.sit` is
byte-identical to `NetFone1400-Mac.sit`. From late 2004, the `NetFone-*` and
`RemoteAdminTool-*` links served the KDX client, and `WorldClock-*` served
Organizer.

A few builds that Wayback does not have come from other archives: Info-Mac
(`archive.info-mac.org`), the Tucows mirror on archive.org, and Macintosh
Repository. Their rows give a URL in place of a Wayback timestamp.

Treat everything here as untrusted, 20-year-old closed-source code. **Never run
it on the host.** Run it only through `lab/kdx-lab` (see `lab/README.md`).

`vendor/haxial/` is git-ignored: KDX was commercial shareware, and the archives
can be fetched again from the URLs below. `vendor/fetch` downloads the four
builds the lab runs (`vendor/fetch all` for every archive), checks each
SHA-256, and unpacks it with `unar` into `vendor/haxial/<name>/extracted/`.
It reads this table, so keep the table's columns as they are. The Source column
holds a Wayback timestamp or a URL. When the URL is a page, not the file,
`vendor/fetch` cannot download the archive; download it by hand into its
directory and run `vendor/fetch` again.

If the archives are already at hand from some other source, put each one
where `vendor/fetch` would download it, `vendor/haxial/<name>/<archive>`
(for example `vendor/haxial/KDXServer1620-Lnx/KDXServer1620-Lnx.zip`), then
run `vendor/fetch`. It skips the download when the checksum matches, and
still verifies and unpacks the archive.

| Archive | Version | Source | SHA-256 |
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
| AppearanceEdit1240-Mac.sit | AppearanceEdit 1.240, Mac PEF | 20051014065356 | 59cd7b3bbd212d4cf1997195fc921ac6880e5d0b8fc88b0ba5f282f4d74ae72e |
| AppearanceEdit1240-Win.zip | AppearanceEdit 1.240, Win32 | 20051014065431 | ade24eb5daa728e2aa950692458dd28a9dce0cdf90afb5758ce3f95b69760d57 |
| experimental/AppearanceEdit1400-Mac.zip | AppearanceEdit 1.400 (experimental), Mac | 20060505063900 | 088f7082dcd786e2cdbd08ed6ce6897d23305a884296ba0df5b4374f7c9a9406 |
| experimental/AppearanceEdit1400-Win.zip | AppearanceEdit 1.400 (experimental), Win32 | 20060505064010 | 31a3c21fea47de167beb1d413446cce92a7dc374f2c5ede6eafb7f4eeec19a46 |
| Appearances040521.zip | Appearances 2004-05-21 (the `.sit` twin has the same files) | 20051014062927 | ece210af318a0cc175420f76931131f388be3234c8cea98e880c43bdf660ecd8 |
| Appearances041205.zip | Appearances 2004-12-05 (the `.sit` twin has the same files) | 20051014063036 | 8d62a7577f78c561643a903d85baa624a56c9b64f639041a61413773208bb1bd |
| AudioTester-Mac.sit | AudioTester (no version number), Mac | 20051014061911 | 30fc30a2ec8cd1ac7070b83840d3d9a20c86672e3f89e06046a11e3eb570728b |
| AudioTester-Win.zip | AudioTester (no version number), Win32 | 20051014061948 | b1c69d92a14ea90dc6510e41e1d9c4a2ae6ab67d0d65a5eed49ee9ba91b243c2 |
| Benchmark1000-Mac.sit | Benchmark 1.000, Mac | 20051014063108 | 38d860f3cd3f4f70723f43cccf77112d46d80c38eadb9a11273510d7ee21a15a |
| Benchmark1000-Win.zip | Benchmark 1.000, Win32 | 20051014063140 | 779493792e7cf6868f5074b346553e25b15d8ddacbe3c1fad3e1a9abea41484d |
| experimental/ButtonBarEdit1000-Mac.zip | ButtonBarEdit 1.000 (experimental), Mac | 20060505063751 | 8cbc65f43a1c9453492ced3a92511f713c3201819cbacd648f00828adf0761ae |
| experimental/ButtonBarEdit1000-Win.zip | ButtonBarEdit 1.000 (experimental), Win32 | 20060505063811 | 303088474638c3708ca05c379cb109ce2d8b8d7ae3f0eebfa1337e70bb1b063b |
| old/Calculator1100-Mac.sit | Calculator 1.100, Mac | 20060321235156 | 0030e6b02b0816eacdbe637de58ed84e8a1164843e3419778f2e1e871af64abd |
| old/Calculator1100-Win.zip | Calculator 1.100, Win32 | 20060515145345 | bfd23243396ad85eeb42544f58e9ce1f04f842104bcf0ea5af8c913f0907cab6 |
| Calculator1200-Mac.sit | Calculator 1.200, Mac | 20051014063909 | a407a8e1b782e97af4d596b9cf5c6ce8c6e715954d5ef7ce68287e0f45291d47 |
| Calculator1200-Win.zip | Calculator 1.200, Win32 | 20051014063946 | 865b5d4d4ab965c6dc89d4897ead4eaa7ba292813e54aa2725141238e5b1db9c |
| Calculator1200beta-Lnx.zip | Calculator 1.200 beta, Linux | 20051014065504 | 3ed42316d44b8a278137bf6531ff553f98df42ca30eb11d921875dae0b789d91 |
| DiskCatalog1300-Mac.sit | DiskCatalog 1.300, Mac | 20051014060815 | a160e2d82b894d751846f3f711ae5f9df32efb6f13d3ec269a1e2a034ec07fe2 |
| DiskCatalog1300-Win.zip | DiskCatalog 1.300, Win32 | 20051014064309 | 8ea25615bd270cdc5718778d723aa5a7a4b948f29d2ef2aa56e846b952a443db |
| FontEdit1000-Mac.sit | FontEdit 1.000, Mac | 20051014062225 | 98edca64cadaed2b07aba05a18dc7f20163fcd2ed60c44a7cdee5c91039ab1fa |
| FontEdit1000-Win.zip | FontEdit 1.000, Win32 | 20051014062313 | 160e1dd4132714e85ae1ad258b6f889c34a8bf8199b9592983d33b03047a5501 |
| Fonts040426.zip | Fonts 2004-04-26 (the `.sit` twin has the same files) | 20051014060902 | 714736c1106a3f1c9e9e4c466ca0ffaf121f105f9910709b02642113dd1ded79 |
| Hash1010-Mac.sit | Hash 1.010, Mac | 20051014060927 | 8563cc4544d3ea8c6c4633b583affc4064f39846b1f592af292861dd2936ef25 |
| Hash1010-Win.zip | Hash 1.010, Win32 | 20051014060953 | a599b106486e4e13bf0621bfac19b3241c877b6771b80e98efc83c88a30a1f78 |
| HxlLauncher0902-Mac.sit | Launcher 0.902, Mac | 20051014064337 | e1d6de066ad45d5b6cb7310bc9be34e3978f018cfb8d876b7abf25c00c3c585b |
| HxlLauncher0902-Win.zip | Launcher 0.902, Win32 | 20051014064410 | daa18691fce26ca90f26c4e1330ebc314277a67391f172b66544a196527c1bf1 |
| Organizer1070-Mac.sit | Organizer 1.070, Mac (`/downloadstatic/WorldClock-*` served this too) | 20051014063601 | 2f4c8427a80573ec60d376c96c100e0dcf80fe8aed03e8b7bcf9a1fe6c3b2e93 |
| Organizer1070-Win.zip | Organizer 1.070, Win32 | 20051014063625 | 031e4ba30c85e123a5fe0ea1de4d29c623b9023f241003c0e0b4fece550601a0 |
| old/TextEdit1610-Mac.sit | TextEdit 1.610, Mac | 20060321235226 | b76c3f851e4e89c17710125605e5dfdf3d326a7d44d74dce2e8e6ea4829c24f1 |
| old/TextEdit1610-Win.zip | TextEdit 1.610, Win32 | 20060515144858 | 3084166a556a0e1a0e3ea0f23da963a9d2a60ee6c25dcd3a50b790ec0ce27651 |
| old/TextEdit1631-Mac.sit | TextEdit 1.631, Mac | 20060321235211 | 13b41b43cbd74f0b4f498b17f2bc4829813db5b96b98a396be1f846a95a59f0f |
| old/TextEdit1631-Win.zip | TextEdit 1.631, Win32 | 20060515144759 | f8a8649a93162c76701f1df5acb50c98a2e0d18fc34c2e95163247e70a7149ec |
| TextEdit1700-Mac.sit | TextEdit 1.700, Mac | 20051014062533 | 77a7445ae75fee3b5dad3f537b7b73b311c55c898f102c004be0ff7652b27df2 |
| TextEdit1700-Win.zip | TextEdit 1.700, Win32 | 20051014062557 | 96d76dc3fb59a3b638cb5d81a54bb2774ea3c35ab2a92debfbd4d19135c3e1ce |
| xyzzy1000-Mac.sit | xyzzy 1.000, Mac | 20051014061748 | 2ab06f447e84d4c0da64852a6e1f0e476af5c5855b9c9df89b5ea963f9ded1c5 |
| xyzzy1000-Win.zip | xyzzy 1.000, Win32 | 20051014061816 | ca222b05c3ad32c6bf45628eb35c9b71411d492a773c8bb0483aeaface478cd3 |
| kdx-client-1000.hqx | Client 1.000, Mac PEF (Info-Mac) | https://archive.info-mac.org/inet/kdx-client-1000.hqx | 7e84bcce1981a35cc58d41a42fc008f67ac52f6c907a7af1b86c08e5758a2ca2 |
| kdx-server-1000.hqx | Server 1.000, Mac PEF (Info-Mac) | https://archive.info-mac.org/inet/kdx-server-1000.hqx | d4274dd991d627dc803df82caf9083fcfaa5dd474c6faf5497763f4529d2b7d9 |
| haxial-disk-catalog-12.hqx | DiskCatalog 1.200, Mac (Info-Mac) | https://archive.info-mac.org/disk/haxial-disk-catalog-12.hqx | 302a802423be752c8d5b1ed10248711c4e3dcf841d21b59dba4ab97e8b895d3c |
| haxial-textedit-141.hqx | TextEdit 1.410, Mac (Info-Mac) | https://archive.info-mac.org/text/haxial-textedit-141.hqx | 2f291f027b14f74022a5789ec281204a46d6a2dc6daf5c0610c4f80616e5660c |
| haxialtextedit.sit | TextEdit 1.420, Mac (Tucows) | https://archive.org/download/tucows_269348_Haxial_TextEdit/haxialtextedit.sit | 17094890d571c53d48b341d8cd641733715f50f2ed1f903638564366508493c4 |
| haxialworldclock.sit | WorldClock 1.210, Mac (Tucows) | https://archive.org/download/tucows_269341_Haxial_WorldClock/haxialworldclock.sit | 3b07bbd265ac96917cf1705b2e9ace3b482a68fabbef95937710321957e1372d |
| KDX___Haxial_Bundle.sit | A user's bundle of Mac builds, with RemoteAdminTool 1.080 (see below) | https://www.macintoshrepository.org/8216-kdx-haxial-bundle-macos-9-and-mac-osx-ppc- | 6648c9c3328ecb34f1bf7a51a5435572a6f913653b3a29a0becb8d56ddfc081f |

URL pattern:
`https://web.archive.org/web/<timestamp>id_/http://www.haxial.com:80/download/<archive>`

Older builds (client 1.320–1.520, server 1.300–1.600, and older Calculator and
TextEdit builds) are in Wayback under `haxial.com/download/old/`. Use them to
diff protocol versions if needed. A row for one names it with the `old/`
prefix, a row for a build in `download/experimental/` with `experimental/`, and
a `/downloadstatic/` row with `../downloadstatic/`. `vendor/fetch` keeps each in
`vendor/haxial/<name>/` like the others, so Client 1.110 lands in
`vendor/haxial/KDXClient-Win/`.

The bundle from Macintosh Repository is a user's collection of Mac builds,
repacked with folder names such as `Haxial Remote Access Tool 108`. It is the
only known copy of RemoteAdminTool 1.080 (Mac), ColorSchemeEdit 1.00, and
IconListEdit 1.01. The user also added a `Launch Items` folder to the
RemoteAdminTool folder. Some of its other builds were changed: the data forks
of its KDX Client 1.600 and DiskCatalog 1.300 have the same size as the
originals but different bytes, and its page says that its KDX Server is
"already registered". Never use its KDX builds as a reference. Its programs
named Benchmark, Calculator, FontEdit, Hash, Launcher, LogView (1.120, though
the folder says 100), NetFone, Organizer, SoundListEdit, TextEdit, WorldClock,
xyzzy, and Tracker have the same data forks as the builds above.

Builds that are lost: the directory listings on `haxial.com` and `haxial.net`
(2001 to 2009) name 498 files, but Wayback has no successful capture of most of
them. The missing builds include:

- Client and Server 0.933 to 0.990, 1.000 for Windows, 1.100, 1.110 (except
  Client 1.110 for Windows), 1.200, and 1.230, and Client 1.300. Info-Mac
  digests also name Client and Server 0.925 to 0.938 for Mac.
- Linux Client betas 1 to 3, and the Linux betas `KDXServer-lnx-b2.zip` and
  `KDXTrackerServ-lnx-b2.zip`.
- Tracker 0.904 to 1.100, LogView 0.900 to 1.100, and SoundListEdit 0.900.
- RemoteAdminTool 1.000, 1.050, 1.060, and 1.070, and 1.080 for Windows.
  Haxial sold it as a separate program in 2002 and 2003. From late 2004, its
  download links served the KDX client, which probably took over its
  features.
- NetFone 0.900 to 1.300, and 1.400 for Windows. From late 2004, its download
  links also served the KDX client, and Haxial's NetFone page told users to
  use KDX as an Internet telephone.
- ColorSchemeEdit, IconListEdit, the ColorSchemes and IconLists packs, and the
  older builds of the other programs.
- The Haxial Compiler (KRE) alphas a15 to a22 and 0.23 to 0.25 (2008), for
  Windows.

Info-Mac also lists `haxial-world-clock-11.hqx` (WorldClock 1.100), but its
copy is cut short and does not unpack.

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
- Haxial's version history pages (local snapshots only, not committed because
  they are Haxial's text). Each one is the last good Wayback capture of
  `http://www.haxial.com:80/products/<path>`; fetch it again with
  `https://web.archive.org/web/<timestamp>id_/<that URL>`.

  | File | Path | Timestamp | SHA-256 |
  |---|---|---|---|
  | `kdx-vh.html` | `kdx/kdx-vh.html` | 20041205094958 | `d12f471304c5cf301e3d86443ce90d5d102d0e166c0e308b23ced3f5fbeae427` |
  | `kdx-vh-old.html` | `kdx/kdx-vh-old.html` | 20020529042135 | `07712d67cfed686e0b97a9e7536ad587ed934b53b5335be94c944632a9f3fcef` |
  | `kdxtracker-vh.html` | `kdx/kdxtracker-vh.html` | 20040409140425 | `73daea4abc6e9ca76f39bf6f4bd65164f1c9acd8d6f2b62d2d73e8e67874bf5e` |
  | `rat-vh.html` | `rat/rat-vh.html` | 20050211155503 | `ef599998283532e424dad6456303afda3ef0d777a71839141c1506d401415453` |
  | `netfone-vh.html` | `netfone/netfone-vh.html` | 20060113082401 | `dc75714b1fc8d701945ad8181d5be1a3bf3d16ff98068be57f45817cfe469d0d` |
  | `logview-vh.html` | `kdx/logview/logview-vh.html` | 20090215122050 | `e739c274d553e31fa70b3db7e5d44e4cc0b44f62ed0b45264d110438143d6bae` |
