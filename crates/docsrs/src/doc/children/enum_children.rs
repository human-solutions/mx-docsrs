use anyhow::Result;
use rustdoc_types::Crate;

use crate::doc::render::RenderingContext;

/// Format child items for an enum (variants, methods and trait implementations)
///
/// TODO: Implement enum child formatting
pub(crate) fn format_enum_children(
    _krate: &Crate,
    _enum_: &rustdoc_types::Enum,
    _output: &mut String,
    _use_colors: bool,
    _context: &RenderingContext,
) -> Result<()> {
    // TODO: Implement enum variants display
    // TODO: Implement enum methods display
    // TODO: Implement enum trait implementations display
    Ok(())
}
