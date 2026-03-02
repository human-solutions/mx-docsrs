mod common;

use common::run_cli;
use insta::assert_snapshot;

// --- Public items ARE found ---

#[test]
fn public_struct_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "PublicStruct"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)

    /// A fully public struct
    pub struct test_visibility::PublicStruct {
        /// A public field
        pub public_field: String,
    }

    /* ======== Methods ======== */
    /// Public constructor
    pub fn new(public_field: String, private_field: i32) -> Self
    ");
}

#[test]
fn public_enum_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "PublicEnum"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)

    /// A public enum
    pub enum test_visibility::PublicEnum {
        /// Public variant
        Variant1,
        /// Another public variant
        Variant2(String),
    }
    ");
}

#[test]
fn public_function_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "public_function"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)

    /// A public function
    pub fn test_visibility::public_function() -> String
    ");
}

#[test]
fn public_const_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "PUBLIC_CONST"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)

    /// Public constant
    pub const test_visibility::PUBLIC_CONST: i32
    ");
}

#[test]
fn public_type_alias_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "PublicAlias"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)

    /// Public type alias
    pub type test_visibility::PublicAlias = test_visibility::PublicStruct
    ");
}

#[test]
fn public_trait_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "PublicTrait"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)

    /// A trait to test trait visibility
    pub trait test_visibility::PublicTrait {
        /// Associated type
        type Item;
        /// Trait method
        fn method(&self) -> Self::Item;
    }
    ");
}

#[test]
fn nested_public_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "NestedPublic"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)

    /// Public item in public module
    pub struct test_visibility::public_module::NestedPublic
    ");
}

#[test]
fn deeply_nested_is_found() {
    let (stdout, stderr, success) = run_cli(&["test-visibility", "DeeplyNested"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)

    /// Public item in nested module
    pub struct test_visibility::public_module::inner::DeeplyNested
    ");
}

// --- Non-public items are NOT found ---

#[test]
fn crate_visible_struct_not_found() {
    let (stdout, _stderr, success) = run_cli(&["test-visibility", "CrateVisibleStruct"]);
    assert!(success, "CLI should succeed (no results is not an error)");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)

    mod test_visibility
    const test_visibility::PUBLIC_CONST
    type test_visibility::PublicAlias
    enum test_visibility::PublicEnum
    struct test_visibility::PublicStruct
    trait test_visibility::PublicTrait
    fn test_visibility::PublicTrait::method
    struct test_visibility::PublicTupleStruct
    fn test_visibility::public_function
    mod test_visibility::public_module
    struct test_visibility::public_module::NestedPublic
    mod test_visibility::public_module::inner
    struct test_visibility::public_module::inner::DeeplyNested
    ");
}

#[test]
fn crate_visible_enum_not_found() {
    let (stdout, _stderr, success) = run_cli(&["test-visibility", "CrateVisibleEnum"]);
    assert!(success, "CLI should succeed (no results is not an error)");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)

    mod test_visibility
    const test_visibility::PUBLIC_CONST
    type test_visibility::PublicAlias
    enum test_visibility::PublicEnum
    struct test_visibility::PublicStruct
    trait test_visibility::PublicTrait
    fn test_visibility::PublicTrait::method
    struct test_visibility::PublicTupleStruct
    fn test_visibility::public_function
    mod test_visibility::public_module
    struct test_visibility::public_module::NestedPublic
    mod test_visibility::public_module::inner
    struct test_visibility::public_module::inner::DeeplyNested
    ");
}

#[test]
fn private_struct_not_found() {
    let (stdout, _stderr, success) = run_cli(&["test-visibility", "PrivateStruct"]);
    assert!(success, "CLI should succeed (no results is not an error)");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)

    mod test_visibility
    const test_visibility::PUBLIC_CONST
    type test_visibility::PublicAlias
    enum test_visibility::PublicEnum
    struct test_visibility::PublicStruct
    trait test_visibility::PublicTrait
    fn test_visibility::PublicTrait::method
    struct test_visibility::PublicTupleStruct
    fn test_visibility::public_function
    mod test_visibility::public_module
    struct test_visibility::public_module::NestedPublic
    mod test_visibility::public_module::inner
    struct test_visibility::public_module::inner::DeeplyNested
    ");
}

#[test]
fn super_visible_not_found() {
    let (stdout, _stderr, success) = run_cli(&["test-visibility", "NestedSuperVisible"]);
    assert!(success, "CLI should succeed (no results is not an error)");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)

    mod test_visibility
    const test_visibility::PUBLIC_CONST
    type test_visibility::PublicAlias
    enum test_visibility::PublicEnum
    struct test_visibility::PublicStruct
    trait test_visibility::PublicTrait
    fn test_visibility::PublicTrait::method
    struct test_visibility::PublicTupleStruct
    fn test_visibility::public_function
    mod test_visibility::public_module
    struct test_visibility::public_module::NestedPublic
    mod test_visibility::public_module::inner
    struct test_visibility::public_module::inner::DeeplyNested
    ");
}
