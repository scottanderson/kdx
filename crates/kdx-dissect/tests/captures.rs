//! Dissects every capture in `captures/` and checks it against its notes and fixtures.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Every lab password the captures' notes give.
const PASSWORDS: [&str; 7] = [
    "lab-only-pw!1",
    "lab-peer-pw!1",
    "lab-tracker-pw!1",
    "lab-tracker-pw!2",
    "rs-test-pw",
    "rs-test-pw2",
    "rs-test-pw3",
];

/// Every problem the dissector reports, as `<capture> <source port>: <line>`.
///
/// Each one is a step in the capture's notes, not a decoding fault.
const EXPECTED: &[&str] = &[
    // Step 2: admin's password tried for `rs-test`.
    "kdx-accounts 40714: key: --password #1 (server proof DOES NOT match)",
    "kdx-accounts 40718: key: --password #1 (server proof DOES NOT match)",
    "kdx-accounts 40732: key: --password #5 (server proof DOES NOT match)",
    "kdx-accounts 40740: key: --password #5 (server proof DOES NOT match)",
    "kdx-accounts 40754: key: --password #5 (server proof DOES NOT match)",
    "kdx-accounts 40756: key: --password #6 (server proof DOES NOT match)",
    "kdx-accounts 40762: key: --password #6 (server proof DOES NOT match)",
    // Step 7: the flood ban closes connections before the hello.
    "kdx-accounts 48046: stopped: S->C hello: need 36 bytes",
    "kdx-accounts 48060: stopped: S->C hello: need 36 bytes",
    "kdx-accounts 48076: stopped: S->C hello: need 36 bytes",
    "kdx-accounts 48080: stopped: S->C hello: need 36 bytes",
    "kdx-accounts 53018: stopped: S->C hello: need 36 bytes",
    // Step 4: `who` with the old password.
    "kdx-accounts 58350: key: --password #5 (server proof DOES NOT match)",
    // Step 8: `groups --login admin` with a wrong password.
    "kdx-cli-tracker 53576: stopped: no candidate key matches the client's proof; try --password",
    // Step 1: a DCC login with a name, which the peer rejects.
    "linux-dcc-voice-failed 44420: key: --password #1 (server proof DOES NOT match)",
    // A login with a wrong password.
    "linux-first-connect 35552: stopped: no candidate key matches the client's proof; try --password",
    // Step 6: the changed admin password, which the tracker ignored.
    "linux-tracker 48232: key: --password #4 (server proof DOES NOT match)",
    // 400 with a bad path, sent on purpose.
    "probe 32980: ! invalid path name count",
    // Step 2: `who --login admin` with a wrong password.
    "win-view-display 34156: stopped: no candidate key matches the client's proof; try --password",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixtures() -> PathBuf {
    root().join("crates/kdx-proto/tests/fixtures")
}

/// File stems in `dir` with `extension`, sorted.
fn stems(dir: &Path, extension: &str) -> BTreeSet<String> {
    std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            (path.extension()? == extension)
                .then(|| path.file_stem().unwrap().to_str().unwrap().to_owned())
        })
        .collect()
}

