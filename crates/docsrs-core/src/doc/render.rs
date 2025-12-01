use jsondoc::{JsonDocItem, NameableItem, PathComponent};
use rustdoc_fmt::Output;
use std::{borrow::Cow, cmp::Ordering, collections::HashMap};

use rustdoc_types::{
    Abi, AssocItemConstraint, AssocItemConstraintKind, Attribute, AttributeRepr, Constant, Crate,
    FunctionHeader, FunctionPointer, FunctionSignature, GenericArg, GenericArgs, GenericBound,
    GenericParamDef, GenericParamDefKind, Generics, Id, Impl, Item, ItemEnum, MacroKind, Path,
    PolyTrait, ReprKind, StructKind, Term, Trait, TraitBoundModifier, Type, VariantKind,
    WherePredicate,
};

/// When we render an item, it might contain references to other parts of the
/// public API. For such cases, the rendering code can use the fields in this
/// struct.
pub struct RenderingContext<'c> {
    /// The original and unmodified rustdoc JSON, in deserialized form.
    pub crate_: &'c Crate,

    /// Given a rustdoc JSON ID, keeps track of what public items that have this Id.
    pub id_to_items: HashMap<&'c Id, Vec<&'c JsonDocItem<'c>>>,
}

impl<'c> RenderingContext<'c> {
    pub fn token_stream(&self, public_item: &JsonDocItem<'c>) -> Output {
        let item = public_item.item();
        let item_path = public_item.path();

        let mut tokens = Output::new();

        for attr in &item.attrs {
            if let Some(annotation) = match attr {
                Attribute::ExportName(name) => Some(format!("#[export_name = \"{name}\"]")),
                Attribute::LinkSection(section) => Some(format!("#[link_section = \"{section}\"]")),
                Attribute::NoMangle => Some("#[no_mangle]".to_string()),
                Attribute::NonExhaustive => Some("#[non_exhaustive]".to_string()),
                Attribute::Repr(AttributeRepr {
                    kind,
                    align,
                    packed,
                    int,
                }) => {
                    let mut items: Vec<Cow<'static, str>> = vec![];
                    if let Some(kind) = match kind {
                        ReprKind::Rust => None,
                        ReprKind::C => Some("C"),
                        ReprKind::Transparent => Some("transparent"),
                        ReprKind::Simd => Some("simd"),
                    } {
                        items.push(Cow::Borrowed(kind));
                    }
                    if let Some(align) = align {
                        items.push(Cow::Owned(format!("align({align})")))
                    }
                    if let Some(packed) = packed {
                        items.push(Cow::Owned(format!("packed({packed})")))
                    }
                    if let Some(int) = int {
                        items.push(Cow::Owned(int.to_string()))
                    }
                    (!items.is_empty()).then(|| {
                        let mut s = String::new();
                        s.push_str("#[repr(");
                        s.push_str(&items.join(", "));
                        s.push_str(")]");
                        s
                    })
                }
                _ => None,
            } {
                tokens.annotation(annotation).whitespace();
            }
        }

        let inner_tokens = match &item.inner {
            ItemEnum::Module(_) => self.render_simple(&["mod"], item_path),
            ItemEnum::ExternCrate { .. } => self.render_simple(&["extern", "crate"], item_path),
            ItemEnum::Use(_) => self.render_simple(&["use"], item_path),
            ItemEnum::Union(_) => self.render_simple(&["union"], item_path),
            ItemEnum::Struct(s) => {
                let mut output = self.render_simple(&["struct"], item_path);
                output.extend(self.render_generics(&s.generics));
                if let StructKind::Tuple(fields) = &s.kind {
                    let prefix = Output::new().qualifier_pub();
                    output.extend(
                        self.render_option_tuple(&self.resolve_tuple_fields(fields), Some(&prefix)),
                    );
                }
                output
            }
            ItemEnum::StructField(inner) => {
                let mut output = self.render_simple(&[], item_path);
                output.extend(Output::new().symbol_colon());
                output.extend(self.render_type(inner));
                output
            }
            ItemEnum::Enum(e) => {
                let mut output = self.render_simple(&["enum"], item_path);
                output.extend(self.render_generics(&e.generics));
                output
            }
            ItemEnum::Variant(inner) => {
                let mut output = self.render_simple(&[], item_path);
                match &inner.kind {
                    VariantKind::Struct { .. } => {} // Each struct field is printed individually
                    VariantKind::Plain => {
                        if let Some(discriminant) = &inner.discriminant {
                            output.extend(Output::new().symbol_equals());
                            output.identifier(&discriminant.value);
                        }
                    }
                    VariantKind::Tuple(fields) => {
                        output.extend(
                            self.render_option_tuple(&self.resolve_tuple_fields(fields), None),
                        );
                    }
                }
                output
            }
            ItemEnum::Function(inner) => self.render_function(
                self.render_path(item_path),
                &inner.sig,
                &inner.generics,
                &inner.header,
            ),
            ItemEnum::Trait(trait_) => self.render_trait(trait_, item_path),
            ItemEnum::TraitAlias(_) => self.render_simple(&["trait", "alias"], item_path),
            ItemEnum::Impl(impl_) => {
                self.render_impl(impl_, item_path, false /* disregard_negativity */)
            }
            ItemEnum::TypeAlias(inner) => {
                let mut output = self.render_simple(&["type"], item_path);
                output.extend(self.render_generics(&inner.generics));
                output.extend(Output::new().symbol_equals());
                output.extend(self.render_type(&inner.type_));
                output
            }
            ItemEnum::AssocType {
                generics,
                bounds,
                type_,
            } => {
                let mut output = self.render_simple(&["type"], item_path);
                output.extend(self.render_generics(generics));
                output.extend(self.render_generic_bounds_with_colon(bounds));
                if let Some(ty) = type_ {
                    output.extend(Output::new().symbol_equals());
                    output.extend(self.render_type(ty));
                }
                output
            }
            ItemEnum::Constant { const_, type_ } => {
                let mut output = self.render_simple(&["const"], item_path);
                output.extend(Output::new().symbol_colon());
                output.extend(self.render_constant(const_, Some(type_)));
                output
            }
            ItemEnum::AssocConst { type_, .. } => {
                let mut output = self.render_simple(&["const"], item_path);
                output.extend(Output::new().symbol_colon());
                output.extend(self.render_type(type_));
                output
            }
            ItemEnum::Static(inner) => {
                let tags = if inner.is_mutable {
                    vec!["mut", "static"]
                } else {
                    vec!["static"]
                };
                let mut output = self.render_simple(&tags, item_path);
                output.extend(Output::new().symbol_colon());
                output.extend(self.render_type(&inner.type_));
                output
            }
            ItemEnum::ExternType => self.render_simple(&["type"], item_path),
            ItemEnum::Macro(_definition) => {
                let mut output = self.render_simple(&["macro"], item_path);
                output.symbol("!");
                output
            }
            ItemEnum::ProcMacro(inner) => {
                let mut output = self.render_simple(&["proc", "macro"], item_path);
                output.pop(); // Remove name of macro to possibly wrap it in `#[]`
                let name = item.name.as_deref().unwrap_or("");
                match inner.kind {
                    MacroKind::Bang => {
                        output.identifier(name).symbol("!()");
                    }
                    MacroKind::Attr => {
                        output.symbol("#[").identifier(name).symbol("]");
                    }
                    MacroKind::Derive => {
                        output.symbol("#[derive(").identifier(name).symbol(")]");
                    }
                }
                output
            }
            ItemEnum::Primitive(primitive) => {
                let mut output = Output::new().qualifier_pub();
                output.kind("type").whitespace().primitive(&primitive.name);
                output
            }
        };

        tokens.extend(inner_tokens);

        tokens
    }

