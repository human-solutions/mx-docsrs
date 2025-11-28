pub use crate::list::list_item::ListItem;
use jsondoc::JsonDoc;

mod list_item;

/// Extract public API from a crate.
pub(crate) fn list_items(doc: &JsonDoc) -> Vec<ListItem> {
    doc.items()
        .iter()
        .filter_map(ListItem::from_jsondoc_item)
        .collect()
}
