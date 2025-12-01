//! Statistics aggregation and reporting

use crate::markdown_analyzer::MarkdownStats;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Aggregate statistics across all analyzed crates
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AggregateStats {
    pub total_crates: usize,
    pub total_docs: usize,
    pub total_chars: usize,
    pub aggregate: MarkdownStats,
    pub per_crate: HashMap<String, CrateStats>,
}

/// Statistics for a single crate
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CrateStats {
    pub docs_count: usize,
    pub total_chars: usize,
    pub stats: MarkdownStats,
}

/// Full analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub generated_at: String,
    pub stats: AggregateStats,
    pub feature_prevalence: HashMap<String, f64>,
}

impl AnalysisReport {
    pub fn new(stats: AggregateStats) -> Self {
        let feature_prevalence = calculate_prevalence(&stats);
        Self {
            generated_at: chrono_lite_now(),
            stats,
            feature_prevalence,
        }
    }
}

/// Simple ISO 8601 timestamp without chrono dependency
pub fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    // Simple conversion (not handling leap seconds, but good enough for timestamps)
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    // Calculate year, month, day from days since 1970-01-01
    let mut year = 1970;
    let mut remaining_days = days;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let (month, day) = day_of_year_to_month_day(remaining_days as u32, is_leap_year(year));

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