    /// Tuple fields are referenced by ID in JSON, but we need to look up the
    /// actual types that the IDs correspond to, in order to render the fields.
    /// This helper does that for a slice of fields.
    fn resolve_tuple_fields(&self, fields: &[Option<Id>]) -> Vec<Option<&'c Type>> {
        let mut resolved_fields: Vec<Option<&Type>> = vec![];

        for id in fields {
            resolved_fields.push(
                if let Some(Item {
                    inner: ItemEnum::StructField(type_),
                    ..
                }) = id.as_ref().and_then(|id| self.crate_.index.get(id))
                {
                    Some(type_)
                } else {
                    None
                },
            );
        }

        resolved_fields
    }

    fn render_simple(&self, tags: &[&str], path: &[PathComponent]) -> Output {
        let mut output = Output::new().qualifier_pub();
        for tag in tags {
            output.kind(*tag).whitespace();
        }
        output.extend(self.render_path(path));
        output
    }

    fn render_path(&self, path: &[PathComponent]) -> Output {
        let mut output = Output::new();
        for component in path {
            if component.hide {
                continue;
            }

            let (tokens, push_a_separator) = component.type_.map_or_else(
                || self.render_nameable_item(&component.item),
                |ty| self.render_type_and_separator(ty),
            );

            output.extend(tokens);

            if push_a_separator {
                output.symbol("::");
            }
        }
        if !path.is_empty() {
            output.pop(); // Remove last "::" so "a::b::c::" becomes "a::b::c"
        }
        output
    }

    fn render_nameable_item(&self, item: &NameableItem) -> (Output, bool) {
        let mut push_a_separator = false;
        let mut output = Output::new();

        if let Some(name) = item.name() {
            // Only push a name if it exists (impls don't have names)
            if matches!(item.item.inner, ItemEnum::Function(_)) {
                output.function(name.to_string());
            } else if matches!(
                item.item.inner,
                ItemEnum::Trait(_)
                    | ItemEnum::Struct(_)
                    | ItemEnum::Union(_)
                    | ItemEnum::Enum(_)
                    | ItemEnum::TypeAlias(_)
            ) {
                output.type_(name.to_string());
            } else {
                output.identifier(name.to_string());
            }
            push_a_separator = true;
        }
        (output, push_a_separator)
    }

    fn render_sequence<T>(
        &self,
        start: Output,
        end: Output,
        between: Output,
        sequence: &[T],
        render: impl Fn(&T) -> Output,
    ) -> Output {
        self.render_sequence_impl(start, end, between, false, sequence, render)
    }

    fn render_sequence_if_not_empty<T>(
        &self,
        start: Output,
        end: Output,
        between: Output,
        sequence: &[T],
        render: impl Fn(&T) -> Output,
    ) -> Output {
        self.render_sequence_impl(start, end, between, true, sequence, render)
    }

    fn render_sequence_impl<T>(
        &self,
        start: Output,
        end: Output,
        between: Output,
        return_nothing_if_empty: bool,
        sequence: &[T],
        render: impl Fn(&T) -> Output,
    ) -> Output {
        if return_nothing_if_empty && sequence.is_empty() {
            return Output::new();
        }
        let mut output = start;
        for (index, seq) in sequence.iter().enumerate() {
            output.extend(render(seq));
            if index < sequence.len() - 1 {
                output.extend(between.clone());
            }
        }
        output.extend(end);
        output
    }

    pub fn render_type(&self, ty: &Type) -> Output {
        self.render_option_type(&Some(ty))
    }

    fn render_type_and_separator(&self, ty: &Type) -> (Output, bool) {
        (self.render_type(ty), true)
    }

    fn render_option_type(&self, ty: &Option<&Type>) -> Output {
        let Some(ty) = ty else {
            let mut output = Output::new();
            output.symbol("_");
            return output;
        }; // The `_` in `EnumWithStrippedTupleVariants::DoubleFirstHidden(_, bool)`
        match ty {
            Type::ResolvedPath(path) => self.render_resolved_path(path),
            Type::DynTrait(dyn_trait) => self.render_dyn_trait(dyn_trait),
            Type::Generic(name) => {
                let mut output = Output::new();
                output.generic(name);
                output
            }
            Type::Primitive(name) => {
                let mut output = Output::new();
                output.primitive(name);
                output
            }
            Type::FunctionPointer(ptr) => self.render_function_pointer(ptr),
            Type::Tuple(types) => self.render_tuple(types),
            Type::Slice(ty) => self.render_slice(ty),
            Type::Array { type_, len } => self.render_array(type_, len),
            Type::ImplTrait(bounds) => self.render_impl_trait(bounds),
            Type::Infer => {
                let mut output = Output::new();
                output.symbol("_");
                output
            }
            Type::RawPointer { is_mutable, type_ } => self.render_raw_pointer(*is_mutable, type_),
            Type::BorrowedRef {
                lifetime,
                is_mutable,
                type_,
            } => self.render_borrowed_ref(lifetime.as_deref(), *is_mutable, type_),
            Type::QualifiedPath {
                name,
                args: _,
                self_type,
                trait_,
            } => self.render_qualified_path(self_type, trait_.as_ref(), name),
            Type::Pat { .. } => {
                let mut output = Output::new();
                output.symbol(
                    "https://github.com/rust-lang/rust/issues/123646 is unstable and not supported",
                );
                output
            }
        }
    }

    fn render_trait(&self, trait_: &Trait, path: &[PathComponent]) -> Output {
        let mut output = Output::new().qualifier_pub();
        if trait_.is_unsafe {
            output.qualifier("unsafe").whitespace();
        };
        output.kind("trait").whitespace();
        output.extend(self.render_path(path));
        output.extend(self.render_generics(&trait_.generics));
        output.extend(self.render_generic_bounds_with_colon(&trait_.bounds));
        output
    }

    fn render_dyn_trait(&self, dyn_trait: &rustdoc_types::DynTrait) -> Output {
        let mut output = Output::new();

        let more_than_one = dyn_trait.traits.len() > 1 || dyn_trait.lifetime.is_some();
        if more_than_one {
            output.symbol("(");
        }

        let mut start = Output::new();
        start.keyword("dyn").whitespace();
        output.extend(self.render_sequence_if_not_empty(
            start,
            Output::new(),
            Output::new().symbol_plus(),
            &dyn_trait.traits,
            |p| self.render_poly_trait(p),
        ));

        if let Some(lt) = &dyn_trait.lifetime {
            output.extend(Output::new().symbol_plus());
            output.lifetime(lt);
        }

        if more_than_one {
            output.symbol(")");
        }

        output
    }

    pub fn render_function(
        &self,
        name: Output,
        sig: &FunctionSignature,
        generics: &Generics,
        header: &FunctionHeader,
    ) -> Output {
        let mut output = Output::new().qualifier_pub();
        if header.is_unsafe {
            output.qualifier("unsafe").whitespace();
        };
        if header.is_const {
            output.qualifier("const").whitespace();
        };
        if header.is_async {
            output.qualifier("async").whitespace();
        };
        if header.abi != Abi::Rust {
            let abi_str = match &header.abi {
                Abi::C { .. } => "c",
                Abi::Cdecl { .. } => "cdecl",
                Abi::Stdcall { .. } => "stdcall",
                Abi::Fastcall { .. } => "fastcall",
                Abi::Aapcs { .. } => "aapcs",
                Abi::Win64 { .. } => "win64",
                Abi::SysV64 { .. } => "sysV64",
                Abi::System { .. } => "system",
                Abi::Other(text) => text,
                Abi::Rust => unreachable!(),
            };
            output.qualifier(abi_str).whitespace();
        }

        output.kind("fn").whitespace();
        output.extend(name);

        // Generic parameters
        output.extend(self.render_generic_param_defs(&generics.params));

        // Regular parameters and return type
        output.extend(self.render_fn_decl(sig, true));

        // Where predicates
        output.extend(self.render_where_predicates(&generics.where_predicates));

        output
    }

    fn render_fn_decl(&self, sig: &FunctionSignature, include_underscores: bool) -> Output {
        let mut start = Output::new();
        start.symbol("(");
        let mut end = Output::new();
        end.symbol(")");

        let mut output = self.render_sequence(
            start,
            end,
            Output::new().symbol_comma(),
            &sig.inputs,
            |(name, ty)| {
                self.simplified_self(name, ty).unwrap_or_else(|| {
                    let mut output = Output::new();
                    let ignore_name = name.is_empty() || (name == "_" && !include_underscores);
                    if !ignore_name {
                        output.identifier(name).symbol(":").whitespace();
                    }
                    output.extend(self.render_type(ty));
                    output
                })
            },
        );
        // Return type
        if let Some(ty) = &sig.output {
            output.extend(Output::new().symbol_arrow());
            output.extend(self.render_type(ty));
        }
        output
    }

    fn simplified_self(&self, name: &str, ty: &Type) -> Option<Output> {
        if name == "self" {
            match ty {
                Type::Generic(name) if name == "Self" => {
                    let mut output = Output::new();
                    output.self_("self");
                    Some(output)
                }
                Type::BorrowedRef {
                    lifetime,
                    is_mutable,
                    type_,
                } => match type_.as_ref() {
                    Type::Generic(name) if name == "Self" => {
                        let mut output = Output::new();
                        output.symbol("&");
                        if let Some(lt) = lifetime {
                            output.lifetime(lt).whitespace();
                        }
                        if *is_mutable {
                            output.keyword("mut").whitespace();
                        }
                        output.self_("self");
                        Some(output)
                    }
                    _ => None,
                },
                _ => None,
            }
        } else {
            None
        }
    }

    fn render_resolved_path(&self, path: &Path) -> Output {
        let mut output = Output::new();
        if let Some(item) = self.best_item_for_id(&path.id) {
            output.extend(self.render_path(item.path()));
        } else if let Some(item) = self.crate_.paths.get(&path.id) {
            output.extend(self.render_path_components(item.path.iter()));
        } else if !path.path.is_empty() {
            output.extend(self.render_path_name(&path.path));
        }
        if let Some(args) = &path.args {
            output.extend(self.render_generic_args(args));
        }
        output
    }

    fn render_path_name(&self, name: &str) -> Output {
        self.render_path_components(name.split("::"))
    }

    fn render_path_components(&self, path_iter: impl Iterator<Item = impl AsRef<str>>) -> Output {
        let mut output = Output::new();
        let path: Vec<_> = path_iter.collect();
        let len = path.len();
        for (index, part) in path.into_iter().enumerate() {
            if index == len - 1 {
                output.type_(part.as_ref());
            } else {
                output.identifier(part.as_ref());
            }
            output.symbol("::");
        }
        if len > 0 {
            output.pop();
        }
        output
    }

    fn render_function_pointer(&self, ptr: &FunctionPointer) -> Output {
        let mut output = self.render_higher_rank_trait_bounds(&ptr.generic_params);
        output.kind("fn");
        output.extend(self.render_fn_decl(&ptr.sig, false));
        output
    }

    fn render_tuple(&self, types: &[Type]) -> Output {
        let option_tuple: Vec<Option<&Type>> = types.iter().map(Some).collect();
        self.render_option_tuple(&option_tuple, None)
    }

    /// `prefix` is to handle the difference  between tuple structs and enum variant
    /// tuple structs. The former marks public fields as `pub ` whereas all fields
    /// of enum tuple structs are always implicitly `pub`.
    fn render_option_tuple(&self, types: &[Option<&Type>], prefix: Option<&Output>) -> Output {
        let mut start = Output::new();
        start.symbol("(");
        let mut end = Output::new();
        end.symbol(")");

        self.render_sequence(start, end, Output::new().symbol_comma(), types, |type_| {
            let mut output = Output::new();
            if let (Some(prefix), Some(_)) = (prefix, type_) {
                output.extend(prefix.clone());
            }
            output.extend(self.render_option_type(type_));
            output
        })
    }

    fn render_slice(&self, ty: &Type) -> Output {
        let mut output = Output::new();
        output.symbol("[");
        output.extend(self.render_type(ty));
        output.symbol("]");
        output
    }

    fn render_array(&self, type_: &Type, len: &str) -> Output {
        let mut output = Output::new();
        output.symbol("[");
        output.extend(self.render_type(type_));
        output.symbol(";").whitespace().primitive(len).symbol("]");
        output
    }

    pub fn render_impl(
        &self,
        impl_: &Impl,
        _path: &[PathComponent],
        disregard_negativity: bool,
    ) -> Output {
        let mut output = Output::new();

        if impl_.is_unsafe {
            output.keyword("unsafe").whitespace();
        }

        output.keyword("impl");

        output.extend(self.render_generic_param_defs(&impl_.generics.params));

        output.whitespace();

        if let Some(trait_) = &impl_.trait_ {
            if !disregard_negativity && impl_.is_negative {
                output.symbol("!");
            }
            output.extend(self.render_resolved_path(trait_));
            output.whitespace().keyword("for").whitespace();
            output.extend(self.render_type(&impl_.for_));
        } else {
            output.extend(self.render_type(&impl_.for_));
        }

        output.extend(self.render_where_predicates(&impl_.generics.where_predicates));

        output
    }

    fn render_impl_trait(&self, bounds: &[GenericBound]) -> Output {
        let mut output = Output::new();
        output.keyword("impl").whitespace();
        output.extend(self.render_generic_bounds(bounds));
        output
    }

    fn render_raw_pointer(&self, is_mutable: bool, type_: &Type) -> Output {
        let mut output = Output::new();
        output.symbol("*");
        output.keyword(if is_mutable { "mut" } else { "const" });
        output.whitespace();
        output.extend(self.render_type(type_));
        output
    }

    fn render_borrowed_ref(
        &self,
        lifetime: Option<&str>,
        is_mutable: bool,
        type_: &Type,
    ) -> Output {
        let mut output = Output::new();
        output.symbol("&");
        if let Some(lt) = lifetime {
            output.lifetime(lt).whitespace();
        }
        if is_mutable {
            output.keyword("mut").whitespace();
        }
        output.extend(self.render_type(type_));
        output
    }

    fn render_qualified_path(&self, type_: &Type, trait_: Option<&Path>, name: &str) -> Output {
        let mut output = Output::new();
        match (type_, trait_) {
            (Type::Generic(name), Some(trait_)) if name == "Self" && trait_.path.is_empty() => {
                output.keyword("Self");
            }
            (_, trait_) => {
                if trait_.is_some() {
                    output.symbol("<");
                }
                output.extend(self.render_type(type_));
                if let Some(trait_) = trait_ {
                    output.whitespace().keyword("as").whitespace();
                    output.extend(self.render_resolved_path(trait_));
                    output.symbol(">");
                }
            }
        }
        output.symbol("::").identifier(name);
        output
    }

    fn render_generic_args(&self, args: &GenericArgs) -> Output {
        match args {
            GenericArgs::AngleBracketed { args, constraints } => {
                self.render_angle_bracketed(args, constraints)
            }
            GenericArgs::Parenthesized { inputs, output } => {
                self.render_parenthesized(inputs, output)
            }
            GenericArgs::ReturnTypeNotation => {
                let mut output = Output::new();
                output.symbol("ReturnTypeNotation not supported");
                output
            }
        }
    }

    fn render_parenthesized(&self, inputs: &[Type], return_ty: &Option<Type>) -> Output {
        let mut start = Output::new();
        start.symbol("(");
        let mut end = Output::new();
        end.symbol(")");

        let mut output =
            self.render_sequence(start, end, Output::new().symbol_comma(), inputs, |type_| {
                self.render_type(type_)
            });
        if let Some(return_ty) = return_ty {
            output.extend(Output::new().symbol_arrow());
            output.extend(self.render_type(return_ty));
        }
        output
    }

    fn render_angle_bracketed(
        &self,
        args: &[GenericArg],
        constraints: &[AssocItemConstraint],
    ) -> Output {
        enum Arg<'c> {
            GenericArg(&'c GenericArg),
            AssocItemConstraint(&'c AssocItemConstraint),
        }
        let mut start = Output::new();
        start.symbol("<");
        let mut end = Output::new();
        end.symbol(">");

        self.render_sequence_if_not_empty(
            start,
            end,
            Output::new().symbol_comma(),
            &args
                .iter()
                .map(Arg::GenericArg)
                .chain(constraints.iter().map(Arg::AssocItemConstraint))
                .collect::<Vec<_>>(),
            |arg| match arg {
                Arg::GenericArg(arg) => self.render_generic_arg(arg),
                Arg::AssocItemConstraint(constraints) => {
                    self.render_assoc_item_constraint(constraints)
                }
            },
        )
    }

    fn render_term(&self, term: &Term) -> Output {
        match term {
            Term::Type(ty) => self.render_type(ty),
            Term::Constant(c) => self.render_constant(c, None),
        }
    }

    fn render_poly_trait(&self, poly_trait: &PolyTrait) -> Output {
        let mut output = self.render_higher_rank_trait_bounds(&poly_trait.generic_params);
        output.extend(self.render_resolved_path(&poly_trait.trait_));
        output
    }

    fn render_generic_arg(&self, arg: &GenericArg) -> Output {
        match arg {
            GenericArg::Lifetime(name) => {
                let mut output = Output::new();
                output.lifetime(name);
                output
            }
            GenericArg::Type(ty) => self.render_type(ty),
            GenericArg::Const(c) => self.render_constant(c, None),
            GenericArg::Infer => {
                let mut output = Output::new();
                output.symbol("_");
                output
            }
        }
    }

    fn render_assoc_item_constraint(&self, constraints: &AssocItemConstraint) -> Output {
        let mut output = Output::new();
        output.identifier(&constraints.name);
        if let Some(generic_args) = &constraints.args {
            output.extend(self.render_generic_args(generic_args));
        }
        match &constraints.binding {
            AssocItemConstraintKind::Equality(term) => {
                output.extend(Output::new().symbol_equals());
                output.extend(self.render_term(term));
            }
            AssocItemConstraintKind::Constraint(bounds) => {
                output.extend(self.render_generic_bounds(bounds));
            }
        }
        output
    }

    fn render_constant(&self, constant: &Constant, type_: Option<&Type>) -> Output {
        let mut output = Output::new();
        if let Some(type_) = type_ {
            output.extend(self.render_type(type_));
        } else if let Some(value) = &constant.value {
            if constant.is_literal {
                output.primitive(value);
            } else {
                output.identifier(value);
            }
        } else {
            output.identifier(&constant.expr);
        }
        output
    }

    fn render_generics(&self, generics: &Generics) -> Output {
        let mut output = Output::new();
        output.extend(self.render_generic_param_defs(&generics.params));
        output.extend(self.render_where_predicates(&generics.where_predicates));
        output
    }

    fn render_generic_param_defs(&self, params: &[GenericParamDef]) -> Output {
        let params_without_synthetics: Vec<_> = params
            .iter()
            .filter(|p| {
                if let GenericParamDefKind::Type { is_synthetic, .. } = p.kind {
                    !is_synthetic
                } else {
                    true
                }
            })
            .collect();

        let mut start = Output::new();
        start.symbol("<");
        let mut end = Output::new();
        end.symbol(">");

        self.render_sequence_if_not_empty(
            start,
            end,
            Output::new().symbol_comma(),
            &params_without_synthetics,
            |param| self.render_generic_param_def(param),
        )
    }

    fn render_generic_param_def(&self, generic_param_def: &GenericParamDef) -> Output {
        let mut output = Output::new();
        match &generic_param_def.kind {
            GenericParamDefKind::Lifetime { outlives } => {
                output.lifetime(&generic_param_def.name);
                if !outlives.is_empty() {
                    output.extend(Output::new().symbol_colon());
                    output.extend(self.render_sequence(
                        Output::new(),
                        Output::new(),
                        Output::new().symbol_plus(),
                        outlives,
                        |s| {
                            let mut out = Output::new();
                            out.lifetime(s);
                            out
                        },
                    ));
                }
            }
            GenericParamDefKind::Type { bounds, .. } => {
                output.generic(&generic_param_def.name);
                output.extend(self.render_generic_bounds_with_colon(bounds));
            }
            GenericParamDefKind::Const { type_, .. } => {
                output
                    .qualifier("const")
                    .whitespace()
                    .identifier(&generic_param_def.name);
                output.extend(Output::new().symbol_colon());
                output.extend(self.render_type(type_));
            }
        }
        output
    }

    fn render_where_predicates(&self, where_predicates: &[WherePredicate]) -> Output {
        let mut output = Output::new();
        if !where_predicates.is_empty() {
            output.whitespace();
            output.keyword("where");
            output.whitespace();
            output.extend(self.render_sequence(
                Output::new(),
                Output::new(),
                Output::new().symbol_comma(),
                where_predicates,
                |p| self.render_where_predicate(p),
            ));
        }
        output
    }

    fn render_where_predicate(&self, where_predicate: &WherePredicate) -> Output {
        let mut output = Output::new();
        match where_predicate {
            WherePredicate::BoundPredicate {
                type_,
                bounds,
                generic_params,
            } => {
                output.extend(self.render_higher_rank_trait_bounds(generic_params));
                output.extend(self.render_type(type_));
                output.extend(self.render_generic_bounds_with_colon(bounds));
            }
            WherePredicate::LifetimePredicate { lifetime, outlives } => {
                output.lifetime(lifetime);
                output.extend(self.render_sequence_if_not_empty(
                    Output::new().symbol_colon(),
                    Output::new(),
                    Output::new().symbol_plus(),
                    outlives,
                    |s| {
                        let mut out = Output::new();
                        out.lifetime(s);
                        out
                    },
                ));
            }
            WherePredicate::EqPredicate { lhs, rhs } => {
                output.extend(self.render_type(lhs));
                output.extend(Output::new().symbol_equals());
                output.extend(self.render_term(rhs));
            }
        }
        output
    }

    fn render_generic_bounds_with_colon(&self, bounds: &[GenericBound]) -> Output {
        let mut output = Output::new();
        if !bounds.is_empty() {
            output.extend(Output::new().symbol_colon());
            output.extend(self.render_generic_bounds(bounds));
        }
        output
    }

    fn render_generic_bounds(&self, bounds: &[GenericBound]) -> Output {
        self.render_sequence_if_not_empty(
            Output::new(),
            Output::new(),
            Output::new().symbol_plus(),
            bounds,
            |bound| {
                match bound {
                    GenericBound::TraitBound {
                        trait_,
                        generic_params,
                        modifier,
                    } => {
                        let mut output = Output::new();
                        output.extend(self.render_higher_rank_trait_bounds(generic_params));
                        match modifier {
                            TraitBoundModifier::None | TraitBoundModifier::MaybeConst => {}
                            TraitBoundModifier::Maybe => {
                                output.symbol("?");
                            }
                        }
                        output.extend(self.render_resolved_path(trait_));
                        output
                    }
                    GenericBound::Outlives(id) => {
                        let mut output = Output::new();
                        output.lifetime(id);
                        output
                    }
                    GenericBound::Use(args) => {
                        let mut output = Output::new();
                        output.keyword("use").symbol("<");

                        for i in 0..args.len() {
                            match &args[i] {
                                rustdoc_types::PreciseCapturingArg::Lifetime(lifetime) => {
                                    output.lifetime(lifetime);
                                }
                                rustdoc_types::PreciseCapturingArg::Param(param) => {
                                    output.generic(param);
                                }
                            }

                            // Insert a ", " in between parameters, but not after the final one.
                            if i < args.len() - 1 {
                                output.symbol(",").whitespace();
                            }
                        }

                        output.symbol(">");

                        output
                    }
                }
            },
        )
    }

    fn render_higher_rank_trait_bounds(&self, generic_params: &[GenericParamDef]) -> Output {
        let mut output = Output::new();
        if !generic_params.is_empty() {
            output
                .keyword("for")
                .extend(self.render_generic_param_defs(generic_params))
                .whitespace();
        }
        output
    }

    fn best_item_for_id(&self, id: &'c Id) -> Option<&'c JsonDocItem<'c>> {
        match self.id_to_items.get(&id) {
            None => None,
            Some(items) => {
                items
                    .iter()
                    .max_by(|a, b| {
                        // If there is any item in the path that has been
                        // renamed/re-exported, i.e. that is not the original
                        // path, prefer that less than an item with a path where
                        // all items are original.
                        let mut ordering = match (
                            a.path_contains_renamed_item(),
                            b.path_contains_renamed_item(),
                        ) {
                            (true, false) => Ordering::Less,
                            (false, true) => Ordering::Greater,
                            _ => Ordering::Equal,
                        };

                        // If we still can't make up our mind, go with the shortest path
                        if ordering == Ordering::Equal {
                            ordering = b.path().len().cmp(&a.path().len());
                        }

                        ordering
                    })
                    .copied()
            }
        }
    }
}
