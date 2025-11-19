use anyhow::{Context, Result};
use directories::ProjectDirs;
use rustdoc_types::Crate;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

/// Fetch and search documentation from docs.rs
/// Returns the search results and the parsed crate data
pub fn fetch_docs(crate_name: &str, version: &str, use_cache: bool) -> Result<Crate> {
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

    Ok(krate)
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
