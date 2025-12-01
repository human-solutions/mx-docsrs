//! Centralized colorization for terminal output.
//!
//! This module provides a unified `Colorizer` that handles all color/styling
//! decisions, extracting colors from syntect themes for consistency between
//! code block syntax highlighting and token-based signature coloring.

use std::sync::LazyLock;

use colored::Colorize;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color as SyntectColor, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{LinesWithEndings, as_24_bit_terminal_escaped};
use terminal_colorsaurus::{QueryOptions, ThemeMode, theme_mode};

use crate::Token;

/// Global syntax set (loaded once on first use)
static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);

/// Global theme set (loaded once on first use)
static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

/// Global colorizer instance
static COLORIZER: LazyLock<Colorizer> = LazyLock::new(Colorizer::new);

/// Color scheme extracted from a syntect theme for token coloring.
#[derive(Debug, Clone)]
#[allow(dead_code)] // foreground kept for potential future use
struct ColorScheme {
    /// Keywords, qualifiers, self, lifetime, kind
    keyword: SyntectColor,
    /// Function names
    function: SyntectColor,
    /// Types, generics, primitives
    type_: SyntectColor,
    /// Identifiers
    identifier: SyntectColor,
    /// Inline code in markdown
    string: SyntectColor,
    /// Default foreground color
    foreground: SyntectColor,
}

impl ColorScheme {
    /// Extract a color scheme from the given syntect theme.
    fn from_theme(theme: &syntect::highlighting::Theme) -> Self {
        let foreground = theme.settings.foreground.unwrap_or(SyntectColor {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        });

        // Extract colors by matching scope patterns in theme items
        let mut keyword = None;
        let mut function = None;
        let mut type_ = None;
        let mut identifier = None;
        let mut string = None;

        for item in &theme.scopes {
            // Convert scope selectors to string for pattern matching
            let scope_str = format!("{:?}", item.scope);

            if let Some(fg) = item.style.foreground {
                // Keywords (check first as it's most specific)
                if keyword.is_none()
                    && (scope_str.contains("keyword")
                        || scope_str.contains("storage.modifier")
                        || scope_str.contains("storage.type.rust"))
                {
                    keyword = Some(fg);
                }

                // Functions
                if function.is_none()
                    && (scope_str.contains("entity.name.function")
                        || scope_str.contains("support.function"))
                {
                    function = Some(fg);
                }

                // Types
                if type_.is_none()
                    && (scope_str.contains("entity.name.type")
                        || scope_str.contains("entity.name.class")
                        || scope_str.contains("support.type")
                        || scope_str.contains("storage.type"))
                {
                    type_ = Some(fg);
                }

                // Identifiers/Variables
                if identifier.is_none() && scope_str.contains("variable") {
                    identifier = Some(fg);
                }

                // Strings (for inline code)
                if string.is_none()
                    && (scope_str.contains("string") || scope_str.contains("constant.character"))
                {
                    string = Some(fg);
                }
            }
        }

        Self {
            keyword: keyword.unwrap_or(foreground),
            function: function.unwrap_or(foreground),
            type_: type_.unwrap_or(foreground),
            identifier: identifier.unwrap_or(foreground),
            string: string.unwrap_or(foreground),
            foreground,
        }
    }
}

/// Centralized colorizer for all terminal output.
///
/// Provides methods for:
/// - Token coloring (signatures, fields, methods)
/// - Markdown element styling (headings, emphasis, code)
/// - Syntax highlighting for code blocks
pub struct Colorizer {
    scheme: ColorScheme,
    theme_name: &'static str,
    is_dark: bool,
}

