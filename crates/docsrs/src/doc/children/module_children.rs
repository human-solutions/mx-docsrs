use anyhow::Result;
use rustdoc_types::Crate;

use crate::doc::render::RenderingContext;

/// Format child items for a module
///
/// TODO: Implement module child formatting (if needed)
pub(crate) fn format_module_children(
    _krate: &Crate,
    _module: &rustdoc_types::Module,
    _output: &mut String,
    _use_colors: bool,
    _context: &RenderingContext,
) -> Result<()> {
    // TODO: Decide if modules should display child items or just serve as navigation
    Ok(())
}
