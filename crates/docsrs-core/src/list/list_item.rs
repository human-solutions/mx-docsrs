use jsondoc::JsonDocItem;
use rustdoc_fmt::Output;
use rustdoc_types::{Id, ItemEnum};

#[derive(Clone, Copy)]
pub enum EntryKind {
    Module,
    Struct,
    Enum,
    Trait,
    Function,
    Constant,
    Static,
    TypeAlias,
    Macro,
}

impl EntryKind {
    fn from_item_enum(item: &ItemEnum) -> Option<Self> {
        Some(match item {
            ItemEnum::Constant { .. } => EntryKind::Constant,
            ItemEnum::Function(_) => EntryKind::Function,
            ItemEnum::Module(_) => EntryKind::Module,
            ItemEnum::Struct(_) => EntryKind::Struct,
            ItemEnum::Enum(_) => EntryKind::Enum,
            ItemEnum::Trait(_) => EntryKind::Trait,
            ItemEnum::TypeAlias(_) => EntryKind::TypeAlias,
            ItemEnum::Macro(_) => EntryKind::Macro,
            ItemEnum::Static(_) => EntryKind::Static,
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

    fn keyword(self) -> &'static str {
        match self {
            EntryKind::Module => "mod",
            EntryKind::Struct => "struct",
            EntryKind::Enum => "enum",
            EntryKind::Trait => "trait",
            EntryKind::Function => "fn",
            EntryKind::Constant => "const",
            EntryKind::Static => "static",
            EntryKind::TypeAlias => "type",
            EntryKind::Macro => "macro",
        }
    }
}

/// Represent a public item of an analyzed crate, i.e. an item that forms part
/// of the public API of a crate.
#[derive(Clone)]
pub struct ListItem {
    module: Vec<(String, EntryKind)>,
    pub path: String,
    kind: EntryKind,
    pub id: Id,
}

impl ListItem {
    pub fn from_jsondoc_item(item: &JsonDocItem<'_>) -> Option<Self> {
        let kind = EntryKind::from_item_enum(&item.item().inner)?;

        // Skip items whose path contains hidden components (e.g., impl methods)
        if item.path().iter().any(|seg| seg.hide) {
            return None;
        }

        let module: Vec<(String, EntryKind)> = item
            .path()
            .iter()
            .filter_map(|seg| {
                let name = seg.item.name()?.to_string();
                let kind = EntryKind::from_item_enum(&seg.item.item.inner)?;
                Some((name, kind))
            })
            .collect();

        let path = module
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>()
            .join("::");

        Some(Self {
            module,
            path,
            kind,
            id: item.id(),
        })
    }

    /// Full-path output for search results: `fn crate::path::name`
    pub fn as_output(&self) -> Output {
        let mut out = Output::new();

        out.kind(self.kind.keyword()).whitespace();

        let last_idx = self.module.len().saturating_sub(1);
        for (i, (seg, seg_kind)) in self.module.iter().enumerate() {
            let is_last = i == last_idx;
            if is_last {
                Self::color_last_segment(&mut out, seg, self.kind);
            } else {
                Self::color_path_segment(&mut out, seg, seg_kind);
                out.symbol("::");
            }
        }

        out
    }

    /// Module-relative output for child listings: `pub struct Name`
    pub fn as_module_child(&self) -> Output {
        let mut out = Output::new();

        out.qualifier("pub").whitespace();
        out.kind(self.kind.keyword()).whitespace();

        // Use just the last segment (the item name)
        if let Some((name, _)) = self.module.last() {
            Self::color_last_segment(&mut out, name, self.kind);
        }

        out
    }

    fn color_last_segment(out: &mut Output, seg: &str, kind: EntryKind) {
        match kind {
            EntryKind::Macro => {
                out.identifier(seg).symbol("!");
            }
            EntryKind::Constant | EntryKind::Static => {
                out.symbol(seg);
            }
            EntryKind::Enum | EntryKind::Struct | EntryKind::Trait | EntryKind::TypeAlias => {
                out.type_(seg);
            }
            EntryKind::Function => {
                out.function(seg);
            }
            _ => {
                out.identifier(seg);
            }
        };
    }

    fn color_path_segment(out: &mut Output, seg: &str, seg_kind: &EntryKind) {
        match seg_kind {
            EntryKind::Enum | EntryKind::Struct | EntryKind::Trait | EntryKind::TypeAlias => {
                out.type_(seg);
            }
            EntryKind::Function => {
                out.function(seg);
            }
            EntryKind::Macro => {
                out.identifier(seg).symbol("!");
            }
            EntryKind::Constant | EntryKind::Static => {
                out.symbol(seg);
            }
            _ => {
                out.identifier(seg);
            }
        };
    }
}
