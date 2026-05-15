//! Tests for the four output modes:
//!   1. Crate root         (no path, no filter)        → `// showing mod {name} (crate root)`
//!   2. Path navigation    (path, no filter)           → `// found {kind} {full_path}`
//!   3. Filter single      (filter resolves uniquely)  → `// found {kind} {path}`
//!   4. Filter multi       (multiple matches)          → `// N items matching "{filter}"`
//!   5. Filter no-match    (zero matches)              → `// no matches for "{filter}" — showing all N items`

mod common;

use common::run_cli;
use insta::assert_snapshot;

#[test]
fn mode_crate_root() {
    let (stdout, stderr, success) = run_cli(&["anyhow@1.0.99"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert!(stdout.starts_with("// showing mod anyhow (crate root)"));
    // Crate-root listing uses `as_module_child` rendering (module-relative,
    // not fully-qualified) — see `list_item.rs::as_module_child`.
    for needle in [
        "pub struct Error",
        "pub trait Context",
        "pub type Result",
        "pub macro anyhow!",
    ] {
        assert!(
            stdout.contains(needle),
            "expected crate-root body to contain `{needle}`; got:\n{stdout}"
        );
    }
}

#[test]
fn mode_path_navigation() {
    let (stdout, stderr, success) = run_cli(&["anyhow@1.0.99::Context"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(
        stdout.lines().next().unwrap(),
        @"// found trait anyhow::Context"
    );
    assert!(stdout.contains("pub trait anyhow::Context"));
}

#[test]
fn mode_filter_single_via_exact_suffix() {
    // `Error` is suffix-unique within anyhow → filter returns just that one item.
    let (stdout, stderr, success) = run_cli(&["anyhow@1.0.99", "Error"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert!(
        stdout.starts_with("// found struct anyhow::Error"),
        "expected single-result description; got:\n{stdout}"
    );
    assert!(stdout.contains("pub struct anyhow::Error"));
}

#[test]
fn mode_filter_multi() {
    // `context` matches `Context::context` and `Context::with_context` →
    // exact-suffix returns 2, falls through to substring (also 2) → list mode.
    let (stdout, stderr, success) = run_cli(&["anyhow@1.0.99", "context"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    // 2 items matching "context"

    fn anyhow::Context::context
    fn anyhow::Context::with_context
    "#);
}

#[test]
fn mode_filter_no_match_falls_back_to_full_list() {
    let (stdout, stderr, success) = run_cli(&["anyhow@1.0.99", "Zzzzz_no_such_thing"]);
    assert!(
        success,
        "CLI should succeed (no results is not an error): {stderr}"
    );
    assert_snapshot!(stdout, @r#"
    // no matches for "Zzzzz_no_such_thing" — showing all 12 items

    mod anyhow
    struct anyhow::Chain
    trait anyhow::Context
    fn anyhow::Context::context
    fn anyhow::Context::with_context
    struct anyhow::Error
    fn anyhow::Ok
    type anyhow::Result
    macro anyhow::anyhow!
    macro anyhow::bail!
    macro anyhow::ensure!
    macro anyhow::format_err!
    "#);
}
