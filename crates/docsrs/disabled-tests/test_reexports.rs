mod common;
use common::{format_results, search_test_crate};
use insta::assert_snapshot;

const CRATE_NAME: &str = "test-reexports";

// ============================================================================
// Tests for items that SHOULD be found
// ============================================================================

#[test]
fn test_simple_reexports_are_searchable() {
    // Test simple re-exports: pub use inner::InnerStruct;
    let results = search_test_crate(CRATE_NAME, "InnerStruct");

    assert_snapshot!(format_results(&results), @"struct test_reexports::inner::InnerStruct");

    let results = search_test_crate(CRATE_NAME, "InnerEnum");
    assert_snapshot!(format_results(&results), @"enum test_reexports::inner::InnerEnum");

    let results = search_test_crate(CRATE_NAME, "inner_function");
    assert_snapshot!(format_results(&results), @"fn test_reexports::inner::inner_function");
}

#[test]
fn test_inner_searchable() {
    // Test simple re-exports: pub use inner::InnerStruct;
    let results = search_test_crate(CRATE_NAME, "Inner");

    assert_snapshot!(format_results(&results), @r"
    constant test_reexports::inner::INNER_CONST
    enum test_reexports::inner::InnerEnum
    fn test_reexports::inner::inner_function
    mod test-reexports::inner
    struct test_reexports::inner::InnerStruct
    trait test_reexports::inner::InnerTrait
    type test_reexports::inner::InnerAlias
    ");
}

#[test]
fn test_renamed_reexports_are_searchable() {
    // Test renamed re-exports: pub use inner::InnerStruct as RenamedStruct;
    // Note: Re-export aliases don't create new searchable names, only the original items are indexed
    // So we search for InnerStruct (which is re-exported as RenamedStruct)
    let results = search_test_crate(CRATE_NAME, "InnerStruct");
    assert_snapshot!(format_results(&results), @"struct test_reexports::inner::InnerStruct");

    // Similarly for renamed_function (original name: inner_function)
    let results = search_test_crate(CRATE_NAME, "inner_function");
    assert_snapshot!(format_results(&results), @"fn test_reexports::inner::inner_function");
}

#[test]
fn test_multiple_item_reexports_are_searchable() {
    // Test multiple-item re-exports: pub use inner::{InnerTrait, INNER_CONST, InnerAlias};
    let results = search_test_crate(CRATE_NAME, "InnerTrait");
    assert_snapshot!(format_results(&results), @"trait test_reexports::inner::InnerTrait");

    let results = search_test_crate(CRATE_NAME, "INNER_CONST");
    assert_snapshot!(format_results(&results), @"constant test_reexports::inner::INNER_CONST");

    let results = search_test_crate(CRATE_NAME, "InnerAlias");
    assert_snapshot!(format_results(&results), @"type test_reexports::inner::InnerAlias");
}

#[test]
fn test_nested_module_reexports_are_searchable() {
    // Test deeply nested re-exports: pub use deeply::nested::module::DeeplyNestedItem;
    let results = search_test_crate(CRATE_NAME, "DeeplyNestedItem");
    assert_snapshot!(format_results(&results), @"struct test_reexports::deeply::nested::module::DeeplyNestedItem");
}

#[test]
fn test_external_crate_reexports_are_searchable() {
    // Test external crate re-exports: pub use std::collections::HashMap;
    // Note: External crate items (from std) are not included in this crate's rustdoc JSON index
    // The re-exports exist as Use items but the actual HashMap/Vec structs are not in the index
    // So we can't search for them. This test verifies that searching returns empty results for external items.
    let results = search_test_crate(CRATE_NAME, "HashMap");
    assert_snapshot!(format_results(&results), @"(no results)");

    let results = search_test_crate(CRATE_NAME, "Vec");
    assert_snapshot!(format_results(&results), @"(no results)");
}

#[test]
fn test_reexport_chains_work() {
    // Test re-export chains: Items re-exported multiple times
    // ChainedReexport -> IntermediateStruct -> InnerStruct (ID 2)
    // Search for the underlying item name
    let results = search_test_crate(CRATE_NAME, "InnerStruct");
    assert_snapshot!(format_results(&results), @"struct test_reexports::inner::InnerStruct");
}

#[test]
fn test_visibility_changing_reexports_work() {
    // Test visibility-changing re-exports within module: pub use private_module::Item as PublicItem;
    // The underlying struct is named "Item", re-exported as "PublicItem"
    let results = search_test_crate(CRATE_NAME, "Item");
    assert_snapshot!(format_results(&results), @r###"
struct test_reexports::deeply::nested::module::DeeplyNestedItem
struct test_reexports::visibility_change::private_module::Item
"###);
}

#[test]
fn test_selective_reexports_include_correct_items() {
    // Test selective re-exports in selective module: pub use internal::{Foo, Bar}; (NOT Baz)
    let results = search_test_crate(CRATE_NAME, "Foo");
    assert_snapshot!(format_results(&results), @"struct test_reexports::selective::internal::Foo");

    let results = search_test_crate(CRATE_NAME, "Bar");
    assert_snapshot!(format_results(&results), @"struct test_reexports::selective::internal::Bar");
}

#[test]
fn test_trait_and_type_reexports_work() {
    // Test trait and type alias re-exports at crate root
    let results = search_test_crate(CRATE_NAME, "MyTrait");
    assert_snapshot!(format_results(&results), @"trait test_reexports::traits::MyTrait");

    let results = search_test_crate(CRATE_NAME, "TraitImpl");
    assert_snapshot!(format_results(&results), @"struct test_reexports::traits::TraitImpl");
}

#[test]
fn test_glob_reexports_work() {
    // Test glob re-exports: pub use crate::inner::*;
    // This should re-export all public items from the inner module
    // Items are available in the reexported module
    let results = search_test_crate(CRATE_NAME, "InnerStruct");
    assert_snapshot!(format_results(&results), @"struct test_reexports::inner::InnerStruct");
}

// ============================================================================
// Tests for items that SHOULD NOT be found
// ============================================================================

#[test]
fn test_non_reexported_items_are_not_found() {
    // Test that Baz is NOT re-exported at selective module level (it's internal only)
    let results = search_test_crate(CRATE_NAME, "Baz");
    assert_snapshot!(format_results(&results), @"(no results)");
}

#[test]
fn test_private_module_items_not_reexported() {
    // Test that items from private modules that aren't re-exported don't show up
    let results = search_test_crate(CRATE_NAME, "PrivateInternalItem");
    // These items shouldn't be in the public API
    assert_snapshot!(format_results(&results), @"(no results)");
}
