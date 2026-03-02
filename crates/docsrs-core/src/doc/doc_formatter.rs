use anyhow::Result;
use rustdoc_fmt::{Colorizer, format_markdown};
use rustdoc_types::{Crate, ItemEnum};

use super::children::{
    format_enum_children, format_module_children, format_struct_children, format_trait_children,
};
use super::link_resolver::RustdocLinkResolver;
use super::public_item::PublicItem;
use super::render::RenderingContext;

/// Format documentation for a single PublicItem
pub fn format_doc(krate: &Crate, item: &PublicItem, context: &RenderingContext) -> Result<String> {
    let colorizer = Colorizer::get();
    let mut output = String::new();

    // Build the signature string (without trailing newline yet)
    let signature = colorizer.tokens(&item.tokens);

    // Try to get the full Item from the crate index to access documentation
    if let Some(full_item) = krate.index.get(&item._id) {
        // 1. Format docs with "/// " prefix on each line (above signature)
        if let Some(docs) = &full_item.docs {
            let resolver = RustdocLinkResolver {
                item_links: &full_item.links,
                krate,
                id_to_items: &context.id_to_items,
            };
            let formatted_docs = format_markdown(docs, &resolver);
            for line in formatted_docs.lines() {
                if line.is_empty() {
                    output.push_str("///\n");
                } else {
                    output.push_str("/// ");
                    output.push_str(line);
                    output.push('\n');
                }
            }
        }

        // 2. Signature
        output.push_str(&signature);

        // 3. Format child items based on item type
        // Children that produce body blocks (struct/enum/trait) append " { ... }\n"
        // Others just append "\n"
        match &full_item.inner {
            ItemEnum::Struct(struct_) => {
                format_struct_children(krate, struct_, &mut output, context)?;
            }
            ItemEnum::Enum(enum_) => {
                format_enum_children(krate, enum_, &mut output, context)?;
            }
            ItemEnum::Trait(trait_) => {
                format_trait_children(krate, trait_, &mut output, context)?;
            }
            ItemEnum::Module(module) => {
                output.push('\n');
                format_module_children(krate, module, &mut output, context)?;
            }
            _ => {
                output.push('\n');
            }
        }
    } else {
        output.push_str(&signature);
        output.push('\n');
    }

    Ok(output)
}
