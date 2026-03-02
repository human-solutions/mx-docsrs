use anyhow::Result;
use jsondoc::ImplKind;
use rustdoc_fmt::{Colorizer, Output};
use rustdoc_types::{Crate, ItemEnum, StructKind, Visibility};

use super::{first_doc_line, write_body_block, write_comment_section, write_trait_impls};
use crate::doc::render::RenderingContext;

/// Format child items for a struct (fields, methods and trait implementations)
pub(crate) fn format_struct_children(
    krate: &Crate,
    struct_: &rustdoc_types::Struct,
    output: &mut String,
    context: &RenderingContext,
) -> Result<()> {
    let colorizer = Colorizer::get();
    let mut plain_fields: Vec<(Option<String>, String)> = Vec::new();
    let mut methods: Vec<(Option<String>, String)> = Vec::new();
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
                    let mut field_output = Output::new();
                    field_output.qualifier("pub");
                    field_output.whitespace();
                    field_output.function(field_name);
                    field_output.symbol(":");
                    field_output.whitespace();
                    field_output.extend(context.render_type(field_type));

                    let field_str = colorizer.tokens(&field_output.into_tokens());
                    let doc = first_doc_line(&field_item.docs);
                    plain_fields.push((doc, field_str));
                }
            }
        }
        StructKind::Tuple(_) | StructKind::Unit => {
            // Tuple structs have fields in the signature already; unit structs have no fields
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
                let impl_str = colorizer.tokens(&impl_tokens.into_tokens());
                trait_impls.push(impl_str);
            } else {
                // This is an inherent impl - extract methods
                for item_id in &impl_.items {
                    if let Some(item) = krate.index.get(item_id) {
                        // Only include functions (methods)
                        if let ItemEnum::Function(func) = &item.inner {
                            let mut name_output = Output::new();
                            name_output.function(item.name.as_deref().unwrap_or("unknown"));
                            let method_output = context.render_function(
                                name_output,
                                &func.sig,
                                &func.generics,
                                &func.header,
                            );
                            let method_str = colorizer.tokens(&method_output.into_tokens());
                            let doc = first_doc_line(&item.docs);
                            methods.push((doc, method_str));
                        }
                    }
                }
            }
        }
    }

    // Output: fields in { } block, methods and trait impls after
    if !plain_fields.is_empty() {
        write_body_block(output, &plain_fields, ",");
        output.push('\n');
    } else {
        output.push('\n');
    }

    write_comment_section(output, "Methods", &methods);
    write_trait_impls(output, &trait_impls);

    Ok(())
}
