//! Syntax highlighting for code blocks using syntect.

use std::sync::LazyLock;

use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::{LinesWithEndings, as_24_bit_terminal_escaped};
use terminal_colorsaurus::{QueryOptions, ThemeMode, theme_mode};

/// Global syntax set (loaded once on first use)
static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);

/// Global theme set (loaded once on first use)
static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

/// Detected theme name based on terminal background (dark or light)
static THEME_NAME: LazyLock<&'static str> =
    LazyLock::new(|| match theme_mode(QueryOptions::default()) {
        Ok(ThemeMode::Light) => "InspiredGitHub",
        Ok(ThemeMode::Dark) | Err(_) => "base16-eighties.dark",
    });

/// Highlight a code block for terminal output.
///
/// - For Rust code, processes hidden lines (`# ` prefix)
/// - Applies syntax highlighting based on language
/// - Falls back to plain text for unknown languages
/// - Adds 4-space indentation to each line
pub fn highlight_code_block(code: &str, language: &str, use_colors: bool) -> String {
    // Determine if this is Rust code
    let is_rust = is_rust_language(language);

    // Process hidden lines for Rust
    let processed_code = if is_rust {
        process_rust_hidden_lines(code)
    } else {
        code.to_string()
    };

    if !use_colors {
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

    // Get theme based on detected terminal background (dark/light)
    let theme = &THEME_SET.themes[*THEME_NAME];

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut output = String::new();

    for line in LinesWithEndings::from(&processed_code) {
        match highlighter.highlight_line(line, &SYNTAX_SET) {
            Ok(ranges) => {
                output.push_str("    "); // 4-space indent
                let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                output.push_str(&escaped);
                // Ensure line ends with newline
                if !line.ends_with('\n') {
                    output.push('\n');
                }
            }
            Err(_) => {
                // Fallback on error
                output.push_str("    ");
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

/// Format code without syntax highlighting (plain text)
fn format_plain(code: &str) -> String {
    let mut output = String::new();
    for line in code.lines() {
        output.push_str("    ");
        output.push_str(line);
        output.push('\n');
    }
    output
}

/// Check if the language identifier indicates Rust
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
        assert_eq!(result, "    let x = 1;\n    let y = 2;\n");
    }

    #[test]
    fn test_highlight_no_colors() {
        let code = "fn main() {}";
        let result = highlight_code_block(code, "rust", false);
        assert_eq!(result, "    fn main() {}\n");
    }

    #[test]
    fn test_highlight_with_hidden_lines() {
        let code = "# use std::io;\nfn main() {}";
        let result = highlight_code_block(code, "rust", false);
        assert_eq!(result, "    fn main() {}\n");
    }
}
