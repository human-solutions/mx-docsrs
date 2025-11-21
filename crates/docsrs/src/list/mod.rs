use crate::{list::list_item::ListItem, proc::ItemProcessor};

mod list_item;

/// Extract public API from a crate.
pub(crate) fn list_items<'c>(item_processor: &ItemProcessor<'c>) -> Vec<ListItem<'c>> {
    item_processor
        .output
        .iter()
        .filter_map(ListItem::from_intermediate)
        .collect::<Vec<_>>()
}
