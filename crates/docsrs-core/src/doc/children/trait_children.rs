use anyhow::Result;
use rustdoc_fmt::{Colorizer, Output};
use rustdoc_types::{Crate, ItemEnum};

use super::{first_doc_line, write_section};
use crate::doc::render::RenderingContext;

/// Format child items for a trait (associated types, methods, etc.)
pub(crate) fn format_trait_children(
    krate: &Crate,
    trait_: &rustdoc_types::Trait,
    output: &mut String,
    context: &RenderingContext,
) -> Result<()> {
    let colorizer = Colorizer::get();
    let mut assoc_types: Vec<(Option<String>, String)> = Vec::new();
    let mut assoc_consts: Vec<(Option<String>, String)> = Vec::new();
    let mut required_methods: Vec<(Option<String>, String)> = Vec::new();
    let mut provided_methods: Vec<(Option<String>, String)> = Vec::new();

    // Process trait items
    for item_id in &trait_.items {
        if let Some(item) = krate.index.get(item_id) {
            let doc = first_doc_line(&item.docs);
            match &item.inner {
                ItemEnum::AssocType { type_, .. } => {
                    // Associated type
                    let mut type_output = Output::new();
                    type_output.keyword("type");
                    type_output.whitespace();
                    type_output.function(item.name.as_deref().unwrap_or("unknown"));

                    // Add default type if present
                    if let Some(default_type) = type_ {
                        type_output.whitespace();
                        type_output.symbol("=");
                        type_output.whitespace();
                        type_output.extend(context.render_type(default_type));
                    }

                    let type_str = colorizer.tokens(&type_output.into_tokens());
                    assoc_types.push((doc, type_str));
                }
                ItemEnum::AssocConst { type_, value } => {
                    // Associated constant
                    let mut const_output = Output::new();
                    const_output.keyword("const");
                    const_output.whitespace();
                    const_output.function(item.name.as_deref().unwrap_or("unknown"));
                    const_output.symbol(":");
                    const_output.whitespace();
                    const_output.extend(context.render_type(type_));

                    // Add value if present (default value)
                    if let Some(val) = value {
                        const_output.whitespace();
                        const_output.symbol("=");
                        const_output.whitespace();
                        const_output.symbol(val);
                    }

                    let const_str = colorizer.tokens(&const_output.into_tokens());
                    assoc_consts.push((doc, const_str));
                }
                ItemEnum::Function(func) => {
                    // Method (required or provided)
                    let mut name_output = Output::new();
                    name_output.function(item.name.as_deref().unwrap_or("unknown"));
                    let method_output = context.render_function(
                        name_output,
                        &func.sig,
                        &func.generics,
                        &func.header,
                    );
                    let method_str = colorizer.tokens(&method_output.into_tokens());

                    // Check if this is a required or provided method
                    if func.has_body {
                        provided_methods.push((doc, method_str));
                    } else {
                        required_methods.push((doc, method_str));
                    }
                }
                _ => {
                    // Other item types in traits are not displayed for now
                }
            }
        }
    }

    // Output sections
    write_section(output, "Associated Types", &assoc_types);
    write_section(output, "Associated Constants", &assoc_consts);
    write_section(output, "Required Methods", &required_methods);
    write_section(output, "Provided Methods", &provided_methods);

    Ok(())
}
