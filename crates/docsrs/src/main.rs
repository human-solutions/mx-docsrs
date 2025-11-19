mod cli;
mod crate_spec;
mod doc;
mod docfetch;
mod docrender;
mod ext;
mod terminal_render;
mod version_resolver;

use anyhow::Result;
use cli::Cli;
use docfetch::{clear_cache, fetch_docs};
use docrender::extract_doc;
use terminal_render::{render_search_results_list, render_to_terminal};
use version_resolver::VersionResolver;

fn main() -> Result<()> {
    let args = Cli::parse_args();

    // Handle --clear-cache flag
    if args.clear_cache {
        return clear_cache();
    }

    // Require crate_spec and symbol if not clearing cache
    let crate_spec = args
        .crate_spec
        .ok_or_else(|| anyhow::anyhow!("Missing required argument: CRATE_SPEC"))?;
    let symbol = args
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
    let use_cache = !args.no_cache;

    // Fetch and search documentation
    let (results, krate) = fetch_docs(&crate_spec.name, &version, &symbol, use_cache)?;

    // Handle results
    if results.is_empty() {
        println!("\nNo items found matching '{}'", symbol);
        return Ok(());
    }

    if results.len() > 1 {
        // Multiple results - list them with FQDNs and exit
        let search_data: Vec<(&String, &String, &Vec<String>)> = results
            .iter()
            .map(|r| (&r.item_type, &r.name, &r.path))
            .collect();

        render_search_results_list(&search_data);
        return Ok(());
    }

    // Only one result - show full documentation
    let selected_result = &results[0];

    if let Some(item) = krate.index.get(&selected_result.id) {
        let doc = extract_doc(item, &krate)?;
        render_to_terminal(&doc);
    } else {
        anyhow::bail!("Failed to find item in crate index");
    }

    Ok(())
}
