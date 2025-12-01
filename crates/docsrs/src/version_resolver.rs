use anyhow::{Context, Result};
use cargo_metadata::{DependencyKind as CargoDependencyKind, Metadata, MetadataCommand, PackageId};
use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::path::PathBuf;

use crate::util::normalize_crate_name;

/// The kind of dependency
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyKind {
    Normal,
    Dev,
    Build,
}

impl std::fmt::Display for DependencyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyKind::Normal => write!(f, "dependency"),
            DependencyKind::Dev => write!(f, "dev-dependency"),
            DependencyKind::Build => write!(f, "build-dependency"),
        }
    }
}

impl From<CargoDependencyKind> for DependencyKind {
    fn from(kind: CargoDependencyKind) -> Self {
        match kind {
            CargoDependencyKind::Normal => DependencyKind::Normal,
            CargoDependencyKind::Development => DependencyKind::Dev,
            CargoDependencyKind::Build => DependencyKind::Build,
            _ => DependencyKind::Normal,
        }
    }
}

/// Information about a resolved crate dependency
#[derive(Debug, Clone)]
pub struct ResolvedCrate {
    /// The actual package name (not the alias)
    pub name: String,
    /// The resolved version
    pub version: String,
    /// The kind of dependency (normal, dev, build)
    pub kind: DependencyKind,
    /// Whether this is a local workspace crate
    pub is_local: bool,
    /// Path for local crates
    pub local_path: Option<PathBuf>,
    /// If the dependency was renamed in Cargo.toml (the alias used)
    pub alias: Option<String>,
    /// For transitive dependencies: the chain from workspace member to target
    /// e.g., ["my-app", "serde", "serde_derive"]
    pub dep_chain: Option<Vec<String>>,
}

impl ResolvedCrate {
    /// Format the resolution message for display
    pub fn format_message(&self) -> String {
        if self.is_local {
            let path = self
                .local_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| ".".to_string());
            format!(
                "Using local dependency at {} (version {})",
                path, self.version
            )
        } else if let Some(chain) = &self.dep_chain {
            // Transitive dependency
            let chain_str = format_dep_chain(chain);
            format!("Found {}@{} via: {}", self.name, self.version, chain_str)
        } else {
            // Direct dependency
            let alias_suffix = self
                .alias
                .as_ref()
                .map(|a| format!(" (aliased as '{}')", a))
                .unwrap_or_default();
            format!(
                "Using {} {}@{}{}",
                self.kind, self.name, self.version, alias_suffix
            )
        }
    }
}

