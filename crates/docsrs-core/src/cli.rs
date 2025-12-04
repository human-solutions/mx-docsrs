use clap::Parser;

use crate::color::Color;
use crate::crate_spec::CrateSpec;

/// Search for documentation of a symbol in a crate
#[derive(Parser, Debug)]
#[command(name = "docsrs")]
#[command(about = "Search for documentation of a symbol in a crate or list all symbols", long_about = None)]
#[command(after_help = "\
EXAMPLES:
  docsrs tokio                   Crate docs (version from Cargo.toml)
  docsrs tokio::spawn            Specific item
  docsrs serde@1.0::Deserialize  Explicit version
  docsrs tokio task              Search for 'task' in tokio")]
#[command(after_long_help = "\
VERSION RESOLUTION:
  When no version is specified, docsrs resolves it automatically:

  1. Direct dependency    Uses the version from your Cargo.toml
  2. Transitive dep       Resolves through the dependency chain
  3. Local/workspace      Builds docs with: cargo +nightly doc
  4. Not found            Falls back to latest version on docs.rs

LOCAL CRATES:
  Workspace crates are detected automatically and documentation is
  built using `cargo +nightly doc`. Requires nightly toolchain:
    rustup toolchain install nightly

  If the build fails but cached docs exist, they are used with a warning.

EXAMPLES:
  docsrs tokio                   Crate root (version from Cargo.toml)
  docsrs tokio::spawn            Specific item
  docsrs serde@1.0::Deserialize  Explicit version
  docsrs tokio task              Search for 'task' in tokio")]
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
