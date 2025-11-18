use anyhow::{Context, Result};
use directories::ProjectDirs;
use rustdoc_types::{Crate, ItemEnum};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

use rustdoc_types::Id;

/// Represents a search result from the documentation
#[derive(Debug)]
pub struct DocResult {
    pub id: Id,
    pub name: String,
    pub item_type: String,
    pub path: Vec<String>,
    #[allow(dead_code)]
    pub url: String,
}

/// Fetch and search documentation from docs.rs
/// Returns the search results and the parsed crate data
pub fn fetch_docs(
    crate_name: &str,
    version: &str,
    symbol: &str,
    use_cache: bool,
) -> Result<(Vec<DocResult>, Crate)> {
    // Try to load from cache first
    let compressed_data = if use_cache {
        match load_from_cache(crate_name, version) {
            Ok(data) => data,
            Err(_) => {
                // Cache miss, download
                download_and_cache(crate_name, version)?
            }
        }
    } else {
        // Skip cache, download directly
        download_rustdoc_json(crate_name, version)?
    };

    // Decompress with zstd
    let decompressed_data =
        zstd::decode_all(&compressed_data[..]).context("Failed to decompress zstd data")?;

    // Parse rustdoc JSON
    let krate: Crate =
        serde_json::from_slice(&decompressed_data).context("Failed to parse rustdoc JSON")?;

    // Search for the symbol
    let mut results = search_items(&krate, symbol, crate_name, version);

    // Deduplicate results by FQDN (module_path + name)
    // The path now always contains only parent modules (not the item name)
    let mut seen = std::collections::HashSet::new();
    results.retain(|result| {
        // Build FQDN for deduplication
        let mut parts = result.path.clone();
        parts.push(result.name.clone());
        let fqdn = parts.join("::");
        seen.insert(fqdn)
    });

    Ok((results, krate))
}

/// Get the cache directory path for rustdoc JSON files
fn get_cache_dir() -> Result<PathBuf> {
    let proj_dirs =
        ProjectDirs::from("", "", "docsrs").context("Failed to determine cache directory")?;
    Ok(proj_dirs.cache_dir().to_path_buf())
}

/// Get the cache file path for a specific crate and version
fn get_cache_path(crate_name: &str, version: &str) -> Result<PathBuf> {
    let cache_dir = get_cache_dir()?;
    Ok(cache_dir.join(crate_name).join(format!("{}.zst", version)))
}

/// Load compressed rustdoc JSON from cache
fn load_from_cache(crate_name: &str, version: &str) -> Result<Vec<u8>> {
    let cache_path = get_cache_path(crate_name, version)?;
    fs::read(&cache_path).context("Cache miss")
}

/// Save compressed rustdoc JSON to cache
fn save_to_cache(crate_name: &str, version: &str, data: &[u8]) -> Result<()> {
    let cache_path = get_cache_path(crate_name, version)?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&cache_path, data).context("Failed to save to cache")?;
    println!("Saved to cache: {}", cache_path.display());
    Ok(())
}

/// Download rustdoc JSON from docs.rs
fn download_rustdoc_json(crate_name: &str, version: &str) -> Result<Vec<u8>> {
    println!("Fetching rustdoc JSON from docs.rs...");

    let url = format!("https://docs.rs/crate/{}/{}/json", crate_name, version);
    println!("URL: {}", url);

    let mut response = ureq::get(&url).call()?;
    let status = response.status();
    println!("Status: {}", status);

    if status != 200 {
        anyhow::bail!("Failed to fetch rustdoc JSON (status: {})", status);
    }

    let mut compressed_data = Vec::new();
    response
        .body_mut()
        .as_reader()
        .read_to_end(&mut compressed_data)?;
    println!("Downloaded {} bytes (compressed)", compressed_data.len());

    Ok(compressed_data)
}

/// Download and cache rustdoc JSON
fn download_and_cache(crate_name: &str, version: &str) -> Result<Vec<u8>> {
    let compressed_data = download_rustdoc_json(crate_name, version)?;

    // Save to cache (ignore errors)
    if let Err(e) = save_to_cache(crate_name, version, &compressed_data) {
        eprintln!("Warning: Failed to cache data: {}", e);
    }

    Ok(compressed_data)
}

/// Clear the entire cache directory
pub fn clear_cache() -> Result<()> {
    let cache_dir = get_cache_dir()?;

    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir).context("Failed to clear cache")?;
        println!("Cache cleared: {}", cache_dir.display());
    } else {
        println!("Cache directory does not exist");
    }

    Ok(())
}

/// Search through rustdoc items for matches
fn search_items(krate: &Crate, query: &str, crate_name: &str, version: &str) -> Vec<DocResult> {
    let mut results = Vec::new();

    // Check if query is a fully qualified domain name (contains "::")
    let is_fqdn = query.contains("::");

    if is_fqdn {
        // FQDN search: match exact path + name
        let parts: Vec<&str> = query.split("::").collect();
        if parts.is_empty() {
            return results;
        }

        let query_name = parts.last().unwrap();
        let query_path_parts: Vec<String> = parts[..parts.len() - 1]
            .iter()
            .map(|s| s.to_string())
            .collect();

        for (id, item) in &krate.index {
            let Some(ref name) = item.name else {
                continue;
            };

            // Check if name matches (case-insensitive)
            if name.to_lowercase() != query_name.to_lowercase() {
                continue;
            }

            // Build the module path (parent modules, without item name)
            // ItemSummary.path includes the item name as the last element, so we need to remove it
            let module_path = if let Some(summary) = krate.paths.get(id) {
                let mut path = summary.path.clone();
                path.pop(); // Remove the item name (last element)
                path
            } else {
                vec![crate_name.to_string()]
            };

            // Check if module path matches the query path
            if module_path.len() >= query_path_parts.len() {
                let path_lower: Vec<String> =
                    module_path.iter().map(|s| s.to_lowercase()).collect();
                let query_path_lower: Vec<String> =
                    query_path_parts.iter().map(|s| s.to_lowercase()).collect();

                // Check if the relevant portion of the path matches
                let matches = if query_path_parts.is_empty() {
                    true
                } else {
                    // Find the query path within the actual path
                    path_lower
                        .windows(query_path_lower.len())
                        .any(|window| window == query_path_lower.as_slice())
                };

                if matches {
                    let item_type = get_item_type(&item.inner);
                    let url = generate_url(crate_name, version, &module_path, name, item_type);

                    results.push(DocResult {
                        id: *id,
                        name: name.clone(),
                        item_type: item_type.to_string(),
                        path: module_path,
                        url,
                    });
                }
            }
        }
    } else {
        // Simple search: substring match on name (case-insensitive)
        let query_lower = query.to_lowercase();

        for (id, item) in &krate.index {
            let Some(ref name) = item.name else {
                continue;
            };

            if name.to_lowercase().contains(&query_lower) {
                // Build the module path (parent modules, without item name)
                // ItemSummary.path includes the item name as the last element, so we need to remove it
                let module_path = if let Some(summary) = krate.paths.get(id) {
                    let mut path = summary.path.clone();
                    path.pop(); // Remove the item name (last element)
                    path
                } else {
                    vec![crate_name.to_string()]
                };

                let item_type = get_item_type(&item.inner);
                let url = generate_url(crate_name, version, &module_path, name, item_type);

                results.push(DocResult {
                    id: *id,
                    name: name.clone(),
                    item_type: item_type.to_string(),
                    path: module_path,
                    url,
                });
            }
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
fn generate_url(
    crate_name: &str,
    version: &str,
    path: &[String],
    name: &str,
    item_type: &str,
) -> String {
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
