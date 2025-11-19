use super::nameable_item::NameableItem;
use crate::doc::{
    crate_wrapper::CrateWrapper, intermediate_public_item::IntermediatePublicItem,
    path_component::PathComponent, public_item::PublicItem, render::RenderingContext,
};
use rustdoc_types::{
    Attribute, Crate, Id, Impl, Item, ItemEnum, Module, Struct, StructKind, Type, Use, VariantKind,
};
use std::{
    collections::{HashMap, VecDeque},
    vec,
};

/// Items in rustdoc JSON reference each other by Id. The [`ItemProcessor`]
/// essentially takes one Id at a time and figure out what to do with it. Once
/// complete, the item is ready to be listed as part of the public API, and
/// optionally can also be used as part of a path to another (child) item.
///
/// This struct contains a (processed) path to an item that is about to be
/// processed further, and the Id of that item.
#[derive(Debug)]
struct UnprocessedItem<'c> {
    /// The path to the item to process.
    parent_path: Vec<PathComponent<'c>>,

    /// The Id of the item's logical parent (if any).
    parent_id: Option<Id>,

    /// The Id of the item to process.
    id: Id,
}

/// Processes items to find more items and to figure out the path to each item.
pub struct ItemProcessor<'c> {
    /// The original and unmodified rustdoc JSON, in deserialized form.
    crate_: CrateWrapper<'c>,

    /// A queue of unprocessed items to process.
    work_queue: VecDeque<UnprocessedItem<'c>>,

    /// The output. A list of processed items.
    output: Vec<IntermediatePublicItem<'c>>,
}

impl<'c> ItemProcessor<'c> {
    pub(crate) fn new(crate_: &'c Crate) -> Self {
        ItemProcessor {
            crate_: CrateWrapper::new(crate_),
            work_queue: VecDeque::new(),
            output: vec![],
        }
    }

    /// Adds an item to the front of the work queue.
    fn add_to_work_queue(
        &mut self,
        parent_path: Vec<PathComponent<'c>>,
        parent_id: Option<Id>,
        id: Id,
    ) {
        self.work_queue.push_front(UnprocessedItem {
            parent_path,
            parent_id,
            id,
        });
    }

    /// Processes the entire work queue.
    fn run(&mut self) {
        while let Some(unprocessed_item) = self.work_queue.pop_front() {
            if let Some(item) = self.crate_.get_item(unprocessed_item.id) {
                self.process_any_item(item, unprocessed_item);
            }
        }
    }

    /// Process any item.
    fn process_any_item(&mut self, item: &'c Item, unprocessed_item: UnprocessedItem<'c>) {
        match &item.inner {
            ItemEnum::Use(use_) => {
                if use_.is_glob {
                    self.process_use_glob_item(use_, unprocessed_item, item);
                } else {
                    self.process_use_item(item, use_, unprocessed_item);
                }
            }
            ItemEnum::Impl(impl_) => {
                self.process_impl_item(unprocessed_item, item, impl_);
            }
            _ => {
                self.process_item_unless_recursive(unprocessed_item, item, None);
            }
        }
    }

    /// Handle `pub use foo::*` wildcard imports.
    fn process_use_glob_item(
        &mut self,
        use_: &'c Use,
        unprocessed_item: UnprocessedItem<'c>,
        item: &'c Item,
    ) {
        if let Some(Item {
            inner: ItemEnum::Module(Module { items, .. }),
            ..
        }) = use_
            .id
            .and_then(|id| self.get_item_if_not_in_path(&unprocessed_item.parent_path, id))
        {
            for &item_id in items {
                self.add_to_work_queue(
                    unprocessed_item.parent_path.clone(),
                    unprocessed_item.parent_id,
                    item_id,
                );
            }
        } else {
            self.process_item(
                unprocessed_item,
                item,
                Some(format!("<<{}::*>>", use_.source)),
            );
        }
    }

    /// Inline public imports by replacing use with the actual item.
    fn process_use_item(
        &mut self,
        item: &'c Item,
        use_: &'c Use,
        unprocessed_item: UnprocessedItem<'c>,
    ) {
        let mut actual_item = item;

        if let Some(used_item) = use_
            .id
            .and_then(|id| self.get_item_if_not_in_path(&unprocessed_item.parent_path, id))
        {
            actual_item = used_item;
        }

        self.process_item(unprocessed_item, actual_item, Some(use_.name.clone()));
    }

    /// Processes impls with filtering (blanket and auto trait impls are omitted).
    fn process_impl_item(
        &mut self,
        unprocessed_item: UnprocessedItem<'c>,
        item: &'c Item,
        impl_: &'c Impl,
    ) {
        if !ImplKind::from(item, impl_).is_active() {
            return;
        }

        self.process_item_for_type(unprocessed_item, item, None, Some(&impl_.for_));
    }

    /// Make sure we don't process items recursively.
    fn process_item_unless_recursive(
        &mut self,
        unprocessed_item: UnprocessedItem<'c>,
        item: &'c Item,
        overridden_name: Option<String>,
    ) {
        if unprocessed_item
            .parent_path
            .iter()
            .any(|m| m.item.item.id == item.id)
        {
            let recursion_breaker = unprocessed_item.finish(
                item,
                Some(format!("<<{}>>", item.name.as_deref().unwrap_or(""))),
                None,
            );
            self.output.push(recursion_breaker);
        } else {
            self.process_item(unprocessed_item, item, overridden_name);
        }
    }

