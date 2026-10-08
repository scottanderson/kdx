# kdx

A Rust client, and later a server and a tracker, for the Haxial KDX protocol,
compatible with the original KDX Server 1.620, Client 1.600, and Tracker 1.110.

- `README.md`: the repository layout
- `docs/protocol/PLAN.md`: phases, licensing, and open work
- `docs/protocol/*.md`: the protocol spec
- `lab/README.md`: how to run the original programs and capture traffic
- `re/README.md`: the Ghidra tooling and a table of server address landmarks
- `vendor/MANIFEST.md`: where each Haxial archive comes from, with checksums

Put details in those files. Keep this file to rules.

## Legal boundaries

- Never commit Haxial material. `vendor/haxial/`, `re/out/` (decompiled
  code), `re/projects/`, and `vendor/refs/*.html` are git-ignored. Keep it
  that way.
- Never commit the 1739-byte challenge table (the "story"). Code reads it at
  run time through `ChallengeTable::extract` or `from_bytes`. Tests that need
  it skip when `vendor/haxial/` is missing.
- Don't accept Haxial's license agreement for the user. The user clicks
  "I Agree" in the client.

## Lab safety

- Run Haxial binaries only through `lab/kdx-lab`, never directly on the host.
- Run `lab/kdx-lab status` before any other lab command. `down` and `reset`
  end the user's running session and capture, so never chain them after
  other commands.
- Agents may drive the GUI clients with `xdotool`, except for the license
  agreement. Record each capture's steps in `captures/<scenario>.md`.
- Run Python that reads downloaded or extracted files with `python3 -I`.

## Docs

- Tag each claim with its evidence: **[O]** observed in a capture, **[S]**
  read in a binary, **[I]** inferred.
- Cite static findings by function address (`FUN_08085a04`), and cite
  captures by file and frame number.
- Write numbers in the protocol as hex where the binary uses hex. Write all
  multi-byte fields as big-endian unless the doc says otherwise.
- Put each verified fact in `docs/protocol/`, never in `re/out/`. Add new
  landmarks to the table in `re/README.md`.
- Mark a fact as verified only after a test or a capture confirms it.
- Wrap Markdown prose at 80 columns. Leave tables and code blocks unwrapped.

## Code

- `kdx-proto` has no I/O and no async.
- A fixed-size wire struct has a `pub const LEN`.
- Types that hold key material (`AccountKey`, `SessionCipher`) have a
  redacted `Debug`.
- Test wire code against capture fixtures in
  `crates/kdx-proto/tests/fixtures/`. Prove encoders by re-encoding a
  decoded capture byte for byte.
- Name a provisional message type with a doc comment that says
  "Provisional name" until a handler or log string confirms it.

## Comments

- Write one line when possible. Put the reasons in the commit message.
- Write rustdoc like other Rust crates.
- Change a comment only with the code it describes.

## Checks

```sh
cargo fmt --all
cargo clippy --workspace --all-targets   # no warnings
cargo test --workspace
cargo run -p kdx-dissect -- captures/first-connect.pcapng --password 'lab-only-pw!1'
```

Static analysis runs headless; one call takes about 30 seconds:

```sh
re/ghidra /work/projects kdx -process KDXServer.lexe -noanalysis -readOnly \
  -scriptPath /work/scripts -postScript decompile.py 0x0804fdac
```

## Git

- Make small commits that each build and pass the tests.
- Write the subject as a short imperative sentence with no area prefix.
  Put the reasons in the body.
- Stage files by name. Keep the changes the user already staged.
- Put the doc update in the same commit as the work it describes.
- End each commit message with a `Co-Authored-By:` line naming the AI model
  that wrote it, such as
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
