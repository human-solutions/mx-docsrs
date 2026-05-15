//! End-to-end CrateSpec parsing tests against real docs.rs.
//!
//! Uses pinned versions of external crates. Bumping a version requires
//! re-running and re-snapshotting — don't do it casually.

mod common;

use common::run_cli;
use insta::assert_snapshot;

#[test]
fn crate_only_resolves_to_crate_root() {
    let (stdout, stderr, success) = run_cli(&["anyhow@1.0.99"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert!(
        stdout.starts_with("// showing mod anyhow (crate root)"),
        "expected crate-root description; got:\n{stdout}"
    );
    // Crate-root listing uses module-relative form: `pub struct Error`.
    assert!(
        stdout.contains("pub struct Error"),
        "expected `pub struct Error` in crate-root body"
    );
}

#[test]
fn crate_with_path_navigates_to_item() {
    let (stdout, stderr, success) = run_cli(&["anyhow@1.0.99::Error"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert!(
        stdout.starts_with("// found struct anyhow::Error"),
        "expected struct description; got:\n{stdout}"
    );
    assert!(stdout.contains("pub struct anyhow::Error"));
}

#[test]
fn hyphenated_name_is_normalized_via_alternate_retry() {
    // `serde-json` is published as `serde_json` on docs.rs. The hyphen form
    // 404s on the first fetch and the alternate-name retry succeeds.
    let (stdout, stderr, success) = run_cli(&["serde-json@1.0.149"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert!(
        stdout.starts_with("// showing mod serde_json (crate root)"),
        "expected crate-root description for serde_json; got:\n{stdout}"
    );
}

#[test]
fn hyphenated_name_with_path_navigates() {
    let (stdout, stderr, success) = run_cli(&["serde_json@1.0.149::Value"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert!(
        stdout.starts_with("// found enum serde_json::Value"),
        "expected enum description; got:\n{stdout}"
    );
    // Verify body shows the enum and its discriminant variants.
    assert!(stdout.contains("pub enum serde_json::Value"));
    for variant in ["Null", "Bool", "Number", "String", "Array", "Object"] {
        assert!(
            stdout.contains(variant),
            "expected variant `{variant}` in Value enum body"
        );
    }
}

#[test]
fn deep_path_navigates_into_nested_module() {
    let (stdout, stderr, success) = run_cli(&["tokio@1.40.5::sync::mpsc::channel"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(
        stdout.lines().next().unwrap(),
        @"// found fn tokio::sync::mpsc::channel"
    );
    assert!(stdout.contains("pub fn tokio::sync::mpsc::channel"));
}

#[test]
fn trailing_double_colon_is_trimmed() {
    // `anyhow@1.0.99::` should behave identically to `anyhow@1.0.99` because
    // CrateSpec trims trailing `::` and treats empty path as None.
    let (stdout, stderr, success) = run_cli(&["anyhow@1.0.99::"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert!(
        stdout.starts_with("// showing mod anyhow (crate root)"),
        "expected crate-root description after trailing-:: trim; got:\n{stdout}"
    );
}
