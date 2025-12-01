//! Extract documentation strings from rustdoc JSON

use rustdoc_types::{Crate, Id, Item, ItemEnum};

/// Represents an extracted documentation string with metadata
#[derive(Debug, Clone)]
pub struct DocEntry {
    pub crate_name: String,
    pub item_path: String,
    pub item_kind: String,
    pub doc_string: String,
}

/// Extract all documentation strings from a rustdoc Crate
pub fn extract_docs(krate: &Crate, crate_name: &str) -> Vec<DocEntry> {
    let mut entries = Vec::new();

    for (id, item) in &krate.index {
        if let Some(docs) = &item.docs
            && !docs.trim().is_empty()
        {
            entries.push(DocEntry {
                crate_name: crate_name.to_string(),
                item_path: build_item_path(krate, id, item),
                item_kind: item_kind_name(&item.inner).to_string(),
                doc_string: docs.clone(),
            });
        }
    }

    entries
}

fn build_item_path(krate: &Crate, id: &Id, item: &Item) -> String {
    // Try to get full path from krate.paths
    if let Some(summary) = krate.paths.get(id) {
        return summary.path.join("::");
    }

    // Fall back to item name
    if let Some(name) = &item.name {
        name.clone()
    } else {
        format!("anonymous_{}", id.0)
    }
}

fn item_kind_name(inner: &ItemEnum) -> &'static str {
    match inner {
        ItemEnum::Function(_) => "function",
        ItemEnum::Struct(_) => "struct",
        ItemEnum::Enum(_) => "enum",
        ItemEnum::Trait(_) => "trait",
        ItemEnum::Module(_) => "module",
        ItemEnum::Constant { .. } => "constant",
        ItemEnum::Macro(_) => "macro",
        ItemEnum::TypeAlias(_) => "type_alias",
        ItemEnum::Static(_) => "static",
        ItemEnum::Union(_) => "union",
        ItemEnum::StructField(_) => "field",
        ItemEnum::Variant(_) => "variant",
        ItemEnum::TraitAlias(_) => "trait_alias",
        ItemEnum::Impl(_) => "impl",
        ItemEnum::AssocConst { .. } => "assoc_const",
        ItemEnum::AssocType { .. } => "assoc_type",
        ItemEnum::ExternCrate { .. } => "extern_crate",
        ItemEnum::Use(_) => "use",
        ItemEnum::ProcMacro(_) => "proc_macro",
        ItemEnum::Primitive(_) => "primitive",
        ItemEnum::ExternType => "extern_type",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustdoc_types::Module;

    #[test]
    fn test_item_kind_names() {
        // Verify item kind name function works
        let module = ItemEnum::Module(Module {
            is_crate: false,
            items: vec![],
            is_stripped: false,
        });
        assert_eq!(item_kind_name(&module), "module");
    }
}
