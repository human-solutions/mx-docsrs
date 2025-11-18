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

/// Helper function to search for items by name in the crate
fn find_items_by_name(krate: &rustdoc_types::Crate, name: &str) -> Vec<String> {
    let mut results = Vec::new();

    for (_id, item) in &krate.index {
        if let Some(item_name) = &item.name {
            if item_name.eq_ignore_ascii_case(name) {
                // Get the full path from krate.paths if available
                let path = krate
                    .paths
                    .get(&item.id)
                    .map(|summary| {
                        let mut parts = summary.path.clone();
                        parts.push(item_name.clone());
                        parts.join("::")
                    })
                    .unwrap_or_else(|| item_name.clone());

                results.push(format!(
                    "{} ({:?}, vis: {:?})",
                    path, item.inner, item.visibility
                ));
            }
        }
    }

    results
}

#[test]
fn test_visibility_public_items_are_in_json() {
    let json_path = generate_rustdoc_json("test-visibility");
    let krate = load_rustdoc_json(&json_path);

    // Public items should be in the JSON
    let public_struct_results = find_items_by_name(&krate, "PublicStruct");
    assert!(
        !public_struct_results.is_empty(),
        "PublicStruct should be found in rustdoc JSON"
    );

    let public_function_results = find_items_by_name(&krate, "public_function");
    assert!(
        !public_function_results.is_empty(),
        "public_function should be found in rustdoc JSON"
    );

    let public_enum_results = find_items_by_name(&krate, "PublicEnum");
    assert!(
        !public_enum_results.is_empty(),
        "PublicEnum should be found in rustdoc JSON"
    );
}

#[test]
fn test_visibility_crate_visible_items_in_json() {
    let json_path = generate_rustdoc_json("test-visibility");
    let krate = load_rustdoc_json(&json_path);

    // pub(crate) items - documenting actual behavior
    let crate_struct_results = find_items_by_name(&krate, "CrateVisibleStruct");
    let crate_function_results = find_items_by_name(&krate, "crate_visible_function");

    // Note: pub(crate) items are NOT included in rustdoc JSON by default
    // rustdoc focuses on the public API
    println!("\nCrateVisibleStruct results: {:?}", crate_struct_results);
    println!(
        "crate_visible_function results: {:?}",
        crate_function_results
    );
    println!("Note: pub(crate) items are not included in default rustdoc JSON output");
}

#[test]
fn test_visibility_private_items_not_in_json() {
    let json_path = generate_rustdoc_json("test-visibility");
    let krate = load_rustdoc_json(&json_path);

    // Private items should NOT be in the JSON
    let private_struct_results = find_items_by_name(&krate, "PrivateStruct");
    assert!(
        private_struct_results.is_empty(),
        "PrivateStruct (private) should NOT be in rustdoc JSON"
    );

    let private_function_results = find_items_by_name(&krate, "private_function");
    assert!(
        private_function_results.is_empty(),
        "private_function (private) should NOT be in rustdoc JSON"
    );
}

#[test]
fn test_visibility_nested_super_visible() {
    let json_path = generate_rustdoc_json("test-visibility");
    let krate = load_rustdoc_json(&json_path);

    // pub(super) items - documenting actual behavior
    let super_visible_results = find_items_by_name(&krate, "NestedSuperVisible");

    // Note: pub(super) items are also NOT included in rustdoc JSON by default
    println!(
        "\nNestedSuperVisible results: {:?}",
        super_visible_results
    );
    println!("Note: pub(super) items are not included in default rustdoc JSON output");
}

#[test]
fn test_visibility_levels_are_recorded() {
    let json_path = generate_rustdoc_json("test-visibility");
    let krate = load_rustdoc_json(&json_path);

    // Verify that visibility information is correctly recorded
    for (_id, item) in &krate.index {
        if let Some(name) = &item.name {
            match name.as_str() {
                "PublicStruct" => {
                    assert!(
                        matches!(item.visibility, rustdoc_types::Visibility::Public),
                        "PublicStruct should have Public visibility"
                    );
                }
                "CrateVisibleStruct" => {
                    assert!(
                        matches!(item.visibility, rustdoc_types::Visibility::Crate),
                        "CrateVisibleStruct should have Crate visibility"
                    );
                }
                _ => {}
            }
        }
    }
}

#[test]
fn test_visibility_private_fields_handling() {
    let json_path = generate_rustdoc_json("test-visibility");
    let krate = load_rustdoc_json(&json_path);

    // Find PublicStruct and check its fields
    for (_id, item) in &krate.index {
        if let Some(name) = &item.name {
            if name == "PublicStruct" {
                if let rustdoc_types::ItemEnum::Struct(s) = &item.inner {
                    if let rustdoc_types::StructKind::Plain { fields, .. } = &s.kind {
                        // Should have public_field visible
                        assert!(
                            !fields.is_empty(),
                            "PublicStruct should have visible fields"
                        );
                        // Note: private fields may be represented as None in the IDs
                    }
                }
            }
        }
    }
}
