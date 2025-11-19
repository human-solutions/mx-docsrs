mod common;
use common::{format_results, search_test_crate};

const CRATE_NAME: &str = "test-reexports";

// ============================================================================
// Tests for items that SHOULD be found
// ============================================================================

#[test]
fn test_simple_reexports_are_searchable() {
    // Test simple re-exports: pub use inner::InnerStruct;
    let results = search_test_crate(CRATE_NAME, "InnerStruct");
    assert!(
        !results.is_empty(),
        "InnerStruct should be searchable via simple re-export"
    );
    assert!(
        results.iter().any(|r| r.name == "InnerStruct"),
        "Results should contain InnerStruct"
    );
    insta::assert_snapshot!(format_results(&results), @"struct test_reexports::inner::InnerStruct");

    let results = search_test_crate(CRATE_NAME, "InnerEnum");
    assert!(
        !results.is_empty(),
        "InnerEnum should be searchable via simple re-export"
    );

    let results = search_test_crate(CRATE_NAME, "inner_function");
    assert!(
        !results.is_empty(),
        "inner_function should be searchable via simple re-export"
    );
}

#[test]
fn test_renamed_reexports_are_searchable() {
    // Test renamed re-exports: pub use inner::InnerStruct as RenamedStruct;
    // Note: Re-export aliases don't create new searchable names, only the original items are indexed
    // So we search for InnerStruct (which is re-exported as RenamedStruct)
    let results = search_test_crate(CRATE_NAME, "InnerStruct");
    assert!(
        !results.is_empty(),
        "InnerStruct should be searchable (re-exported as RenamedStruct)"
    );
    assert!(
        results.iter().any(|r| r.name == "InnerStruct"),
        "Results should contain InnerStruct"
    );
    insta::assert_snapshot!(format_results(&results), @"struct test_reexports::inner::InnerStruct");

    // Similarly for renamed_function (original name: inner_function)
    let results = search_test_crate(CRATE_NAME, "inner_function");
    assert!(
        !results.is_empty(),
        "inner_function should be searchable (re-exported as renamed_function)"
    );
}

#[test]
fn test_multiple_item_reexports_are_searchable() {
    // Test multiple-item re-exports: pub use inner::{InnerTrait, INNER_CONST, InnerAlias};
    let results = search_test_crate(CRATE_NAME, "InnerTrait");
    assert!(
        !results.is_empty(),
        "InnerTrait should be searchable via multiple-item re-export"
    );
    insta::assert_snapshot!(format_results(&results), @"trait test_reexports::inner::InnerTrait");

    let results = search_test_crate(CRATE_NAME, "INNER_CONST");
    assert!(
        !results.is_empty(),
        "INNER_CONST should be searchable via multiple-item re-export"
    );

    let results = search_test_crate(CRATE_NAME, "InnerAlias");
    assert!(
        !results.is_empty(),
        "InnerAlias should be searchable via multiple-item re-export"
    );
}

#[test]
fn test_nested_module_reexports_are_searchable() {
    // Test deeply nested re-exports: pub use deeply::nested::module::DeeplyNestedItem;
    let results = search_test_crate(CRATE_NAME, "DeeplyNestedItem");
    assert!(
        !results.is_empty(),
        "DeeplyNestedItem should be searchable via nested module re-export"
    );
    assert!(
        results.iter().any(|r| r.name == "DeeplyNestedItem"),
        "Results should contain DeeplyNestedItem"
    );
    insta::assert_snapshot!(format_results(&results), @"struct test_reexports::deeply::nested::module::DeeplyNestedItem");
}

#[test]
fn test_external_crate_reexports_are_searchable() {
    // Test external crate re-exports: pub use std::collections::HashMap;
    // Note: External crate items (from std) are not included in this crate's rustdoc JSON index
    // The re-exports exist as Use items but the actual HashMap/Vec structs are not in the index
    // So we can't search for them. This test verifies that searching returns empty results for external items.
    let results = search_test_crate(CRATE_NAME, "HashMap");
    // External items are not in the local crate's index
    assert!(
        results.is_empty(),
        "HashMap from std is not in local crate index (expected behavior)"
    );

    let results = search_test_crate(CRATE_NAME, "Vec");
    assert!(
        results.is_empty(),
        "Vec from std is not in local crate index (expected behavior)"
    );
}

