use anyhow::Result;
use rustdoc_types::{Crate, ItemEnum, StructKind, Visibility};

use super::impl_kind::ImplKind;
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
            output.push_str(docs);
            output.push('\n');
        }

        // For structs, add child items (methods and trait implementations)
        if let ItemEnum::Struct(struct_) = &full_item.inner {
            format_struct_children(krate, struct_, &mut output, use_colors, context)?;
        }
    }

    Ok(output)
}

/// Format child items for a struct (fields, methods and trait implementations)
fn format_struct_children(
    krate: &Crate,
    struct_: &rustdoc_types::Struct,
    output: &mut String,
    use_colors: bool,
    context: &RenderingContext,
) -> Result<()> {
    let mut plain_fields = Vec::new();
    let mut tuple_fields = Vec::new();
    let mut methods = Vec::new();
    let mut trait_impls = Vec::new();

    // Process struct fields based on kind
    match &struct_.kind {
        StructKind::Plain { fields, .. } => {
            // Process named fields
            for field_id in fields {
                if let Some(field_item) = krate.index.get(field_id)
                    && matches!(field_item.visibility, Visibility::Public)
                    && let ItemEnum::StructField(field_type) = &field_item.inner
                {
                    let field_name = field_item.name.as_deref().unwrap_or("unknown");
                    let mut field_output = crate::fmt::Output::new();
                    field_output.qualifier("pub");
                    field_output.whitespace();
                    field_output.function(field_name);
                    field_output.symbol(":");
                    field_output.whitespace();
                    field_output.extend(context.render_type(field_type));

                    let field_str = if use_colors {
                        tokens_to_colored_string(&field_output.into_tokens())
                    } else {
                        tokens_to_string(&field_output.into_tokens())
                    };
                    plain_fields.push(field_str);
                }
            }
        }
        StructKind::Tuple(fields) => {
            // Process tuple fields
            for (index, field_id_opt) in fields.iter().enumerate() {
                if let Some(field_id) = field_id_opt
                    && let Some(field_item) = krate.index.get(field_id)
                    && matches!(field_item.visibility, Visibility::Public)
                    && let ItemEnum::StructField(field_type) = &field_item.inner
                {
                    let mut field_output = crate::fmt::Output::new();
                    field_output.symbol(index.to_string());
                    field_output.symbol(":");
                    field_output.whitespace();
                    field_output.qualifier("pub");
                    field_output.whitespace();
                    field_output.extend(context.render_type(field_type));

                    let field_str = if use_colors {
                        tokens_to_colored_string(&field_output.into_tokens())
                    } else {
                        tokens_to_string(&field_output.into_tokens())
                    };
                    tuple_fields.push(field_str);
                }
            }
        }
        StructKind::Unit => {
            // Unit structs have no fields
        }
    }

    // Process each impl block
    for impl_id in &struct_.impls {
        if let Some(impl_item) = krate.index.get(impl_id)
            && let ItemEnum::Impl(impl_) = &impl_item.inner
        {
            // Check if this impl should be included (filter out auto-traits and blanket impls)
            let impl_kind = ImplKind::from(impl_item, impl_);
            if !impl_kind.is_active() {
                continue;
            }

            if impl_.trait_.is_some() {
                // This is a trait implementation
                let impl_tokens = context.render_impl(impl_, &[], false);
                let impl_str = if use_colors {
                    tokens_to_colored_string(&impl_tokens.into_tokens())
                } else {
                    tokens_to_string(&impl_tokens.into_tokens())
                };
                trait_impls.push(impl_str);
            } else {
                // This is an inherent impl - extract methods
                for item_id in &impl_.items {
                    if let Some(item) = krate.index.get(item_id) {
                        // Only include functions (methods)
                        if let ItemEnum::Function(func) = &item.inner {
                            let mut name_output = crate::fmt::Output::new();
                            name_output.function(item.name.as_deref().unwrap_or("unknown"));
                            let method_output = context.render_function(
                                name_output,
                                &func.sig,
                                &func.generics,
                                &func.header,
                            );
                            let method_str = if use_colors {
                                tokens_to_colored_string(&method_output.into_tokens())
                            } else {
                                tokens_to_string(&method_output.into_tokens())
                            };
                            methods.push(method_str);
                        }
                    }
                }
            }
        }
    }

    // Output Fields section (for plain structs)
    if !plain_fields.is_empty() {
        output.push('\n');
        output.push_str("Fields:\n");
        for field in plain_fields {
            output.push_str("  ");
            output.push_str(&field);
            output.push('\n');
        }
    }

    // Output Tuple Fields section (for tuple structs)
    if !tuple_fields.is_empty() {
        output.push('\n');
        output.push_str("Tuple Fields:\n");
        for field in tuple_fields {
            output.push_str("  ");
            output.push_str(&field);
            output.push('\n');
        }
    }

    // Output Methods section
    if !methods.is_empty() {
        output.push('\n');
        output.push_str("Methods:\n");
        for method in methods {
            output.push_str("  ");
            output.push_str(&method);
            output.push('\n');
        }
    }

    // Output Trait Implementations section
    if !trait_impls.is_empty() {
        output.push('\n');
        output.push_str("Trait Implementations:\n");
        for trait_impl in trait_impls {
            output.push_str("  ");
            output.push_str(&trait_impl);
            output.push('\n');
        }
    }

    Ok(())
}
