use anyhow::{Context, Result};
use cargo_metadata::{Metadata, MetadataCommand};
use std::env;
use std::path::PathBuf;

use crate::util::normalize_crate_name;

pub struct VersionResolver {
    metadata: Metadata,
}

impl VersionResolver {
    /// Create a new VersionResolver by finding and loading the nearest Cargo.toml
    pub fn new() -> Result<Self> {
        let manifest_path = Self::find_cargo_toml()
            .context("No Cargo.toml found in current directory or parent directories")?;

        let metadata = MetadataCommand::new()
            .manifest_path(&manifest_path)
            .exec()
            .context("Failed to execute cargo metadata")?;

        Ok(Self { metadata })
    }

    /// Find Cargo.toml by searching current directory and parent directories
    fn find_cargo_toml() -> Option<PathBuf> {
        let mut current_dir = env::current_dir().ok()?;

        loop {
            let manifest_path = current_dir.join("Cargo.toml");
            if manifest_path.exists() {
                return Some(manifest_path);
            }

            // Move to parent directory
            if !current_dir.pop() {
                // Reached filesystem root
                return None;
            }
        }
    }

    /// Resolve the actual version of a crate from the resolved dependency graph
    ///
    /// Returns the exact version from Cargo.lock (e.g., "1.0.5" instead of "^1.0")
    /// by looking at the resolved dependency graph.
    ///
    /// Returns the resolved version string if found, None otherwise
    pub fn resolve_version(&self, crate_name: &str) -> Option<String> {
        // Get the resolve graph
        let resolve = self.metadata.resolve.as_ref()?;

        // Collect all package IDs that are in the resolved dependency graph
        let resolved_ids: std::collections::HashSet<_> =
            resolve.nodes.iter().map(|node| &node.id).collect();

        // Find workspace member packages that depend on this crate
        for package in &self.metadata.packages {
            if self.metadata.workspace_members.contains(&package.id) {
                // Check if any dependencies match the crate name (normalize for comparison)
                if package
                    .dependencies
                    .iter()
                    .any(|dep| normalize_crate_name(&dep.name) == crate_name)
                {
                    // Now find the actual resolved version in the packages list
                    for pkg in &self.metadata.packages {
                        if normalize_crate_name(&pkg.name) == crate_name
                            && resolved_ids.contains(&pkg.id)
                        {
                            return Some(pkg.version.to_string());
                        }
                    }
                }
            }
        }

        None
    }

    /// Check if a crate is a local workspace member
    pub fn is_local_crate(&self, crate_name: &str) -> bool {
        self.metadata.workspace_members.iter().any(|member_id| {
            self.metadata
                .packages
                .iter()
                .any(|pkg| pkg.id == *member_id && normalize_crate_name(&pkg.name) == crate_name)
        })
    }

    /// Get the path to the rustdoc JSON file for a local workspace crate
    ///
    /// Returns None if the crate is not a workspace member or if the path doesn't exist
    pub fn get_local_crate_doc_path(&self, crate_name: &str) -> Option<PathBuf> {
        if !self.is_local_crate(crate_name) {
            return None;
        }

        // Crate names are already normalized (hyphens → underscores) at input
        let doc_path: PathBuf = self
            .metadata
            .target_directory
            .join("doc")
            .join(format!("{}.json", crate_name))
            .into();

        if doc_path.exists() {
            Some(doc_path)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_cargo_toml_in_current_project() {
        // This test runs in the context of the project, so it should find Cargo.toml
        let result = VersionResolver::find_cargo_toml();
        assert!(result.is_some());
        let path = result.unwrap();
        assert!(path.exists());
        assert!(path.ends_with("Cargo.toml"));
    }

    #[test]
    fn test_new_resolver_succeeds() {
        // Should successfully create a resolver in the project directory
        let result = VersionResolver::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_version_for_known_dependency() {
        // Test with clap which is a dependency in this project
        let resolver = VersionResolver::new().unwrap();
        let version = resolver.resolve_version("clap");
        assert!(version.is_some());
        let ver = version.unwrap();
        // Should be a version string like "4.5.52"
        assert!(ver.starts_with("4."));
    }

    #[test]
    fn test_resolve_version_for_cargo_metadata() {
        // Test with cargo_metadata which is also a dependency
        let resolver = VersionResolver::new().unwrap();
        let version = resolver.resolve_version("cargo_metadata");
        assert!(version.is_some());
        let ver = version.unwrap();
        // Should be a version string like "0.23.1"
        assert!(ver.starts_with("0."));
    }

    #[test]
    fn test_resolve_version_for_anyhow() {
        // Test with anyhow which is also a dependency
        let resolver = VersionResolver::new().unwrap();
        let version = resolver.resolve_version("anyhow");
        assert!(version.is_some());
        let ver = version.unwrap();
        // Should be a version string like "1.0.100"
        assert!(ver.starts_with("1."));
    }

    #[test]
    fn test_resolve_version_for_unknown_crate() {
        // Test with a crate that's not a dependency
        let resolver = VersionResolver::new().unwrap();
        let version = resolver.resolve_version("some_unknown_crate_xyz");
        assert!(version.is_none());
    }

    #[test]
    fn test_resolve_version_returns_exact_version() {
        // Ensure we get exact versions, not version requirements
        let resolver = VersionResolver::new().unwrap();
        let version = resolver.resolve_version("clap").unwrap();
        // Should not contain requirement characters like ^, >=, etc.
        assert!(!version.contains('^'));
        assert!(!version.contains(">="));
        assert!(!version.contains("<="));
    }
}
