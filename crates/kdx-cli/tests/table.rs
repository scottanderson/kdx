//! Runs `kdx table`.

use std::process::{Command, Output};

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