impl Colorizer {
    /// Create a new colorizer, detecting theme from terminal.
    fn new() -> Self {
        // Detect terminal theme (dark/light)
        // Skip terminal detection in test environments to avoid hangs with cargo-nextest
        // See: https://github.com/bash/terminal-colorsaurus/issues/38
        let (theme_name, is_dark) = if Self::is_test_environment() {
            ("base16-eighties.dark", true)
        } else {
            match theme_mode(QueryOptions::default()) {
                Ok(ThemeMode::Light) => ("InspiredGitHub", false),
                Ok(ThemeMode::Dark) | Err(_) => ("base16-eighties.dark", true),
            }
        };

        let theme = &THEME_SET.themes[theme_name];
        let scheme = ColorScheme::from_theme(theme);

        Self {
            scheme,
            theme_name,
            is_dark,
        }
    }

    /// Check if we're running in a test environment where terminal queries may hang.
    fn is_test_environment() -> bool {
        // NEXTEST is set by cargo-nextest
        // RUST_TEST_THREADS is set by cargo test
        std::env::var("NEXTEST").is_ok() || std::env::var("RUST_TEST_THREADS").is_ok()
    }

    /// Get the global colorizer instance.
    #[inline]
    pub fn get() -> &'static Self {
        &COLORIZER
    }

    /// Check if colors are enabled (respects global override).
    #[inline]
    pub fn is_enabled() -> bool {
        colored::control::SHOULD_COLORIZE.should_colorize()
    }

    // ========== Token Coloring ==========

    /// Colorize a single token.
    fn colorize_token(&self, token: &Token) -> String {
        if !Self::is_enabled() {
            return token.text().to_string();
        }

        match token {
            Token::Symbol(text) | Token::Annotation(text) => text.to_string(),
            Token::Qualifier(text)
            | Token::Kind(text)
            | Token::Self_(text)
            | Token::Lifetime(text)
            | Token::Keyword(text) => self.apply_color(text, self.scheme.keyword),
            Token::Function(text) => self.apply_color(text, self.scheme.function),
            Token::Generic(text) | Token::Primitive(text) | Token::Type(text) => {
                self.apply_color(text, self.scheme.type_)
            }
            Token::Identifier(text) => self.apply_color(text, self.scheme.identifier),
            Token::Whitespace => " ".to_string(),
        }
    }

    /// Colorize a slice of tokens to a string.
    pub fn tokens(&self, tokens: &[Token]) -> String {
        tokens.iter().map(|t| self.colorize_token(t)).collect()
    }

    // ========== Markdown Styling ==========

    /// Style text as a heading with inverse background based on level.
    ///
    /// Uses graduated intensity - h1 has strongest contrast, h4 most subtle.
    /// - Dark theme: light-gray background + black foreground
    /// - Light theme: dark-gray background + white foreground
    pub fn heading(&self, text: &str, level: u32) -> String {
        if !Self::is_enabled() {
            return text.to_string();
        }

        // Get background gray level based on heading level (1-4)
        // Higher level = less contrast
        let (bg_r, bg_g, bg_b, fg_r, fg_g, fg_b) = if self.is_dark {
            // Dark theme: light backgrounds with black text
            match level {
                1 => (220, 220, 220, 0, 0, 0), // h1: very light gray
                2 => (180, 180, 180, 0, 0, 0), // h2: light gray
                3 => (140, 140, 140, 0, 0, 0), // h3: medium gray
                _ => (100, 100, 100, 0, 0, 0), // h4+: darker gray
            }
        } else {
            // Light theme: dark backgrounds with white text
            match level {
                1 => (50, 50, 50, 255, 255, 255),    // h1: very dark gray
                2 => (80, 80, 80, 255, 255, 255),    // h2: dark gray
                3 => (110, 110, 110, 255, 255, 255), // h3: medium gray
                _ => (140, 140, 140, 255, 255, 255), // h4+: lighter gray
            }
        };

        // Add # prefix based on level, with padding
        let prefix = "#".repeat(level as usize);
        let padded = format!(" {} {} ", prefix, text);
        padded
            .bold()
            .truecolor(fg_r, fg_g, fg_b)
            .on_truecolor(bg_r, bg_g, bg_b)
            .to_string()
    }

    /// Style text as emphasis (italic).
    pub fn emphasis(&self, text: &str) -> String {
        if Self::is_enabled() {
            text.italic().to_string()
        } else {
            text.to_string()
        }
    }

    /// Style text as strong (bold).
    pub fn strong(&self, text: &str) -> String {
        if Self::is_enabled() {
            text.bold().to_string()
        } else {
            text.to_string()
        }
    }

    /// Style text as inline code.
    pub fn inline_code(&self, code: &str) -> String {
        if Self::is_enabled() {
            self.apply_color(code, self.scheme.string)
        } else {
            format!("`{}`", code)
        }
    }

    /// Get the blockquote prefix.
    pub fn blockquote_prefix(&self) -> String {
        if Self::is_enabled() {
            "\u{2502} ".dimmed().to_string()
        } else {
            "\u{2502} ".to_string()
        }
    }

    /// Style text as blockquote content.
    pub fn blockquote_line(&self, text: &str) -> String {
        if Self::is_enabled() {
            text.dimmed().to_string()
        } else {
            text.to_string()
        }
    }

    // ========== Syntax Highlighting ==========

    /// Highlight a code block for terminal output.
    ///
    /// - For Rust code, processes hidden lines (`# ` prefix)
    /// - Applies syntax highlighting based on language
    /// - Falls back to plain text for unknown languages
    /// - Adds 4-space indentation to each line
    pub fn code_block(&self, code: &str, language: &str) -> String {
        // Determine if this is Rust code
        let is_rust = is_rust_language(language);

        // Process hidden lines for Rust
        let processed_code = if is_rust {
            process_rust_hidden_lines(code)
        } else {
            code.to_string()
        };

        if !Self::is_enabled() {
            return format_plain(&processed_code);
        }

        // Find syntax definition
        let syntax = SYNTAX_SET
            .find_syntax_by_token(language)
            .or_else(|| {
                if is_rust {
                    SYNTAX_SET.find_syntax_by_extension("rs")
                } else {
                    None
                }
            })
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

        // Get theme
        let theme = &THEME_SET.themes[self.theme_name];

        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut output = String::new();

        for line in LinesWithEndings::from(&processed_code) {
            match highlighter.highlight_line(line, &SYNTAX_SET) {
                Ok(ranges) => {
                    output.push_str("  "); // 2-space indent
                    let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                    output.push_str(&escaped);
                    // Ensure line ends with newline
                    if !line.ends_with('\n') {
                        output.push('\n');
                    }
                }
                Err(_) => {
                    // Fallback on error
                    output.push_str("  ");
                    output.push_str(line);
                    if !line.ends_with('\n') {
                        output.push('\n');
                    }
                }
            }
        }

        // Reset terminal colors at end
        output.push_str("\x1b[0m");
        output
    }

    // ========== Helpers ==========

    /// Apply a syntect color to text using the colored crate.
    fn apply_color(&self, text: &str, color: SyntectColor) -> String {
        text.truecolor(color.r, color.g, color.b).to_string()
    }
}

