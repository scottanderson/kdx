# Contributing

[AGENTS.md](AGENTS.md) has the full rules. They apply to people as much as to
coding agents. This file covers the ones a newcomer needs first.

## Setup

1. Install Rust 1.96 or newer, then run `cargo test --workspace`.
2. For lab captures, run `vendor/fetch` and follow `lab/README.md`.
3. For static analysis, follow `re/README.md`.

## Legal rules

- Never commit Haxial material: the archives in `vendor/haxial/`, decompiled
  code from `re/out/`, Ghidra projects, or the 1739-byte challenge table.
  `.gitignore` covers the first three. Code reads the table at run time.
- Run the Haxial binaries only through `lab/kdx-lab`, never on the host.

## Checks

Run these before each commit. Clippy must report no warnings. CI runs the
same checks, with `--check` for rustfmt and `-D warnings` for clippy.

```sh
cargo fmt --all
cargo clippy --workspace --all-targets
cargo test --workspace
python3 -I ci/check-docs.py
cargo run -p kdx-dissect -- captures/first-connect.pcapng --password 'lab-only-pw!1'
```

## Protocol docs

Facts go in `docs/protocol/`. Tag each claim with its evidence:

- **[O]** observed in a capture; cite the file and frame number.
- **[S]** read in a binary; cite the function address, as in `FUN_08085a04`.
- **[I]** inferred.

Write each tag in bold, in tables and headings too, as in
`**[S FUN_08085a04]**`. Leave tags in code blocks plain.

Name a message by its type and title, as in 412 Download File. Write
multi-byte fields as big-endian.

## Code

- `kdx-proto` does no I/O and has no async.
- Test wire code against the capture fixtures in
  `crates/kdx-proto/tests/fixtures/` (see
  [its README](crates/kdx-proto/tests/fixtures/README.md)). Prove an encoder by
  re-encoding a decoded capture byte for byte.

## Commits

Make small commits that each build and pass the checks. Write the subject as
a short imperative sentence, and put the reasons in the body. Put a doc
update in the same commit as the work it describes.

## Backlog

Open work is tracked as markdown under `.scratch/`; see
`docs/agents/issue-tracker.md`. `docs/PLAN.md` lists the protocol
areas still to document.
