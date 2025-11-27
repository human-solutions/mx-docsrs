use anyhow::Result;
use rustdoc_types::{Crate, ItemEnum, Variant};

use crate::colorizer::Colorizer;
use crate::doc::impl_kind::ImplKind;
use crate::doc::render::RenderingContext;

/// Format child items for an enum (variants, methods and trait implementations)
pub(crate) fn format_enum_children(
    krate: &Crate,
    enum_: &rustdoc_types::Enum,
    output: &mut String,
    context: &RenderingContext,
) -> Result<()> {
    let colorizer = Colorizer::get();
    let mut variants = Vec::new();
    let mut methods = Vec::new();
    let mut trait_impls = Vec::new();

    // Process enum variants
    for variant_id in &enum_.variants {
        if let Some(variant_item) = krate.index.get(variant_id)
            && let ItemEnum::Variant(variant) = &variant_item.inner
        {
            let variant_str =
                format_variant(variant_item.name.as_deref(), variant, colorizer, context);
            variants.push(variant_str);
        }
    }

    // Process each impl block
    for impl_id in &enum_.impls {
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
                            let mut name_output = crate::fmt::Output::new();
                            name_output.function(item.name.as_deref().unwrap_or("unknown"));
                            let method_output = context.render_function(
                                name_output,
                                &func.sig,
                                &func.generics,
                                &func.header,
                            );
                            let method_str = colorizer.tokens(&method_output.into_tokens());
                            methods.push(method_str);
                        }
                    }
                }
            }
        }
    }

    // Output Variants section
    if !variants.is_empty() {
        output.push('\n');
        output.push_str("Variants:\n");
        for variant in variants {
            output.push_str("  ");
            output.push_str(&variant);
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

/// Format a single enum variant
fn format_variant(
    name: Option<&str>,
    variant: &Variant,
    colorizer: &Colorizer,
    context: &RenderingContext,
) -> String {
    let mut variant_output = crate::fmt::Output::new();
    variant_output.function(name.unwrap_or("unknown"));

    // Handle different variant kinds
    match &variant.kind {
        rustdoc_types::VariantKind::Plain => {
            // Unit variant - no additional formatting needed
        }
        rustdoc_types::VariantKind::Tuple(fields) => {
            // Tuple variant - resolve field IDs to types
            variant_output.symbol("(");
            let resolved_fields = resolve_tuple_fields(context, fields);
            for (i, field_type_opt) in resolved_fields.iter().enumerate() {
                if i > 0 {
                    variant_output.symbol(",");
                    variant_output.whitespace();
                }
                if let Some(field_type) = field_type_opt {
                    variant_output.extend(context.render_type(field_type));
                }
            }
            variant_output.symbol(")");
        }
        rustdoc_types::VariantKind::Struct { .. } => {
            // Struct variant
            variant_output.whitespace();
            variant_output.symbol("{");
            variant_output.whitespace();
            variant_output.symbol("...");
            variant_output.whitespace();
            variant_output.symbol("}");
        }
    }

    colorizer.tokens(&variant_output.into_tokens())
}

/// Resolve tuple field IDs to their actual types
fn resolve_tuple_fields<'a>(
    context: &RenderingContext<'a>,
    fields: &[Option<rustdoc_types::Id>],
) -> Vec<Option<&'a rustdoc_types::Type>> {
    let mut resolved_fields = Vec::new();
    for id in fields {
        resolved_fields.push(
            if let Some(rustdoc_types::Item {
                inner: ItemEnum::StructField(type_),
                ..
            }) = id.as_ref().and_then(|id| context.crate_.index.get(id))
            {
                Some(type_)
            } else {
                None
            },
        );
    }
    resolved_fields
}
