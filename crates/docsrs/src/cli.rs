use clap::Parser;

use crate::crate_spec::CrateSpec;

/// Search for documentation of a symbol in a crate
#[derive(Parser, Debug)]
#[command(name = "docsrs")]
#[command(about = "Search for documentation of a symbol in a crate", long_about = None)]
pub struct Cli {
    /// The crate name to search in, optionally with version (e.g., "serde" or "serde@1.0")
    #[arg(value_parser = parse_crate_spec)]
    pub crate_spec: Option<CrateSpec>,

    /// The symbol to search for
    pub symbol: Option<String>,

    /// Skip cache and download fresh rustdoc JSON
    #[arg(long)]
    pub no_cache: bool,

    /// Clear the entire cache directory
    #[arg(long)]
    pub clear_cache: bool,
}


fn parse_crate_spec(s: &str) -> Result<CrateSpec, String> {
    CrateSpec::parse(s).map_err(|e| e.to_string())
}
