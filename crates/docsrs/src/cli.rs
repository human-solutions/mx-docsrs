use clap::Parser;

use crate::color::Color;
use crate::crate_spec::CrateSpec;

/// Search for documentation of a symbol in a crate
#[derive(Parser, Debug)]
#[command(name = "docsrs")]
#[command(about = "Search for documentation of a symbol in a crate or list all symbols", long_about = None)]
pub struct Cli {
    /// Crate path: crate[@version][::path] (e.g., "tokio", "serde@1.0", "tokio::task::spawn")
    #[arg(value_parser = parse_crate_spec)]
    pub crate_spec: Option<CrateSpec>,

    /// Filter to search within the path (optional - if omitted, lists all items in path)
    pub filter: Option<String>,

    /// Skip cache and download fresh rustdoc JSON
    #[arg(long)]
    pub no_cache: bool,

    /// Clear the entire cache directory
    #[arg(long)]
    pub clear_cache: bool,

    /// When to use colors in output.
    ///
    /// By default, `--color=auto` is active. Using just `--color` without an
    /// arg is equivalent to `--color=always`.
    #[arg(long, value_name = "WHEN", default_value = "auto")]
    pub color: Color,
}

fn parse_crate_spec(s: &str) -> Result<CrateSpec, String> {
    CrateSpec::parse(s).map_err(|e| e.to_string())
}
