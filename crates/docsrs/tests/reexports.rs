mod common;

use common::run_cli;
use insta::assert_snapshot;

// --- Simple re-exports ---

#[test]
fn simple_struct_reexport() {
    let (stdout, stderr, success) = run_cli(&["test-reexports", "InnerStruct"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    struct crate::InnerStruct
    struct crate::reexported::InnerStruct
    "#);
}

#[test]
fn simple_enum_reexport() {
    let (stdout, stderr, success) = run_cli(&["test-reexports", "InnerEnum"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    enum   crate::InnerEnum
    enum   crate::reexported::InnerEnum
    "#);
}

#[test]
fn simple_function_reexport() {
    let (stdout, stderr, success) = run_cli(&["test-reexports", "inner_function"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    fn     crate::inner_function
    fn     crate::reexported::inner_function
    "#);
}

// --- Renamed re-exports ---

#[test]
fn renamed_struct_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-reexports", "RenamedStruct"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    pub struct crate::ChainedReexport

    A struct defined in inner module

    Fields:
      pub field: alloc::string::String
    "#);
}

// --- Deeply nested re-exports ---

#[test]
fn deeply_nested_reexport() {
    let (stdout, stderr, success) = run_cli(&["test-reexports", "DeeplyNestedItem"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    pub struct crate::DeeplyNestedItem

    A deeply nested struct

    Fields:
      pub value: usize
    "#);
}

// --- Selective re-exports ---

#[test]
fn selective_foo_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-reexports::selective", "Foo"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @"pub struct crate::selective::Foo");
}

#[test]
fn selective_bar_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-reexports::selective", "Bar"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @"pub struct crate::selective::Bar");
}

#[test]
fn selective_baz_not_found() {
    let (stdout, _stderr, success) = run_cli(&["test-reexports::selective", "Baz"]);
    assert!(success, "CLI should succeed (no results is not an error)");
    assert_snapshot!(stdout, @r#"
    mod    crate::selective
    struct crate::selective::Bar
    struct crate::selective::Foo
    "#);
}

// --- Trait re-exports ---

#[test]
fn trait_reexport() {
    let (stdout, stderr, success) = run_cli(&["test-reexports", "MyTrait"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    trait  crate::MyTrait
    fn     crate::MyTrait::do_something
    trait  crate::traits::MyTrait
    fn     crate::traits::MyTrait::do_something
    "#);
}

#[test]
fn trait_impl_reexport() {
    let (stdout, stderr, success) = run_cli(&["test-reexports", "TraitImpl"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    struct crate::TraitImpl
    struct crate::traits::TraitImpl
    "#);
}
