use anyhow::Result;
use rustdoc_types::Crate;

use super::public_item::PublicItem;
use crate::{
    color::Color,
    fmt::{tokens_to_colored_string, tokens_to_string},
};

/// Format documentation for a single PublicItem
pub fn format_doc(krate: &Crate, item: &PublicItem, color: Color) -> Result<String> {
    let use_colors = color.is_active();
    let mut output = String::new();

    // Display the item signature
    let signature = if use_colors {
        tokens_to_colored_string(&item.tokens)
    } else {
        tokens_to_string(&item.tokens)
    };

    output.push_str(&signature);
    output.push('\n');

    // Try to get the full Item from the crate index to access documentation
    if let Some(full_item) = krate.index.get(&item._id)
        && let Some(docs) = &full_item.docs
    {
        output.push('\n');
        output.push_str(docs);
        output.push('\n');
    }

    Ok(output)
}
