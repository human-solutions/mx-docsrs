mod cli;
mod crate_spec;
mod doc;
mod docfetch;
mod ext;
mod fmt;
mod version_resolver;

use clap::Parser;
use cli::Cli;
use doc::extract::extract_doc;
use docfetch::{clear_cache, fetch_docs};
use fmt::{format_search_results_list, format_to_terminal};
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
    let symbol = parsed_args
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
    let (results, krate) = fetch_docs(&crate_spec.name, &version, &symbol, use_cache)?;

    // Handle results
    if results.is_empty() {
        output.push_str(&format!("\nNo items found matching '{}'\n", symbol));
        return Ok(output);
    }

    if results.len() > 1 {
        // Multiple results - list them with FQDNs
        let search_data: Vec<(&String, &String, &Vec<String>)> = results
            .iter()
            .map(|r| (&r.item_type, &r.name, &r.path))
            .collect();

        output.push_str(&format_search_results_list(&search_data));
        return Ok(output);
    }

    // Only one result - show full documentation
    let selected_result = &results[0];

    if let Some(item) = krate.index.get(&selected_result.id) {
        let doc = extract_doc(item, &krate)?;
        output.push_str(&format_to_terminal(&doc));
    } else {
        anyhow::bail!("Failed to find item in crate index");
    }

    Ok(output)
}
