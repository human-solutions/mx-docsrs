use anyhow::Result;
use rustdoc_types::{Crate, ItemEnum};

use super::children::{
    format_enum_children, format_module_children, format_struct_children, format_trait_children,
};
use super::link_resolver::resolve_doc_links;
use super::public_item::PublicItem;
use super::render::RenderingContext;
use crate::{
    color::Color,
    fmt::{tokens_to_colored_string, tokens_to_string},
};

/// Format documentation for a single PublicItem
pub fn format_doc(
    krate: &Crate,
    item: &PublicItem,
    color: Color,
    context: &RenderingContext,
) -> Result<String> {
    let use_colors = color.is_active();
    let mut output = String::new();

    // Display the item signature
    let signature = if use_colors {
        tokens_to_colored_string(&item.tokens)
    } else {
        tokens_to_string(&item.tokens)
    };

    output.push_str(&signature);
    output.push('\n');

    // Try to get the full Item from the crate index to access documentation
    if let Some(full_item) = krate.index.get(&item._id) {
        if let Some(docs) = &full_item.docs {
            output.push('\n');
            let resolved_docs =
                resolve_doc_links(docs, &full_item.links, krate, &context.id_to_items);
            output.push_str(&resolved_docs);
            output.push('\n');
        }

        // Format child items based on item type
        match &full_item.inner {
            ItemEnum::Struct(struct_) => {
                format_struct_children(krate, struct_, &mut output, use_colors, context)?;
            }
            ItemEnum::Enum(enum_) => {
                format_enum_children(krate, enum_, &mut output, use_colors, context)?;
            }
            ItemEnum::Trait(trait_) => {
                format_trait_children(krate, trait_, &mut output, use_colors, context)?;
            }
            ItemEnum::Module(module) => {
                format_module_children(krate, module, &mut output, use_colors, context)?;
            }
            _ => {
                // Other item types don't have child items to display
            }
        }
    }

    Ok(output)
}
