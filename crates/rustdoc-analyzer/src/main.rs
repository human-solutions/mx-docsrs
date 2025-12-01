//! CLI for rustdoc markdown analysis

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rustdoc_analyzer::{
    AggregateStats, AnalysisReport, CRATES, CrateStats, SnippetSelector, TestCorpus,
    analyze_markdown, extract_docs, fetch_docs, generate_markdown_report,
};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "rustdoc-analyzer")]
#[command(about = "Analyze markdown patterns in Rust documentation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze all configured crates
    Analyze {
        /// Output directory for results
        #[arg(short, long, default_value = "output")]
        output: PathBuf,

        /// Skip crate download cache
        #[arg(long)]
        no_cache: bool,
    },
    /// Analyze a single crate
    Single {
        /// Crate name
        name: String,

        /// Crate version (default: latest)
        #[arg(short, long, default_value = "latest")]
        version: String,
    },
    /// Clear the download cache
    ClearCache,
    /// List configured crates
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { output, no_cache } => {
            run_full_analysis(&output, !no_cache)?;
        }
        Commands::Single { name, version } => {
            run_single_analysis(&name, &version)?;
        }
        Commands::ClearCache => {
            rustdoc_analyzer::clear_cache()?;
        }
        Commands::List => {
            list_crates();
        }
    }

    Ok(())
}

fn run_full_analysis(output_dir: &PathBuf, use_cache: bool) -> Result<()> {
    // Create output directory
    fs::create_dir_all(output_dir)?;
    fs::create_dir_all(output_dir.join("corpus"))?;

    let mut stats = AggregateStats::default();
    let mut all_entries: Vec<(rustdoc_analyzer::DocEntry, rustdoc_analyzer::MarkdownStats)> =
        Vec::new();

    let total_crates = CRATES.len();

    for (i, crate_info) in CRATES.iter().enumerate() {
        println!(
            "[{}/{}] Analyzing {} ({})",
            i + 1,
            total_crates,
            crate_info.name,
            crate_info.category.as_str()
        );

        match analyze_crate(crate_info.name, crate_info.version, use_cache) {
            Ok((crate_stats, entries)) => {
                stats.total_crates += 1;
                stats.total_docs += crate_stats.docs_count;
                stats.total_chars += crate_stats.total_chars;
                stats.aggregate.merge(&crate_stats.stats);
                stats
                    .per_crate
                    .insert(crate_info.name.to_string(), crate_stats);
                all_entries.extend(entries);
                println!(
                    "  ✓ {} docs analyzed",
                    stats.per_crate[crate_info.name].docs_count
                );
            }
            Err(e) => {
                eprintln!("  ✗ Failed: {}", e);
            }
        }
    }

    println!("\n=== Analysis Complete ===");
    println!("Crates analyzed: {}/{}", stats.total_crates, total_crates);
    println!("Total docs: {}", stats.total_docs);

    // Generate report
    let report = AnalysisReport::new(stats);

    // Write JSON report
    let json_path = output_dir.join("stats.json");
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&json_path, &json)?;
    println!("Wrote: {}", json_path.display());

    // Write markdown report
    let md_path = output_dir.join("stats.md");
    let md = generate_markdown_report(&report);
    fs::write(&md_path, &md)?;
    println!("Wrote: {}", md_path.display());

    // Generate corpus
    let selector = SnippetSelector::new();
    let corpus = selector.select(&all_entries);

    // Write corpus index
    let corpus_path = output_dir.join("corpus").join("index.json");
    let corpus_json = serde_json::to_string_pretty(&corpus)?;
    fs::write(&corpus_path, &corpus_json)?;
    println!("Wrote: {}", corpus_path.display());

    // Write individual snippet files
    write_corpus_files(output_dir, &corpus)?;

    println!("\nCorpus: {} snippets extracted", corpus.snippet_count);

    Ok(())
}

fn analyze_crate(
    name: &str,
    version: &str,
    use_cache: bool,
) -> Result<(
    CrateStats,
    Vec<(rustdoc_analyzer::DocEntry, rustdoc_analyzer::MarkdownStats)>,
)> {
    let krate = fetch_docs(name, version, use_cache)
        .with_context(|| format!("Failed to fetch docs for {}", name))?;

    let docs = extract_docs(&krate, name);

    let mut crate_stats = CrateStats::default();
    let mut entries = Vec::new();

    for doc in docs {
        let md_stats = analyze_markdown(&doc.doc_string);
        crate_stats.docs_count += 1;
        crate_stats.total_chars += doc.doc_string.len();
        crate_stats.stats.merge(&md_stats);
        entries.push((doc, md_stats));
    }

    Ok((crate_stats, entries))
}

fn run_single_analysis(name: &str, version: &str) -> Result<()> {
    println!("Analyzing {} @ {}", name, version);

    let (crate_stats, _entries) = analyze_crate(name, version, true)?;

    println!("\n=== {} ===", name);
    println!("Docs: {}", crate_stats.docs_count);
    println!("Total chars: {}", crate_stats.total_chars);
    println!("\nFeatures:");
    println!("  Code blocks: {}", crate_stats.stats.code_blocks);
    println!("  Inline code: {}", crate_stats.stats.inline_code);
    println!("  Links: {}", crate_stats.stats.links);
    println!("  Lists: {}", crate_stats.stats.lists);
    println!("  Nested lists: {}", crate_stats.stats.nested_lists);
    println!("  Tables: {}", crate_stats.stats.tables);
    println!("  Block quotes: {}", crate_stats.stats.block_quotes);
    println!("  Headings: {}", crate_stats.stats.headings);

    if !crate_stats.stats.code_block_languages.is_empty() {
        println!("\nCode block languages:");
        for (lang, count) in &crate_stats.stats.code_block_languages {
            println!("  {}: {}", lang, count);
        }
    }

    Ok(())
}

fn write_corpus_files(output_dir: &Path, corpus: &TestCorpus) -> Result<()> {
    use rustdoc_analyzer::SnippetCategory;

    // Create category directories
    for cat in SnippetCategory::all() {
        let dir = output_dir.join("corpus").join(cat.as_str());
        fs::create_dir_all(&dir)?;
    }

    // Write each snippet
    for snippet in &corpus.snippets {
        let filename = format!("{}.md", sanitize_filename(&snippet.id));
        let filepath = output_dir
            .join("corpus")
            .join(snippet.category.as_str())
            .join(&filename);

        let content = format!(
            "<!-- Source: {} -->\n<!-- Features: {} -->\n\n{}",
            snippet.source_path,
            snippet.features.join(", "),
            snippet.markdown
        );

        fs::write(&filepath, content)?;
    }

    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn list_crates() {
    use rustdoc_analyzer::{all_categories, crates_by_category};

    println!("Configured crates for analysis:\n");

    for category in all_categories() {
        println!("{}:", category.as_str());
        for crate_info in crates_by_category(category) {
            println!("  - {} @ {}", crate_info.name, crate_info.version);
        }
        println!();
    }

    println!("Total: {} crates", CRATES.len());
}
