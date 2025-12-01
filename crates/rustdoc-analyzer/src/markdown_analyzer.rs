//! Analyze markdown features using pulldown-cmark

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Statistics for markdown features in a document
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownStats {
    // Tag counts
    pub headings: usize,
    pub heading_levels: HashMap<String, usize>,
    pub paragraphs: usize,
    pub code_blocks: usize,
    pub code_block_languages: HashMap<String, usize>,
    pub inline_code: usize,
    pub links: usize,
    pub link_types: LinkTypeStats,
    pub images: usize,
    pub lists: usize,
    pub ordered_lists: usize,
    pub unordered_lists: usize,
    pub nested_lists: usize,
    pub list_items: usize,
    pub block_quotes: usize,
    pub emphasis: usize,
    pub strong: usize,
    pub strikethrough: usize,
    pub tables: usize,
    pub table_rows: usize,
    pub table_cells: usize,
    pub footnote_definitions: usize,
    pub footnote_references: usize,
    pub html_blocks: usize,
    pub inline_html: usize,
    pub hard_breaks: usize,
    pub soft_breaks: usize,
    pub horizontal_rules: usize,
    pub task_list_markers: usize,

    // Derived metrics
    pub total_events: usize,
    pub nesting_depth_max: usize,
}

impl MarkdownStats {
    /// Merge another MarkdownStats into this one
    pub fn merge(&mut self, other: &MarkdownStats) {
        self.headings += other.headings;
        for (level, count) in &other.heading_levels {
            *self.heading_levels.entry(level.clone()).or_insert(0) += count;
        }
        self.paragraphs += other.paragraphs;
        self.code_blocks += other.code_blocks;
        for (lang, count) in &other.code_block_languages {
            *self.code_block_languages.entry(lang.clone()).or_insert(0) += count;
        }
        self.inline_code += other.inline_code;
        self.links += other.links;
        self.link_types.merge(&other.link_types);
        self.images += other.images;
        self.lists += other.lists;
        self.ordered_lists += other.ordered_lists;
        self.unordered_lists += other.unordered_lists;
        self.nested_lists += other.nested_lists;
        self.list_items += other.list_items;
        self.block_quotes += other.block_quotes;
        self.emphasis += other.emphasis;
        self.strong += other.strong;
        self.strikethrough += other.strikethrough;
        self.tables += other.tables;
        self.table_rows += other.table_rows;
        self.table_cells += other.table_cells;
        self.footnote_definitions += other.footnote_definitions;
        self.footnote_references += other.footnote_references;
        self.html_blocks += other.html_blocks;
        self.inline_html += other.inline_html;
        self.hard_breaks += other.hard_breaks;
        self.soft_breaks += other.soft_breaks;
        self.horizontal_rules += other.horizontal_rules;
        self.task_list_markers += other.task_list_markers;
        self.total_events += other.total_events;
        self.nesting_depth_max = self.nesting_depth_max.max(other.nesting_depth_max);
    }

    /// Check if this document has any "interesting" features beyond plain text
    pub fn has_interesting_features(&self) -> bool {
        self.code_blocks > 0
            || self.tables > 0
            || self.lists > 0
            || self.block_quotes > 0
            || self.links > 0
            || self.images > 0
    }
}

/// Statistics for different link types
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct LinkTypeStats {
    pub external_http: usize,
    pub internal_doc: usize,
    pub intra_doc: usize,
    pub anchor: usize,
}

impl LinkTypeStats {
    pub fn merge(&mut self, other: &LinkTypeStats) {
        self.external_http += other.external_http;
        self.internal_doc += other.internal_doc;
        self.intra_doc += other.intra_doc;
        self.anchor += other.anchor;
    }
}

