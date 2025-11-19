use anyhow::Result;
use rustdoc_types::Crate;

mod crate_wrapper;
mod intermediate_public_item;
mod item_processor;
mod nameable_item;
mod path_component;
mod public_item;
mod render;
mod tokens;

use item_processor::public_api_in_crate;
use public_item::PublicItem;

pub fn extract_list(krate: &Crate) -> Result<String> {
    let mut items = public_api_in_crate(krate);
    items.sort_by(PublicItem::grouping_cmp);

    let mut output = items
        .iter()
        .map(|item| item.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    if !output.is_empty() {
        output.push('\n');
    }

    Ok(output)
}
