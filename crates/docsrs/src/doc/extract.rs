use anyhow::Result;
use rustdoc_types::{Crate, Id, Item, ItemEnum};

use crate::ext::{GenericsFormattingExt, ItemEnumExt, TypeFormattingExt, VisibilityExt};

/// Represents a fully rendered documentation item
#[derive(Debug, Clone)]
pub struct RenderedDoc {
    pub item_type: String,
    pub name: String,
    pub signature: String,
    pub docs: Option<String>,
    pub metadata: DocMetadata,
    pub sections: Vec<DocSection>,
}

/// Metadata about a documented item
#[derive(Debug, Clone)]
pub struct DocMetadata {
    #[allow(dead_code)]
    pub visibility: String,
    pub deprecation: Option<String>,
    pub attributes: Vec<String>,
}

/// A section of documentation (e.g., "Fields", "Methods", "Variants")
#[derive(Debug, Clone)]
pub struct DocSection {
    pub title: String,
    pub items: Vec<RenderedDoc>,
}

/// Extract documentation for an item and its children
pub fn extract_doc(item: &Item, krate: &Crate) -> Result<RenderedDoc> {
    let name = item
        .name
        .clone()
        .unwrap_or_else(|| "<anonymous>".to_string());
    let item_type = item.inner.type_name();
    let signature = generate_signature(item, krate);
    let docs = item.docs.clone();
    let metadata = extract_metadata(item);
    let sections = extract_sections(item, krate)?;

    Ok(RenderedDoc {
        item_type: item_type.to_string(),
        name,
        signature,
        docs,
        metadata,
        sections,
    })
}

/// Extract metadata from an item
fn extract_metadata(item: &Item) -> DocMetadata {
    let visibility = item.visibility.format();

    let deprecation = item.deprecation.as_ref().map(|dep| {
        if let Some(note) = &dep.note {
            format!("deprecated: {}", note)
        } else if let Some(since) = &dep.since {
            format!("deprecated since {}", since)
        } else {
            "deprecated".to_string()
        }
    });

    let attributes = item
        .attrs
        .iter()
        .filter_map(|attr| {
            // Format interesting attributes
            match attr {
                rustdoc_types::Attribute::Repr(repr) => Some(format!("#[repr({:?})]", repr.kind)),
                rustdoc_types::Attribute::MustUse { reason } => {
                    if let Some(r) = reason {
                        Some(format!("#[must_use = \"{}\"]", r))
                    } else {
                        Some("#[must_use]".to_string())
                    }
                }
                rustdoc_types::Attribute::NonExhaustive => Some("#[non_exhaustive]".to_string()),
                rustdoc_types::Attribute::NoMangle => Some("#[no_mangle]".to_string()),
                rustdoc_types::Attribute::Other(s) => Some(s.clone()),
                _ => None, // Skip other attributes
            }
        })
        .collect();

    DocMetadata {
        visibility,
        deprecation,
        attributes,
    }
}

/// Extract child sections based on item type
fn extract_sections(item: &Item, krate: &Crate) -> Result<Vec<DocSection>> {
    let mut sections = Vec::new();

    match &item.inner {
        ItemEnum::Struct(s) => {
            // Extract fields
            if let rustdoc_types::StructKind::Plain { fields, .. } = &s.kind {
                if !fields.is_empty() {
                    let field_docs = extract_child_items(fields, krate)?;
                    sections.push(DocSection {
                        title: "Fields".to_string(),
                        items: field_docs,
                    });
                }
            } else if let rustdoc_types::StructKind::Tuple(fields) = &s.kind {
                // Filter out None (private/hidden fields) and extract docs for visible fields
                let visible_fields: Vec<Id> = fields.iter().filter_map(|f| *f).collect();
                if !visible_fields.is_empty() {
                    let field_docs = extract_child_items(&visible_fields, krate)?;
                    sections.push(DocSection {
                        title: "Fields".to_string(),
                        items: field_docs,
                    });
                }
            }
        }

        ItemEnum::Enum(e) => {
            // Extract variants
            if !e.variants.is_empty() {
                let variant_docs = extract_child_items(&e.variants, krate)?;
                sections.push(DocSection {
                    title: "Variants".to_string(),
                    items: variant_docs,
                });
            }
        }

        ItemEnum::Trait(t) => {
            // Extract associated items (methods, types, constants)
            if !t.items.is_empty() {
                let assoc_docs = extract_child_items(&t.items, krate)?;
                sections.push(DocSection {
                    title: "Associated Items".to_string(),
                    items: assoc_docs,
                });
            }
        }

        ItemEnum::Module(m) => {
            // For modules, we could show contents but that might be too verbose
            // Skip for now to keep "medium detail" level manageable
            let _ = m;
        }

        _ => {
            // Other types don't have children to extract
        }
    }

    Ok(sections)
}

