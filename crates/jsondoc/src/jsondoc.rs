use ouroboros::self_referencing;
use rustdoc_types::{Crate, Id, Impl, Item, ItemEnum, Module, Type, Use};
use std::collections::{HashMap, VecDeque};

use crate::{
    crate_wrapper::CrateWrapper, impl_kind::ImplKind, item_ext::ItemExt, jsondoc_item::JsonDocItem,
    path_component::PathComponent, unprocessed_item::UnprocessedItem,
};

/// JSON documentation for a Rust crate.
///
/// Owns the rustdoc `Crate` data and provides structured access to public API items
/// with their full paths.
#[self_referencing]
pub struct JsonDoc {
    /// The owned crate data.
    crate_data: Crate,

    /// Processed items that borrow from crate_data.
    #[borrows(crate_data)]
    #[covariant]
    items: Vec<JsonDocItem<'this>>,
}

impl From<Crate> for JsonDoc {
    /// Create a new JsonDoc by processing the given Crate.
    /// Takes ownership of the Crate.
    fn from(crate_: Crate) -> Self {
        JsonDocBuilder {
            crate_data: crate_,
            items_builder: |crate_ref: &Crate| process_crate(crate_ref),
        }
        .build()
    }
}

impl JsonDoc {
    /// Returns the processed items.
    pub fn items(&self) -> &[JsonDocItem<'_>] {
        self.borrow_items()
    }

    /// Map IDs to their public items.
    pub fn id_to_items(&self) -> HashMap<&Id, Vec<&JsonDocItem<'_>>> {
        let mut map: HashMap<&Id, Vec<&JsonDocItem<'_>>> = HashMap::new();
        for item in self.borrow_items() {
            map.entry(&item.item().id).or_default().push(item);
        }
        map
    }

    /// Returns the crate root module ID.
    pub fn crate_root_id(&self) -> Id {
        self.borrow_crate_data().root
    }

    /// Find the item ID for an exact path like "tokio::task".
    /// Returns None if no item exists at that exact path.
    pub fn find_item_by_path(&self, path: &str) -> Option<Id> {
        for item in self.borrow_items() {
            // Skip hidden items
            if item.path().iter().any(|seg| seg.hide) {
                continue;
            }

            // Build the path string
            let item_path: String = item
                .path()
                .iter()
                .filter_map(|seg| seg.item.name())
                .collect::<Vec<_>>()
                .join("::");

            if item_path == path {
                return Some(item.id());
            }
        }
        None
    }

    /// Access the underlying crate data.
    pub fn crate_data(&self) -> &Crate {
        self.borrow_crate_data()
    }
}

/// Process a crate into a list of public items.
fn process_crate(crate_: &Crate) -> Vec<JsonDocItem<'_>> {
    let mut processor = Processor::new(crate_);
    processor.add_to_work_queue(vec![], None, crate_.root);
    processor.run();
    processor.output
}

/// Internal processor that works on borrowed crate data.
struct Processor<'c> {
    /// The original and unmodified rustdoc JSON, in deserialized form.
    crate_: CrateWrapper<'c>,

    /// A queue of unprocessed items to process.
    work_queue: VecDeque<UnprocessedItem<'c>>,

    /// The output. A list of processed items.
    output: Vec<JsonDocItem<'c>>,
}

impl<'c> Processor<'c> {
    fn new(crate_: &'c Crate) -> Self {
        Processor {
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

        let children = item.children();
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
}