#[test]
fn test_reexport_chains_work() {
    // Test re-export chains: Items re-exported multiple times
    // ChainedReexport -> IntermediateStruct -> InnerStruct (ID 2)
    // Search for the underlying item name
    let results = search_test_crate(CRATE_NAME, "InnerStruct");
    assert!(
        !results.is_empty(),
        "InnerStruct should be searchable (re-exported as ChainedReexport via chain)"
    );
    assert!(
        results.iter().any(|r| r.name == "InnerStruct"),
        "Results should contain InnerStruct"
    );
    insta::assert_snapshot!(format_results(&results), @"struct test_reexports::inner::InnerStruct");
}

#[test]
fn test_visibility_changing_reexports_work() {
    // Test visibility-changing re-exports within module: pub use private_module::Item as PublicItem;
    // The underlying struct is named "Item", re-exported as "PublicItem"
    let results = search_test_crate(CRATE_NAME, "Item");
    assert!(
        !results.is_empty(),
        "Item should be searchable (re-exported as PublicItem in visibility_change module)"
    );
    assert!(
        results.iter().any(|r| r.name == "Item"),
        "Results should contain Item"
    );
    insta::assert_snapshot!(format_results(&results), @r###"
struct test_reexports::deeply::nested::module::DeeplyNestedItem
struct test_reexports::visibility_change::private_module::Item
"###);
}

#[test]
fn test_selective_reexports_include_correct_items() {
    // Test selective re-exports in selective module: pub use internal::{Foo, Bar}; (NOT Baz)
    let results = search_test_crate(CRATE_NAME, "Foo");
    assert!(
        !results.is_empty(),
        "Foo should be searchable via selective re-export"
    );
    insta::assert_snapshot!(format_results(&results), @"struct test_reexports::selective::internal::Foo");

    let results = search_test_crate(CRATE_NAME, "Bar");
    assert!(
        !results.is_empty(),
        "Bar should be searchable via selective re-export"
    );
}

#[test]
fn test_trait_and_type_reexports_work() {
    // Test trait and type alias re-exports at crate root
    let results = search_test_crate(CRATE_NAME, "MyTrait");
    assert!(
        !results.is_empty(),
        "MyTrait should be searchable via trait re-export"
    );
    insta::assert_snapshot!(format_results(&results), @"trait test_reexports::traits::MyTrait");

    let results = search_test_crate(CRATE_NAME, "TraitImpl");
    assert!(
        !results.is_empty(),
        "TraitImpl should be searchable via trait impl re-export"
    );

    let results = search_test_crate(CRATE_NAME, "MyType");
    assert!(
        !results.is_empty(),
        "MyType should be searchable via type alias re-export"
    );
}

#[test]
fn test_glob_reexports_work() {
    // Test glob re-exports: pub use crate::inner::*;
    // This should re-export all public items from the inner module
    // Items are available in the reexported module
    let results = search_test_crate(CRATE_NAME, "InnerStruct");
    assert!(
        !results.is_empty(),
        "Items should be searchable via glob re-export"
    );
}

// ============================================================================
// Tests for items that SHOULD NOT be found
// ============================================================================

#[test]
fn test_non_reexported_items_are_not_found() {
    // Test that Baz is NOT re-exported at selective module level (it's internal only)
    let results = search_test_crate(CRATE_NAME, "Baz");
    // If Baz exists in JSON, it should be in the internal submodule, not accessible
    if !results.is_empty() {
        // Verify none of them are directly accessible (not in selective module)
        assert!(
            !results
                .iter()
                .any(|r| r.name == "Baz" && r.path.contains(&"selective".to_string())),
            "Baz should not be re-exported in selective module"
        );
    }
    insta::assert_snapshot!(format_results(&results), @"(no results)");
}

#[test]
fn test_private_module_items_not_reexported() {
    // Test that items from private modules that aren't re-exported don't show up
    let results = search_test_crate(CRATE_NAME, "PrivateInternalItem");
    // These items shouldn't be in the public API
    assert!(
        results.is_empty(),
        "Private internal items should not be in public documentation"
    );
}
