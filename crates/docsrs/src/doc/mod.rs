use anyhow::Result;
use rustdoc_types::Crate;

use crate::color::Color;

mod crate_wrapper;
mod doc_formatter;
mod impl_kind;
mod intermediate_public_item;
mod item_processor;
mod matcher;
mod nameable_item;
mod output;
mod path_component;
mod public_item;
mod render;
mod tokens;
mod unprocessed_item;

use doc_formatter::format_doc;
use item_processor::public_api_in_crate;
use matcher::match_items;
use public_item::PublicItem;
use tokens::{tokens_to_colored_string, tokens_to_string};

pub fn extract_list(krate: &Crate, color: Color, pattern: Option<&str>) -> Result<String> {
    let mut items = public_api_in_crate(krate);
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

    Ok(output)
}
