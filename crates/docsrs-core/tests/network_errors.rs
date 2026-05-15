//! Error-path tests.

mod common;

use common::run_cli;
use insta::assert_snapshot;

#[test]
fn unknown_crate_returns_clear_error() {
    let (stdout, stderr, success) = run_cli(&["this_crate_definitely_does_not_exist_xyz_2026"]);
    assert!(!success, "CLI should fail for unknown crate");
    assert!(stdout.is_empty());
    assert_snapshot!(
        stderr,
        @"Crate 'this_crate_definitely_does_not_exist_xyz_2026@latest' not found on docs.rs. Check the crate name and version."
    );
}

#[test]
fn unknown_version_returns_clear_error() {
    let (stdout, stderr, success) = run_cli(&["anyhow@99.99.99"]);
    assert!(!success, "CLI should fail for unknown version");
    assert!(stdout.is_empty());
    assert_snapshot!(
        stderr,
        @"Crate 'anyhow@99.99.99' not found on docs.rs. Check the crate name and version."
    );
}

#[test]
fn unknown_path_returns_no_item_found() {
    let (stdout, stderr, success) = run_cli(&["anyhow@1.0.99::NonexistentItem"]);
    assert!(!success, "CLI should fail when path doesn't resolve");
    assert!(stdout.is_empty());
    assert!(
        stderr.contains("No item found at anyhow::NonexistentItem"),
        "stderr did not contain expected message:\n{stderr}"
    );
}

#[test]
fn unknown_deep_path_returns_no_item_found() {
    // Path navigation only resolves top-level items, so even an existing method
    // is not reachable via `Error::new` — and a clearly bogus method behaves the same.
    let (stdout, stderr, success) = run_cli(&["anyhow@1.0.99::Error::nonexistent_method"]);
    assert!(!success, "CLI should fail for deep nonexistent path");
    assert!(stdout.is_empty());
    assert!(
        stderr.contains("No item found at anyhow::Error::nonexistent_method"),
        "stderr did not contain expected message:\n{stderr}"
    );
}

#[test]
fn missing_crate_name_with_version_fails() {
    let (stdout, stderr, success) = run_cli(&["@1.0.0"]);
    assert!(!success, "CLI should fail when crate name is missing");
    assert!(stdout.is_empty());
    assert!(
        stderr.contains("Crate name cannot be empty"),
        "stderr did not contain expected validation error:\n{stderr}"
    );
}
