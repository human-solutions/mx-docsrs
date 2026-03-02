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

/// Write items inside a `{ }` body block with given trailing punctuation.
///
/// Each item is indented 4 spaces. If an item has a doc comment, it's written
/// as `/// doc` above the item line.
fn write_body_block(output: &mut String, items: &[(Option<String>, String)], trailing: &str) {
    if items.is_empty() {
        return;
    }
    output.push_str(" {\n");
    for (doc, signature) in items {
        if let Some(doc_line) = doc {
            output.push_str("    /// ");
            output.push_str(doc_line);
            output.push('\n');
        }
        output.push_str("    ");
        output.push_str(signature);
        output.push_str(trailing);
        output.push('\n');
    }
    output.push('}');
}

/// Format a block comment section header: `/* ======== Heading ======== */`
fn format_block_header(heading: &str) -> String {
    format!("/* ======== {heading} ======== */")
}

/// Write a section header followed by items with `///` doc comments.
fn write_comment_section(output: &mut String, heading: &str, items: &[(Option<String>, String)]) {
    if items.is_empty() {
        return;
    }
    output.push('\n');
    output.push_str(&format_block_header(heading));
    output.push('\n');
    for (doc, signature) in items {
        if let Some(doc_line) = doc {
            output.push_str("/// ");
            output.push_str(doc_line);
            output.push('\n');
        }
        output.push_str(signature);
        output.push('\n');
    }
}

/// Write a trait implementations section with `impl ... { .. }` lines.
fn write_trait_impls(output: &mut String, impls: &[String]) {
    if impls.is_empty() {
        return;
    }
    output.push('\n');
    output.push_str(&format_block_header("Trait Implementations"));
    output.push('\n');
    for trait_impl in impls {
        output.push_str(trait_impl);
        output.push_str(" { .. }\n");
    }
}
