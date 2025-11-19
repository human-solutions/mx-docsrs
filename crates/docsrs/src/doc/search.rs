use rustdoc_types::{Crate, Id, ItemEnum};
use std::fmt;

/// Represents a search result from the documentation
#[derive(Debug)]
pub struct DocResult {
    pub id: Id,
    pub name: String,
    pub item_type: String,
    pub path: Vec<String>,
}

impl fmt::Display for DocResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path_str = if self.path.is_empty() {
            String::new()
        } else {
            format!("{}::", self.path.join("::"))
        };
        write!(f, "{} {}{}", self.item_type, path_str, self.name)
    }
}

/// Search through rustdoc items for matches
pub fn search_items(krate: &Crate, query: &str, crate_name: &str) -> Vec<DocResult> {
    let mut results = Vec::new();

    // Check if query is a fully qualified domain name (contains "::")
    let is_fqdn = query.contains("::");

    if is_fqdn {
        // FQDN search: match exact path + name
        let parts: Vec<&str> = query.split("::").collect();
        if parts.is_empty() {
            return results;
        }

        let query_name = parts.last().unwrap();
        let query_path_parts: Vec<String> = parts[..parts.len() - 1]
            .iter()
            .map(|s| s.to_string())
            .collect();

        for (id, item) in &krate.index {
            let Some(ref name) = item.name else {
                continue;
            };

            // Check if name matches (case-insensitive)
            if name.to_lowercase() != query_name.to_lowercase() {
                continue;
            }

            // Build the module path (parent modules, without item name)
            // ItemSummary.path includes the item name as the last element, so we need to remove it
            let module_path = if let Some(summary) = krate.paths.get(id) {
                let mut path = summary.path.clone();
                path.pop(); // Remove the item name (last element)
                path
            } else {
                vec![crate_name.to_string()]
            };

            // Check if module path matches the query path
            if module_path.len() >= query_path_parts.len() {
                let path_lower: Vec<String> =
                    module_path.iter().map(|s| s.to_lowercase()).collect();
                let query_path_lower: Vec<String> =
                    query_path_parts.iter().map(|s| s.to_lowercase()).collect();

                // Check if the relevant portion of the path matches
                let matches = if query_path_parts.is_empty() {
                    true
                } else {
                    // Find the query path within the actual path
                    path_lower
                        .windows(query_path_lower.len())
                        .any(|window| window == query_path_lower.as_slice())
                };

                if matches {
                    let item_type = get_item_type(&item.inner);

                    results.push(DocResult {
                        id: *id,
                        name: name.clone(),
                        item_type: item_type.to_string(),
                        path: module_path,
                    });
                }
            }
        }
    } else {
        // Simple search: substring match on name (case-insensitive)
        let query_lower = query.to_lowercase();

        for (id, item) in &krate.index {
            let Some(ref name) = item.name else {
                continue;
            };

            if name.to_lowercase().contains(&query_lower) {
                // Build the module path (parent modules, without item name)
                // ItemSummary.path includes the item name as the last element, so we need to remove it
                let module_path = if let Some(summary) = krate.paths.get(id) {
                    let mut path = summary.path.clone();
                    path.pop(); // Remove the item name (last element)
                    path
                } else {
                    vec![crate_name.to_string()]
                };

                let item_type = get_item_type(&item.inner);

                results.push(DocResult {
                    id: *id,
                    name: name.clone(),
                    item_type: item_type.to_string(),
                    path: module_path,
                });
            }
        }
    }

    // Sort results for deterministic output (fixes HashMap iteration non-determinism)
    // Sort by: 1) item_type, 2) FQDN (path + name)
    results.sort_by(|a, b| {
        // First compare by item_type
        let type_cmp = a.item_type.cmp(&b.item_type);
        if type_cmp != std::cmp::Ordering::Equal {
            return type_cmp;
        }

        // Then compare by FQDN
        let a_fqdn = format!("{}::{}", a.path.join("::"), a.name);
        let b_fqdn = format!("{}::{}", b.path.join("::"), b.name);
        a_fqdn.cmp(&b_fqdn)
    });

    results
}

/// Get the type of an item as a string
fn get_item_type(item: &ItemEnum) -> &'static str {
    match item {
        ItemEnum::Module(_) => "mod",
        ItemEnum::ExternCrate { .. } => "externcrate",
        ItemEnum::Union(_) => "union",
        ItemEnum::Struct(_) => "struct",
        ItemEnum::StructField(_) => "structfield",
        ItemEnum::Enum(_) => "enum",
        ItemEnum::Variant(_) => "variant",
        ItemEnum::Function(_) => "fn",
        ItemEnum::Trait(_) => "trait",
        ItemEnum::TraitAlias(_) => "traitalias",
        ItemEnum::Impl(_) => "impl",
        ItemEnum::TypeAlias(_) => "type",
        ItemEnum::Constant { .. } => "constant",
        ItemEnum::Static(_) => "static",
        ItemEnum::Macro(_) => "macro",
        ItemEnum::ProcMacro(_) => "derive",
        ItemEnum::Primitive(_) => "primitive",
        ItemEnum::AssocConst { .. } => "associatedconstant",
        ItemEnum::AssocType { .. } => "associatedtype",
        _ => "item",
    }
}
