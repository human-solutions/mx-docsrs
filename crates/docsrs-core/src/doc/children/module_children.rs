use anyhow::Result;
use rustdoc_fmt::Colorizer;
use rustdoc_types::{Crate, Item, ItemEnum};

use crate::doc::render::RenderingContext;
use crate::list::ListItem;

/// Format child items for a module
pub(crate) fn format_module_children(
    krate: &Crate,
    module: &rustdoc_types::Module,
    output: &mut String,
    context: &RenderingContext,
) -> Result<()> {
    let colorizer = Colorizer::get();
    let mut items: Vec<ListItem> = Vec::new();

    for item_id in &module.items {
        // Resolve Use items to their targets, since id_to_items
        // is keyed by the target's ID (Use items are inlined during processing)
        let lookup_id = match krate.index.get(item_id) {
            Some(Item {
                inner: ItemEnum::Use(use_),
                ..
            }) => use_.id.as_ref().unwrap_or(item_id),
            _ => item_id,
        };

        if let Some(jsondoc_items) = context.id_to_items.get(lookup_id)
            && let Some(jsondoc_item) = jsondoc_items.first()
            && let Some(list_item) = ListItem::from_jsondoc_item(jsondoc_item)
        {
            items.push(list_item);
        }
    }

    // Sort items by path for consistent output
    items.sort_by(|a, b| a.path.cmp(&b.path));

    // Output all items using module-relative rendering: "pub TYPE Name"
    if !items.is_empty() {
        output.push('\n');
        for item in items {
            output.push_str(&colorizer.tokens(&item.as_module_child().into_tokens()));
            output.push('\n');
        }
    }

    Ok(())
}
