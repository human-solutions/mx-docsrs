mod common;

use common::run_cli;
use insta::assert_snapshot;

#[test]
fn empty_crate_name_fails() {
    let (stdout, stderr, success) = run_cli(&["", "symbol"]);
    assert!(!success, "CLI should fail with empty crate name");
    assert!(stdout.is_empty());
    assert_snapshot!(stderr, @r#"
    error: invalid value '' for '[CRATE_SPEC]': Crate name cannot be empty

    For more information, try '--help'.
    "#);
}

#[test]
fn empty_version_fails() {
    let (stdout, stderr, success) = run_cli(&["crate@", "symbol"]);
    assert!(!success, "CLI should fail with empty version");
    assert!(stdout.is_empty());
    assert_snapshot!(stderr, @r#"
    error: invalid value 'crate@' for '[CRATE_SPEC]': Version cannot be empty after '@'

    For more information, try '--help'.
    "#);
}

#[test]
fn help_shows_usage() {
    let (stdout, stderr, success) = run_cli(&["--help"]);
    assert!(success, "Help should succeed");
    assert!(stderr.is_empty());
    assert_snapshot!(stdout);
}
