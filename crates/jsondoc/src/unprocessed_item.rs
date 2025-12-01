use rustdoc_types::{Id, Item, Type};

use crate::jsondoc_item::JsonDocItem;
use crate::nameable_item::NameableItem;
use crate::path_component::PathComponent;

/// Items in rustdoc JSON reference each other by Id. The processor
/// essentially takes one Id at a time and figure out what to do with it. Once
/// complete, the item is ready to be listed as part of the public API, and
/// optionally can also be used as part of a path to another (child) item.
///
/// This struct contains a (processed) path to an item that is about to be
/// processed further, and the Id of that item.
#[derive(Debug)]
pub(crate) struct UnprocessedItem<'c> {
    /// The path to the item to process.
    pub(crate) parent_path: Vec<PathComponent<'c>>,

    /// The Id of the item's logical parent (if any).
    pub(crate) parent_id: Option<Id>,

    /// The Id of the item to process.
    pub(crate) id: Id,
}

impl<'c> UnprocessedItem<'c> {
    /// Turns an [`UnprocessedItem`] into a finished [`JsonDocItem`].
    pub(crate) fn finish(
        mut self,
        item: &'c Item,
        overridden_name: Option<String>,
        type_: Option<&'c Type>,
    ) -> JsonDocItem<'c> {
        let mut path = self.parent_path.split_off(0);

        path.push(PathComponent {
            item: NameableItem {
                item,
                overridden_name,
            },
            type_,
            hide: false,
        });

        JsonDocItem::new(path, self.parent_id, item.id)
    }
}