/// Analyze markdown and return statistics
pub fn analyze_markdown(markdown: &str) -> MarkdownStats {
    let options = Options::all();
    let parser = Parser::new_ext(markdown, options);

    let mut stats = MarkdownStats::default();
    let mut depth = 0usize;
    let mut list_depth = 0usize;

    for event in parser {
        stats.total_events += 1;

        match event {
            Event::Start(ref tag) => {
                depth += 1;
                stats.nesting_depth_max = stats.nesting_depth_max.max(depth);

                // Track list nesting
                if matches!(tag, Tag::List(_)) {
                    if list_depth > 0 {
                        stats.nested_lists += 1;
                    }
                    list_depth += 1;
                }

                count_start_tag(&mut stats, tag);
            }
            Event::End(ref tag_end) => {
                depth = depth.saturating_sub(1);

                // Track list nesting
                if matches!(tag_end, pulldown_cmark::TagEnd::List(_)) {
                    list_depth = list_depth.saturating_sub(1);
                }
            }
            Event::Text(_) => {}
            Event::Code(_) => {
                stats.inline_code += 1;
            }
            Event::Html(_) => {
                stats.html_blocks += 1;
            }
            Event::InlineHtml(_) => {
                stats.inline_html += 1;
            }
            Event::SoftBreak => {
                stats.soft_breaks += 1;
            }
            Event::HardBreak => {
                stats.hard_breaks += 1;
            }
            Event::Rule => {
                stats.horizontal_rules += 1;
            }
            Event::FootnoteReference(_) => {
                stats.footnote_references += 1;
            }
            Event::TaskListMarker(_) => {
                stats.task_list_markers += 1;
            }
            Event::InlineMath(_) | Event::DisplayMath(_) => {
                // Math is rare in rustdoc, just count as events
            }
        }
    }

    stats
}

fn count_start_tag(stats: &mut MarkdownStats, tag: &Tag) {
    match tag {
        Tag::Heading { level, .. } => {
            stats.headings += 1;
            let level_str = format!("h{}", *level as u8);
            *stats.heading_levels.entry(level_str).or_insert(0) += 1;
        }
        Tag::Paragraph => {
            stats.paragraphs += 1;
        }
        Tag::CodeBlock(kind) => {
            stats.code_blocks += 1;
            let lang = match kind {
                CodeBlockKind::Fenced(lang) => normalize_rust_language(lang),
                CodeBlockKind::Indented => "indented".to_string(),
            };
            *stats.code_block_languages.entry(lang).or_insert(0) += 1;
        }
        Tag::List(first_item) => {
            stats.lists += 1;
            if first_item.is_some() {
                stats.ordered_lists += 1;
            } else {
                stats.unordered_lists += 1;
            }
        }
        Tag::Item => {
            stats.list_items += 1;
        }
        Tag::BlockQuote(_) => {
            stats.block_quotes += 1;
        }
        Tag::Emphasis => {
            stats.emphasis += 1;
        }
        Tag::Strong => {
            stats.strong += 1;
        }
        Tag::Strikethrough => {
            stats.strikethrough += 1;
        }
        Tag::Link { dest_url, .. } => {
            stats.links += 1;
            classify_link(&mut stats.link_types, dest_url);
        }
        Tag::Image { .. } => {
            stats.images += 1;
        }
        Tag::Table(_) => {
            stats.tables += 1;
        }
        Tag::TableHead | Tag::TableRow => {
            stats.table_rows += 1;
        }
        Tag::TableCell => {
            stats.table_cells += 1;
        }
        Tag::FootnoteDefinition(_) => {
            stats.footnote_definitions += 1;
        }
        Tag::HtmlBlock
        | Tag::MetadataBlock(_)
        | Tag::DefinitionList
        | Tag::DefinitionListTitle
        | Tag::DefinitionListDefinition
        | Tag::Superscript
        | Tag::Subscript => {}
    }
}

