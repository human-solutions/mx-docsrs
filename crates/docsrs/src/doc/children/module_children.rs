use anyhow::Result;
use rustdoc_types::{Crate, ItemEnum, Visibility};

use crate::doc::render::RenderingContext;

/// Format child items for a module
pub(crate) fn format_module_children(
    krate: &Crate,
    module: &rustdoc_types::Module,
    output: &mut String,
    _context: &RenderingContext,
) -> Result<()> {
    let mut modules = Vec::new();
    let mut structs = Vec::new();
    let mut enums = Vec::new();
    let mut traits = Vec::new();
    let mut functions = Vec::new();
    let mut type_aliases = Vec::new();
    let mut constants = Vec::new();
    let mut statics = Vec::new();

    // Process module items
    for item_id in &module.items {
        if let Some(item) = krate.index.get(item_id) {
            // Only include public items
            if !matches!(item.visibility, Visibility::Public) {
                continue;
            }

            let name = item.name.as_deref().unwrap_or("unknown").to_string();

            match &item.inner {
                ItemEnum::Module(_) => modules.push(name),
                ItemEnum::Struct(_) => structs.push(name),
                ItemEnum::Enum(_) => enums.push(name),
                ItemEnum::Trait(_) => traits.push(name),
                ItemEnum::Function(_) => functions.push(name),
                ItemEnum::TypeAlias(_) => type_aliases.push(name),
                ItemEnum::Constant { .. } => constants.push(name),
                ItemEnum::Static(_) => statics.push(name),
                _ => {
                    // Other item types are not displayed for modules
                }
            }
        }
    }

    // Output Modules section
    if !modules.is_empty() {
        output.push('\n');
        output.push_str("Modules:\n");
        for module in modules {
            output.push_str("  ");
            output.push_str(&module);
            output.push('\n');
        }
    }

    // Output Structs section
    if !structs.is_empty() {
        output.push('\n');
        output.push_str("Structs:\n");
        for struct_ in structs {
            output.push_str("  ");
            output.push_str(&struct_);
            output.push('\n');
        }
    }

    // Output Enums section
    if !enums.is_empty() {
        output.push('\n');
        output.push_str("Enums:\n");
        for enum_ in enums {
            output.push_str("  ");
            output.push_str(&enum_);
            output.push('\n');
        }
    }

    // Output Traits section
    if !traits.is_empty() {
        output.push('\n');
        output.push_str("Traits:\n");
        for trait_ in traits {
            output.push_str("  ");
            output.push_str(&trait_);
            output.push('\n');
        }
    }

    // Output Functions section
    if !functions.is_empty() {
        output.push('\n');
        output.push_str("Functions:\n");
        for function in functions {
            output.push_str("  ");
            output.push_str(&function);
            output.push('\n');
        }
    }

    // Output Type Aliases section
    if !type_aliases.is_empty() {
        output.push('\n');
        output.push_str("Type Aliases:\n");
        for type_alias in type_aliases {
            output.push_str("  ");
            output.push_str(&type_alias);
            output.push('\n');
        }
    }

    // Output Constants section
    if !constants.is_empty() {
        output.push('\n');
        output.push_str("Constants:\n");
        for constant in constants {
            output.push_str("  ");
            output.push_str(&constant);
            output.push('\n');
        }
    }

    // Output Statics section
    if !statics.is_empty() {
        output.push('\n');
        output.push_str("Statics:\n");
        for static_ in statics {
            output.push_str("  ");
            output.push_str(&static_);
            output.push('\n');
        }
    }

    Ok(())
}
