use anyhow::{Context, Result, bail};
use directories::ProjectDirs;
use rustdoc_types::Crate;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Load documentation from a local rustdoc JSON file
pub fn load_local_docs(path: &Path) -> Result<Crate> {
    let json_data = fs::read_to_string(path)
        .with_context(|| format!("Failed to read local rustdoc JSON at {}", path.display()))?;

    let krate: Crate =
        serde_json::from_str(&json_data).context("Failed to parse local rustdoc JSON")?;

    Ok(krate)
}

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

/// Check if a character is valid for crate names and versions.
/// Allows alphanumeric characters, hyphens, underscores, dots, and plus signs.
fn is_valid_path_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '+'
}

/// Validate that a string is safe to use as a path component.
/// Rejects empty strings, path separators, and path traversal components.
fn validate_path_component(value: &str, component_name: &str) -> Result<()> {
    if value.is_empty() {
        bail!("{} cannot be empty", component_name);
    }

    // Reject path separators
    if value.contains('/') || value.contains('\\') {
        bail!("{} contains invalid path separator", component_name);
    }

    // Reject path traversal components
    if value == "." || value == ".." || value.contains("..") {
        bail!("{} contains invalid path component", component_name);
    }

    // First character must be alphanumeric
    let first_char = value.chars().next().unwrap();
    if !first_char.is_ascii_alphanumeric() {
        bail!(
            "{} contains invalid characters (allowed: alphanumeric, hyphen, underscore, dot, plus)",
            component_name
        );
    }

    // All characters must be valid
    if !value.chars().all(is_valid_path_char) {
        bail!(
            "{} contains invalid characters (allowed: alphanumeric, hyphen, underscore, dot, plus)",
            component_name
        );
    }

    Ok(())
}

/// Get the cache file path for a specific crate and version.
/// Validates inputs and ensures the resulting path stays within the cache directory.
fn get_cache_path(crate_name: &str, version: &str) -> Result<PathBuf> {
    // Validate inputs
    validate_path_component(crate_name, "crate name")?;
    validate_path_component(version, "version")?;

    let cache_dir = get_cache_dir()?;

    // Verify the path stays within the cache directory.
    // We need to handle the case where the path doesn't exist yet,
    // so we canonicalize the cache_dir and check the joined path components.
    let canonical_cache_dir = cache_dir
        .canonicalize()
        .unwrap_or_else(|_| cache_dir.clone());

    // For the cache path, we need to build it from the canonical cache dir
    // since the file may not exist yet
    let safe_cache_path = canonical_cache_dir
        .join(crate_name)
        .join(format!("{}.zst", version));

    // Double-check that no path traversal occurred by verifying the path starts with cache_dir
    if !safe_cache_path.starts_with(&canonical_cache_dir) {
        bail!("Path traversal detected: resulting path escapes cache directory");
    }

    Ok(safe_cache_path)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path_component_valid() {
        // Valid crate names
        assert!(validate_path_component("serde", "crate name").is_ok());
        assert!(validate_path_component("serde_json", "crate name").is_ok());
        assert!(validate_path_component("my-crate", "crate name").is_ok());
        assert!(validate_path_component("my_crate-v2", "crate name").is_ok());

        // Valid versions
        assert!(validate_path_component("1.0.0", "version").is_ok());
        assert!(validate_path_component("0.1.0-beta.1", "version").is_ok());
        assert!(validate_path_component("1.0.0+build123", "version").is_ok());
    }

    #[test]
    fn test_validate_path_component_empty() {
        let result = validate_path_component("", "crate name");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_path_component_path_separator() {
        // Forward slash
        let result = validate_path_component("../etc", "crate name");
        assert!(result.is_err());

        let result = validate_path_component("foo/bar", "crate name");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("path separator"));

        // Backslash
        let result = validate_path_component("..\\etc", "crate name");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("path separator"));
    }

    #[test]
    fn test_validate_path_component_path_traversal() {
        let result = validate_path_component("..", "crate name");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("invalid path component")
        );

        let result = validate_path_component(".", "crate name");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_component_invalid_chars() {
        // Starting with non-alphanumeric
        let result = validate_path_component("-foo", "crate name");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("invalid characters")
        );

        let result = validate_path_component(".hidden", "crate name");
        assert!(result.is_err());

        // Special characters
        let result = validate_path_component("foo@bar", "crate name");
        assert!(result.is_err());

        let result = validate_path_component("foo bar", "crate name");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_cache_path_valid() {
        let result = get_cache_path("serde", "1.0.0");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("serde"));
        assert!(path.to_string_lossy().ends_with("1.0.0.zst"));
    }

    #[test]
    fn test_get_cache_path_path_traversal_rejected() {
        // Attempt path traversal in crate name
        let result = get_cache_path("../../../etc", "passwd");
        assert!(result.is_err());

        // Attempt path traversal in version
        let result = get_cache_path("serde", "../../../etc/passwd");
        assert!(result.is_err());

        // Attempt via embedded path separator
        let result = get_cache_path("foo/bar", "1.0.0");
        assert!(result.is_err());
    }
}
