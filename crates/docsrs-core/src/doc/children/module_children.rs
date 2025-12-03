use anyhow::Result;
use rustdoc_fmt::Colorizer;
use rustdoc_types::Crate;

use crate::doc::render::RenderingContext;
use crate::list::ListItem;

/// Format child items for a module
pub(crate) fn format_module_children(
    _krate: &Crate,
    module: &rustdoc_types::Module,
    output: &mut String,
    context: &RenderingContext,
) -> Result<()> {
    let colorizer = Colorizer::get();
    let mut items: Vec<ListItem> = Vec::new();

    for item_id in &module.items {
        // Look up the JsonDocItem via id_to_items
        if let Some(jsondoc_items) = context.id_to_items.get(item_id) {
            // Get the first (best) item for this ID
            if let Some(jsondoc_item) = jsondoc_items.first() {
                // Convert to ListItem (handles visibility filtering internally)
                if let Some(list_item) = ListItem::from_jsondoc_item(jsondoc_item) {
                    items.push(list_item);
                }
            }
        }
    }

    // Sort items by path for consistent output
    items.sort_by(|a, b| a.path.cmp(&b.path));

    // Output all items in list format
    if !items.is_empty() {
        output.push('\n');
        for item in items {
            output.push_str(&colorizer.tokens(&item.as_output().into_tokens()));
            output.push('\n');
        }
    }

    Ok(())
}
