//! Trait for resolving documentation links.

/// Trait for resolving documentation links in markdown.
///
/// Implement this trait to provide custom link resolution logic,
/// such as resolving intra-doc links to fully qualified paths.
pub trait LinkResolver {
    /// Resolve a link to display text.
    ///
    /// # Arguments
    /// * `link_text` - The visible text of the link
    /// * `dest_url` - The destination URL or path
    ///
    /// # Returns
    /// The text to display (may include URL for external links)
    fn resolve_link(&self, link_text: &str, dest_url: &str) -> String;
}

/// Default resolver that formats external URLs and returns text as-is for others.
///
/// - External URLs (http/https) are formatted as "text (url)"
/// - All other links return the link text unchanged
pub struct DefaultLinkResolver;

impl LinkResolver for DefaultLinkResolver {
    fn resolve_link(&self, link_text: &str, dest_url: &str) -> String {
        if dest_url.starts_with("http://") || dest_url.starts_with("https://") {
            format!("{link_text} ({dest_url})")
        } else {
            link_text.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_external_url() {
        let resolver = DefaultLinkResolver;
        let result = resolver.resolve_link("docs", "https://docs.rs/tokio");
        assert_eq!(result, "docs (https://docs.rs/tokio)");
    }

    #[test]
    fn test_http_url() {
        let resolver = DefaultLinkResolver;
        let result = resolver.resolve_link("example", "http://example.com");
        assert_eq!(result, "example (http://example.com)");
    }

    #[test]
    fn test_internal_link() {
        let resolver = DefaultLinkResolver;
        let result = resolver.resolve_link("SomeType", "SomeType");
        assert_eq!(result, "SomeType");
    }

    #[test]
    fn test_empty_dest() {
        let resolver = DefaultLinkResolver;
        let result = resolver.resolve_link("text", "");
        assert_eq!(result, "text");
    }
}
