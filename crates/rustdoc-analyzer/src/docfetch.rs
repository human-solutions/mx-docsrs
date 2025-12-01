//! Download and cache rustdoc JSON from docs.rs

use anyhow::{Context, Result, bail};
use directories::ProjectDirs;
use rustdoc_types::Crate;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

/// Fetch and parse documentation from docs.rs
pub fn fetch_docs(crate_name: &str, version: &str, use_cache: bool) -> Result<Crate> {
    let compressed_data = if use_cache {
        match load_from_cache(crate_name, version) {
            Ok(data) => data,
            Err(_) => download_and_cache(crate_name, version)?,
        }
    } else {
        download_rustdoc_json(crate_name, version)?
    };

    let decompressed_data =
        zstd::decode_all(&compressed_data[..]).context("Failed to decompress zstd data")?;

    let krate: Crate =
        serde_json::from_slice(&decompressed_data).context("Failed to parse rustdoc JSON")?;

    Ok(krate)
}

/// Get the cache directory path for rustdoc JSON files
fn get_cache_dir() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("", "", "rustdoc-analyzer")
        .context("Failed to determine cache directory")?;
    Ok(proj_dirs.cache_dir().to_path_buf())
}

fn is_valid_path_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '+'
}

fn validate_path_component(value: &str, component_name: &str) -> Result<()> {
    if value.is_empty() {
        bail!("{} cannot be empty", component_name);
    }

    if value.contains('/') || value.contains('\\') {
        bail!("{} contains invalid path separator", component_name);
    }

    if value == "." || value == ".." || value.contains("..") {
        bail!("{} contains invalid path component", component_name);
    }

    let first_char = value.chars().next().unwrap();
    if !first_char.is_ascii_alphanumeric() {
        bail!(
            "{} contains invalid characters (allowed: alphanumeric, hyphen, underscore, dot, plus)",
            component_name
        );
    }

    if !value.chars().all(is_valid_path_char) {
        bail!(
            "{} contains invalid characters (allowed: alphanumeric, hyphen, underscore, dot, plus)",
            component_name
        );
    }

    Ok(())
}

fn get_cache_path(crate_name: &str, version: &str) -> Result<PathBuf> {
    validate_path_component(crate_name, "crate name")?;
    validate_path_component(version, "version")?;

    let cache_dir = get_cache_dir()?;
    let canonical_cache_dir = cache_dir
        .canonicalize()
        .unwrap_or_else(|_| cache_dir.clone());

    let safe_cache_path = canonical_cache_dir
        .join(crate_name)
        .join(format!("{}.zst", version));

    if !safe_cache_path.starts_with(&canonical_cache_dir) {
        bail!("Path traversal detected: resulting path escapes cache directory");
    }

    Ok(safe_cache_path)
}

fn load_from_cache(crate_name: &str, version: &str) -> Result<Vec<u8>> {
    let cache_path = get_cache_path(crate_name, version)?;
    fs::read(&cache_path).context("Cache miss")
}

fn save_to_cache(crate_name: &str, version: &str, data: &[u8]) -> Result<()> {
    let cache_path = get_cache_path(crate_name, version)?;

    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&cache_path, data).context("Failed to save to cache")?;
    Ok(())
}

fn download_rustdoc_json(crate_name: &str, version: &str) -> Result<Vec<u8>> {
    let url = format!("https://docs.rs/crate/{}/{}/json", crate_name, version);

    let mut response = ureq::get(&url).call()?;

    let mut compressed_data = Vec::new();
    response
        .body_mut()
        .as_reader()
        .read_to_end(&mut compressed_data)?;

    Ok(compressed_data)
}

fn download_and_cache(crate_name: &str, version: &str) -> Result<Vec<u8>> {
    let compressed_data = download_rustdoc_json(crate_name, version)?;

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
