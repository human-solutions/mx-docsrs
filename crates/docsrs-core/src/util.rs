/// Normalize crate name by replacing hyphens with underscores (Cargo convention)
pub fn normalize_crate_name(name: &str) -> String {
    name.replace('-', "_")
}

#[cfg(test)]
mod tests {
    use super::*;

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
