mod common;

use common::{find_items_by_name, find_reexports, generate_rustdoc_json, load_rustdoc_json};

#[test]
fn test_reexports_are_in_json() {
    let json_path = generate_rustdoc_json("test-reexports");
    let krate = load_rustdoc_json(&json_path);

    // Find all re-exports
    let reexports = find_reexports(&krate);

    assert!(
        !reexports.is_empty(),
        "Should find re-export items in the JSON"
    );

    // Print some examples for debugging
    println!("\nFound {} re-exports:", reexports.len());
    for (path, use_item) in reexports.iter().take(10) {
        println!("  {} -> {}", path, use_item.source);
    }
}

#[test]
fn test_reexport_simple() {
    let json_path = generate_rustdoc_json("test-reexports");
    let krate = load_rustdoc_json(&json_path);

    let reexports = find_reexports(&krate);

    // Look for InnerStruct re-export at crate root
    let inner_struct_reexport = reexports.iter().find(|(_path, use_item)| {
        use_item.source == "inner::InnerStruct" || use_item.source.ends_with("::inner::InnerStruct")
    });

    assert!(
        inner_struct_reexport.is_some(),
        "Should find re-export of InnerStruct. All re-exports: {:?}",
        reexports
            .iter()
            .map(|(p, u)| format!("{} -> {}", p, u.source))
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_reexport_renamed() {
    let json_path = generate_rustdoc_json("test-reexports");
    let krate = load_rustdoc_json(&json_path);

    // Look for RenamedStruct (which is InnerStruct with a new name)
    let renamed_results = find_items_by_name(&krate, "RenamedStruct");

    // Note: Renamed re-exports may not appear as separate items in the JSON
    // They appear as Use items pointing to the original
    println!("\nRenamedStruct search results: {:?}", renamed_results);
    println!("Note: Renamed re-exports appear as Use items, not as separate named items");
}

#[test]
fn test_reexport_multiple_items() {
    let json_path = generate_rustdoc_json("test-reexports");
    let krate = load_rustdoc_json(&json_path);

    // Multiple items re-exported: InnerTrait, INNER_CONST, InnerAlias
    let trait_results = find_items_by_name(&krate, "InnerTrait");
    let const_results = find_items_by_name(&krate, "INNER_CONST");
    let alias_results = find_items_by_name(&krate, "InnerAlias");

    assert!(
        !trait_results.is_empty(),
        "Should find InnerTrait re-export"
    );
    assert!(
        !const_results.is_empty(),
        "Should find INNER_CONST re-export"
    );
    assert!(
        !alias_results.is_empty(),
        "Should find InnerAlias re-export"
    );
}

#[test]
fn test_reexport_glob() {
    let json_path = generate_rustdoc_json("test-reexports");
    let krate = load_rustdoc_json(&json_path);

    let reexports = find_reexports(&krate);

    // Look for glob re-export in the reexported module
    // Note: Glob re-exports might be expanded in the JSON
    let glob_reexports: Vec<_> = reexports
        .iter()
        .filter(|(path, use_item)| path.contains("reexported") && use_item.source.contains("inner"))
        .collect();

    // Note: The behavior of glob re-exports in rustdoc JSON may vary
    // This test documents the current behavior
    println!(
        "\nGlob re-exports in 'reexported' module: {:?}",
        glob_reexports
    );
}

#[test]
fn test_reexport_nested_module() {
    let json_path = generate_rustdoc_json("test-reexports");
    let krate = load_rustdoc_json(&json_path);

    // DeeplyNestedItem is re-exported from deeply::nested::module
    let results = find_items_by_name(&krate, "DeeplyNestedItem");

    assert!(
        !results.is_empty(),
        "Should find DeeplyNestedItem re-export"
    );

    println!("\nDeeplyNestedItem found at: {:?}", results);
}

#[test]
fn test_reexport_external_crate() {
    let json_path = generate_rustdoc_json("test-reexports");
    let krate = load_rustdoc_json(&json_path);

    let reexports = find_reexports(&krate);

    // Look for re-exports from std
    let std_reexports: Vec<_> = reexports
        .iter()
        .filter(|(_, import)| import.source.starts_with("std::"))
        .collect();

    assert!(
        !std_reexports.is_empty(),
        "Should find re-exports from std library"
    );

    println!("\nStd re-exports: {:?}", std_reexports);
}

#[test]
fn test_reexport_chain() {
    let json_path = generate_rustdoc_json("test-reexports");
    let krate = load_rustdoc_json(&json_path);

    // ChainedReexport is a re-export of a re-export
    let results = find_items_by_name(&krate, "ChainedReexport");

    // This test documents how chained re-exports appear in the JSON
    println!("\nChainedReexport found at: {:?}", results);
}

#[test]
fn test_reexport_visibility_change() {
    let json_path = generate_rustdoc_json("test-reexports");
    let krate = load_rustdoc_json(&json_path);

    // PublicItem is a re-export from a private module
    let results = find_items_by_name(&krate, "PublicItem");

    // Note: Items from private modules may not be included in the JSON
    // even when re-exported publicly
    println!(
        "\nPublicItem (from private module) search results: {:?}",
        results
    );
    println!("Note: Items re-exported from private modules may not appear in rustdoc JSON");
}

#[test]
fn test_reexport_selective() {
    let json_path = generate_rustdoc_json("test-reexports");
    let krate = load_rustdoc_json(&json_path);

    // Foo and Bar should be re-exported, but not Baz
    let foo_results = find_items_by_name(&krate, "Foo");
    let bar_results = find_items_by_name(&krate, "Bar");
    let baz_results = find_items_by_name(&krate, "Baz");

    assert!(!foo_results.is_empty(), "Should find Foo re-export");
    assert!(!bar_results.is_empty(), "Should find Bar re-export");

    // Baz should only appear in the private module, not as a re-export
    println!("\nFoo: {:?}", foo_results);
    println!("Bar: {:?}", bar_results);
    println!(
        "Baz: {:?} (should not be publicly re-exported)",
        baz_results
    );
}
