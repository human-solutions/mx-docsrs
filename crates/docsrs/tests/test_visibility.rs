mod common;
use common::{format_results, search_test_crate};

const CRATE_NAME: &str = "test-visibility";

// ============================================================================
// Tests for items that SHOULD be found (public items)
// ============================================================================

#[test]
fn test_public_struct_is_searchable() {
    // Test that pub items are searchable
    let results = search_test_crate(CRATE_NAME, "PublicStruct");
    assert!(
        !results.is_empty(),
        "PublicStruct should be searchable (public item)"
    );
    assert!(
        results.iter().any(|r| r.name == "PublicStruct"),
        "Results should contain PublicStruct"
    );
    insta::assert_snapshot!(format_results(&results), @"struct test_visibility::PublicStruct");
}

#[test]
fn test_public_enum_is_searchable() {
    let results = search_test_crate(CRATE_NAME, "PublicEnum");
    assert!(
        !results.is_empty(),
        "PublicEnum should be searchable (public item)"
    );
    insta::assert_snapshot!(format_results(&results), @"enum test_visibility::PublicEnum");
}

#[test]
fn test_public_function_is_searchable() {
    let results = search_test_crate(CRATE_NAME, "public_function");
    assert!(
        !results.is_empty(),
        "public_function should be searchable (public item)"
    );
    insta::assert_snapshot!(format_results(&results), @"fn test_visibility::public_function");
}

#[test]
fn test_public_const_is_searchable() {
    let results = search_test_crate(CRATE_NAME, "PUBLIC_CONST");
    assert!(
        !results.is_empty(),
        "PUBLIC_CONST should be searchable (public item)"
    );
    insta::assert_snapshot!(format_results(&results), @"constant test_visibility::PUBLIC_CONST");
}

#[test]
fn test_public_type_alias_is_searchable() {
    let results = search_test_crate(CRATE_NAME, "PublicAlias");
    assert!(
        !results.is_empty(),
        "PublicAlias should be searchable (public item)"
    );
    insta::assert_snapshot!(format_results(&results), @"type test_visibility::PublicAlias");
}

// ============================================================================
// Tests for items that SHOULD NOT be found (non-public items)
// ============================================================================

#[test]
fn test_crate_visible_struct_not_searchable() {
    // pub(crate) items should not be in the public rustdoc JSON
    let results = search_test_crate(CRATE_NAME, "CrateVisibleStruct");
    // CrateVisibleStruct should either not exist or not be at the crate root level
    assert!(
        results.is_empty() || !results.iter().any(|r| r.path.len() == 1),
        "CrateVisibleStruct should not be in public API (crate-visible)"
    );
    insta::assert_snapshot!(format_results(&results), @"(no results)");
}

#[test]
fn test_crate_visible_enum_not_searchable() {
    let results = search_test_crate(CRATE_NAME, "CrateVisibleEnum");
    assert!(
        results.is_empty() || !results.iter().any(|r| r.path.len() == 1),
        "CrateVisibleEnum should not be in public API (crate-visible)"
    );
    insta::assert_snapshot!(format_results(&results), @"(no results)");
}

#[test]
fn test_crate_visible_function_not_searchable() {
    let results = search_test_crate(CRATE_NAME, "crate_visible_function");
    assert!(
        results.is_empty() || !results.iter().any(|r| r.path.len() == 1),
        "crate_visible_function should not be in public API (crate-visible)"
    );
    insta::assert_snapshot!(format_results(&results), @"(no results)");
}

#[test]
fn test_crate_visible_const_not_searchable() {
    let results = search_test_crate(CRATE_NAME, "CRATE_CONST");
    assert!(
        results.is_empty() || !results.iter().any(|r| r.path.len() == 1),
        "CRATE_CONST should not be in public API (crate-visible)"
    );
    insta::assert_snapshot!(format_results(&results), @"(no results)");
}

#[test]
fn test_crate_visible_type_alias_not_searchable() {
    let results = search_test_crate(CRATE_NAME, "CrateAlias");
    assert!(
        results.is_empty() || !results.iter().any(|r| r.path.len() == 1),
        "CrateAlias should not be in public API (crate-visible)"
    );
    insta::assert_snapshot!(format_results(&results), @"(no results)");
}

#[test]
fn test_private_struct_not_searchable() {
    // Private items should not be in the public rustdoc JSON
    let results = search_test_crate(CRATE_NAME, "PrivateStruct");
    assert!(
        results.is_empty() || !results.iter().any(|r| r.path.len() == 1),
        "PrivateStruct should not be in public API (private)"
    );
    insta::assert_snapshot!(format_results(&results), @"(no results)");
}

#[test]
fn test_private_function_not_searchable() {
    let results = search_test_crate(CRATE_NAME, "private_function");
    assert!(
        results.is_empty() || !results.iter().any(|r| r.path.len() == 1),
        "private_function should not be in public API (private)"
    );
    insta::assert_snapshot!(format_results(&results), @"(no results)");
}

#[test]
fn test_private_const_not_searchable() {
    let results = search_test_crate(CRATE_NAME, "PRIVATE_CONST");
    assert!(
        results.is_empty() || !results.iter().any(|r| r.path.len() == 1),
        "PRIVATE_CONST should not be in public API (private)"
    );
    insta::assert_snapshot!(format_results(&results), @"(no results)");
}

#[test]
fn test_super_visible_not_searchable() {
    // pub(super) items should not be in the public rustdoc JSON
    let results = search_test_crate(CRATE_NAME, "NestedSuperVisible");
    assert!(
        results.is_empty() || !results.iter().any(|r| r.path.len() == 1),
        "NestedSuperVisible should not be in public API (super-visible)"
    );
    insta::assert_snapshot!(format_results(&results), @"(no results)");
}

#[test]
fn test_path_restricted_not_searchable() {
    // pub(in path) items should not be in the public rustdoc JSON
    let results = search_test_crate(CRATE_NAME, "VisibleToOuterModule");
    assert!(
        results.is_empty() || !results.iter().any(|r| r.path.len() == 1),
        "VisibleToOuterModule should not be in public API (path-restricted)"
    );
    insta::assert_snapshot!(format_results(&results), @"(no results)");
}
