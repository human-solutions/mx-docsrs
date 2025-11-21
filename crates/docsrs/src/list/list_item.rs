use rustdoc_types::{
    Constant, Enum, Function, Id, ItemEnum, Module, Static, Struct, Trait, TypeAlias,
};

use crate::{fmt::Output, proc::IntermediatePublicItem};

#[allow(dead_code)]
#[derive(Clone)]
pub enum EntryKind<'c> {
    Module(&'c Module),
    Struct(&'c Struct),
    Enum(&'c Enum),
    Trait(&'c Trait),
    Function(&'c Function),
    Constant(&'c Constant),
    Static(&'c Static),
    TypeAlias(&'c TypeAlias),
    Macro(&'c str),
}

impl<'c> EntryKind<'c> {
    fn from_item_enum(item: &'c ItemEnum) -> Option<Self> {
        Some(match item {
            ItemEnum::Constant { const_, .. } => EntryKind::Constant(const_),
            ItemEnum::Function(f) => EntryKind::Function(f),
            ItemEnum::Module(module) => EntryKind::Module(module),
            ItemEnum::Struct(s) => EntryKind::Struct(s),
            ItemEnum::Enum(e) => EntryKind::Enum(e),
            ItemEnum::Trait(t) => EntryKind::Trait(t),
            ItemEnum::TypeAlias(ta) => EntryKind::TypeAlias(ta),
            ItemEnum::Macro(m) => EntryKind::Macro(m),
            ItemEnum::Static(s) => EntryKind::Static(s),
            ItemEnum::ProcMacro(_)
            | ItemEnum::Primitive(_)
            | ItemEnum::Variant(_)
            | ItemEnum::TraitAlias(_)
            | ItemEnum::ExternCrate { .. }
            | ItemEnum::StructField(_)
            | ItemEnum::Use(_)
            | ItemEnum::Union(_)
            | ItemEnum::AssocConst { .. }
            | ItemEnum::ExternType
            | ItemEnum::Impl { .. }
            | ItemEnum::AssocType { .. } => return None,
        })
    }

    fn keyword(&self) -> &'static str {
        match self {
            EntryKind::Module(_) => "mod   ",
            EntryKind::Struct(_) => "struct",
            EntryKind::Enum(_) => "enum  ",
            EntryKind::Trait(_) => "trait ",
            EntryKind::Function(_) => "fn    ",
            EntryKind::Constant(_) => "const ",
            EntryKind::Static(_) => "static",
            EntryKind::TypeAlias(_) => "type  ",
            EntryKind::Macro(_) => "macro ",
        }
    }
}

/// Represent a public item of an analyzed crate, i.e. an item that forms part
/// of the public API of a crate.
#[derive(Clone)]
pub struct ListItem<'c> {
    module: Vec<String>,
    pub path: String,
    kind: EntryKind<'c>,
    _id: Id,
}

impl<'c> ListItem<'c> {
    pub fn from_intermediate(intermediate: &IntermediatePublicItem<'c>) -> Option<Self> {
        let kind = EntryKind::from_item_enum(&intermediate.item().inner)?;
        let module = intermediate
            .path()
            .iter()
            .filter_map(|seg| seg.item.name().map(|n| n.to_string()))
            .collect::<Vec<String>>();

        let path = module.join("::");

        Some(Self {
            module,
            path,
            kind,
            _id: intermediate.id(),
        })
    }

    pub fn as_output(&self) -> Output {
        let mut out = Output::new();

        out.kind(self.kind.keyword()).whitespace();

        for (i, seg) in self.module.iter().enumerate() {
            let is_last = i == self.module.len() - 1;
            if is_last {
                match self.kind {
                    EntryKind::Macro(_) => out.identifier(seg).symbol("!"),
                    EntryKind::Constant(_) | EntryKind::Static(_) => out.symbol(seg),
                    EntryKind::Enum(_)
                    | EntryKind::Struct(_)
                    | EntryKind::Trait(_)
                    | EntryKind::TypeAlias(_) => out.type_(seg),
                    EntryKind::Function(_) => out.function(seg),
                    _ => out.identifier(seg),
                };
            } else {
                out.identifier(seg).symbol("::");
            }
        }

        out
    }
}
