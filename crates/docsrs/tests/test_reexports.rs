use std::path::PathBuf;
use std::process::Command;

/// Helper function to generate rustdoc JSON for a test crate
fn generate_rustdoc_json(crate_name: &str) -> PathBuf {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let target_dir = workspace_root.join("target");

    // Run cargo +nightly rustdoc to generate JSON
    let output = Command::new("cargo")
        .arg("+nightly")
        .arg("rustdoc")
        .arg("-p")
        .arg(crate_name)
        .arg("--")
        .arg("-Zunstable-options")
        .arg("--output-format")
        .arg("json")
        .current_dir(&workspace_root)
        .output()
        .expect("Failed to generate rustdoc JSON");

    if !output.status.success() {
        panic!(
            "Failed to generate rustdoc JSON:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // The JSON file is in target/doc/{crate_name}.json
    // Note: Rust converts hyphens to underscores in crate names for file names
    let json_filename = crate_name.replace('-', "_");
    target_dir
        .join("doc")
        .join(format!("{}.json", json_filename))
}

/// Helper function to load and parse rustdoc JSON
fn load_rustdoc_json(path: &PathBuf) -> rustdoc_types::Crate {
    let json_content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read JSON file {:?}: {}", path, e));
    serde_json::from_str(&json_content).unwrap_or_else(|e| panic!("Failed to parse JSON: {}", e))
}

/// Helper function to find all Use items (re-exports) in the crate
fn find_reexports(krate: &rustdoc_types::Crate) -> Vec<(String, rustdoc_types::Use)> {
    let mut results = Vec::new();

    for (_id, item) in &krate.index {
        if let rustdoc_types::ItemEnum::Use(use_item) = &item.inner {
            // Get the path where this re-export appears
            let path = krate
                .paths
                .get(&item.id)
                .map(|summary| summary.path.join("::"))
                .unwrap_or_else(|| String::from("<unknown>"));

            results.push((path, use_item.clone()));
        }
    }

    results
}

/// Helper function to search for items by name
fn find_items_by_name(krate: &rustdoc_types::Crate, name: &str) -> Vec<String> {
    let mut results = Vec::new();

    for (_id, item) in &krate.index {
        if let Some(item_name) = &item.name {
            if item_name.eq_ignore_ascii_case(name) {
                let path = krate
                    .paths
                    .get(&item.id)
                    .map(|summary| {
                        let mut parts = summary.path.clone();
                        parts.push(item_name.clone());
                        parts.join("::")
                    })
                    .unwrap_or_else(|| item_name.clone());

                results.push(format!("{} ({:?})", path, item.inner));
            }
        }
    }

    results
}

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
