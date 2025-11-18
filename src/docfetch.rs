use anyhow::{Context, Result};
use rustdoc_types::{Crate, ItemEnum};
use std::io::Read;

/// Represents a search result from the documentation
#[derive(Debug)]
pub struct DocResult {
    pub name: String,
    pub item_type: String,
    pub path: Vec<String>,
    pub url: String,
}

/// Fetch and search documentation from docs.rs
pub fn fetch_docs(crate_name: &str, version: &str, symbol: &str) -> Result<Vec<DocResult>> {
    println!("Fetching rustdoc JSON from docs.rs...");

    // Construct URL to fetch rustdoc JSON
    let url = format!("https://docs.rs/crate/{}/{}/json", crate_name, version);
    println!("URL: {}", url);

    // Fetch the compressed JSON
    let mut response = ureq::get(&url).call()?;
    let status = response.status();
    println!("Status: {}", status);

    if status != 200 {
        anyhow::bail!("Failed to fetch rustdoc JSON (status: {})", status);
    }

    // Read compressed data
    let mut compressed_data = Vec::new();
    response.body_mut().as_reader().read_to_end(&mut compressed_data)?;
    println!("Downloaded {} bytes (compressed)", compressed_data.len());

    // Decompress with zstd
    let decompressed_data = zstd::decode_all(&compressed_data[..])
        .context("Failed to decompress zstd data")?;
    println!("Decompressed to {} bytes", decompressed_data.len());

    // Parse rustdoc JSON
    let krate: Crate = serde_json::from_slice(&decompressed_data)
        .context("Failed to parse rustdoc JSON")?;
    let crate_name_from_json = krate.index.get(&krate.root)
        .and_then(|i| i.name.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("?");
    println!("Parsed crate: {} (format version {})",
             crate_name_from_json,
             krate.format_version);

    // Search for the symbol
    let results = search_items(&krate, symbol, crate_name, version);
    println!("Found {} matching items", results.len());

    Ok(results)
}

/// Search through rustdoc items for matches
fn search_items(krate: &Crate, query: &str, crate_name: &str, version: &str) -> Vec<DocResult> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    for (id, item) in &krate.index {
        // Skip items without names
        let Some(ref name) = item.name else {
            continue;
        };

        // Check if item name matches the query (case-insensitive)
        if name.to_lowercase().contains(&query_lower) {
            // Build the path to this item using the paths map
            let path = if let Some(summary) = krate.paths.get(id) {
                summary.path.clone()
            } else {
                vec![crate_name.to_string()]
            };

            // Get item type as string
            let item_type = get_item_type(&item.inner);

            // Generate documentation URL
            let url = generate_url(crate_name, version, &path, name, item_type);

            results.push(DocResult {
                name: name.clone(),
                item_type: item_type.to_string(),
                path,
                url,
            });
        }
    }

    results
}

/// Get the type of an item as a string
fn get_item_type(item: &ItemEnum) -> &'static str {
    match item {
        ItemEnum::Module(_) => "mod",
        ItemEnum::ExternCrate { .. } => "externcrate",
        ItemEnum::Union(_) => "union",
        ItemEnum::Struct(_) => "struct",
        ItemEnum::StructField(_) => "structfield",
        ItemEnum::Enum(_) => "enum",
        ItemEnum::Variant(_) => "variant",
        ItemEnum::Function(_) => "fn",
        ItemEnum::Trait(_) => "trait",
        ItemEnum::TraitAlias(_) => "traitalias",
        ItemEnum::Impl(_) => "impl",
        ItemEnum::TypeAlias(_) => "type",
        ItemEnum::Constant { .. } => "constant",
        ItemEnum::Static(_) => "static",
        ItemEnum::Macro(_) => "macro",
        ItemEnum::ProcMacro(_) => "derive",
        ItemEnum::Primitive(_) => "primitive",
        ItemEnum::AssocConst { .. } => "associatedconstant",
        ItemEnum::AssocType { .. } => "associatedtype",
        _ => "item",
    }
}

/// Generate a documentation URL for an item
fn generate_url(crate_name: &str, version: &str, path: &[String], name: &str, item_type: &str) -> String {
    let base = format!("https://docs.rs/{}/{}", crate_name, version);

    if path.is_empty() {
        return format!("{}/{}/", base, crate_name);
    }

    // Build path string
    let path_str = path.join("/");

    // Different URL formats based on item type
    match item_type {
        "fn" => format!("{}/{}/fn.{}.html", base, path_str, name),
        "struct" => format!("{}/{}/struct.{}.html", base, path_str, name),
        "enum" => format!("{}/{}/enum.{}.html", base, path_str, name),
        "trait" => format!("{}/{}/trait.{}.html", base, path_str, name),
        "type" => format!("{}/{}/type.{}.html", base, path_str, name),
        "mod" => format!("{}/{}/index.html", base, path_str),
        "macro" => format!("{}/{}/macro.{}.html", base, path_str, name),
        "constant" => format!("{}/{}/constant.{}.html", base, path_str, name),
        "static" => format!("{}/{}/static.{}.html", base, path_str, name),
        _ => format!("{}/{}/", base, path_str),
    }
}
