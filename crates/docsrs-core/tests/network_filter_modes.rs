//! Tests for filter precedence and path-prefix scoping.
//!
//! Filter precedence (see `filter_list` in `lib.rs:235`):
//!   1. Exact suffix match → if exactly 1 result, return it as single.
//!   2. Otherwise substring match → return list of matches.
//!   3. If both yield zero, fall through to "no matches" mode.
//!
//! Path-prefix scoping (see `filter_by_path_prefix`) narrows the candidate
//! set BEFORE the filter precedence runs.

mod common;

use common::run_cli;
use insta::assert_snapshot;

#[test]
fn exact_suffix_unique_returns_single_result() {
    // `Error` is suffix-unique within anyhow's public items.
    let (stdout, stderr, success) = run_cli(&["anyhow@1.0.99", "Error"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert!(
        stdout.starts_with("// found struct anyhow::Error"),
        "expected single-result mode; got:\n{stdout}"
    );
}

#[test]
fn substring_with_multiple_matches_returns_list() {
    // `context` is suffix-shared by `Context::context` and `Context::with_context`
    // and substring-shared identically — falls through to list mode.
    let (stdout, stderr, success) = run_cli(&["anyhow@1.0.99", "context"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    // 2 items matching "context"

    fn anyhow::Context::context
    fn anyhow::Context::with_context
    "#);
}

#[test]
fn path_prefix_scopes_filter_to_module() {
    // Filter `channel` inside `tokio::sync::mpsc` finds `channel` and `unbounded_channel`,
    // and nothing else from outside that module.
    let (stdout, stderr, success) = run_cli(&["tokio@1.40.5::sync::mpsc", "channel"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    // 2 items matching "channel"

    fn tokio::sync::mpsc::channel
    fn tokio::sync::mpsc::unbounded_channel
    "#);
}

#[test]
fn filter_returns_substring_when_suffix_not_unique() {
    // `from_str` matches `serde_json::de::from_str` and `serde_json::from_str` —
    // ends_with returns 2, substring returns 2, list mode.
    let (stdout, stderr, success) = run_cli(&["serde_json@1.0.149", "from_str"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert!(
        stdout.starts_with("// 2 items matching \"from_str\""),
        "expected 2-item list mode; got:\n{stdout}"
    );
    assert!(stdout.contains("fn serde_json::from_str"));
    assert!(stdout.contains("fn serde_json::de::from_str"));
}

#[test]
fn no_match_falls_back_to_full_list_for_module() {
    let (stdout, stderr, success) = run_cli(&["anyhow@1.0.99", "Definitely_Not_There_xyz"]);
    assert!(
        success,
        "CLI should succeed (no-match is not an error): {stderr}"
    );
    assert!(
        stdout.starts_with("// no matches for \"Definitely_Not_There_xyz\" — showing all "),
        "expected no-match fallback header; got:\n{stdout}"
    );
    // The fallback shows the full sorted list — assert at least the crate root mod is present.
    assert!(stdout.contains("mod anyhow"));
}
