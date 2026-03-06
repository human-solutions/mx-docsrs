/// Normalize crate name by replacing hyphens with underscores (Cargo convention)
pub fn normalize_crate_name(name: &str) -> String {
    name.replace('-', "_")
}

/// Return an alternate crate name by swapping underscores and hyphens.
/// Returns `None` if the name contains neither.
pub fn alternate_crate_name(name: &str) -> Option<String> {
    if name.contains('_') {
        Some(name.replace('_', "-"))
    } else if name.contains('-') {
        Some(name.replace('-', "_"))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alternate_underscore_to_hyphen() {
        assert_eq!(
            alternate_crate_name("iroh_docs"),
            Some("iroh-docs".to_string())
        );
    }

    #[test]
    fn test_alternate_hyphen_to_underscore() {
        assert_eq!(
            alternate_crate_name("serde-json"),
            Some("serde_json".to_string())
        );
    }

    #[test]
    fn test_alternate_no_separator() {
        assert_eq!(alternate_crate_name("tokio"), None);
    }

    #[test]
    fn test_alternate_multiple_underscores() {
        assert_eq!(alternate_crate_name("a_b_c_d"), Some("a-b-c-d".to_string()));
    }

    #[test]
    fn test_normalize_hyphen_to_underscore() {
        assert_eq!(normalize_crate_name("serde-json"), "serde_json");
    }

    #[test]
    fn test_normalize_underscore_unchanged() {
        assert_eq!(normalize_crate_name("serde_json"), "serde_json");
    }

    #[test]
    fn test_normalize_no_change_needed() {
        assert_eq!(normalize_crate_name("tokio"), "tokio");
    }

    #[test]
    fn test_normalize_multiple_hyphens() {
        assert_eq!(normalize_crate_name("a-b-c-d"), "a_b_c_d");
    }
}
