use anyhow::Result;
use rustdoc_types::Crate;

use crate::{
    color::Color,
    doc::public_item::public_api_in_crate,
    fmt::{tokens_to_colored_string, tokens_to_string},
    proc::ItemProcessor,
};

mod doc_formatter;
pub(crate) mod impl_kind;
mod matcher;
mod public_item;
mod render;

use doc_formatter::format_doc;
use matcher::match_items;
use public_item::PublicItem;

pub fn signatures(krate: &Crate, color: Color, pattern: Option<&str>) -> Result<String> {
    let item_processor = ItemProcessor::process(krate);

    let mut items = public_api_in_crate(krate, &item_processor);

    items.sort_by(PublicItem::grouping_cmp);

    // Apply pattern matching if provided
    if let Some(pattern) = pattern {
        items = match_items(items, pattern);
    }

    // If exactly one match, show documentation instead of list
    if items.len() == 1 {
        return format_doc(krate, &items[0], color);
    }

    let use_colors = color.is_active();

    // Override the colored crate's auto-detection to respect our color setting
    match color {
        Color::Always => colored::control::set_override(true),
        Color::Never => colored::control::set_override(false),
        Color::Auto => {
            // Let colored crate auto-detect
            colored::control::unset_override();
        }
    }

    let mut output = items
        .iter()
        .map(|item| {
            if use_colors {
                tokens_to_colored_string(&item.tokens)
            } else {
                tokens_to_string(&item.tokens)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    if !output.is_empty() {
        output.push('\n');
    }

    // Ok(output)
    Ok(String::new())
}
