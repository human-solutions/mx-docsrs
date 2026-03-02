mod enum_children;
mod module_children;
mod struct_children;
mod trait_children;

pub(crate) use enum_children::format_enum_children;
pub(crate) use module_children::format_module_children;
pub(crate) use struct_children::format_struct_children;
pub(crate) use trait_children::format_trait_children;

/// Extract the first line of a doc comment, if present.
fn first_doc_line(docs: &Option<String>) -> Option<String> {
    docs.as_ref()
        .and_then(|d| d.lines().next())
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
}

/// Write a section of items with optional doc comments above each signature.
fn write_section(output: &mut String, heading: &str, items: &[(Option<String>, String)]) {
    if items.is_empty() {
        return;
    }
    output.push('\n');
    output.push_str(heading);
    output.push_str(":\n");
    for (doc, signature) in items {
        if let Some(doc_line) = doc {
            output.push_str("  /// ");
            output.push_str(doc_line);
            output.push('\n');
        }
        output.push_str("  ");
        output.push_str(signature);
        output.push('\n');
    }
}