/// Normalize Rust code block language identifiers
fn normalize_rust_language(lang: &str) -> String {
    let lang_str = lang.to_string();

    // Empty or explicit rust variants
    if lang_str.is_empty() || lang_str == "rust" || lang_str == "rs" {
        return "rust".to_string();
    }

    // Rustdoc attributes that indicate Rust code
    let rust_attrs = ["no_run", "ignore", "should_panic", "compile_fail"];

    // Check if it starts with rust, or contains rust attributes
    if lang_str.starts_with("rust,") || lang_str.contains(",rust") {
        return "rust".to_string();
    }

    // Check for bare rustdoc attributes
    for attr in rust_attrs {
        if lang_str == attr || lang_str.starts_with(&format!("{},", attr)) {
            return "rust".to_string();
        }
    }

    lang_str
}

fn classify_link(link_stats: &mut LinkTypeStats, url: &str) {
    if url.starts_with("http://") || url.starts_with("https://") {
        link_stats.external_http += 1;
    } else if url.starts_with('#') {
        link_stats.anchor += 1;
    } else if url.contains(".html") || url.starts_with("../") || url.starts_with("./") {
        link_stats.internal_doc += 1;
    } else {
        link_stats.intra_doc += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_plain_text() {
        let stats = analyze_markdown("Hello world");
        assert_eq!(stats.paragraphs, 1);
        assert_eq!(stats.code_blocks, 0);
    }

    #[test]
    fn test_analyze_code_block() {
        let md = "```rust\nfn main() {}\n```";
        let stats = analyze_markdown(md);
        assert_eq!(stats.code_blocks, 1);
        assert_eq!(stats.code_block_languages.get("rust"), Some(&1));
    }

    #[test]
    fn test_analyze_code_block_no_run() {
        let md = "```no_run\nfn main() {}\n```";
        let stats = analyze_markdown(md);
        assert_eq!(stats.code_blocks, 1);
        assert_eq!(stats.code_block_languages.get("rust"), Some(&1));
    }

    #[test]
    fn test_analyze_links() {
        // Note: [`Foo`] style requires a reference definition or rustdoc processing
        let md = "[external](https://example.com) and [Foo](Foo) and [section](#anchor)";
        let stats = analyze_markdown(md);
        assert_eq!(stats.links, 3);
        assert_eq!(stats.link_types.external_http, 1);
        assert_eq!(stats.link_types.anchor, 1);
        assert_eq!(stats.link_types.intra_doc, 1);
    }

    #[test]
    fn test_analyze_list() {
        let md = "- item 1\n- item 2\n- item 3";
        let stats = analyze_markdown(md);
        assert_eq!(stats.lists, 1);
        assert_eq!(stats.unordered_lists, 1);
        assert_eq!(stats.list_items, 3);
        assert_eq!(stats.nested_lists, 0);
    }

    #[test]
    fn test_analyze_nested_list() {
        let md = "- item 1\n  - nested 1\n  - nested 2\n- item 2";
        let stats = analyze_markdown(md);
        assert_eq!(stats.lists, 2); // outer + inner
        assert_eq!(stats.nested_lists, 1); // 1 nested list
        assert_eq!(stats.list_items, 4);
    }

    #[test]
    fn test_analyze_table() {
        let md = "| a | b |\n|---|---|\n| 1 | 2 |";
        let stats = analyze_markdown(md);
        assert_eq!(stats.tables, 1);
        assert!(stats.table_rows > 0);
        assert!(stats.table_cells > 0);
    }

    #[test]
    fn test_normalize_rust_language() {
        assert_eq!(normalize_rust_language(""), "rust");
        assert_eq!(normalize_rust_language("rust"), "rust");
        assert_eq!(normalize_rust_language("rs"), "rust");
        assert_eq!(normalize_rust_language("no_run"), "rust");
        assert_eq!(normalize_rust_language("rust,no_run"), "rust");
        assert_eq!(normalize_rust_language("ignore"), "rust");
        assert_eq!(normalize_rust_language("json"), "json");
        assert_eq!(normalize_rust_language("bash"), "bash");
    }
}
