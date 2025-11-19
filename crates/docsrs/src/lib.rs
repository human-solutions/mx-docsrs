mod cli;
mod crate_spec;
mod doc;
mod docfetch;
mod version_resolver;

use clap::Parser;
use cli::Cli;
use docfetch::{clear_cache, fetch_docs};
use version_resolver::VersionResolver;

/// Run the CLI with the given arguments and return the output as a string.
///
/// # Arguments
/// * `args` - Command line arguments (excluding program name)
///
/// # Returns
/// * `Ok(String)` - Successful output (stdout)
/// * `Err(String)` - Error message (stderr)
pub fn run_cli(args: &[&str]) -> Result<String, String> {
    match run_cli_impl(args) {
        Ok(output) => Ok(output),
        Err(e) => Err(e.to_string()),
    }
}

fn run_cli_impl(args: &[&str]) -> anyhow::Result<String> {
    let mut output = String::new();

    // Parse arguments using the Cli::try_parse_from method
    let parsed_args =
        match Cli::try_parse_from(std::iter::once("docsrs").chain(args.iter().copied())) {
            Ok(args) => args,
            Err(e) => {
                // Handle --help and --version as successful outputs
                if e.kind() == clap::error::ErrorKind::DisplayHelp
                    || e.kind() == clap::error::ErrorKind::DisplayVersion
                {
                    return Ok(e.to_string());
                }
                return Err(e.into());
            }
        };

    // Handle --clear-cache flag
    if parsed_args.clear_cache {
        clear_cache()?;
        output.push_str("Cache cleared successfully\n");
        return Ok(output);
    }

    // Require crate_spec and symbol if not clearing cache
    let crate_spec = parsed_args
        .crate_spec
        .ok_or_else(|| anyhow::anyhow!("Missing required argument: CRATE_SPEC"))?;
    let _symbol = parsed_args
        .symbol
        .ok_or_else(|| anyhow::anyhow!("Missing required argument: SYMBOL"))?;

    // Resolve the version
    let version = if let Some(explicit_version) = crate_spec.version {
        // Use explicitly provided version
        explicit_version
    } else {
        // Try to resolve from Cargo.toml
        match VersionResolver::new() {
            Ok(resolver) => resolver
                .resolve_version(&crate_spec.name)
                .unwrap_or_else(|| "latest".to_string()),
            Err(_) => {
                // No Cargo.toml found, default to latest
                "latest".to_string()
            }
        }
    };

    // Determine whether to use cache
    let use_cache = !parsed_args.no_cache;

    // Fetch and search documentation
    let krate = fetch_docs(&crate_spec.name, &version, use_cache)?;

    doc::extract_list(&krate)
}
