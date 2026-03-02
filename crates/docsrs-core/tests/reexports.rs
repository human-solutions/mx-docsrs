mod common;

use common::run_cli;
use insta::assert_snapshot;

// --- Simple re-exports ---

#[test]
fn simple_struct_reexport() {
    let (stdout, stderr, success) = run_cli(&["test-reexports", "InnerStruct"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    // version 0.1.0 (local)

    // 2 items matching "InnerStruct"
    struct test_reexports::InnerStruct
    struct test_reexports::reexported::InnerStruct
    "#);
}

#[test]
fn simple_enum_reexport() {
    let (stdout, stderr, success) = run_cli(&["test-reexports", "InnerEnum"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    // version 0.1.0 (local)

    // 2 items matching "InnerEnum"
    enum test_reexports::InnerEnum
    enum test_reexports::reexported::InnerEnum
    "#);
}

#[test]
fn simple_function_reexport() {
    let (stdout, stderr, success) = run_cli(&["test-reexports", "inner_function"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    // version 0.1.0 (local)

    // 2 items matching "inner_function"
    fn test_reexports::inner_function
    fn test_reexports::reexported::inner_function
    "#);
}

// --- Renamed re-exports ---

#[test]
fn renamed_struct_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-reexports", "RenamedStruct"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)

    /// A struct defined in inner module
    pub struct test_reexports::ChainedReexport {
        pub field: String,
    }
    ");
}

// --- Deeply nested re-exports ---

#[test]
fn deeply_nested_reexport() {
    let (stdout, stderr, success) = run_cli(&["test-reexports", "DeeplyNestedItem"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)

    /// A deeply nested struct
    pub struct test_reexports::DeeplyNestedItem {
        pub value: usize,
    }
    ");
}

// --- Selective re-exports ---

#[test]
fn selective_foo_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-reexports::selective", "Foo"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)

    pub struct test_reexports::selective::Foo
    ");
}

#[test]
fn selective_bar_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-reexports::selective", "Bar"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)

    pub struct test_reexports::selective::Bar
    ");
}

#[test]
fn selective_baz_not_found() {
    let (stdout, _stderr, success) = run_cli(&["test-reexports::selective", "Baz"]);
    assert!(success, "CLI should succeed (no results is not an error)");
    assert_snapshot!(stdout, @r#"
    // version 0.1.0 (local)

    // no matches for "Baz" — showing all 3 items
    mod test_reexports::selective
    struct test_reexports::selective::Bar
    struct test_reexports::selective::Foo
    "#);
}

// --- Trait re-exports ---

#[test]
fn trait_reexport() {
    let (stdout, stderr, success) = run_cli(&["test-reexports", "MyTrait"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    // version 0.1.0 (local)

    // 4 items matching "MyTrait"
    trait test_reexports::MyTrait
    fn test_reexports::MyTrait::do_something
    trait test_reexports::traits::MyTrait
    fn test_reexports::traits::MyTrait::do_something
    "#);
}

#[test]
fn trait_impl_reexport() {
    let (stdout, stderr, success) = run_cli(&["test-reexports", "TraitImpl"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    // version 0.1.0 (local)

    // 2 items matching "TraitImpl"
    struct test_reexports::TraitImpl
    struct test_reexports::traits::TraitImpl
    "#);
}
