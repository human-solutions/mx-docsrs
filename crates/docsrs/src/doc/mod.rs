use anyhow::Result;
use jsondoc::JsonDoc;
use rustdoc_types::Id;

mod children;
mod doc_formatter;
mod link_resolver;
mod public_item;
mod render;

use doc_formatter::format_doc;
use public_item::PublicItem;
use render::RenderingContext;

pub fn signature_for_id(doc: &JsonDoc, id: &Id) -> Result<String> {
    // Find the item with the matching id
    let item = doc
        .items()
        .iter()
        .find(|item| item.id() == *id)
        .ok_or_else(|| anyhow::anyhow!("Item with id {:?} not found", id))?;

    // Create rendering context
    let context = RenderingContext {
        crate_: doc.crate_data(),
        id_to_items: doc.id_to_items(),
    };

    // Convert to PublicItem
    let public_item = PublicItem::from_jsondoc_item(&context, item);

    // Format the documentation
    format_doc(doc.crate_data(), &public_item, &context)
}
