//! Formats markdown documentation for terminal display with ANSI colors.

use std::collections::HashMap;

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use rustdoc_types::{Crate, Id};

use super::link_resolver::resolve_single_link;
use crate::colorizer::Colorizer;
use crate::proc::IntermediatePublicItem;

/// Formats markdown documentation for terminal display.
///
/// Converts markdown syntax to ANSI-formatted terminal output:
/// - Headers become bold
/// - `**bold**` becomes ANSI bold
/// - `*italic*` becomes ANSI italic/dim
/// - `` `code` `` becomes styled inline code
/// - Code blocks are syntax highlighted
/// - Lists use bullet points
/// - Block quotes use `│` prefix
pub fn format_markdown(
    docs: &str,
    item_links: &HashMap<String, Id>,
    krate: &Crate,
    id_to_items: &HashMap<&Id, Vec<&IntermediatePublicItem<'_>>>,
) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(docs, options);
    let mut formatter = MarkdownFormatter::new(item_links, krate, id_to_items);

    for event in parser {
        formatter.process_event(event);
    }

    formatter.finish()
}

struct MarkdownFormatter<'a> {
    output: String,
    colorizer: &'static Colorizer,
    item_links: &'a HashMap<String, Id>,
    krate: &'a Crate,
    id_to_items: &'a HashMap<&'a Id, Vec<&'a IntermediatePublicItem<'a>>>,

    // State tracking
    in_link: bool,
    link_text: String,
    current_dest_url: String,
    in_code_block: bool,
    code_block_lang: String,
    code_block_content: String,
    in_heading: bool,
    heading_text: String,
    in_emphasis: bool,
    emphasis_text: String,
    in_strong: bool,
    strong_text: String,
    in_block_quote: bool,
    block_quote_text: String,
    in_list: bool,
    list_ordered: bool,
    list_index: u64,
}

impl<'a> MarkdownFormatter<'a> {
    fn new(
        item_links: &'a HashMap<String, Id>,
        krate: &'a Crate,
        id_to_items: &'a HashMap<&'a Id, Vec<&'a IntermediatePublicItem<'a>>>,
    ) -> Self {
        Self {
            output: String::new(),
            colorizer: Colorizer::get(),
            item_links,
            krate,
            id_to_items,
            in_link: false,
            link_text: String::new(),
            current_dest_url: String::new(),
            in_code_block: false,
            code_block_lang: String::new(),
            code_block_content: String::new(),
            in_heading: false,
            heading_text: String::new(),
            in_emphasis: false,
            emphasis_text: String::new(),
            in_strong: false,
            strong_text: String::new(),
            in_block_quote: false,
            block_quote_text: String::new(),
            in_list: false,
            list_ordered: false,
            list_index: 1,
        }
    }

    fn process_event(&mut self, event: Event) {
        match event {
            // Links
            Event::Start(Tag::Link { dest_url, .. }) => {
                self.in_link = true;
                self.link_text.clear();
                self.current_dest_url = dest_url.to_string();
            }
            Event::End(TagEnd::Link) => {
                let resolved = resolve_single_link(
                    &self.link_text,
                    &self.current_dest_url,
                    self.item_links,
                    self.krate,
                    self.id_to_items,
                );
                self.push_text(&resolved);
                self.in_link = false;
            }

            // Headings
            Event::Start(Tag::Heading { .. }) => {
                self.in_heading = true;
                self.heading_text.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                let text = std::mem::take(&mut self.heading_text);
                self.output.push_str(&self.colorizer.heading(&text));
                self.output.push_str("\n\n");
                self.in_heading = false;
            }

            // Code blocks
            Event::Start(Tag::CodeBlock(kind)) => {
                self.in_code_block = true;
                self.code_block_lang = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                self.code_block_content.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                let highlighted = self
                    .colorizer
                    .code_block(&self.code_block_content, &self.code_block_lang);
                self.output.push_str(&highlighted);
                self.in_code_block = false;
            }

            // Paragraphs
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {
                if self.in_block_quote {
                    self.block_quote_text.push_str("\n\n");
                } else {
                    self.output.push_str("\n\n");
                }
            }

            // Emphasis (italic)
            Event::Start(Tag::Emphasis) => {
                self.in_emphasis = true;
                self.emphasis_text.clear();
            }
            Event::End(TagEnd::Emphasis) => {
                let text = std::mem::take(&mut self.emphasis_text);
                let styled = self.colorizer.emphasis(&text);
                if self.in_link {
                    self.link_text.push_str(&styled);
                } else {
                    self.push_text(&styled);
                }
                self.in_emphasis = false;
            }

            // Strong (bold)
            Event::Start(Tag::Strong) => {
                self.in_strong = true;
                self.strong_text.clear();
            }
            Event::End(TagEnd::Strong) => {
                let text = std::mem::take(&mut self.strong_text);
                let styled = self.colorizer.strong(&text);
                if self.in_link {
                    self.link_text.push_str(&styled);
                } else {
                    self.push_text(&styled);
                }
                self.in_strong = false;
            }

            // Lists
            Event::Start(Tag::List(first_index)) => {
                self.in_list = true;
                self.list_ordered = first_index.is_some();
                self.list_index = first_index.unwrap_or(1);
            }
            Event::End(TagEnd::List(_)) => {
                self.in_list = false;
                self.output.push('\n');
            }
            Event::Start(Tag::Item) => {
                if self.list_ordered {
                    self.output.push_str(&format!("  {}. ", self.list_index));
                    self.list_index += 1;
                } else {
                    self.output.push_str("  • ");
                }
            }
            Event::End(TagEnd::Item) => {
                self.output.push('\n');
            }

            // Block quotes
            Event::Start(Tag::BlockQuote(_)) => {
                self.in_block_quote = true;
                self.block_quote_text.clear();
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                let text = std::mem::take(&mut self.block_quote_text);
                for line in text.trim_end().lines() {
                    self.output.push_str(&self.colorizer.blockquote_prefix());
                    self.output.push_str(&self.colorizer.blockquote_line(line));
                    self.output.push('\n');
                }
                self.output.push('\n');
                self.in_block_quote = false;
            }

            // Text content
            Event::Text(text) => {
                if self.in_link {
                    self.link_text.push_str(&text);
                } else if self.in_heading {
                    self.heading_text.push_str(&text);
                } else if self.in_emphasis {
                    self.emphasis_text.push_str(&text);
                } else if self.in_strong {
                    self.strong_text.push_str(&text);
                } else if self.in_code_block {
                    // Accumulate code block content for later highlighting
                    self.code_block_content.push_str(&text);
                } else if self.in_block_quote {
                    self.block_quote_text.push_str(&text);
                } else {
                    self.output.push_str(&text);
                }
            }

            // Inline code
            Event::Code(code) => {
                if self.in_link {
                    self.link_text.push_str(&code);
                } else if self.in_heading {
                    self.heading_text.push_str(&code);
                } else {
                    self.push_text(&self.colorizer.inline_code(&code));
                }
            }

            // Line breaks
            Event::SoftBreak => {
                if self.in_block_quote {
                    self.block_quote_text.push('\n');
                } else if self.in_heading {
                    self.heading_text.push(' ');
                } else if self.in_emphasis {
                    self.emphasis_text.push(' ');
                } else if self.in_strong {
                    self.strong_text.push(' ');
                } else if self.in_link {
                    self.link_text.push(' ');
                } else {
                    self.output.push('\n');
                }
            }
            Event::HardBreak => {
                if self.in_block_quote {
                    self.block_quote_text.push_str("\n\n");
                } else {
                    self.output.push_str("\n\n");
                }
            }

            // Strikethrough
            Event::Start(Tag::Strikethrough) => {}
            Event::End(TagEnd::Strikethrough) => {}

            // Ignore other events
            _ => {}
        }
    }

