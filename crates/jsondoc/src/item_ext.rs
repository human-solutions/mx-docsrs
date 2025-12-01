use rustdoc_types::{Id, Item, ItemEnum, Struct, StructKind, VariantKind};

/// Extension trait for rustdoc_types::Item providing utility methods.
pub trait ItemExt {
    /// Returns an iterator over child items if this item can contain children.
    fn children(&self) -> Box<dyn Iterator<Item = &Id> + '_>;

    /// Returns the impls for this item if applicable.
    fn impls(&self) -> Option<&[Id]>;
}

impl ItemExt for Item {
    fn children(&self) -> Box<dyn Iterator<Item = &Id> + '_> {
        match &self.inner {
            ItemEnum::Module(m) => Box::new(m.items.iter()),
            ItemEnum::Union(u) => Box::new(u.fields.iter()),
            ItemEnum::Struct(Struct {
                kind: StructKind::Plain { fields, .. },
                ..
            })
            | ItemEnum::Variant(rustdoc_types::Variant {
                kind: VariantKind::Struct { fields, .. },
                ..
            }) => Box::new(fields.iter()),
            ItemEnum::Struct(Struct {
                kind: StructKind::Tuple(fields),
                ..
            }) => Box::new(fields.iter().filter_map(|f| f.as_ref())),
            ItemEnum::Enum(e) => Box::new(e.variants.iter()),
            ItemEnum::Trait(t) => Box::new(t.items.iter()),
            ItemEnum::Impl(i) => Box::new(i.items.iter()),
            _ => Box::new(std::iter::empty()),
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
