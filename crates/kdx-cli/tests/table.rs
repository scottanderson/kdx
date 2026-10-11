//! Runs `kdx table`.

use std::process::{Command, Output};

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use kdx_client::table;
use kdx_proto::ChallengeTable;

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
fn fetch_keeps_a_cached_table() {
    // The table is not in the repository; skip unless the environment or `kdx table fetch` provide it.
    let Some(table) = env_table().or_else(table::cached) else {
        let missing = "no challenge table in KDX_CHALLENGE_TABLE or the kdx cache";
        // CI sets KDX_REQUIRE_TABLE so a missing table fails instead of skipping.
        assert!(std::env::var_os("KDX_REQUIRE_TABLE").is_none(), "{missing}");
        eprintln!("skipping: {missing}");
        return;
    };
    let cache = std::env::temp_dir().join(format!("kdx-table-test-{}", std::process::id()));
    std::fs::create_dir_all(cache.join("kdx")).unwrap();
    std::fs::write(cache.join("kdx/challenge-table.bin"), table.as_bytes()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kdx"))
        .args(["table", "fetch"])
        .env("XDG_CACHE_HOME", &cache)
        .output()
        .unwrap();
    std::fs::remove_dir_all(&cache).unwrap();
    let (stdout, stderr) = (
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(output.status.success(), "{stderr}");
    assert!(stdout.contains("already saved"), "{stdout}");
    assert!(!stderr.contains("downloading"), "{stderr}");
}

/// The table in `KDX_CHALLENGE_TABLE`, as base64, which CI sets from a secret.
fn env_table() -> Option<ChallengeTable> {
    let b64 = std::env::var("KDX_CHALLENGE_TABLE")
        .ok()
        .filter(|v| !v.trim().is_empty())?;
    let bytes = STANDARD
        .decode(b64.trim())
        .expect("KDX_CHALLENGE_TABLE is not base64");
    Some(ChallengeTable::from_bytes(&bytes).unwrap())
}
