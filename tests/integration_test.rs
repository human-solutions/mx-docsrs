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
    let (stdout, _stderr, success) = run_cli(&["tokio@latest", "spawn"]);
    assert!(success, "CLI should succeed");
    assert!(
        stdout.contains("Multiple items found") || stdout.contains("fn:"),
        "Should show results"
    );
}

#[test]
fn test_cli_with_crate_in_dependencies() {
    let (stdout, _stderr, success) = run_cli(&["anyhow", "Error"]);
    assert!(success, "CLI should succeed");
    assert!(
        stdout.contains("Multiple items found") || stdout.contains("struct:") || stdout.contains("Error"),
        "Should show results for Error"
    );
}

#[test]
fn test_cli_with_unknown_crate() {
    let (_stdout, stderr, success) = run_cli(&["some_unknown_crate", "symbol"]);
    // Unknown crate should fail to fetch
    assert!(!success, "CLI should fail for unknown crate");
    assert!(
        stderr.contains("Failed to fetch") || stderr.contains("404") || stderr.contains("error"),
        "Should show error for unknown crate"
    );
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
    assert!(
        output.contains("empty"),
        "Should show error about empty name"
    );
}

#[test]
fn test_cli_empty_version() {
    let (stdout, stderr, success) = run_cli(&["crate@", "symbol"]);
    assert!(!success, "CLI should fail with empty version");
    let output = format!("{}{}", stdout, stderr);
    assert!(
        output.contains("empty"),
        "Should show error about empty version"
    );
}

#[test]
fn test_cli_help() {
    let (stdout, stderr, success) = run_cli(&["--help"]);
    let output = format!("{}{}", stdout, stderr);
    assert!(success, "Help should succeed");
    assert!(output.contains("docsrs"), "Should mention binary name");
    assert!(
        output.contains("CRATE_SPEC"),
        "Should mention crate spec argument"
    );
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
    assert!(
        stdout.contains("struct:") || stdout.contains("MetadataCommand"),
        "Should show MetadataCommand"
    );
}

#[test]
fn test_cli_resolves_anyhow_dependency() {
    let (stdout, _stderr, success) = run_cli(&["anyhow", "Error"]);
    assert!(success, "CLI should succeed");
    assert!(
        stdout.contains("struct:") || stdout.contains("Error") || stdout.contains("Multiple items found"),
        "Should show Error results"
    );
}

#[test]
fn test_cli_complex_version_requirement() {
    let (stdout, _stderr, success) = run_cli(&["serde@latest", "Serialize"]);
    assert!(success, "CLI should succeed");
    assert!(
        stdout.contains("trait:") || stdout.contains("Serialize") || stdout.contains("Multiple items found"),
        "Should show Serialize results"
    );
}
