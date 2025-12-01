mod corpus;
pub mod crate_list;
mod doc_extractor;
mod docfetch;
mod markdown_analyzer;
mod stats;

pub use corpus::{CorpusSnippet, SnippetCategory, SnippetSelector, TestCorpus};
pub use crate_list::{CRATES, CrateCategory, CrateInfo, all_categories, crates_by_category};
pub use doc_extractor::{DocEntry, extract_docs};
pub use docfetch::{clear_cache, fetch_docs};
pub use markdown_analyzer::{LinkTypeStats, MarkdownStats, analyze_markdown};
pub use stats::{
    AggregateStats, AnalysisReport, CrateStats, chrono_lite_now, generate_markdown_report,
};
