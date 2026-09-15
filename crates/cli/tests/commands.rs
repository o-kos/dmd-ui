use std::process::Command;

#[test]
fn every_command_reports_not_implemented() {
    for command in ["run", "check", "info", "list", "mask", "convert", "record"] {
        let output = Command::new(env!("CARGO_BIN_EXE_dmd"))
            .arg(command)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains(&format!("{command}: not implemented"))
        );
    }
}

#[test]
fn help_succeeds_and_lists_the_surface() {
    let output = Command::new(env!("CARGO_BIN_EXE_dmd"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8_lossy(&output.stdout);
    for option in [
        "run",
        "check",
        "info",
        "list",
        "mask",
        "convert",
        "record",
        "--search-path",
        "--backend",
        "--events",
    ] {
        assert!(help.contains(option));
    }
}
