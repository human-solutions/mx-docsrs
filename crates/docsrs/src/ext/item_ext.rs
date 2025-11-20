use crate::doc::impl_kind::ImplKind;
use rustdoc_types::{Id, Item, ItemEnum, Struct, StructKind, VariantKind};

/// Extension trait for rustdoc_types::Item providing utility methods.
pub trait ItemExt {
    /// Returns a sorting prefix to group items nicely.
    fn sorting_prefix(&self) -> u8;

    /// Returns the child items if this item can contain children.
    fn children(&self) -> Option<&Vec<Id>>;

    /// Returns the impls for this item if applicable.
    fn impls(&self) -> Option<&[Id]>;
}

impl ItemExt for Item {
    fn sorting_prefix(&self) -> u8 {
        match &self.inner {
            ItemEnum::ExternCrate { .. } => 1,
            ItemEnum::Use(_) => 2,

            ItemEnum::Primitive(_) => 3,

            ItemEnum::Module(_) => 4,

            ItemEnum::Macro(_) => 5,
            ItemEnum::ProcMacro(_) => 6,

            ItemEnum::Enum(_) => 7,
            ItemEnum::Union(_) => 8,
            ItemEnum::Struct(_) => 9,
            ItemEnum::StructField(_) => 10,
            ItemEnum::Variant(_) => 11,

            ItemEnum::Constant { .. } => 12,

            ItemEnum::Static(_) => 13,

            ItemEnum::Trait(_) => 14,

            ItemEnum::AssocType { .. } => 15,
            ItemEnum::AssocConst { .. } => 16,

            ItemEnum::Function(_) => 17,

            ItemEnum::TypeAlias(_) => 19,

            ItemEnum::Impl(impl_) => match ImplKind::from(self, impl_) {
                ImplKind::Inherent => 20,
                ImplKind::Trait | ImplKind::AutoDerived => 21,
                ImplKind::AutoTrait => 23,
                ImplKind::Blanket => 24,
            },

            ItemEnum::ExternType => 25,

            ItemEnum::TraitAlias(_) => 27,
        }
    }

    fn children(&self) -> Option<&Vec<Id>> {
        match &self.inner {
            ItemEnum::Module(m) => Some(&m.items),
            ItemEnum::Union(u) => Some(&u.fields),
            ItemEnum::Struct(Struct {
                kind: StructKind::Plain { fields, .. },
                ..
            })
            | ItemEnum::Variant(rustdoc_types::Variant {
                kind: VariantKind::Struct { fields, .. },
                ..
            }) => Some(fields),
            ItemEnum::Enum(e) => Some(&e.variants),
            ItemEnum::Trait(t) => Some(&t.items),
            ItemEnum::Impl(i) => Some(&i.items),
            _ => None,
        }
    }

    fn impls(&self) -> Option<&[Id]> {
        match &self.inner {
            ItemEnum::Union(u) => Some(&u.impls),
            ItemEnum::Struct(s) => Some(&s.impls),
            ItemEnum::Enum(e) => Some(&e.impls),
            ItemEnum::Primitive(p) => Some(&p.impls),
            ItemEnum::Trait(t) => Some(&t.implementations),
            _ => None,
        }
    }
}
