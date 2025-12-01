mod common;

use common::run_cli;
use insta::assert_snapshot;

// --- Public items ARE found ---

#[test]
fn public_struct_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "PublicStruct"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    Using local dependency at [LOCAL_PATH] (version 0.1.0)
    pub struct test_visibility::PublicStruct

    A fully public struct

    Fields:
      pub public_field: alloc::string::String

    Methods:
      pub fn new(public_field: alloc::string::String, private_field: i32) -> Self
    ");
}

#[test]
fn public_enum_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "PublicEnum"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    Using local dependency at [LOCAL_PATH] (version 0.1.0)
    pub enum test_visibility::PublicEnum

    A public enum

    Variants:
      Variant1
      Variant2(alloc::string::String)
    ");
}

#[test]
fn public_function_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "public_function"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    Using local dependency at [LOCAL_PATH] (version 0.1.0)
    pub fn test_visibility::public_function() -> alloc::string::String

    A public function
    ");
}

#[test]
fn public_const_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "PUBLIC_CONST"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    Using local dependency at [LOCAL_PATH] (version 0.1.0)
    pub const test_visibility::PUBLIC_CONST: i32

    Public constant
    ");
}

#[test]
fn public_type_alias_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "PublicAlias"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    Using local dependency at [LOCAL_PATH] (version 0.1.0)
    pub type test_visibility::PublicAlias = test_visibility::PublicStruct

    Public type alias
    ");
}

#[test]
fn public_trait_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "PublicTrait"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    Using local dependency at [LOCAL_PATH] (version 0.1.0)
    pub trait test_visibility::PublicTrait

    A trait to test trait visibility

    Associated Types:
      type Item

    Required Methods:
      pub fn method(&self) -> Self::Item
    ");
}

#[test]
fn nested_public_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "NestedPublic"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    Using local dependency at [LOCAL_PATH] (version 0.1.0)
    pub struct test_visibility::public_module::NestedPublic

    Public item in public module
    ");
}

#[test]
fn deeply_nested_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "DeeplyNested"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    Using local dependency at [LOCAL_PATH] (version 0.1.0)
    pub struct test_visibility::public_module::inner::DeeplyNested

    Public item in nested module
    ");
}

// --- Non-public items are NOT found ---

#[test]
fn crate_visible_struct_not_found() {
    let (stdout, _stderr, success) = run_cli(&["test-visibility", "CrateVisibleStruct"]);
    assert!(success, "CLI should succeed (no results is not an error)");
    assert_snapshot!(stdout, @r"
    Using local dependency at [LOCAL_PATH] (version 0.1.0)
    mod    test_visibility
    const  test_visibility::PUBLIC_CONST
    type   test_visibility::PublicAlias
    enum   test_visibility::PublicEnum
    struct test_visibility::PublicStruct
    trait  test_visibility::PublicTrait
    fn     test_visibility::PublicTrait::method
    struct test_visibility::PublicTupleStruct
    fn     test_visibility::public_function
    mod    test_visibility::public_module
    struct test_visibility::public_module::NestedPublic
    mod    test_visibility::public_module::inner
    struct test_visibility::public_module::inner::DeeplyNested
    ");
}

#[test]
fn crate_visible_enum_not_found() {
    let (stdout, _stderr, success) = run_cli(&["test-visibility", "CrateVisibleEnum"]);
    assert!(success, "CLI should succeed (no results is not an error)");
    assert_snapshot!(stdout, @r"
    Using local dependency at [LOCAL_PATH] (version 0.1.0)
    mod    test_visibility
    const  test_visibility::PUBLIC_CONST
    type   test_visibility::PublicAlias
    enum   test_visibility::PublicEnum
    struct test_visibility::PublicStruct
    trait  test_visibility::PublicTrait
    fn     test_visibility::PublicTrait::method
    struct test_visibility::PublicTupleStruct
    fn     test_visibility::public_function
    mod    test_visibility::public_module
    struct test_visibility::public_module::NestedPublic
    mod    test_visibility::public_module::inner
    struct test_visibility::public_module::inner::DeeplyNested
    ");
}

#[test]
fn private_struct_not_found() {
    let (stdout, _stderr, success) = run_cli(&["test-visibility", "PrivateStruct"]);
    assert!(success, "CLI should succeed (no results is not an error)");
    assert_snapshot!(stdout, @r"
    Using local dependency at [LOCAL_PATH] (version 0.1.0)
    mod    test_visibility
    const  test_visibility::PUBLIC_CONST
    type   test_visibility::PublicAlias
    enum   test_visibility::PublicEnum
    struct test_visibility::PublicStruct
    trait  test_visibility::PublicTrait
    fn     test_visibility::PublicTrait::method
    struct test_visibility::PublicTupleStruct
    fn     test_visibility::public_function
    mod    test_visibility::public_module
    struct test_visibility::public_module::NestedPublic
    mod    test_visibility::public_module::inner
    struct test_visibility::public_module::inner::DeeplyNested
    ");
}

#[test]
fn super_visible_not_found() {
    let (stdout, _stderr, success) = run_cli(&["test-visibility", "NestedSuperVisible"]);
    assert!(success, "CLI should succeed (no results is not an error)");
    assert_snapshot!(stdout, @r"
    Using local dependency at [LOCAL_PATH] (version 0.1.0)
    mod    test_visibility
    const  test_visibility::PUBLIC_CONST
    type   test_visibility::PublicAlias
    enum   test_visibility::PublicEnum
    struct test_visibility::PublicStruct
    trait  test_visibility::PublicTrait
    fn     test_visibility::PublicTrait::method
    struct test_visibility::PublicTupleStruct
    fn     test_visibility::public_function
    mod    test_visibility::public_module
    struct test_visibility::public_module::NestedPublic
    mod    test_visibility::public_module::inner
    struct test_visibility::public_module::inner::DeeplyNested
    ");
}
