//! Tests covering every item kind that docsrs renders.
//!
//! ProcMacro, Variant, TraitAlias, StructField, Use, Union, AssocConst,
//! ExternType, Impl, AssocType are intentionally filtered out
//! (see `EntryKind::from_item_enum` in `list/list_item.rs`).
//!
//! `const` and `static` kinds are NOT covered here — the natural candidate
//! crate (`libc`) is too large for the docsrs build to process within a
//! reasonable test timeout. The list/path-navigation pipeline for these
//! kinds is exercised indirectly via the local `test-visibility` crate
//! (`tests/visibility.rs::public_const_is_found`).

mod common;

use common::run_cli;
use insta::assert_snapshot;

#[test]
fn struct_kind() {
    let (stdout, stderr, success) = run_cli(&["anyhow@1.0.99::Error"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout.lines().next().unwrap(), @"// found struct anyhow::Error");
    assert!(stdout.contains("pub struct anyhow::Error"));
    assert!(stdout.contains("pub fn new"));
}

#[test]
fn enum_kind() {
    let (stdout, stderr, success) = run_cli(&["serde_json@1.0.149::Value"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout.lines().next().unwrap(), @"// found enum serde_json::Value");
    assert!(stdout.contains("pub enum serde_json::Value"));
}

#[test]
fn trait_kind() {
    let (stdout, stderr, success) = run_cli(&["anyhow@1.0.99::Context"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout.lines().next().unwrap(), @"// found trait anyhow::Context");
    assert!(stdout.contains("pub trait anyhow::Context"));
    assert!(stdout.contains("fn context"));
    assert!(stdout.contains("fn with_context"));
}

#[test]
fn function_kind() {
    let (stdout, stderr, success) = run_cli(&["tokio@1.40.5::spawn"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout.lines().next().unwrap(), @"// found fn tokio::spawn");
    assert!(stdout.contains("pub fn tokio::spawn"));
    assert!(stdout.contains("JoinHandle"));
}

#[test]
fn type_alias_kind() {
    let (stdout, stderr, success) = run_cli(&["anyhow@1.0.99::Result"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout.lines().next().unwrap(), @"// found type anyhow::Result");
    assert!(stdout.contains("pub type anyhow::Result"));
}

#[test]
fn macro_kind() {
    let (stdout, stderr, success) = run_cli(&["lazy_static@1.5.0::lazy_static"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout.lines().next().unwrap(), @"// found macro lazy_static::lazy_static");
    assert!(stdout.contains("pub macro lazy_static::lazy_static!"));
}

#[test]
fn module_kind() {
    let (stdout, stderr, success) = run_cli(&["tokio@1.40.5::sync"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout.lines().next().unwrap(), @"// found mod tokio::sync");
}
