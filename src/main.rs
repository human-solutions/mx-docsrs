mod cli;
mod crate_spec;
mod docfetch;
mod version_resolver;

use anyhow::Result;
use cli::Cli;
use docfetch::{clear_cache, fetch_docs};
use version_resolver::VersionResolver;

fn main() -> Result<()> {
    let args = Cli::parse_args();

    // Handle --clear-cache flag
    if args.clear_cache {
        return clear_cache();
    }

    // Require crate_spec and symbol if not clearing cache
    let crate_spec = args.crate_spec.ok_or_else(|| {
        anyhow::anyhow!("Missing required argument: CRATE_SPEC")
    })?;
    let symbol = args.symbol.ok_or_else(|| {
        anyhow::anyhow!("Missing required argument: SYMBOL")
    })?;

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
    let results = fetch_docs(&crate_spec.name, &version, &symbol, use_cache)?;

    // Display results
    println!("\n=== Search Results ===\n");

    if results.is_empty() {
        println!("No items found matching '{}'", symbol);
    } else {
        for result in &results {
            println!("{} {} ({})",
                     result.item_type,
                     result.name,
                     result.path.join("::"));
            println!("  {}\n", result.url);
        }
        println!("Total: {} result(s)", results.len());
    }

    Ok(())
}
