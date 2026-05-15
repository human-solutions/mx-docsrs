//! Tests for version resolution behavior.
//!
//! When an explicit version is given, no resolution header is emitted — the
//! output begins directly with the description line.
//!
//! Assertions on the header only (not body) — body contents are too volatile
//! to snapshot for these tests.

mod common;

use common::run_cli;

#[test]
fn explicit_exact_version_emits_no_resolution_header() {
    // When the user pins an exact version, lib.rs skips VersionResolver
    // and the first output line is the description, not a resolution comment.
    let (stdout, stderr, success) = run_cli(&["anyhow@1.0.99"]);
    assert!(success, "CLI should succeed: {stderr}");
    let first = stdout.lines().next().unwrap_or("");
    assert!(
        first.starts_with("// showing mod anyhow"),
        "expected first line to be description, got: {first:?}"
    );
}

#[test]
fn caret_version_requirement_is_passed_through() {
    // docs.rs accepts `^1` in the URL and serves a compatible build.
    // We only assert that the CLI succeeds — body content varies by which
    // version docs.rs resolves the requirement to.
    let (stdout, stderr, success) = run_cli(&["anyhow@^1"]);
    assert!(success, "CLI should succeed for `^1` requirement: {stderr}");
    let first = stdout.lines().next().unwrap_or("");
    assert!(
        first.starts_with("// showing mod anyhow"),
        "expected anyhow crate-root output, got: {first:?}"
    );
}

#[test]
fn unknown_crate_fails_with_not_found_message() {
    let (stdout, stderr, success) = run_cli(&["this_crate_definitely_does_not_exist_xyz_2026"]);
    assert!(!success, "CLI should fail for unknown crate");
    assert!(stdout.is_empty());
    assert!(
        stderr.contains(
            "Crate 'this_crate_definitely_does_not_exist_xyz_2026@latest' not found on docs.rs."
        ),
        "stderr did not match expected not-found message:\n{stderr}"
    );
}

#[test]
fn unknown_version_fails_with_not_found_message() {
    let (stdout, stderr, success) = run_cli(&["anyhow@99.99.99"]);
    assert!(!success, "CLI should fail for unknown version");
    assert!(stdout.is_empty());
    assert!(
        stderr.contains("Crate 'anyhow@99.99.99' not found on docs.rs."),
        "stderr did not match expected not-found message:\n{stderr}"
    );
}
