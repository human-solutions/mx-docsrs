use rustdoc_types::{Crate, FunctionSignature, GenericArgs, ItemEnum, Type};

/// Extension trait for rustdoc Visibility formatting
pub trait VisibilityExt {
    fn format(&self) -> String;
    fn prefix(&self) -> String;
}

impl VisibilityExt for rustdoc_types::Visibility {
    fn format(&self) -> String {
        match self {
            rustdoc_types::Visibility::Public => "pub".to_string(),
            rustdoc_types::Visibility::Default => "".to_string(),
            rustdoc_types::Visibility::Crate => "pub(crate)".to_string(),
            rustdoc_types::Visibility::Restricted { parent: _, path } => {
                format!("pub({})", path)
            }
        }
    }

    fn prefix(&self) -> String {
        let vis = self.format();
        if vis.is_empty() {
            String::new()
        } else {
            format!("{} ", vis)
        }
    }
}

/// Extension trait for getting ItemEnum type as string
pub trait ItemEnumExt {
    fn type_name(&self) -> &'static str;
}

impl ItemEnumExt for ItemEnum {
    fn type_name(&self) -> &'static str {
        match self {
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
}

/// Extension trait for formatting Type
pub trait TypeFormattingExt {
    fn format_simple(&self) -> String;
    fn format_with_context(&self, krate: &Crate) -> String;
}

impl TypeFormattingExt for Type {
    fn format_simple(&self) -> String {
        match self {
            Type::ResolvedPath(path) => {
                let name = &path.path;
                if let Some(args) = &path.args {
                    format!("{}{}", name, args.format())
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
                    .map(|(_, ty)| ty.format_simple())
                    .collect();
                let ret = fp
                    .sig
                    .output
                    .as_ref()
                    .map(|t| format!(" -> {}", t.format_simple()))
                    .unwrap_or_default();
                format!("fn({}){}", params.join(", "), ret)
            }
            Type::Tuple(types) => {
                if types.is_empty() {
                    "()".to_string()
                } else {
                    let inner: Vec<String> = types.iter().map(|t| t.format_simple()).collect();
                    format!("({})", inner.join(", "))
                }
            }
            Type::Slice(inner) => {
                format!("[{}]", inner.format_simple())
            }
            Type::Array { type_, len } => {
                format!("[{}; {}]", type_.format_simple(), len)
            }
            Type::Pat { type_, .. } => {
                format!("{} is _", type_.format_simple())
            }
            Type::ImplTrait(bounds) => {
                use crate::ext::GenericBoundsExt;
                format!("impl {}", bounds.format_bounds())
            }
            Type::Infer => "_".to_string(),
            Type::RawPointer { is_mutable, type_ } => {
                let mut_str = if *is_mutable { "mut" } else { "const" };
                format!("*{} {}", mut_str, type_.format_simple())
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
                format!("&{}{}{}", lt_str, mut_str, type_.format_simple())
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
                        self_type.format_simple(),
                        trait_path.path,
                        name
                    )
                } else {
                    format!("<{}>::{}", self_type.format_simple(), name)
                }
            }
        }
    }

    fn format_with_context(&self, _krate: &Crate) -> String {
        // For now, just use simple formatting
        // Could enhance with crate context for resolving paths
        self.format_simple()
    }
}

/// Extension trait for formatting Generics
pub trait GenericsFormattingExt {
    fn format_params(&self) -> String;
    fn format_where_clause(&self) -> String;
}