/// Format code without syntax highlighting (plain text).
fn format_plain(code: &str) -> String {
    let mut output = String::new();
    for line in code.lines() {
        output.push_str("  ");
        output.push_str(line);
        output.push('\n');
    }
    output
}

/// Check if the language identifier indicates Rust.
fn is_rust_language(lang: &str) -> bool {
    // Handle common rustdoc language annotations
    let lang_lower = lang.to_lowercase();
    matches!(
        lang_lower.as_str(),
        "" | "rust" | "rs" | "no_run" | "should_panic" | "ignore" | "compile_fail"
    ) || lang_lower.starts_with("rust,")
        || lang_lower.contains(",no_run")
        || lang_lower.contains(",ignore")
        || lang_lower.contains(",should_panic")
}

/// Process rustdoc hidden lines in Rust code blocks.
///
/// Rustdoc hidden line rules:
/// - `#` alone or `# ` followed by content = hidden
/// - `##` = escape (shows single `#`)
/// - `#!` = NOT hidden (inner attribute like `#![allow(...)]`)
/// - `#[` = NOT hidden (outer attribute like `#[derive(...)]`)
fn process_rust_hidden_lines(code: &str) -> String {
    code.lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();

            // Not a hidden line marker - keep as-is
            if !trimmed.starts_with('#') {
                return Some(line.to_string());
            }

            // Check what follows the #
            let after_hash = &trimmed[1..];

            // Escape sequence: ## becomes #
            if after_hash.starts_with('#') {
                let leading_ws = &line[..line.len() - trimmed.len()];
                return Some(format!("{}{}", leading_ws, after_hash));
            }

            // Attributes are NOT hidden: #! and #[
            if after_hash.starts_with('!') || after_hash.starts_with('[') {
                return Some(line.to_string());
            }

            // Hidden line: # alone, or # followed by space
            // (covers "# code" and just "#")
            if after_hash.is_empty() || after_hash.starts_with(' ') {
                return None;
            }

            // Anything else (e.g., #foo) - keep as-is
            Some(line.to_string())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hidden_line_basic() {
        let code = "# use std::io;\nfn main() {}\n# Ok(())";
        let result = process_rust_hidden_lines(code);
        assert_eq!(result, "fn main() {}");
    }

    #[test]
    fn test_hidden_line_empty() {
        // Lines that are just "#" should be hidden
        let code = "#\n#\nfn main() {}\n#";
        let result = process_rust_hidden_lines(code);
        assert_eq!(result, "fn main() {}");
    }

    #[test]
    fn test_hidden_line_escape() {
        let code = "## This shows as # comment\nfn main() {}";
        let result = process_rust_hidden_lines(code);
        assert_eq!(result, "# This shows as # comment\nfn main() {}");
    }

    #[test]
    fn test_attributes_not_hidden() {
        let code = "#![allow(unused)]\n#[derive(Debug)]\nstruct Foo;";
        let result = process_rust_hidden_lines(code);
        assert_eq!(result, code);
    }

    #[test]
    fn test_is_rust_language() {
        assert!(is_rust_language(""));
        assert!(is_rust_language("rust"));
        assert!(is_rust_language("rs"));
        assert!(is_rust_language("no_run"));
        assert!(is_rust_language("rust,no_run"));
        assert!(is_rust_language("ignore"));
        assert!(!is_rust_language("python"));
        assert!(!is_rust_language("json"));
    }

    #[test]
    fn test_format_plain() {
        let code = "let x = 1;\nlet y = 2;";
        let result = format_plain(code);
        assert_eq!(result, "  let x = 1;\n  let y = 2;\n");
    }

    #[test]
    fn test_colorizer_tokens_no_colors() {
        colored::control::set_override(false);
        let colorizer = Colorizer::get();
        let tokens = vec![
            Token::Keyword("fn".to_string()),
            Token::Whitespace,
            Token::Function("main".to_string()),
            Token::Symbol("()".to_string()),
        ];
        let result = colorizer.tokens(&tokens);
        assert_eq!(result, "fn main()");
        colored::control::unset_override();
    }

    #[test]
    fn test_colorizer_code_block_no_colors() {
        colored::control::set_override(false);
        let colorizer = Colorizer::get();
        let code = "fn main() {}";
        let result = colorizer.code_block(code, "rust");
        assert_eq!(result, "  fn main() {}\n");
        colored::control::unset_override();
    }

    #[test]
    fn test_colorizer_inline_code_no_colors() {
        colored::control::set_override(false);
        let colorizer = Colorizer::get();
        let result = colorizer.inline_code("foo()");
        assert_eq!(result, "`foo()`");
        colored::control::unset_override();
    }

    #[test]
    fn test_colorizer_with_colors() {
        colored::control::set_override(true);
        let colorizer = Colorizer::get();
        let tokens = vec![Token::Keyword("fn".to_string())];
        let result = colorizer.tokens(&tokens);
        // Should contain ANSI escape codes
        assert!(
            result.contains("\x1b["),
            "Expected ANSI codes in: {}",
            result
        );
        colored::control::unset_override();
    }
}
