use anyhow::Result;
use rustdoc_types::Crate;

use crate::doc::render::RenderingContext;

/// Format child items for a trait (associated types, methods, etc.)
///
/// TODO: Implement trait child formatting
pub(crate) fn format_trait_children(
    _krate: &Crate,
    _trait_: &rustdoc_types::Trait,
    _output: &mut String,
    _use_colors: bool,
    _context: &RenderingContext,
) -> Result<()> {
    // TODO: Implement associated types display
    // TODO: Implement associated constants display
    // TODO: Implement required methods display
    // TODO: Implement provided methods display
    Ok(())
}
