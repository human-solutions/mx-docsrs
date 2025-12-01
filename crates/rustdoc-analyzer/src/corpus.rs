//! Test corpus extraction for rustdoc-fmt testing

use crate::doc_extractor::DocEntry;
use crate::markdown_analyzer::MarkdownStats;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Category for snippet organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnippetCategory {
    CodeBlock,
    Link,
    List,
    NestedList,
    Table,
    BlockQuote,
    Heading,
    Complex,
}

impl SnippetCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            SnippetCategory::CodeBlock => "code_blocks",
            SnippetCategory::Link => "links",
            SnippetCategory::List => "lists",
            SnippetCategory::NestedList => "nested_lists",
            SnippetCategory::Table => "tables",
            SnippetCategory::BlockQuote => "block_quotes",
            SnippetCategory::Heading => "headings",
            SnippetCategory::Complex => "complex",
        }
    }

    pub fn all() -> &'static [SnippetCategory] {
        &[
            SnippetCategory::CodeBlock,
            SnippetCategory::Link,
            SnippetCategory::List,
            SnippetCategory::NestedList,
            SnippetCategory::Table,
            SnippetCategory::BlockQuote,
            SnippetCategory::Heading,
            SnippetCategory::Complex,
        ]
    }
}

/// A selected snippet for the test corpus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusSnippet {
    pub id: String,
    pub source_crate: String,
    pub source_path: String,
    pub markdown: String,
    pub features: Vec<String>,
    pub category: SnippetCategory,
    pub char_count: usize,
}

/// The complete test corpus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCorpus {
    pub version: String,
    pub generated_at: String,
    pub snippet_count: usize,
    pub snippets: Vec<CorpusSnippet>,
    pub by_feature: HashMap<String, Vec<String>>,
    pub by_category: HashMap<String, Vec<String>>,
}

/// Configuration for snippet selection
pub struct SnippetSelector {
    pub max_length: usize,
    pub target_per_category: usize,
}

impl Default for SnippetSelector {
    fn default() -> Self {
        Self {
            max_length: 2000,
            target_per_category: 5,
        }
    }
}

impl SnippetSelector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Select interesting snippets from analyzed docs
    pub fn select(&self, entries: &[(DocEntry, MarkdownStats)]) -> TestCorpus {
        let mut selected: HashMap<SnippetCategory, Vec<CorpusSnippet>> = HashMap::new();

        // Initialize all categories
        for cat in SnippetCategory::all() {
            selected.insert(*cat, Vec::new());
        }

        for (entry, stats) in entries {
            // Skip snippets that are too long
            if entry.doc_string.len() > self.max_length {
                continue;
            }

            // Skip plain text with no interesting features
            if !stats.has_interesting_features() {
                continue;
            }

            let features = self.detect_features(stats);
            if features.is_empty() {
                continue;
            }

            let category = self.categorize(&features, stats);
            let snippets = selected.entry(category).or_default();

            // Only add if we haven't reached target for this category
            if snippets.len() < self.target_per_category {
                let snippet = CorpusSnippet {
                    id: make_snippet_id(&entry.crate_name, &entry.item_path),
                    source_crate: entry.crate_name.clone(),
                    source_path: entry.item_path.clone(),
                    markdown: entry.doc_string.clone(),
                    features: features.clone(),
                    category,
                    char_count: entry.doc_string.len(),
                };
                snippets.push(snippet);
            }
        }

        // Build the corpus
        let all_snippets: Vec<CorpusSnippet> = selected.into_values().flatten().collect();

        let mut by_feature: HashMap<String, Vec<String>> = HashMap::new();
        let mut by_category: HashMap<String, Vec<String>> = HashMap::new();

        for snippet in &all_snippets {
            for feature in &snippet.features {
                by_feature
                    .entry(feature.clone())
                    .or_default()
                    .push(snippet.id.clone());
            }
            by_category
                .entry(snippet.category.as_str().to_string())
                .or_default()
                .push(snippet.id.clone());
        }

        TestCorpus {
            version: "1.0".to_string(),
            generated_at: crate::stats::chrono_lite_now(),
            snippet_count: all_snippets.len(),
            snippets: all_snippets,
            by_feature,
            by_category,
        }
    }

    fn detect_features(&self, stats: &MarkdownStats) -> Vec<String> {
        let mut features = Vec::new();

        if stats.code_blocks > 0 {
            features.push("code_block".into());
        }
        if stats.inline_code > 0 {
            features.push("inline_code".into());
        }
        if stats.links > 0 {
            features.push("link".into());
        }
        if stats.lists > 0 {
            features.push("list".into());
        }
        if stats.nested_lists > 0 {
            features.push("nested_list".into());
        }
        if stats.block_quotes > 0 {
            features.push("block_quote".into());
        }
        if stats.headings > 0 {
            features.push("heading".into());
        }
        if stats.emphasis > 0 {
            features.push("emphasis".into());
        }
        if stats.strong > 0 {
            features.push("strong".into());
        }
        if stats.tables > 0 {
            features.push("table".into());
        }
        if stats.strikethrough > 0 {
            features.push("strikethrough".into());
        }
        if stats.task_list_markers > 0 {
            features.push("task_list".into());
        }
        if stats.footnote_references > 0 {
            features.push("footnote".into());
        }
        if stats.images > 0 {
            features.push("image".into());
        }
        if stats.nesting_depth_max > 3 {
            features.push("deep_nesting".into());
        }

        features
    }

    fn categorize(&self, features: &[String], stats: &MarkdownStats) -> SnippetCategory {
        // Prioritize rare/interesting features first
        if stats.tables > 0 {
            return SnippetCategory::Table;
        }
        if stats.nested_lists > 0 {
            return SnippetCategory::NestedList;
        }

        // Complex if multiple interesting features
        let interesting_count = features
            .iter()
            .filter(|f| matches!(f.as_str(), "code_block" | "list" | "block_quote"))
            .count();

        if interesting_count >= 2 || stats.nesting_depth_max > 3 {
            return SnippetCategory::Complex;
        }

        // Primary category by feature priority
        if stats.code_blocks > 0 {
            return SnippetCategory::CodeBlock;
        }
        if stats.lists > 0 {
            return SnippetCategory::List;
        }
        if stats.block_quotes > 0 {
            return SnippetCategory::BlockQuote;
        }
        if stats.links > 0 {
            return SnippetCategory::Link;
        }
        if stats.headings > 0 {
            return SnippetCategory::Heading;
        }

        SnippetCategory::Complex
    }
}

fn make_snippet_id(crate_name: &str, item_path: &str) -> String {
    let path_part = item_path
        .replace("::", "_")
        .replace(['<', '>', ' ', '\'', '"'], "");
    format!("{}_{}", crate_name, path_part)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_snippet_id() {
        assert_eq!(
            make_snippet_id("tokio", "tokio::runtime::Runtime"),
            "tokio_tokio_runtime_Runtime"
        );
    }

    #[test]
    fn test_snippet_category_as_str() {
        assert_eq!(SnippetCategory::CodeBlock.as_str(), "code_blocks");
        assert_eq!(SnippetCategory::Complex.as_str(), "complex");
    }
}
