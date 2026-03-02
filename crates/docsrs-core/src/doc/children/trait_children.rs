use anyhow::Result;
use rustdoc_fmt::{Colorizer, Output};
use rustdoc_types::{Crate, ItemEnum};

use super::first_doc_line;
use crate::doc::render::RenderingContext;

/// Format child items for a trait (associated types, consts, methods)
///
/// All items are rendered inside a single `{ }` block.
pub(crate) fn format_trait_children(
    krate: &Crate,
    trait_: &rustdoc_types::Trait,
    output: &mut String,
    context: &RenderingContext,
) -> Result<()> {
    let colorizer = Colorizer::get();
    let mut body_items: Vec<(Option<String>, String)> = Vec::new();

    // Process trait items in order: types, consts, then methods
    for item_id in &trait_.items {
        if let Some(item) = krate.index.get(item_id) {
            let doc = first_doc_line(&item.docs);
            match &item.inner {
                ItemEnum::AssocType { type_, .. } => {
                    let mut type_output = Output::new();
                    type_output.keyword("type");
                    type_output.whitespace();
                    type_output.function(item.name.as_deref().unwrap_or("unknown"));

                    if let Some(default_type) = type_ {
                        type_output.whitespace();
                        type_output.symbol("=");
                        type_output.whitespace();
                        type_output.extend(context.render_type(default_type));
                    }

                    let type_str = colorizer.tokens(&type_output.into_tokens());
                    // Associated types end with ";"
                    body_items.push((doc, format!("{type_str};")));
                }
                ItemEnum::AssocConst { type_, value } => {
                    let mut const_output = Output::new();
                    const_output.keyword("const");
                    const_output.whitespace();
                    const_output.function(item.name.as_deref().unwrap_or("unknown"));
                    const_output.symbol(":");
                    const_output.whitespace();
                    const_output.extend(context.render_type(type_));

                    if let Some(val) = value {
                        const_output.whitespace();
                        const_output.symbol("=");
                        const_output.whitespace();
                        const_output.symbol(val);
                    }

                    let const_str = colorizer.tokens(&const_output.into_tokens());
                    // Associated consts end with ";"
                    body_items.push((doc, format!("{const_str};")));
                }
                ItemEnum::Function(func) => {
                    // Trait methods don't use `pub` qualifier
                    let mut name_output = Output::new();
                    name_output.function(item.name.as_deref().unwrap_or("unknown"));
                    let method_output =
                        context.render_method(name_output, &func.sig, &func.generics, &func.header);
                    let method_str = colorizer.tokens(&method_output.into_tokens());

                    if func.has_body {
                        // Provided method: has default impl
                        body_items.push((doc, format!("{method_str} {{ .. }}")));
                    } else {
                        // Required method: ends with ";"
                        body_items.push((doc, format!("{method_str};")));
                    }
                }
                _ => {}
            }
        }
    }

    // Write the body block (items already have their own terminators)
    if !body_items.is_empty() {
        output.push_str(" {\n");
        for (doc, signature) in &body_items {
            if let Some(doc_line) = doc {
                output.push_str("    /// ");
                output.push_str(doc_line);
                output.push('\n');
            }
            output.push_str("    ");
            output.push_str(signature);
            output.push('\n');
        }
        output.push_str("}\n");
    } else {
        output.push('\n');
    }

    Ok(())
}
