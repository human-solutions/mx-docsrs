use rustdoc_types::{Id, Item, ItemEnum, Struct, StructKind, VariantKind};

/// Extension trait for rustdoc_types::Item providing utility methods.
pub trait ItemExt {
    /// Returns the child items if this item can contain children.
    fn children(&self) -> Option<&Vec<Id>>;

    /// Returns the impls for this item if applicable.
    fn impls(&self) -> Option<&[Id]>;
}

impl ItemExt for Item {
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