impl GenericsFormattingExt for rustdoc_types::Generics {
    fn format_params(&self) -> String {
        if self.params.is_empty() {
            return String::new();
        }

        let params: Vec<String> = self
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
                    use crate::ext::GenericBoundsExt;
                    let mut s = param.name.clone();
                    if !bounds.is_empty() {
                        s.push_str(": ");
                        s.push_str(&bounds.format_bounds());
                    }
                    if let Some(def) = default {
                        s.push_str(" = ");
                        s.push_str(&def.format_simple());
                    }
                    s
                }
                rustdoc_types::GenericParamDefKind::Const { type_, default } => {
                    let mut s = format!("const {}: {}", param.name, type_.format_simple());
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

    fn format_where_clause(&self) -> String {
        if self.where_predicates.is_empty() {
            return String::new();
        }

        use crate::ext::GenericBoundsExt;

        let predicates: Vec<String> = self
            .where_predicates
            .iter()
            .filter_map(|pred| match pred {
                rustdoc_types::WherePredicate::BoundPredicate { type_, bounds, .. } => Some(
                    format!("{}: {}", type_.format_simple(), bounds.format_bounds()),
                ),
                rustdoc_types::WherePredicate::LifetimePredicate { lifetime, outlives } => {
                    if outlives.is_empty() {
                        None
                    } else {
                        Some(format!("{}: {}", lifetime, outlives.join(" + ")))
                    }
                }
                rustdoc_types::WherePredicate::EqPredicate { lhs, rhs } => {
                    Some(format!("{} = {}", lhs.format_simple(), format_term(rhs)))
                }
            })
            .collect();

        if predicates.is_empty() {
            String::new()
        } else {
            format!("\nwhere\n    {}", predicates.join(",\n    "))
        }
    }
}

/// Extension trait for formatting GenericArgs
pub trait GenericArgsFormattingExt {
    fn format(&self) -> String;
}

impl GenericArgsFormattingExt for GenericArgs {
    fn format(&self) -> String {
        match self {
            GenericArgs::ReturnTypeNotation => "(..)".to_string(),
            GenericArgs::AngleBracketed { args, constraints } => {
                let mut parts = Vec::new();

                for arg in args {
                    match arg {
                        rustdoc_types::GenericArg::Lifetime(lt) => parts.push(lt.clone()),
                        rustdoc_types::GenericArg::Type(ty) => parts.push(ty.format_simple()),
                        rustdoc_types::GenericArg::Const(c) => parts.push(c.expr.clone()),
                        rustdoc_types::GenericArg::Infer => parts.push("_".to_string()),
                    }
                }

                use crate::ext::GenericBoundsExt;
                for constraint in constraints {
                    match &constraint.binding {
                        rustdoc_types::AssocItemConstraintKind::Equality(term) => {
                            parts.push(format!("{} = {}", constraint.name, format_term(term)));
                        }
                        rustdoc_types::AssocItemConstraintKind::Constraint(bounds) => {
                            let bounds_str = bounds.format_bounds();
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
                    .map(|t| t.format_simple())
                    .collect::<Vec<_>>()
                    .join(", ");
                let output_str = output
                    .as_ref()
                    .map(|t| format!(" -> {}", t.format_simple()))
                    .unwrap_or_default();
                format!("({}){}", inputs_str, output_str)
            }
        }
    }
}

/// Extension trait for formatting FunctionSignature
pub trait FunctionSignatureExt {
    fn format_signature(
        &self,
        vis_prefix: &str,
        name: &str,
        generics: &rustdoc_types::Generics,
        header: &rustdoc_types::FunctionHeader,
    ) -> String;
}

impl FunctionSignatureExt for FunctionSignature {
    fn format_signature(
        &self,
        vis_prefix: &str,
        name: &str,
        generics: &rustdoc_types::Generics,
        header: &rustdoc_types::FunctionHeader,
    ) -> String {
        let mut parts = Vec::new();

        // Check if ABI is not Rust (the default)
        match &header.abi {
            rustdoc_types::Abi::Rust => {} // Default, don't show
            rustdoc_types::Abi::C { .. } => parts.push("extern \"C\"".to_string()),
            rustdoc_types::Abi::Other(abi_name) => parts.push(format!("extern \"{}\"", abi_name)),
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

        let gen_params = generics.format_params();

        // Format parameters
        let params: Vec<String> = self
            .inputs
            .iter()
            .map(|(param_name, ty)| format!("{}: {}", param_name, ty.format_simple()))
            .collect();
        let params_str = params.join(", ");

        // Format return type
        let return_str = if let Some(ret) = &self.output {
            format!(" -> {}", ret.format_simple())
        } else {
            String::new()
        };

        // Format where clause
        let where_clause = generics.format_where_clause();

        format!(
            "{}{}fn {}{}({}){}{}",
            vis_prefix, modifiers, name, gen_params, params_str, return_str, where_clause
        )
    }
}

/// Helper function for formatting Terms (used in equality predicates)
fn format_term(term: &rustdoc_types::Term) -> String {
    match term {
        rustdoc_types::Term::Type(ty) => ty.format_simple(),
        rustdoc_types::Term::Constant(c) => c.expr.clone(),
    }
}
