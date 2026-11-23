//! `--inventory` prints the `## Messages` section that each lab capture's notes end with.

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn inventory(capture: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_kdx-dissect"))
        .arg(root().join(format!("captures/{capture}.pcapng")))
        .args([
            "--inventory",
            "--password",
            "openkdx-lab",
            "--password",
            "wrong-password",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{capture}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

/// The fenced block under `## Messages` in a capture's notes.
fn notes_inventory(capture: &str) -> String {
    let notes = std::fs::read_to_string(root().join(format!("captures/{capture}.md"))).unwrap();
    let (_, section) = notes
        .split_once("## Messages\n")
        .expect("a Messages section");
    let section = section.split("\n## ").next().unwrap();
    let fence = section.find("```").expect("a fenced block");
    section[fence..].trim_end().to_owned()
}

/// The captures whose notes carry an inventory.
const LAB_CAPTURES: [&str; 15] = [
    "win-chat-basic",
    "win-files-basic",
    "win-first-connect",
    "win-guest-login",
    "win-news-basic",
    "win-pm-basic",
    "win-tracker-basic",
    "cli-accounts-classes",
    "cli-broadcast",
    "cli-connection-monitor",
    "cli-disconnect-ban",
    "cli-folder-access",
    "cli-server-history",
    "cli-server-settings",
    "cli-upload-resume",
];

#[test]
fn inventory_matches_the_notes_of_every_lab_capture() {
    for stem in LAB_CAPTURES {
        assert_eq!(inventory(stem).trim_end(), notes_inventory(stem), "{stem}");
    }
}

#[test]
fn inventory_is_one_line_per_message_in_frame_order() {
    let text = inventory("win-chat-basic");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], "```");
    assert_eq!(
        lines[1],
        "4  C->S  127.0.0.1:37182 -> 127.0.0.1:10700  hello"
    );
    assert!(lines.contains(&"16  S->C  127.0.0.1:37182 -> 127.0.0.1:10700  215 Chat List (reply)"));
    assert_eq!(*lines.last().unwrap(), "```");
    let frames: Vec<usize> = lines[1..lines.len() - 1]
        .iter()
        .map(|l| l.split("  ").next().unwrap().parse().unwrap())
        .collect();
    assert!(frames.windows(2).all(|w| w[0] <= w[1]));
}