/// Format dependency chain, truncating if > 3 elements
fn format_dep_chain(chain: &[String]) -> String {
    if chain.len() <= 3 {
        chain.join(" → ")
    } else {
        // Show: first → second → ... → last
        format!(
            "{} → {} → ... → {}",
            chain[0],
            chain[1],
            chain.last().unwrap()
        )
    }
}

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

    /// Resolve a crate to get detailed information about the dependency
    ///
    /// Returns `ResolvedCrate` with version, kind, and dependency chain info.
    /// Prioritizes: local workspace crate > direct dependency > transitive dependency
    pub fn resolve_crate(&self, crate_name: &str) -> Option<ResolvedCrate> {
        let normalized_name = normalize_crate_name(crate_name);

        // 1. Check if it's a local workspace crate
        if let Some(resolved) = self.resolve_local_crate(&normalized_name) {
            return Some(resolved);
        }

        // 2. Check for direct dependencies (collect all matches)
        let mut direct_matches = self.find_direct_dependencies(&normalized_name);

        // 3. If no direct matches, look for transitive dependencies
        if direct_matches.is_empty() {
            if let Some(resolved) = self.find_transitive_dependency(&normalized_name) {
                return Some(resolved);
            }
            return None;
        }

        // 4. If multiple direct matches (different versions), we take the first one
        //    (direct always wins, and we note if there are multiple)
        Some(direct_matches.remove(0))
    }

    /// Resolve a local workspace crate
    fn resolve_local_crate(&self, crate_name: &str) -> Option<ResolvedCrate> {
        for member_id in &self.metadata.workspace_members {
            for pkg in &self.metadata.packages {
                if pkg.id == *member_id && normalize_crate_name(&pkg.name) == crate_name {
                    let local_path = pkg.manifest_path.parent().map(|p| p.into());
                    return Some(ResolvedCrate {
                        name: pkg.name.to_string(),
                        version: pkg.version.to_string(),
                        kind: DependencyKind::Normal,
                        is_local: true,
                        local_path,
                        alias: None,
                        dep_chain: None,
                    });
                }
            }
        }
        None
    }

    /// Find direct dependencies of workspace members that match the crate name
    fn find_direct_dependencies(&self, crate_name: &str) -> Vec<ResolvedCrate> {
        let resolve = match self.metadata.resolve.as_ref() {
            Some(r) => r,
            None => return vec![],
        };

        let resolved_ids: HashSet<_> = resolve.nodes.iter().map(|node| &node.id).collect();
        let mut results = vec![];

        // Search through workspace members
        for package in &self.metadata.packages {
            if !self.metadata.workspace_members.contains(&package.id) {
                continue;
            }

            // Check each dependency
            for dep in &package.dependencies {
                // Check if this dependency matches (by name or by rename/alias)
                let dep_normalized = normalize_crate_name(&dep.name);
                let rename_normalized = dep.rename.as_ref().map(|r| normalize_crate_name(r));

                let (matches, alias) = if dep_normalized == crate_name {
                    (true, None)
                } else if rename_normalized.as_deref() == Some(crate_name) {
                    // User searched by alias, resolve to actual package name
                    (true, Some(crate_name.to_string()))
                } else {
                    (false, None)
                };

                if matches {
                    // Find the resolved version in packages
                    for pkg in &self.metadata.packages {
                        if normalize_crate_name(&pkg.name) == dep_normalized
                            && resolved_ids.contains(&pkg.id)
                        {
                            results.push(ResolvedCrate {
                                name: pkg.name.to_string(),
                                version: pkg.version.to_string(),
                                kind: dep.kind.into(),
                                is_local: false,
                                local_path: None,
                                alias,
                                dep_chain: None,
                            });
                            break;
                        }
                    }
                }
            }
        }

        results
    }

    /// Find a transitive dependency using BFS through the resolve graph
    fn find_transitive_dependency(&self, crate_name: &str) -> Option<ResolvedCrate> {
        let resolve = self.metadata.resolve.as_ref()?;

        // Build a map from PackageId to package name for quick lookup
        let pkg_names: HashMap<&PackageId, &str> = self
            .metadata
            .packages
            .iter()
            .map(|p| (&p.id, p.name.as_str()))
            .collect();

        // Build adjacency list from resolve graph
        let mut adj: HashMap<&PackageId, Vec<&PackageId>> = HashMap::new();
        for node in &resolve.nodes {
            let deps: Vec<_> = node.deps.iter().map(|d| &d.pkg).collect();
            adj.insert(&node.id, deps);
        }

        // BFS from each workspace member
        for member_id in &self.metadata.workspace_members {
            if let Some((canonical_name, version, chain)) =
                self.bfs_find_crate(&adj, &pkg_names, member_id, crate_name)
            {
                return Some(ResolvedCrate {
                    name: canonical_name,
                    version,
                    kind: DependencyKind::Normal, // Transitive deps are always "normal" from our perspective
                    is_local: false,
                    local_path: None,
                    alias: None,
                    dep_chain: Some(chain),
                });
            }
        }

        None
    }

    /// BFS to find a crate in the dependency graph, returning the canonical name, version, and path taken
    fn bfs_find_crate(
        &self,
        adj: &HashMap<&PackageId, Vec<&PackageId>>,
        pkg_names: &HashMap<&PackageId, &str>,
        start: &PackageId,
        target: &str,
    ) -> Option<(String, String, Vec<String>)> {
        let mut visited: HashSet<&PackageId> = HashSet::new();
        let mut queue: VecDeque<(&PackageId, Vec<String>)> = VecDeque::new();

        // Get start package name
        let start_name = pkg_names.get(start)?;
        queue.push_back((start, vec![start_name.to_string()]));
        visited.insert(start);

        while let Some((current, path)) = queue.pop_front() {
            if let Some(deps) = adj.get(current) {
                for dep_id in deps {
                    if visited.contains(dep_id) {
                        continue;
                    }
                    visited.insert(dep_id);

                    let dep_name = pkg_names.get(dep_id)?;
                    let mut new_path = path.clone();
                    new_path.push(dep_name.to_string());

                    // Check if this is our target
                    if normalize_crate_name(dep_name) == target {
                        // Find version and canonical name
                        for pkg in &self.metadata.packages {
                            if &pkg.id == *dep_id {
                                return Some((
                                    pkg.name.to_string(),
                                    pkg.version.to_string(),
                                    new_path,
                                ));
                            }
                        }
                    }

                    queue.push_back((dep_id, new_path));
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
        // Normalize crate name: rustdoc generates JSON with underscores (e.g., test_visibility.json)
        let normalized = normalize_crate_name(crate_name);

        if !self.is_local_crate(&normalized) {
            return None;
        }

        let doc_path: PathBuf = self
            .metadata
            .target_directory
            .join("doc")
            .join(format!("{}.json", normalized))
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
    fn test_resolve_crate_for_direct_dependency() {
        // Test with clap which is a direct dependency in this project
        let resolver = VersionResolver::new().unwrap();
        let resolved = resolver.resolve_crate("clap");
        assert!(resolved.is_some());

        let resolved = resolved.unwrap();
        assert_eq!(resolved.name, "clap");
        assert!(resolved.version.starts_with("4."));
        assert_eq!(resolved.kind, DependencyKind::Normal);
        assert!(!resolved.is_local);
        assert!(resolved.dep_chain.is_none());
    }

    #[test]
    fn test_resolve_crate_for_cargo_metadata() {
        // Test with cargo_metadata which is also a dependency
        let resolver = VersionResolver::new().unwrap();
        let resolved = resolver.resolve_crate("cargo_metadata");
        assert!(resolved.is_some());

        let resolved = resolved.unwrap();
        assert_eq!(resolved.name, "cargo_metadata");
        assert!(resolved.version.starts_with("0."));
        assert!(!resolved.is_local);
    }

    #[test]
    fn test_resolve_crate_for_unknown_crate() {
        // Test with a crate that's not a dependency
        let resolver = VersionResolver::new().unwrap();
        let resolved = resolver.resolve_crate("some_unknown_crate_xyz");
        assert!(resolved.is_none());
    }

    #[test]
    fn test_resolve_crate_returns_exact_version() {
        // Ensure we get exact versions, not version requirements
        let resolver = VersionResolver::new().unwrap();
        let resolved = resolver.resolve_crate("clap").unwrap();
        // Should not contain requirement characters like ^, >=, etc.
        assert!(!resolved.version.contains('^'));
        assert!(!resolved.version.contains(">="));
        assert!(!resolved.version.contains("<="));
    }

    #[test]
    fn test_resolve_crate_for_transitive_dependency() {
        // Test with a transitive dependency (e.g., serde_derive comes through serde)
        let resolver = VersionResolver::new().unwrap();
        let resolved = resolver.resolve_crate("clap_builder");
        // clap_builder is a transitive dependency of clap
        assert!(resolved.is_some());

        let resolved = resolved.unwrap();
        assert!(resolved.dep_chain.is_some());
        let chain = resolved.dep_chain.unwrap();
        assert!(chain.len() >= 2); // At least: workspace -> clap -> clap_builder
    }

    #[test]
    fn test_format_message_direct_dependency() {
        let resolved = ResolvedCrate {
            name: "serde".to_string(),
            version: "1.0.210".to_string(),
            kind: DependencyKind::Normal,
            is_local: false,
            local_path: None,
            alias: None,
            dep_chain: None,
        };
        assert_eq!(resolved.format_message(), "Using dependency serde@1.0.210");
    }

    #[test]
    fn test_format_message_dev_dependency() {
        let resolved = ResolvedCrate {
            name: "insta".to_string(),
            version: "1.43.0".to_string(),
            kind: DependencyKind::Dev,
            is_local: false,
            local_path: None,
            alias: None,
            dep_chain: None,
        };
        assert_eq!(
            resolved.format_message(),
            "Using dev-dependency insta@1.43.0"
        );
    }

    #[test]
    fn test_format_message_local_crate() {
        let resolved = ResolvedCrate {
            name: "my-lib".to_string(),
            version: "0.1.0".to_string(),
            kind: DependencyKind::Normal,
            is_local: true,
            local_path: Some(PathBuf::from("./crates/my-lib")),
            alias: None,
            dep_chain: None,
        };
        assert_eq!(
            resolved.format_message(),
            "Using local dependency at ./crates/my-lib (version 0.1.0)"
        );
    }

    #[test]
    fn test_format_message_transitive() {
        let resolved = ResolvedCrate {
            name: "serde_derive".to_string(),
            version: "1.0.210".to_string(),
            kind: DependencyKind::Normal,
            is_local: false,
            local_path: None,
            alias: None,
            dep_chain: Some(vec![
                "my-app".to_string(),
                "serde".to_string(),
                "serde_derive".to_string(),
            ]),
        };
        assert_eq!(
            resolved.format_message(),
            "Found serde_derive@1.0.210 via: my-app → serde → serde_derive"
        );
    }

    #[test]
    fn test_format_message_transitive_truncated() {
        let resolved = ResolvedCrate {
            name: "deep".to_string(),
            version: "0.1.0".to_string(),
            kind: DependencyKind::Normal,
            is_local: false,
            local_path: None,
            alias: None,
            dep_chain: Some(vec![
                "app".to_string(),
                "level1".to_string(),
                "level2".to_string(),
                "level3".to_string(),
                "deep".to_string(),
            ]),
        };
        assert_eq!(
            resolved.format_message(),
            "Found deep@0.1.0 via: app → level1 → ... → deep"
        );
    }

    #[test]
    fn test_format_message_aliased() {
        let resolved = ResolvedCrate {
            name: "serde_json".to_string(),
            version: "1.0.128".to_string(),
            kind: DependencyKind::Normal,
            is_local: false,
            local_path: None,
            alias: Some("json".to_string()),
            dep_chain: None,
        };
        assert_eq!(
            resolved.format_message(),
            "Using dependency serde_json@1.0.128 (aliased as 'json')"
        );
    }

    #[test]
    fn test_format_dep_chain_short() {
        let chain = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(format_dep_chain(&chain), "a → b → c");
    }

    #[test]
    fn test_format_dep_chain_long() {
        let chain = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
        ];
        assert_eq!(format_dep_chain(&chain), "a → b → ... → e");
    }
}
