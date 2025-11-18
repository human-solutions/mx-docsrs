use crate::docrender::{DocSection, RenderedDoc};
use colored::*;
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

/// Render a RenderedDoc to the terminal with formatting
pub fn render_to_terminal(doc: &RenderedDoc) {
    // Header separator
    println!("{}", "─".repeat(60).bright_black());

    // Item type and name
    println!(
        "{}: {}",
        doc.item_type.bright_cyan().bold(),
        doc.name.bright_white().bold()
    );

    println!("{}", "─".repeat(60).bright_black());

    // Signature
    if !doc.signature.is_empty() {
        println!("{}", doc.signature.bright_yellow());
        println!();
    }

    // Metadata
    render_metadata(doc);

    // Documentation
    if let Some(docs) = &doc.docs {
        if !docs.is_empty() {
            render_markdown(docs);
            println!();
        }
    } else {
        println!("{}", "No documentation available.".dimmed());
        println!();
    }

    // Sections (fields, methods, variants, etc.)
    for section in &doc.sections {
        render_section(section);
    }

    // Footer separator
    println!("{}", "─".repeat(60).bright_black());
}

/// Render metadata (deprecation, attributes)
fn render_metadata(doc: &RenderedDoc) {
    if let Some(deprecation) = &doc.metadata.deprecation {
        println!("{} {}", "⚠".yellow().bold(), deprecation.yellow());
        println!();
    }

    if !doc.metadata.attributes.is_empty() {
        for attr in &doc.metadata.attributes {
            println!("{}", attr.bright_black());
        }
        println!();
    }
}

/// Render a documentation section
fn render_section(section: &DocSection) {
    println!("{}", section.title.bright_green().bold());
    println!();

    for item in &section.items {
        println!("  {} {}", "•".bright_blue(), item.signature.white());

        if let Some(docs) = &item.docs
            && !docs.is_empty()
        {
            // Render first line or two of docs for each child item
            let first_line = docs.lines().next().unwrap_or("");
            if !first_line.is_empty() {
                println!("    {}", first_line.dimmed());
            }
        }

        println!();
    }
}

/// Render markdown text to terminal with formatting
fn render_markdown(markdown: &str) {
    let parser = Parser::new(markdown);
    let mut renderer = MarkdownRenderer::new();

    for event in parser {
        renderer.process_event(event);
    }

    renderer.flush();
}

/// Markdown renderer that converts events to styled terminal output
struct MarkdownRenderer {
    current_line: String,
    in_code_block: bool,
    in_emphasis: bool,
    in_strong: bool,
    in_heading: bool,
    heading_level: usize,
    list_depth: usize,
    in_list_item: bool,
}

impl MarkdownRenderer {
    fn new() -> Self {
        Self {
            current_line: String::new(),
            in_code_block: false,
            in_emphasis: false,
            in_strong: false,
            in_heading: false,
            heading_level: 0,
            list_depth: 0,
            in_list_item: false,
        }
    }

    fn process_event(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.start_tag(tag),
            Event::End(tag_end) => self.end_tag(tag_end),
            Event::Text(text) => self.add_text(&text),
            Event::Code(code) => self.add_code(&code),
            Event::SoftBreak => self.add_text(" "),
            Event::HardBreak => self.line_break(),
            Event::Rule => {
                self.flush();
                println!("{}", "─".repeat(40).bright_black());
            }
            _ => {}
        }
    }

    fn start_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Paragraph => {
                // Paragraphs get a line break before
            }
            Tag::Heading { level, .. } => {
                self.in_heading = true;
                self.heading_level = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
            }
            Tag::CodeBlock(_) => {
                self.flush();
                self.in_code_block = true;
            }
            Tag::List(_) => {
                self.flush();
                self.list_depth += 1;
            }
            Tag::Item => {
                self.in_list_item = true;
                let indent = "  ".repeat(self.list_depth.saturating_sub(1));
                self.current_line.push_str(&indent);
                self.current_line.push_str("• ");
            }
            Tag::Emphasis => {
                self.in_emphasis = true;
            }
            Tag::Strong => {
                self.in_strong = true;
            }
            Tag::Link { dest_url, .. } => {
                // For now, just show the link text (no OSC 8 yet)
                // Could enhance later
                let _ = dest_url;
            }
            _ => {}
        }
    }

    fn end_tag(&mut self, tag_end: TagEnd) {
        match tag_end {
            TagEnd::Paragraph => {
                self.flush();
                println!();
            }
            TagEnd::Heading(_) => {
                self.flush_heading();
                self.in_heading = false;
                println!();
            }
            TagEnd::CodeBlock => {
                self.in_code_block = false;
                println!();
            }
            TagEnd::List(_) => {
                self.list_depth = self.list_depth.saturating_sub(1);
            }
            TagEnd::Item => {
                self.flush();
                self.in_list_item = false;
            }
            TagEnd::Emphasis => {
                self.in_emphasis = false;
            }
            TagEnd::Strong => {
                self.in_strong = false;
            }
            _ => {}
        }
    }

    fn add_text(&mut self, text: &str) {
        if self.in_code_block {
            // Code blocks: preserve formatting, add indentation
            for line in text.lines() {
                println!("    {}", line.bright_cyan());
            }
        } else {
            // Regular text: apply styling based on context
            let styled_text = self.style_text(text);
            self.current_line.push_str(&styled_text);
        }
    }

    fn add_code(&mut self, code: &str) {
        let styled = code.bright_magenta().to_string();
        self.current_line.push_str(&styled);
    }

    fn style_text(&self, text: &str) -> String {
        let mut result = text.to_string();

        if self.in_strong && self.in_emphasis {
            result = result.bold().italic().to_string();
        } else if self.in_strong {
            result = result.bold().to_string();
        } else if self.in_emphasis {
            result = result.italic().to_string();
        }

        result
    }

    fn line_break(&mut self) {
        self.flush();
    }

    fn flush(&mut self) {
        if !self.current_line.is_empty() {
            println!("{}", self.current_line);
            self.current_line.clear();
        }
    }

    fn flush_heading(&mut self) {
        if !self.current_line.is_empty() {
            let heading = match self.heading_level {
                1 => self
                    .current_line
                    .bright_white()
                    .bold()
                    .underline()
                    .to_string(),
                2 => self.current_line.bright_white().bold().to_string(),
                _ => self.current_line.bright_green().bold().to_string(),
            };
            println!("{}", heading);
            self.current_line.clear();
        }
    }
}

/// Render a list of search results with fully qualified names
pub fn render_search_results_list(results: &[(&String, &String, &Vec<String>)]) {
    println!(
        "\n{}",
        "Multiple items found. Use the fully qualified name to view a specific item:"
            .bright_cyan()
            .bold()
    );
    println!("{}", "─".repeat(80).bright_black());

    for (item_type, name, module_path) in results.iter() {
        let type_str = format!("{:12}", item_type).bright_blue();

        // Build FQDN by appending name to module path
        // The path now always contains only the parent modules (not the item name)
        let mut fqdn_parts = (*module_path).clone();
        fqdn_parts.push((*name).clone());
        let fqdn = fqdn_parts.join("::");
        let fqdn_str = fqdn.bright_white().bold();

        println!("  {} {}", type_str, fqdn_str);
    }

    println!("{}", "─".repeat(80).bright_black());

    // Build example FQDN the same way
    let mut example_parts = results[0].2.clone();
    example_parts.push(results[0].1.clone());
    let example_fqdn = example_parts.join("::");
    println!(
        "\n{}",
        format!("Example: docsrs tokio {}", example_fqdn).dimmed()
    );
}
