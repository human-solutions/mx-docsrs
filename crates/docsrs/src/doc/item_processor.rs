use super::impl_kind::ImplKind;
use super::unprocessed_item::UnprocessedItem;
use crate::doc::{
    crate_wrapper::CrateWrapper, intermediate_public_item::IntermediatePublicItem,
    path_component::PathComponent, public_item::PublicItem, render::RenderingContext,
};
use crate::ext::item_ext::ItemExt;
use rustdoc_types::{Crate, Id, Impl, Item, ItemEnum, Module, Type, Use};
use std::{
    collections::{HashMap, VecDeque},
    vec,
};

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
    pub fn add_to_work_queue(
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
    pub fn run(&mut self) {
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

        let children = item.children().into_iter().flatten();
        let impls = item.impls().into_iter().flatten();

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

/// Extract public API from a crate.
pub(crate) fn public_api_in_crate(
    crate_: &Crate,
    item_processor: &ItemProcessor,
) -> Vec<PublicItem> {
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
