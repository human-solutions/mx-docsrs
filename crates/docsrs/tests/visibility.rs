mod common;

use common::run_cli;
use insta::assert_snapshot;

// --- Public items ARE found ---

#[test]
fn public_struct_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "PublicStruct"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    pub struct crate::PublicStruct

    A fully public struct

    Fields:
      pub public_field: alloc::string::String

    Methods:
      pub fn new(public_field: alloc::string::String, private_field: i32) -> Self
    "#);
}

#[test]
fn public_enum_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "PublicEnum"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    pub enum crate::PublicEnum

    A public enum

    Variants:
      Variant1
      Variant2(alloc::string::String)
    "#);
}

#[test]
fn public_function_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "public_function"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    pub fn crate::public_function() -> alloc::string::String

    A public function
    "#);
}

#[test]
fn public_const_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "PUBLIC_CONST"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    pub const crate::PUBLIC_CONST: i32

    Public constant
    "#);
}

#[test]
fn public_type_alias_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "PublicAlias"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    pub type crate::PublicAlias = crate::PublicStruct

    Public type alias
    "#);
}

#[test]
fn public_trait_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "PublicTrait"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    pub trait crate::PublicTrait

    A trait to test trait visibility

    Associated Types:
      type Item

    Required Methods:
      pub fn method(&self) -> Self::Item
    "#);
}

#[test]
fn nested_public_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "NestedPublic"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    pub struct crate::public_module::NestedPublic

    Public item in public module
    "#);
}

#[test]
fn deeply_nested_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "DeeplyNested"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    pub struct crate::public_module::inner::DeeplyNested

    Public item in nested module
    "#);
}

// --- Non-public items are NOT found ---

#[test]
fn crate_visible_struct_not_found() {
    let (stdout, _stderr, success) = run_cli(&["test-visibility", "CrateVisibleStruct"]);
    assert!(success, "CLI should succeed (no results is not an error)");
    assert_snapshot!(stdout, @r#"
    mod    crate
    const  crate::PUBLIC_CONST
    type   crate::PublicAlias
    enum   crate::PublicEnum
    struct crate::PublicStruct
    trait  crate::PublicTrait
    fn     crate::PublicTrait::method
    struct crate::PublicTupleStruct
    fn     crate::public_function
    mod    crate::public_module
    struct crate::public_module::NestedPublic
    mod    crate::public_module::inner
    struct crate::public_module::inner::DeeplyNested
    "#);
}

#[test]
fn crate_visible_enum_not_found() {
    let (stdout, _stderr, success) = run_cli(&["test-visibility", "CrateVisibleEnum"]);
    assert!(success, "CLI should succeed (no results is not an error)");
    assert_snapshot!(stdout, @r#"
    mod    crate
    const  crate::PUBLIC_CONST
    type   crate::PublicAlias
    enum   crate::PublicEnum
    struct crate::PublicStruct
    trait  crate::PublicTrait
    fn     crate::PublicTrait::method
    struct crate::PublicTupleStruct
    fn     crate::public_function
    mod    crate::public_module
    struct crate::public_module::NestedPublic
    mod    crate::public_module::inner
    struct crate::public_module::inner::DeeplyNested
    "#);
}

#[test]
fn private_struct_not_found() {
    let (stdout, _stderr, success) = run_cli(&["test-visibility", "PrivateStruct"]);
    assert!(success, "CLI should succeed (no results is not an error)");
    assert_snapshot!(stdout, @r#"
    mod    crate
    const  crate::PUBLIC_CONST
    type   crate::PublicAlias
    enum   crate::PublicEnum
    struct crate::PublicStruct
    trait  crate::PublicTrait
    fn     crate::PublicTrait::method
    struct crate::PublicTupleStruct
    fn     crate::public_function
    mod    crate::public_module
    struct crate::public_module::NestedPublic
    mod    crate::public_module::inner
    struct crate::public_module::inner::DeeplyNested
    "#);
}

#[test]
fn super_visible_not_found() {
    let (stdout, _stderr, success) = run_cli(&["test-visibility", "NestedSuperVisible"]);
    assert!(success, "CLI should succeed (no results is not an error)");
    assert_snapshot!(stdout, @r#"
    mod    crate
    const  crate::PUBLIC_CONST
    type   crate::PublicAlias
    enum   crate::PublicEnum
    struct crate::PublicStruct
    trait  crate::PublicTrait
    fn     crate::PublicTrait::method
    struct crate::PublicTupleStruct
    fn     crate::public_function
    mod    crate::public_module
    struct crate::public_module::NestedPublic
    mod    crate::public_module::inner
    struct crate::public_module::inner::DeeplyNested
    "#);
}