    /// Process an item and add it to output.
    fn process_item(
        &mut self,
        unprocessed_item: UnprocessedItem<'c>,
        item: &'c Item,
        overridden_name: Option<String>,
    ) {
        self.process_item_for_type(unprocessed_item, item, overridden_name, None);
    }

    /// Process an item with optional type information.
    fn process_item_for_type(
        &mut self,
        unprocessed_item: UnprocessedItem<'c>,
        item: &'c Item,
        overridden_name: Option<String>,
        type_: Option<&'c Type>,
    ) {
        let finished_item = unprocessed_item.finish(item, overridden_name, type_);

        let children = children_for_item(item).into_iter().flatten();
        let impls = impls_for_item(item).into_iter().flatten();

        for &id in children {
            self.add_to_work_queue(finished_item.path().into(), Some(item.id), id);
        }

        for &id in impls {
            let mut path = finished_item.path().to_vec();
            for a in &mut path {
                a.hide = true;
            }
            self.add_to_work_queue(path, Some(item.id), id);
        }

        self.output.push(finished_item);
    }

    /// Get item only if it's not in the path (to prevent recursion).
    fn get_item_if_not_in_path(
        &mut self,
        parent_path: &[PathComponent<'c>],
        id: Id,
    ) -> Option<&'c Item> {
        if parent_path.iter().any(|m| m.item.item.id == id) {
            return None;
        }

        self.crate_.get_item(id)
    }

    /// Map IDs to their public items.
    fn id_to_items(&self) -> HashMap<&Id, Vec<&IntermediatePublicItem<'_>>> {
        let mut id_to_items: HashMap<&Id, Vec<&IntermediatePublicItem<'_>>> = HashMap::new();
        for finished_item in &self.output {
            id_to_items
                .entry(&finished_item.item().id)
                .or_default()
                .push(finished_item);
        }
        id_to_items
    }
}

impl<'c> UnprocessedItem<'c> {
    /// Turns an [`UnprocessedItem`] into a finished [`IntermediatePublicItem`].
    fn finish(
        mut self,
        item: &'c Item,
        overridden_name: Option<String>,
        type_: Option<&'c Type>,
    ) -> IntermediatePublicItem<'c> {
        let mut path = self.parent_path.split_off(0);

        path.push(PathComponent {
            item: NameableItem {
                item,
                overridden_name,
                sorting_prefix: sorting_prefix(item),
            },
            type_,
            hide: false,
        });

        IntermediatePublicItem::new(path, self.parent_id, item.id)
    }
}

/// Sorting prefix to group items nicely.
pub(crate) fn sorting_prefix(item: &Item) -> u8 {
    match &item.inner {
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

        ItemEnum::Impl(impl_) => match ImplKind::from(item, impl_) {
            ImplKind::Inherent => 20,
            ImplKind::Trait | ImplKind::AutoDerived => 21,
            ImplKind::AutoTrait => 23,
            ImplKind::Blanket => 24,
        },

        ItemEnum::ExternType => 25,

        ItemEnum::TraitAlias(_) => 27,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ImplKind {
    /// E.g. `impl Foo` or `impl<'a> Foo<'a>`
    Inherent,

    /// E.g. `impl Bar for Foo {}`
    Trait,

    /// Auto-generated by `#[derive(...)]`
    AutoDerived,

    /// Auto-trait impl such as `impl Sync for Foo`
    AutoTrait,

    /// Blanket impls such as `impl<T> Any for T`
    Blanket,
}

impl ImplKind {
    fn from(impl_item: &Item, impl_: &Impl) -> Self {
        let has_blanket_impl = impl_.blanket_impl.is_some();
        let is_automatically_derived = impl_item.attrs.contains(&Attribute::AutomaticallyDerived);

        match (impl_.is_synthetic, has_blanket_impl) {
            (true, false) => ImplKind::AutoTrait,
            (false, true) => ImplKind::Blanket,
            _ if is_automatically_derived => ImplKind::AutoDerived,
            _ if impl_.trait_.is_none() => ImplKind::Inherent,
            _ => ImplKind::Trait,
        }
    }
}

impl ImplKind {
    /// Check if this impl should be included (hardcoded to -s -s behavior).
    fn is_active(&self) -> bool {
        match self {
            ImplKind::Blanket => false,    // omit_blanket_impls: true
            ImplKind::AutoTrait => false,  // omit_auto_trait_impls: true
            ImplKind::AutoDerived => true, // omit_auto_derived_impls: false
            ImplKind::Inherent | ImplKind::Trait => true,
        }
    }
}

/// Get child items.
const fn children_for_item(item: &Item) -> Option<&Vec<Id>> {
    match &item.inner {
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

/// Get impls for item.
pub fn impls_for_item(item: &Item) -> Option<&[Id]> {
    match &item.inner {
        ItemEnum::Union(u) => Some(&u.impls),
        ItemEnum::Struct(s) => Some(&s.impls),
        ItemEnum::Enum(e) => Some(&e.impls),
        ItemEnum::Primitive(p) => Some(&p.impls),
        ItemEnum::Trait(t) => Some(&t.implementations),
        _ => None,
    }
}

/// Extract public API from a crate.
pub(crate) fn public_api_in_crate(crate_: &Crate) -> Vec<PublicItem> {
    let mut item_processor = ItemProcessor::new(crate_);
    item_processor.add_to_work_queue(vec![], None, crate_.root);
    item_processor.run();

    let context = RenderingContext {
        crate_,
        id_to_items: item_processor.id_to_items(),
    };

    item_processor
        .output
        .iter()
        .map(|item| PublicItem::from_intermediate_public_item(&context, item))
        .collect::<Vec<_>>()
}
