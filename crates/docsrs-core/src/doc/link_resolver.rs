//! Resolves intra-doc links in documentation strings to fully qualified paths.

use std::{cmp::Ordering, collections::HashMap};

use jsondoc::JsonDocItem;
use rustdoc_fmt::LinkResolver;
use rustdoc_types::{Crate, Id};

/// A link resolver that uses rustdoc data to resolve intra-doc links.
pub struct RustdocLinkResolver<'a> {
    /// Links from the current item's documentation
    pub item_links: &'a HashMap<String, Id>,
    /// The rustdoc crate data
    pub krate: &'a Crate,
    /// Mapping from IDs to public items
    pub id_to_items: &'a HashMap<&'a Id, Vec<&'a JsonDocItem<'a>>>,
}

impl<'a> RustdocLinkResolver<'a> {
    /// Resolves a single link to its fully qualified path.
    ///
    /// External URLs are formatted as "text (url)".
    /// Intra-doc links are resolved via `Item.links` and `id_to_items`.
    fn resolve_single_link(&self, link_text: &str, dest_url: &str) -> String {
        // External URLs - format as "text (url)"
        if dest_url.starts_with("http://") || dest_url.starts_with("https://") {
            return format!("{link_text} ({dest_url})");
        }

        // Try to resolve via Item.links
        // Strip backticks for lookup - rustdoc normalizes these
        let lookup_key = link_text.trim_matches('`');

        if let Some(resolved_id) = self.item_links.get(lookup_key)
            && let Some(fqn_path) = self.id_to_public_path(resolved_id)
        {
            return fqn_path;
        }

        // Also try the dest_url as a key (for inline links like [text](Type::method))
        let dest_key = dest_url.trim_end_matches("()"); // Strip method parens
        if let Some(resolved_id) = self.item_links.get(dest_key)
            && let Some(fqn_path) = self.id_to_public_path(resolved_id)
        {
            return fqn_path;
        }

        // Unresolvable - return original link text
        link_text.to_string()
    }

    /// Convert an Id to a fully qualified public path string.
    ///
    /// First tries to find the best public path via `id_to_items` (re-exports).
    /// Falls back to `Crate.paths` for external items not in the public API.
    fn id_to_public_path(&self, id: &Id) -> Option<String> {
        // First, try to find a public path via id_to_items (handles re-exports)
        if let Some(items) = self.id_to_items.get(id)
            && let Some(best_item) = Self::best_item_for_id(items)
        {
            let path = Self::item_to_path_string(best_item);
            if !path.is_empty() {
                return Some(path);
            }
        }

        // Fallback: use Crate.paths for external items (std, core, etc.)
        if let Some(item_summary) = self.krate.paths.get(id) {
            let path = item_summary.path.join("::");
            return Some(path);
        }

        // Last resort: try to get name from index
        if let Some(item) = self.krate.index.get(id) {
            return item.name.clone();
        }

        None
    }

    /// Select the best public item path (same logic as RenderingContext::best_item_for_id)
    fn best_item_for_id(items: &[&'a JsonDocItem<'a>]) -> Option<&'a JsonDocItem<'a>> {
        items
            .iter()
            .max_by(|a, b| {
                // Prefer items without renamed path components
                let mut ordering = match (
                    a.path_contains_renamed_item(),
                    b.path_contains_renamed_item(),
                ) {
                    (true, false) => Ordering::Less,
                    (false, true) => Ordering::Greater,
                    _ => Ordering::Equal,
                };

                // If still equal, prefer shorter paths
                if ordering == Ordering::Equal {
                    ordering = b.path().len().cmp(&a.path().len());
                }

                ordering
            })
            .copied()
    }

    /// Convert a JsonDocItem to a path string like "tokio::task::JoinHandle"
    fn item_to_path_string(item: &JsonDocItem<'_>) -> String {
        item.path()
            .iter()
            .filter(|c| !c.hide)
            .filter_map(|c| c.item.name())
            .collect::<Vec<_>>()
            .join("::")
    }
}

impl LinkResolver for RustdocLinkResolver<'_> {
    fn resolve_link(&self, link_text: &str, dest_url: &str) -> String {
        self.resolve_single_link(link_text, dest_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_crate() -> Crate {
        Crate {
            root: Id(0),
            crate_version: None,
            includes_private: false,
            index: HashMap::new(),
            paths: HashMap::new(),
            external_crates: HashMap::new(),
            target: rustdoc_types::Target {
                triple: String::new(),
                target_features: vec![],
            },
            format_version: 0,
        }
    }

    #[test]
    fn test_external_url() {
        let krate = empty_crate();
        let resolver = RustdocLinkResolver {
            item_links: &HashMap::new(),
            krate: &krate,
            id_to_items: &HashMap::new(),
        };
        let result = resolver.resolve_link("docs", "https://docs.rs/tokio");
        assert_eq!(result, "docs (https://docs.rs/tokio)");
    }

    #[test]
    fn test_unresolvable_keeps_text() {
        let krate = empty_crate();
        let resolver = RustdocLinkResolver {
            item_links: &HashMap::new(),
            krate: &krate,
            id_to_items: &HashMap::new(),
        };
        let result = resolver.resolve_link("Unknown", "Unknown");
        assert_eq!(result, "Unknown");
    }
}