/// Runs `kdx-dissect` on a capture, exporting its streams to `export`.
fn dissect(capture: &str, export: &Path) -> String {
    let mut command = Command::new(env!("CARGO_BIN_EXE_kdx-dissect"));
    command
        .arg(root().join(format!("captures/{capture}.pcapng")))
        .arg("--export")
        .arg(export);
    for password in PASSWORDS {
        command.args(["--password", password]);
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{capture}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

/// The lines of `output` that report a problem, tagged with the capture and source port.
fn problems(capture: &str, output: &str) -> Vec<String> {
    let mut port = "";
    let mut found = Vec::new();
    for line in output.lines() {
        if let Some(rest) = line
            .strip_prefix("server connection ")
            .or_else(|| line.strip_prefix("tracker connection "))
            .or_else(|| line.strip_prefix("GConsole connection "))
            .or_else(|| line.strip_prefix("DCC connection "))
        {
            let client = rest.split(' ').next().unwrap();
            port = client.rsplit(':').next().unwrap();
            continue;
        }
        let text = line.trim_start();
        let is_message_error = text.starts_with('[')
            && (text.contains(" trailing bytes")
                || text.contains(" bad header ")
                || text
                    .split(' ')
                    .nth(3)
                    .is_some_and(|txid| txid.ends_with(':')));
        if text.starts_with("! ")
            || text.starts_with("stopped: ")
            || text.contains("DOES NOT match")
            || is_message_error
        {
            found.push(format!("{capture} {port}: {text}"));
        }
    }
    found
}

#[test]
fn every_capture_has_notes() {
    let captures = stems(&root().join("captures"), "pcapng");
    let notes = stems(&root().join("captures"), "md");
    assert!(!captures.is_empty());
    assert_eq!(captures, notes);
}

#[test]
fn captures_dissect_cleanly_and_match_fixtures() {
    let export_root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("captures");
    let _ = std::fs::remove_dir_all(&export_root);
    let fixture_names = stems(&fixtures(), "bin");
    let mut matched = BTreeSet::new();
    let mut found = Vec::new();

    // One process per capture, all at once.
    let outputs: Vec<_> = std::thread::scope(|scope| {
        let runs: Vec<_> = stems(&root().join("captures"), "pcapng")
            .into_iter()
            .map(|capture| {
                let export = export_root.join(&capture);
                scope.spawn(move || {
                    std::fs::create_dir_all(&export).unwrap();
                    let output = dissect(&capture, &export);
                    (capture, export, output)
                })
            })
            .collect();
        runs.into_iter().map(|run| run.join().unwrap()).collect()
    });

    for (capture, export, output) in outputs {
        assert!(
            output.contains("  key: "),
            "{capture}: no connection authenticated"
        );
        found.extend(problems(&capture, &output));

        // A fixture is `<capture>-<port>`, or `<capture>` when it has one connection.
        for name in &fixture_names {
            let Some((stem, dir)) = name.rsplit_once('.') else {
                continue;
            };
            let port = match stem.strip_prefix(capture.as_str()) {
                Some("") => None,
                Some(rest) => match rest.strip_prefix('-') {
                    Some(port) if port.parse::<u16>().is_ok() => Some(port),
                    _ => continue,
                },
                None => continue,
            };
            let exported = match port {
                Some(port) => export.join(format!("{port}.{dir}.bin")),
                None => {
                    let mut all = stems(&export, "bin")
                        .into_iter()
                        .filter(|s| s.ends_with(&format!(".{dir}")));
                    let only = all.next().unwrap();
                    assert!(
                        all.next().is_none(),
                        "{name}: capture has several connections"
                    );
                    export.join(format!("{only}.bin"))
                }
            };
            let fixture = std::fs::read(fixtures().join(format!("{name}.bin"))).unwrap();
            let actual = std::fs::read(&exported)
                .unwrap_or_else(|_| panic!("{name}: no such connection in {capture}"));
            assert!(fixture == actual, "{name}: differs from {capture}");
            matched.insert(name.clone());
        }
    }

    // UDP registrations are not exported, so find them in the capture as sent.
    let tracker = std::fs::read(root().join("captures/linux-tracker.pcapng")).unwrap();
    for name in &fixture_names {
        if name.starts_with("linux-tracker-reg-") {
            let datagram = std::fs::read(fixtures().join(format!("{name}.bin"))).unwrap();
            assert!(
                tracker.windows(datagram.len()).any(|w| w == datagram),
                "{name}: not in linux-tracker"
            );
            matched.insert(name.clone());
        }
    }

    let unmatched: Vec<_> = fixture_names.difference(&matched).collect();
    assert!(
        unmatched.is_empty(),
        "fixtures with no capture: {unmatched:?}"
    );
    assert_eq!(found, EXPECTED);
}