fn is_leap_year(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

fn day_of_year_to_month_day(day_of_year: u32, leap: bool) -> (u32, u32) {
    let days_in_months: [u32; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut remaining = day_of_year;
    for (i, &days) in days_in_months.iter().enumerate() {
        if remaining < days {
            return (i as u32 + 1, remaining + 1);
        }
        remaining -= days;
    }
    (12, 31) // Fallback
}

/// Calculate feature prevalence as percentage of docs containing each feature
fn calculate_prevalence(stats: &AggregateStats) -> HashMap<String, f64> {
    let total = stats.total_docs as f64;
    if total == 0.0 {
        return HashMap::new();
    }

    let mut prevalence = HashMap::new();
    let agg = &stats.aggregate;

    // Count docs with each feature (approximation based on aggregate counts)
    // For accurate prevalence, we'd need per-doc tracking, but this gives a good estimate
    prevalence.insert(
        "code_blocks".to_string(),
        estimate_prevalence(agg.code_blocks, total),
    );
    prevalence.insert(
        "inline_code".to_string(),
        estimate_prevalence(agg.inline_code, total),
    );
    prevalence.insert("links".to_string(), estimate_prevalence(agg.links, total));
    prevalence.insert("lists".to_string(), estimate_prevalence(agg.lists, total));
    prevalence.insert(
        "headings".to_string(),
        estimate_prevalence(agg.headings, total),
    );
    prevalence.insert(
        "emphasis".to_string(),
        estimate_prevalence(agg.emphasis, total),
    );
    prevalence.insert("strong".to_string(), estimate_prevalence(agg.strong, total));
    prevalence.insert(
        "block_quotes".to_string(),
        estimate_prevalence(agg.block_quotes, total),
    );
    prevalence.insert("tables".to_string(), estimate_prevalence(agg.tables, total));
    prevalence.insert(
        "strikethrough".to_string(),
        estimate_prevalence(agg.strikethrough, total),
    );
    prevalence.insert("images".to_string(), estimate_prevalence(agg.images, total));
    prevalence.insert(
        "footnotes".to_string(),
        estimate_prevalence(agg.footnote_definitions, total),
    );
    prevalence.insert(
        "task_lists".to_string(),
        estimate_prevalence(agg.task_list_markers, total),
    );

    prevalence
}

/// Estimate prevalence (capped at 1.0 since count can exceed doc count)
fn estimate_prevalence(count: usize, total: f64) -> f64 {
    (count as f64 / total).min(1.0)
}

/// Generate a markdown report from the analysis
pub fn generate_markdown_report(report: &AnalysisReport) -> String {
    let mut md = String::new();

    md.push_str("# Rustdoc Markdown Analysis Report\n\n");
    md.push_str(&format!("Generated: {}\n\n", report.generated_at));

    // Summary
    md.push_str("## Summary\n\n");
    md.push_str("| Metric | Value |\n");
    md.push_str("|--------|-------|\n");
    md.push_str(&format!(
        "| Crates analyzed | {} |\n",
        report.stats.total_crates
    ));
    md.push_str(&format!(
        "| Doc strings analyzed | {} |\n",
        report.stats.total_docs
    ));
    md.push_str(&format!(
        "| Total markdown characters | {} |\n\n",
        format_number(report.stats.total_chars)
    ));

    // Feature frequency table
    md.push_str("## Feature Frequency\n\n");

    let agg = &report.stats.aggregate;
    md.push_str("| Feature | Count | Est. Prevalence |\n");
    md.push_str("|---------|-------|----------------|\n");

    let mut features: Vec<(&str, usize)> = vec![
        ("Code blocks", agg.code_blocks),
        ("Inline code", agg.inline_code),
        ("Links", agg.links),
        ("Lists", agg.lists),
        ("Nested lists", agg.nested_lists),
        ("Headings", agg.headings),
        ("Emphasis", agg.emphasis),
        ("Strong", agg.strong),
        ("Block quotes", agg.block_quotes),
        ("Tables", agg.tables),
        ("Strikethrough", agg.strikethrough),
        ("Images", agg.images),
        ("Footnotes", agg.footnote_definitions),
        ("Task lists", agg.task_list_markers),
        ("HTML blocks", agg.html_blocks),
    ];

    // Sort by count descending
    features.sort_by(|a, b| b.1.cmp(&a.1));

    for (name, count) in features {
        let prevalence = report
            .feature_prevalence
            .get(&name.to_lowercase().replace(' ', "_"))
            .copied()
            .unwrap_or(0.0);
        md.push_str(&format!(
            "| {} | {} | {:.1}% |\n",
            name,
            format_number(count),
            prevalence * 100.0
        ));
    }

    // Code block languages
    md.push_str("\n## Code Block Languages\n\n");
    md.push_str("| Language | Count | % of code blocks |\n");
    md.push_str("|----------|-------|------------------|\n");

    let total_code_blocks = agg.code_blocks.max(1) as f64;
    let mut langs: Vec<_> = agg.code_block_languages.iter().collect();
    langs.sort_by(|a, b| b.1.cmp(a.1));

    for (lang, count) in langs.iter().take(10) {
        let pct = **count as f64 / total_code_blocks * 100.0;
        md.push_str(&format!("| {} | {} | {:.1}% |\n", lang, count, pct));
    }

    // Link types
    md.push_str("\n## Link Types\n\n");
    md.push_str("| Type | Count | % of links |\n");
    md.push_str("|------|-------|------------|\n");

    let total_links = agg.links.max(1) as f64;
    let link_types = [
        ("Intra-doc (`[`Foo`]`)", agg.link_types.intra_doc),
        ("Internal (`.html`)", agg.link_types.internal_doc),
        ("External (http/https)", agg.link_types.external_http),
        ("Anchor (`#`)", agg.link_types.anchor),
    ];

    for (name, count) in link_types {
        let pct = count as f64 / total_links * 100.0;
        md.push_str(&format!("| {} | {} | {:.1}% |\n", name, count, pct));
    }

    // Feature gap analysis
    md.push_str("\n## Feature Gap Analysis\n\n");
    md.push_str("Features that rustdoc-fmt does NOT currently handle:\n\n");

    if agg.tables > 0 {
        md.push_str(&format!(
            "- **Tables**: {} occurrences ({:.2}% of docs)\n",
            agg.tables,
            agg.tables as f64 / report.stats.total_docs as f64 * 100.0
        ));
    }
    if agg.footnote_definitions > 0 {
        md.push_str(&format!(
            "- **Footnotes**: {} occurrences\n",
            agg.footnote_definitions
        ));
    }
    if agg.task_list_markers > 0 {
        md.push_str(&format!(
            "- **Task lists**: {} occurrences\n",
            agg.task_list_markers
        ));
    }
    if agg.images > 0 {
        md.push_str(&format!("- **Images**: {} occurrences\n", agg.images));
    }

    md
}

fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(500), "500");
        assert_eq!(format_number(1500), "1.5K");
        assert_eq!(format_number(1_500_000), "1.5M");
    }

    #[test]
    fn test_chrono_lite_now() {
        let ts = chrono_lite_now();
        assert!(ts.contains('T'));
        assert!(ts.ends_with('Z'));
    }
}
