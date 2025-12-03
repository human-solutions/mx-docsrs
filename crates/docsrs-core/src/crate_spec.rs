use anyhow::{Result, bail};
use std::str::FromStr;

use crate::util::normalize_crate_name;

/// Represents a crate specification with optional version and path prefix
///
/// Syntax: `crate[@version][::path::segments]`
///
/// Examples:
/// - `tokio` → name="tokio", version=None, path_prefix=None
/// - `tokio@1.0` → name="tokio", version=Some("1.0"), path_prefix=None
/// - `tokio::task` → name="tokio", version=None, path_prefix=Some("task")
/// - `tokio@1.0::task::spawn` → name="tokio", version=Some("1.0"), path_prefix=Some("task::spawn")
#[derive(Debug, Clone)]
pub struct CrateSpec {
    pub name: String,
    pub version: Option<String>,
    pub path_prefix: Option<String>,
}

impl CrateSpec {
    pub fn parse(input: &str) -> Result<Self> {
        // First, split on '@' to separate name from version+path
        let (name, remainder) = if let Some(at_pos) = input.find('@') {
            let name = &input[..at_pos];
            let remainder = &input[at_pos + 1..];
            (name, Some(remainder))
        } else {
            // No '@', check for '::' to separate name from path
            if let Some(colons_pos) = input.find("::") {
                let name = &input[..colons_pos];
                let path = &input[colons_pos + 2..];
                return Self::build(name, None, Some(path));
            }
            (input, None)
        };

        // Parse remainder (version and optional path)
        let (version, path_prefix) = if let Some(rem) = remainder {
            if let Some(colons_pos) = rem.find("::") {
                let version = &rem[..colons_pos];
                let path = &rem[colons_pos + 2..];
                (Some(version), Some(path))
            } else {
                (Some(rem), None)
            }
        } else {
            (None, None)
        };

        Self::build(name, version, path_prefix)
    }

    fn build(name: &str, version: Option<&str>, path_prefix: Option<&str>) -> Result<Self> {
        if name.trim().is_empty() {
            bail!("Crate name cannot be empty");
        }
        if let Some(v) = version
            && v.trim().is_empty()
        {
            bail!("Version cannot be empty after '@'");
        }

        // Normalize path_prefix: trim trailing '::', treat empty as None
        let path_prefix = path_prefix.map(|p| p.trim_end_matches("::")).and_then(|p| {
            if p.is_empty() {
                None
            } else {
                Some(p.to_string())
            }
        });

        Ok(CrateSpec {
            name: normalize_crate_name(name),
            version: version.map(|v| v.to_string()),
            path_prefix,
        })
    }
}

impl FromStr for CrateSpec {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_crate_only() {
        let spec = CrateSpec::parse("tokio").unwrap();
        assert_eq!(spec.name, "tokio");
        assert_eq!(spec.version, None);
        assert_eq!(spec.path_prefix, None);
    }

    #[test]
    fn test_parse_crate_with_version() {
        let spec = CrateSpec::parse("tokio@1.0.0").unwrap();
        assert_eq!(spec.name, "tokio");
        assert_eq!(spec.version, Some("1.0.0".to_string()));
        assert_eq!(spec.path_prefix, None);
    }

    #[test]
    fn test_parse_crate_with_version_requirement() {
        let spec = CrateSpec::parse("serde@^1.0").unwrap();
        assert_eq!(spec.name, "serde");
        assert_eq!(spec.version, Some("^1.0".to_string()));
        assert_eq!(spec.path_prefix, None);
    }

    #[test]
    fn test_parse_crate_with_complex_version() {
        let spec = CrateSpec::parse("clap@>=4.0,<5.0").unwrap();
        assert_eq!(spec.name, "clap");
        assert_eq!(spec.version, Some(">=4.0,<5.0".to_string()));
        assert_eq!(spec.path_prefix, None);
    }

    #[test]
    fn test_parse_crate_with_path() {
        let spec = CrateSpec::parse("tokio::task").unwrap();
        assert_eq!(spec.name, "tokio");
        assert_eq!(spec.version, None);
        assert_eq!(spec.path_prefix, Some("task".to_string()));
    }

    #[test]
    fn test_parse_crate_with_deep_path() {
        let spec = CrateSpec::parse("tokio::task::spawn").unwrap();
        assert_eq!(spec.name, "tokio");
        assert_eq!(spec.version, None);
        assert_eq!(spec.path_prefix, Some("task::spawn".to_string()));
    }

    #[test]
    fn test_parse_crate_with_version_and_path() {
        let spec = CrateSpec::parse("tokio@1.0::task").unwrap();
        assert_eq!(spec.name, "tokio");
        assert_eq!(spec.version, Some("1.0".to_string()));
        assert_eq!(spec.path_prefix, Some("task".to_string()));
    }

    #[test]
    fn test_parse_crate_with_version_and_deep_path() {
        let spec = CrateSpec::parse("tokio@1.0::task::spawn").unwrap();
        assert_eq!(spec.name, "tokio");
        assert_eq!(spec.version, Some("1.0".to_string()));
        assert_eq!(spec.path_prefix, Some("task::spawn".to_string()));
    }

    #[test]
    fn test_parse_path_with_trailing_colons() {
        let spec = CrateSpec::parse("tokio::task::").unwrap();
        assert_eq!(spec.name, "tokio");
        assert_eq!(spec.path_prefix, Some("task".to_string()));
    }

    #[test]
    fn test_parse_empty_name_fails() {
        let result = CrateSpec::parse("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_parse_empty_name_with_version_fails() {
        let result = CrateSpec::parse("@1.0.0");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_parse_empty_version_fails() {
        let result = CrateSpec::parse("tokio@");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_parse_whitespace_only_name_fails() {
        let result = CrateSpec::parse("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str_trait() {
        let spec: CrateSpec = "tokio@1.0.0".parse().unwrap();
        assert_eq!(spec.name, "tokio");
        assert_eq!(spec.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_multiple_at_signs() {
        // Only split on first @, rest is part of version
        let spec = CrateSpec::parse("crate@1.0@beta").unwrap();
        assert_eq!(spec.name, "crate");
        assert_eq!(spec.version, Some("1.0@beta".to_string()));
    }

    #[test]
    fn test_empty_path_becomes_none() {
        let spec = CrateSpec::parse("tokio::").unwrap();
        assert_eq!(spec.name, "tokio");
        assert_eq!(spec.path_prefix, None);
    }

    #[test]
    fn test_normalize_hyphen_to_underscore() {
        let spec = CrateSpec::parse("serde-json").unwrap();
        assert_eq!(spec.name, "serde_json");
    }

    #[test]
    fn test_normalize_underscore_unchanged() {
        let spec = CrateSpec::parse("serde_json").unwrap();
        assert_eq!(spec.name, "serde_json");
    }
}