    fn push_text(&mut self, text: &str) {
        if self.in_block_quote {
            self.block_quote_text.push_str(text);
        } else {
            self.output.push_str(text);
        }
    }

    fn finish(self) -> String {
        self.output.trim_end().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format_test(docs: &str) -> String {
        colored::control::set_override(false);
        let result = format_markdown(docs, &HashMap::new(), &empty_crate(), &HashMap::new());
        colored::control::unset_override();
        result
    }

    fn format_test_colored(docs: &str) -> String {
        colored::control::set_override(true);
        let result = format_markdown(docs, &HashMap::new(), &empty_crate(), &HashMap::new());
        colored::control::unset_override();
        result
    }

    fn empty_crate() -> Crate {
        Crate {
            root: Id(0),
            crate_version: None,
            includes_private: false,
            index: HashMap::new(),
            paths: HashMap::new(),
            external_crates: HashMap::new(),
            target: rustdoc_types::Target {
                triple: String::new(),
                target_features: vec![],
            },
            format_version: 0,
        }
    }

    #[test]
    fn test_plain_text() {
        let result = format_test("Hello world");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_paragraphs() {
        let result = format_test("First paragraph.\n\nSecond paragraph.");
        assert_eq!(result, "First paragraph.\n\nSecond paragraph.");
    }

    #[test]
    fn test_heading() {
        let result = format_test("# Hello");
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_inline_code_plain() {
        let result = format_test("Use `foo()` here");
        assert_eq!(result, "Use `foo()` here");
    }

    #[test]
    fn test_inline_code_colored() {
        let result = format_test_colored("Use `foo()` here");
        // Should contain ANSI codes
        assert!(
            result.contains("\x1b["),
            "Expected ANSI codes in: {}",
            result
        );
        assert!(result.contains("foo()"));
    }

    #[test]
    fn test_code_block() {
        let result = format_test("```\nlet x = 1;\n```");
        assert_eq!(result, "    let x = 1;");
    }

    #[test]
    fn test_unordered_list() {
        let result = format_test("- first\n- second");
        assert_eq!(result, "  • first\n  • second");
    }

    #[test]
    fn test_ordered_list() {
        let result = format_test("1. first\n2. second");
        assert_eq!(result, "  1. first\n  2. second");
    }

    #[test]
    fn test_block_quote_plain() {
        let result = format_test("> quoted text");
        assert_eq!(result, "\u{2502} quoted text");
    }

    #[test]
    fn test_bold_plain() {
        let result = format_test("**bold text**");
        assert_eq!(result, "bold text");
    }

    #[test]
    fn test_italic_plain() {
        let result = format_test("*italic text*");
        assert_eq!(result, "italic text");
    }

    #[test]
    fn test_external_link() {
        let result = format_test("[docs](https://docs.rs)");
        assert_eq!(result, "docs (https://docs.rs)");
    }
}
