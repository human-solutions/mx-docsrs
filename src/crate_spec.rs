use anyhow::{bail, Result};
use std::str::FromStr;

/// Represents a crate specification with optional version
#[derive(Debug, Clone)]
pub struct CrateSpec {
    pub name: String,
    pub version: Option<String>,
}

impl CrateSpec {
    /// Parse a crate specification from a string
    ///
    /// Format: `crate_name` or `crate_name@version`
    ///
    /// # Examples
    ///
    /// ```
    /// let spec = CrateSpec::parse("tokio")?;
    /// assert_eq!(spec.name, "tokio");
    /// assert_eq!(spec.version, None);
    ///
    /// let spec = CrateSpec::parse("tokio@1.0.0")?;
    /// assert_eq!(spec.name, "tokio");
    /// assert_eq!(spec.version, Some("1.0.0".to_string()));
    /// ```
    pub fn parse(input: &str) -> Result<Self> {
        let parts: Vec<&str> = input.splitn(2, '@').collect();

        match parts.as_slice() {
            [name] => {
                if name.trim().is_empty() {
                    bail!("Crate name cannot be empty");
                }
                Ok(CrateSpec {
                    name: name.to_string(),
                    version: None,
                })
            }
            [name, version] => {
                if name.trim().is_empty() {
                    bail!("Crate name cannot be empty");
                }
                if version.trim().is_empty() {
                    bail!("Version cannot be empty after '@'");
                }
                Ok(CrateSpec {
                    name: name.to_string(),
                    version: Some(version.to_string()),
                })
            }
            _ => unreachable!("splitn(2) always returns 1 or 2 elements"),
        }
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
    }

    #[test]
    fn test_parse_crate_with_version() {
        let spec = CrateSpec::parse("tokio@1.0.0").unwrap();
        assert_eq!(spec.name, "tokio");
        assert_eq!(spec.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_parse_crate_with_version_requirement() {
        let spec = CrateSpec::parse("serde@^1.0").unwrap();
        assert_eq!(spec.name, "serde");
        assert_eq!(spec.version, Some("^1.0".to_string()));
    }

    #[test]
    fn test_parse_crate_with_complex_version() {
        let spec = CrateSpec::parse("clap@>=4.0,<5.0").unwrap();
        assert_eq!(spec.name, "clap");
        assert_eq!(spec.version, Some(">=4.0,<5.0".to_string()));
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
}
