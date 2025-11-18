use std::process::Command;

fn run_cli(args: &[&str]) -> (String, String, bool) {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .args(args)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let success = output.status.success();

    (stdout, stderr, success)
}

#[test]
fn test_cli_with_explicit_version() {
    let (stdout, _stderr, success) = run_cli(&["tokio@1.0.0", "spawn"]);
    assert!(success, "CLI should succeed");
    assert!(stdout.contains("tokio @ 1.0.0"), "Should use explicit version");
    assert!(stdout.contains("Symbol: spawn"), "Should show symbol");
}

#[test]
fn test_cli_with_crate_in_dependencies() {
    let (stdout, _stderr, success) = run_cli(&["clap", "Parser"]);
    assert!(success, "CLI should succeed");
    assert!(stdout.contains("clap @"), "Should show crate name");
    assert!(stdout.contains("4."), "Should resolve to version 4.x");
    assert!(!stdout.contains("^"), "Should not contain requirement characters");
    assert!(stdout.contains("Symbol: Parser"), "Should show symbol");
}

#[test]
fn test_cli_with_unknown_crate() {
    let (stdout, _stderr, success) = run_cli(&["some_unknown_crate", "symbol"]);
    assert!(success, "CLI should succeed");
    assert!(stdout.contains("some_unknown_crate @ latest"), "Should default to latest");
    assert!(stdout.contains("Symbol: symbol"), "Should show symbol");
}

#[test]
fn test_cli_missing_arguments() {
    let (stdout, stderr, success) = run_cli(&["clap"]);
    assert!(!success, "CLI should fail with missing arguments");
    let output = format!("{}{}", stdout, stderr);
    assert!(
        output.contains("required arguments were not provided") || output.contains("SYMBOL"),
        "Should show error about missing symbol argument"
    );
}

#[test]
fn test_cli_empty_crate_name() {
    let (stdout, stderr, success) = run_cli(&["", "symbol"]);
    assert!(!success, "CLI should fail with empty crate name");
    let output = format!("{}{}", stdout, stderr);
    assert!(output.contains("empty"), "Should show error about empty name");
}

#[test]
fn test_cli_empty_version() {
    let (stdout, stderr, success) = run_cli(&["crate@", "symbol"]);
    assert!(!success, "CLI should fail with empty version");
    let output = format!("{}{}", stdout, stderr);
    assert!(output.contains("empty"), "Should show error about empty version");
}

#[test]
fn test_cli_help() {
    let (stdout, stderr, success) = run_cli(&["--help"]);
    let output = format!("{}{}", stdout, stderr);
    assert!(success, "Help should succeed");
    assert!(output.contains("docsrs"), "Should mention binary name");
    assert!(output.contains("CRATE_SPEC"), "Should mention crate spec argument");
    assert!(output.contains("SYMBOL"), "Should mention symbol argument");
    assert!(
        output.contains("@version") || output.contains("optionally"),
        "Should mention optional version syntax"
    );
}

#[test]
fn test_cli_resolves_cargo_metadata_dependency() {
    let (stdout, _stderr, success) = run_cli(&["cargo_metadata", "MetadataCommand"]);
    assert!(success, "CLI should succeed");
    assert!(stdout.contains("cargo_metadata @"), "Should show crate name");
    assert!(stdout.contains("0."), "Should resolve to version 0.x");
    assert!(!stdout.contains("latest"), "Should not use latest");
}

#[test]
fn test_cli_resolves_anyhow_dependency() {
    let (stdout, _stderr, success) = run_cli(&["anyhow", "Error"]);
    assert!(success, "CLI should succeed");
    assert!(stdout.contains("anyhow @"), "Should show crate name");
    assert!(stdout.contains("1."), "Should resolve to version 1.x");
    assert!(!stdout.contains("latest"), "Should not use latest");
}

#[test]
fn test_cli_complex_version_requirement() {
    let (stdout, _stderr, success) = run_cli(&["serde@>=1.0,<2.0", "Serialize"]);
    assert!(success, "CLI should succeed");
    assert!(
        stdout.contains("serde @ >=1.0,<2.0"),
        "Should preserve complex version requirement"
    );
}