/// Extract documentation for child items given their IDs
fn extract_child_items(ids: &[Id], krate: &Crate) -> Result<Vec<RenderedDoc>> {
    let mut results = Vec::new();

    for id in ids {
        if let Some(child_item) = krate.index.get(id) {
            // Extract doc for child (non-recursive to avoid deep nesting)
            let child_doc = extract_doc_shallow(child_item, krate)?;
            results.push(child_doc);
        }
    }

    Ok(results)
}

/// Extract documentation for an item without recursing into its children
fn extract_doc_shallow(item: &Item, krate: &Crate) -> Result<RenderedDoc> {
    let name = item
        .name
        .clone()
        .unwrap_or_else(|| "<anonymous>".to_string());
    let item_type = item.inner.type_name();
    let signature = generate_signature(item, krate);
    let docs = item.docs.clone();
    let metadata = extract_metadata(item);

    Ok(RenderedDoc {
        item_type: item_type.to_string(),
        name,
        signature,
        docs,
        metadata,
        sections: Vec::new(), // No nested sections for shallow extraction
    })
}

/// Generate a signature for an item
fn generate_signature(item: &Item, krate: &Crate) -> String {
    let vis_prefix = item.visibility.prefix();

    match &item.inner {
        ItemEnum::Function(func) => {
            use crate::ext::FunctionSignatureExt;
            func.sig.format_signature(
                &vis_prefix,
                item.name.as_deref().unwrap_or("<fn>"),
                &func.generics,
                &func.header,
            )
        }

        ItemEnum::Struct(s) => {
            let name = item.name.as_deref().unwrap_or("<struct>");
            let generics = s.generics.format_params();

            match &s.kind {
                rustdoc_types::StructKind::Unit => {
                    format!("{}struct {}{};", vis_prefix, name, generics)
                }
                rustdoc_types::StructKind::Tuple(_) => {
                    format!("{}struct {}{}(...);", vis_prefix, name, generics)
                }
                rustdoc_types::StructKind::Plain { .. } => {
                    format!("{}struct {}{} {{ ... }}", vis_prefix, name, generics)
                }
            }
        }

        ItemEnum::Enum(e) => {
            let name = item.name.as_deref().unwrap_or("<enum>");
            let generics = e.generics.format_params();
            format!("{}enum {}{} {{ ... }}", vis_prefix, name, generics)
        }

        ItemEnum::Trait(t) => {
            let name = item.name.as_deref().unwrap_or("<trait>");
            let generics = t.generics.format_params();
            let unsafe_prefix = if t.is_unsafe { "unsafe " } else { "" };
            format!(
                "{}{}trait {}{} {{ ... }}",
                vis_prefix, unsafe_prefix, name, generics
            )
        }

        ItemEnum::TypeAlias(ta) => {
            let name = item.name.as_deref().unwrap_or("<type>");
            let generics = ta.generics.format_params();
            let type_str = ta.type_.format_with_context(krate);
            format!("{}type {}{} = {};", vis_prefix, name, generics, type_str)
        }

        ItemEnum::Constant { type_, const_: _ } => {
            let name = item.name.as_deref().unwrap_or("<const>");
            let type_str = type_.format_with_context(krate);
            format!("{}const {}: {};", vis_prefix, name, type_str)
        }

        ItemEnum::Static(s) => {
            let name = item.name.as_deref().unwrap_or("<static>");
            let type_str = s.type_.format_with_context(krate);
            let mut_str = if s.is_mutable { "mut " } else { "" };
            format!("{}static {}{}: {};", vis_prefix, mut_str, name, type_str)
        }

        ItemEnum::StructField(ty) => {
            let name = item.name.as_deref().unwrap_or("<field>");
            let type_str = ty.format_with_context(krate);
            format!("{}{}: {}", vis_prefix, name, type_str)
        }

        ItemEnum::Variant(v) => {
            let name = item.name.as_deref().unwrap_or("<variant>");
            match &v.kind {
                rustdoc_types::VariantKind::Plain => name.to_string(),
                rustdoc_types::VariantKind::Tuple(_) => format!("{}(...)", name),
                rustdoc_types::VariantKind::Struct { .. } => format!("{} {{ ... }}", name),
            }
        }

        ItemEnum::Macro(_) => {
            let name = item.name.as_deref().unwrap_or("<macro>");
            format!("macro_rules! {} {{ ... }}", name)
        }

        ItemEnum::Module(_) => {
            let name = item.name.as_deref().unwrap_or("<mod>");
            format!("{}mod {}", vis_prefix, name)
        }

        _ => {
            // Fallback for other types
            let type_name = item.inner.type_name();
            let name = item.name.as_deref().unwrap_or("<item>");
            format!("{}{} {}", vis_prefix, type_name, name)
        }
    }
}
