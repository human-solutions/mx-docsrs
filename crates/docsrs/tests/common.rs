use std::path::PathBuf;
use std::process::Command;

/// Helper function to generate rustdoc JSON for a test crate
pub fn generate_rustdoc_json(crate_name: &str) -> PathBuf {
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
pub fn load_rustdoc_json(path: &PathBuf) -> rustdoc_types::Crate {
    let json_content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read JSON file {:?}: {}", path, e));
    serde_json::from_str(&json_content).unwrap_or_else(|e| panic!("Failed to parse JSON: {}", e))
}

/// Helper function to search for items by name in the crate
pub fn find_items_by_name(krate: &rustdoc_types::Crate, name: &str) -> Vec<String> {
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

/// Helper function to find all Use items (re-exports) in the crate
pub fn find_reexports(krate: &rustdoc_types::Crate) -> Vec<(String, rustdoc_types::Use)> {
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

    // Sort for deterministic output (fixes HashMap iteration non-determinism)
    results.sort_by(|a, b| {
        // First compare by path
        let path_cmp = a.0.cmp(&b.0);
        if path_cmp != std::cmp::Ordering::Equal {
            return path_cmp;
        }

        // Then by source
        let source_cmp = a.1.source.cmp(&b.1.source);
        if source_cmp != std::cmp::Ordering::Equal {
            return source_cmp;
        }

        // Finally by name
        a.1.name.cmp(&b.1.name)
    });

    results
}
