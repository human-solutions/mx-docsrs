//! Tests for command-line flags: --color (auto/always/never/invalid) and --no-cache.
//!
//! `--version` is not implemented in this CLI (the `Cli` struct has no
//! `#[command(version)]` attribute), so there's no test for it here.
//!
//! NOTE: The color tests mutate the process-global `colored::control` override.
//! They use `run_cli_raw` (which does NOT force colors off) and explicitly
//! reset the override at the end so other tests in this binary aren't affected.

mod common;

use common::{run_cli, run_cli_raw};

#[test]
fn color_never_produces_no_ansi_escapes() {
    // Reset override to auto so --color=never can take effect.
    colored::control::unset_override();
    let (stdout, _stderr, success) = run_cli_raw(&["--color=never", "anyhow@1.0.99"]);
    colored::control::set_override(false);
    assert!(success, "CLI should succeed with --color=never");
    assert!(
        !stdout.contains('\x1b'),
        "stdout should contain no ANSI escapes with --color=never"
    );
}

#[test]
fn color_always_produces_ansi_escapes() {
    colored::control::unset_override();
    let (stdout, _stderr, success) = run_cli_raw(&["--color=always", "anyhow@1.0.99"]);
    colored::control::set_override(false);
    assert!(success, "CLI should succeed with --color=always");
    assert!(
        stdout.contains('\x1b'),
        "stdout should contain ANSI escapes with --color=always"
    );
}

#[test]
fn color_invalid_value_fails() {
    let (stdout, stderr, success) = run_cli(&["--color=invalid", "anyhow"]);
    assert!(!success, "invalid --color value should fail");
    assert!(stdout.is_empty());
    assert!(
        stderr.contains("Invalid color option: invalid"),
        "stderr did not contain expected error:\n{stderr}"
    );
}

#[test]
fn no_cache_flag_still_succeeds() {
    let (stdout, stderr, success) = run_cli(&["--no-cache", "anyhow@1.0.99::Error"]);
    assert!(success, "CLI should succeed with --no-cache: {stderr}");
    assert!(stdout.starts_with("// found struct anyhow::Error"));
}
