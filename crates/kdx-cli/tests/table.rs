//! Runs `kdx table`.

use std::process::{Command, Output};

use kdx_proto::Story;

fn kdx(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kdx"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn rejects_server_options() {
    for option in ["--server", "--login", "--password", "--nick", "--table"] {
        let output = kdx(&["table", "path", option, "x"]);
        assert!(!output.status.success(), "{option} was accepted");
    }
    let help = String::from_utf8(kdx(&["table", "fetch", "--help"]).stdout).unwrap();
    assert!(!help.contains("--server"), "{help}");
}

#[test]
fn server_commands_take_server_options() {
    let help = String::from_utf8(kdx(&["who", "--help"]).stdout).unwrap();
    assert!(
        help.contains("--server") && help.contains("--table"),
        "{help}"
    );
}

#[test]
fn fetch_keeps_cached_tables() {
    let (table, gconsole) = (
        kdx_test_tables::table(Story::Kdx),
        kdx_test_tables::table(Story::Hgc),
    );
    let cache = std::env::temp_dir().join(format!("kdx-table-test-{}", std::process::id()));
    // The KDX table is at its legacy path, to be moved; the stale legacy GConsole table is removed.
    std::fs::create_dir_all(cache.join("kdx/table")).unwrap();
    std::fs::write(cache.join("kdx/challenge-table.bin"), table.as_bytes()).unwrap();
    std::fs::write(cache.join("kdx/table/hgc.bin"), gconsole.as_bytes()).unwrap();
    std::fs::write(cache.join("kdx/gconsole-table.bin"), b"stale").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kdx"))
        .args(["table", "fetch"])
        .env("XDG_CACHE_HOME", &cache)
        .output()
        .unwrap();
    let moved = std::fs::read(cache.join("kdx/table/kdx.bin")).ok();
    let left: Vec<_> = ["challenge-table.bin", "gconsole-table.bin"]
        .into_iter()
        .filter(|f| cache.join("kdx").join(f).exists())
        .collect();
    std::fs::remove_dir_all(&cache).unwrap();
    let (stdout, stderr) = (
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(output.status.success(), "{stderr}");
    assert_eq!(stdout.matches("already saved").count(), 2, "{stdout}");
    assert!(!stderr.contains("downloading"), "{stderr}");
    assert_eq!(moved.as_deref(), Some(table.as_bytes()));
    assert!(left.is_empty(), "legacy files left: {left:?}");
}
