use anyhow::Result;
use rustdoc_types::{Crate, Id};

use crate::{color::Color, proc::ItemProcessor};

mod children;
mod doc_formatter;
pub(crate) mod impl_kind;
mod link_resolver;
mod markdown_formatter;
mod public_item;
mod render;
mod syntax_highlighter;

use doc_formatter::format_doc;
use public_item::PublicItem;
use render::RenderingContext;

pub fn signature_for_id(
    krate: &Crate,
    item_processor: &ItemProcessor,
    id: &Id,
    color: Color,
) -> Result<String> {
    // Find the intermediate item with the matching id
    let intermediate_item = item_processor
        .output
        .iter()
        .find(|item| item.id() == *id)
        .ok_or_else(|| anyhow::anyhow!("Item with id {:?} not found", id))?;

    // Create rendering context
    let context = RenderingContext {
        crate_: krate,
        id_to_items: item_processor.id_to_items(),
    };

    // Convert to PublicItem
    let public_item = PublicItem::from_intermediate_public_item(&context, intermediate_item);

    // Format the documentation
    format_doc(krate, &public_item, color, &context)
}
