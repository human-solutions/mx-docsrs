use anyhow::Result;
use rustdoc_types::{Crate, FunctionSignature, GenericArgs, Id, Item, ItemEnum, Type};

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
    let item_type = get_item_type(&item.inner);
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
    let visibility = format_visibility(&item.visibility);

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

/// Format visibility as a string
fn format_visibility(vis: &rustdoc_types::Visibility) -> String {
    match vis {
        rustdoc_types::Visibility::Public => "pub".to_string(),
        rustdoc_types::Visibility::Default => "".to_string(),
        rustdoc_types::Visibility::Crate => "pub(crate)".to_string(),
        rustdoc_types::Visibility::Restricted { parent: _, path } => {
            format!("pub({})", path)
        }
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
    let item_type = get_item_type(&item.inner);
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

/// Get the type of an item as a string
fn get_item_type(item: &ItemEnum) -> &'static str {
    match item {
        ItemEnum::Module(_) => "mod",
        ItemEnum::ExternCrate { .. } => "extern crate",
        ItemEnum::Union(_) => "union",
        ItemEnum::Struct(_) => "struct",
        ItemEnum::StructField(_) => "field",
        ItemEnum::Enum(_) => "enum",
        ItemEnum::Variant(_) => "variant",
        ItemEnum::Function(_) => "fn",
        ItemEnum::Trait(_) => "trait",
        ItemEnum::TraitAlias(_) => "trait alias",
        ItemEnum::Impl(_) => "impl",
        ItemEnum::TypeAlias(_) => "type",
        ItemEnum::Constant { .. } => "const",
        ItemEnum::Static(_) => "static",
        ItemEnum::Macro(_) => "macro",
        ItemEnum::ProcMacro(_) => "proc macro",
        ItemEnum::Primitive(_) => "primitive",
        ItemEnum::AssocConst { .. } => "const",
        ItemEnum::AssocType { .. } => "type",
        _ => "item",
    }
}

/// Generate a signature for an item
fn generate_signature(item: &Item, krate: &Crate) -> String {
    let vis = format_visibility(&item.visibility);
    let vis_prefix = if vis.is_empty() {
        String::new()
    } else {
        format!("{} ", vis)
    };

    match &item.inner {
        ItemEnum::Function(func) => generate_function_signature(
            &vis_prefix,
            item.name.as_deref().unwrap_or("<fn>"),
            &func.sig,
            &func.generics,
            &func.header,
        ),

        ItemEnum::Struct(s) => {
            let name = item.name.as_deref().unwrap_or("<struct>");
            let generics = format_generics(&s.generics);

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
            let generics = format_generics(&e.generics);
            format!("{}enum {}{} {{ ... }}", vis_prefix, name, generics)
        }

        ItemEnum::Trait(t) => {
            let name = item.name.as_deref().unwrap_or("<trait>");
            let generics = format_generics(&t.generics);
            let unsafe_prefix = if t.is_unsafe { "unsafe " } else { "" };
            format!(
                "{}{}trait {}{} {{ ... }}",
                vis_prefix, unsafe_prefix, name, generics
            )
        }

        ItemEnum::TypeAlias(ta) => {
            let name = item.name.as_deref().unwrap_or("<type>");
            let generics = format_generics(&ta.generics);
            let type_str = format_type(&ta.type_, krate);
            format!("{}type {}{} = {};", vis_prefix, name, generics, type_str)
        }

        ItemEnum::Constant { type_, const_: _ } => {
            let name = item.name.as_deref().unwrap_or("<const>");
            let type_str = format_type(type_, krate);
            format!("{}const {}: {};", vis_prefix, name, type_str)
        }

        ItemEnum::Static(s) => {
            let name = item.name.as_deref().unwrap_or("<static>");
            let type_str = format_type(&s.type_, krate);
            let mut_str = if s.is_mutable { "mut " } else { "" };
            format!("{}static {}{}: {};", vis_prefix, mut_str, name, type_str)
        }

        ItemEnum::StructField(ty) => {
            let name = item.name.as_deref().unwrap_or("<field>");
            let type_str = format_type(ty, krate);
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
            let type_name = get_item_type(&item.inner);
            let name = item.name.as_deref().unwrap_or("<item>");
            format!("{}{} {}", vis_prefix, type_name, name)
        }
    }
}

/// Generate a function signature
fn generate_function_signature(
    vis_prefix: &str,
    name: &str,
    sig: &FunctionSignature,
    generics: &rustdoc_types::Generics,
    header: &rustdoc_types::FunctionHeader,
) -> String {
    let mut parts = Vec::new();

    // Check if ABI is not Rust (the default)
    match &header.abi {
        rustdoc_types::Abi::Rust => {} // Default, don't show
        rustdoc_types::Abi::C { .. } => parts.push("extern \"C\"".to_string()),
        rustdoc_types::Abi::Other(name) => parts.push(format!("extern \"{}\"", name)),
        _ => {} // Other ABI variants
    }
    if header.is_const {
        parts.push("const".to_string());
    }
    if header.is_async {
        parts.push("async".to_string());
    }
    if header.is_unsafe {
        parts.push("unsafe".to_string());
    }

    let modifiers = if parts.is_empty() {
        String::new()
    } else {
        format!("{} ", parts.join(" "))
    };

    let gen_params = format_generics(generics);

    // Format parameters
    let params: Vec<String> = sig
        .inputs
        .iter()
        .map(|(name, ty)| format!("{}: {}", name, format_type_simple(ty)))
        .collect();
    let params_str = params.join(", ");

    // Format return type
    let return_str = if let Some(ret) = &sig.output {
        format!(" -> {}", format_type_simple(ret))
    } else {
        String::new()
    };

    // Format where clause
    let where_clause = format_where_clause(generics);

    format!(
        "{}{}fn {}{}({}){}{}",
        vis_prefix, modifiers, name, gen_params, params_str, return_str, where_clause
    )
}

/// Format generics (simple version)
fn format_generics(generics: &rustdoc_types::Generics) -> String {
    if generics.params.is_empty() {
        return String::new();
    }

    let params: Vec<String> = generics
        .params
        .iter()
        .map(|param| match &param.kind {
            rustdoc_types::GenericParamDefKind::Lifetime { outlives } => {
                if outlives.is_empty() {
                    param.name.clone()
                } else {
                    format!("{}: {}", param.name, outlives.join(" + "))
                }
            }
            rustdoc_types::GenericParamDefKind::Type {
                bounds, default, ..
            } => {
                let mut s = param.name.clone();
                if !bounds.is_empty() {
                    s.push_str(": ");
                    s.push_str(&format_bounds(bounds));
                }
                if let Some(def) = default {
                    s.push_str(" = ");
                    s.push_str(&format_type_simple(def));
                }
                s
            }
            rustdoc_types::GenericParamDefKind::Const { type_, default } => {
                let mut s = format!("const {}: {}", param.name, format_type_simple(type_));
                if let Some(def) = default {
                    s.push_str(" = ");
                    s.push_str(def);
                }
                s
            }
        })
        .collect();

    format!("<{}>", params.join(", "))
}

/// Format where clause
fn format_where_clause(generics: &rustdoc_types::Generics) -> String {
    if generics.where_predicates.is_empty() {
        return String::new();
    }

    let predicates: Vec<String> =
        generics
            .where_predicates
            .iter()
            .filter_map(|pred| match pred {
                rustdoc_types::WherePredicate::BoundPredicate { type_, bounds, .. } => Some(
                    format!("{}: {}", format_type_simple(type_), format_bounds(bounds)),
                ),
                rustdoc_types::WherePredicate::LifetimePredicate { lifetime, outlives } => {
                    if outlives.is_empty() {
                        None
                    } else {
                        Some(format!("{}: {}", lifetime, outlives.join(" + ")))
                    }
                }
                rustdoc_types::WherePredicate::EqPredicate { lhs, rhs } => Some(format!(
                    "{} = {}",
                    format_type_simple(lhs),
                    format_term_simple(rhs)
                )),
            })
            .collect();

    if predicates.is_empty() {
        String::new()
    } else {
        format!("\nwhere\n    {}", predicates.join(",\n    "))
    }
}

/// Format generic bounds
fn format_bounds(bounds: &[rustdoc_types::GenericBound]) -> String {
    bounds
        .iter()
        .map(|bound| match bound {
            rustdoc_types::GenericBound::TraitBound { trait_, .. } => trait_.path.clone(),
            rustdoc_types::GenericBound::Outlives(lifetime) => lifetime.clone(),
            rustdoc_types::GenericBound::Use(args) => {
                // Format precise capturing args
                let arg_strs: Vec<String> = args
                    .iter()
                    .map(|arg| match arg {
                        rustdoc_types::PreciseCapturingArg::Lifetime(lt) => lt.clone(),
                        rustdoc_types::PreciseCapturingArg::Param(name) => name.clone(),
                    })
                    .collect();
                format!("use<{}>", arg_strs.join(", "))
            }
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

/// Format a type (simplified version)
fn format_type_simple(ty: &Type) -> String {
    match ty {
        Type::ResolvedPath(path) => {
            let name = &path.path; // path.path is the String
            if let Some(args) = &path.args {
                format!("{}{}", name, format_generic_args(args))
            } else {
                name.clone()
            }
        }
        Type::DynTrait(dt) => {
            let traits: Vec<String> = dt
                .traits
                .iter()
                .map(|poly_trait| poly_trait.trait_.path.clone())
                .collect();
            format!("dyn {}", traits.join(" + "))
        }
        Type::Generic(name) => name.clone(),
        Type::Primitive(name) => name.clone(),
        Type::FunctionPointer(fp) => {
            let params: Vec<String> = fp
                .sig
                .inputs
                .iter()
                .map(|(_, ty)| format_type_simple(ty))
                .collect();
            let ret = fp
                .sig
                .output
                .as_ref()
                .map(|t| format!(" -> {}", format_type_simple(t)))
                .unwrap_or_default();
            format!("fn({}){}", params.join(", "), ret)
        }
        Type::Tuple(types) => {
            if types.is_empty() {
                "()".to_string()
            } else {
                let inner: Vec<String> = types.iter().map(format_type_simple).collect();
                format!("({})", inner.join(", "))
            }
        }
        Type::Slice(inner) => {
            format!("[{}]", format_type_simple(inner))
        }
        Type::Array { type_, len } => {
            format!("[{}; {}]", format_type_simple(type_), len)
        }
        Type::Pat { type_, .. } => {
            format!("{} is _", format_type_simple(type_))
        }
        Type::ImplTrait(bounds) => {
            format!("impl {}", format_bounds(bounds))
        }
        Type::Infer => "_".to_string(),
        Type::RawPointer { is_mutable, type_ } => {
            let mut_str = if *is_mutable { "mut" } else { "const" };
            format!("*{} {}", mut_str, format_type_simple(type_))
        }
        Type::BorrowedRef {
            lifetime,
            is_mutable,
            type_,
        } => {
            let mut_str = if *is_mutable { " mut" } else { "" };
            let lt = lifetime.as_deref().unwrap_or("");
            let lt_str = if lt.is_empty() {
                String::new()
            } else {
                format!("{} ", lt)
            };
            format!("&{}{}{}", lt_str, mut_str, format_type_simple(type_))
        }
        Type::QualifiedPath {
            self_type,
            trait_,
            name,
            ..
        } => {
            if let Some(trait_path) = trait_ {
                format!(
                    "<{} as {}>::{}",
                    format_type_simple(self_type),
                    trait_path.path,
                    name
                )
            } else {
                format!("<{}>::{}", format_type_simple(self_type), name)
            }
        }
    }
}

/// Format generic arguments
fn format_generic_args(args: &GenericArgs) -> String {
    match args {
        GenericArgs::ReturnTypeNotation => "(..)".to_string(),
        GenericArgs::AngleBracketed { args, constraints } => {
            let mut parts = Vec::new();

            for arg in args {
                match arg {
                    rustdoc_types::GenericArg::Lifetime(lt) => parts.push(lt.clone()),
                    rustdoc_types::GenericArg::Type(ty) => parts.push(format_type_simple(ty)),
                    rustdoc_types::GenericArg::Const(c) => parts.push(c.expr.clone()),
                    rustdoc_types::GenericArg::Infer => parts.push("_".to_string()),
                }
            }

            for constraint in constraints {
                match &constraint.binding {
                    rustdoc_types::AssocItemConstraintKind::Equality(term) => {
                        parts.push(format!(
                            "{} = {}",
                            constraint.name,
                            format_term_simple(term)
                        ));
                    }
                    rustdoc_types::AssocItemConstraintKind::Constraint(bounds) => {
                        let bounds_str = format_bounds(bounds);
                        parts.push(format!("{}: {}", constraint.name, bounds_str));
                    }
                }
            }

            if parts.is_empty() {
                String::new()
            } else {
                format!("<{}>", parts.join(", "))
            }
        }
        GenericArgs::Parenthesized { inputs, output } => {
            let inputs_str = inputs
                .iter()
                .map(format_type_simple)
                .collect::<Vec<_>>()
                .join(", ");
            let output_str = output
                .as_ref()
                .map(|t| format!(" -> {}", format_type_simple(t)))
                .unwrap_or_default();
            format!("({}){}", inputs_str, output_str)
        }
    }
}

/// Format a term (for equality predicates)
fn format_term_simple(term: &rustdoc_types::Term) -> String {
    match term {
        rustdoc_types::Term::Type(ty) => format_type_simple(ty),
        rustdoc_types::Term::Constant(c) => c.expr.clone(),
    }
}

/// Format a type with crate context (for more detailed rendering if needed)
fn format_type(ty: &Type, _krate: &Crate) -> String {
    // For now, just use simple formatting
    // Could enhance with crate context for resolving paths
    format_type_simple(ty)
}
