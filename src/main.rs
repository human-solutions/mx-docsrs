mod cli;
mod crate_spec;
mod docfetch;
mod version_resolver;

use anyhow::Result;
use cli::Cli;
use docfetch::fetch_docs;
use version_resolver::VersionResolver;

fn main() -> Result<()> {
    let args = Cli::parse_args();

    // Resolve the version
    let version = if let Some(explicit_version) = args.crate_spec.version {
        // Use explicitly provided version
        explicit_version
    } else {
        // Try to resolve from Cargo.toml
        match VersionResolver::new() {
            Ok(resolver) => resolver
                .resolve_version(&args.crate_spec.name)
                .unwrap_or_else(|| "latest".to_string()),
            Err(_) => {
                // No Cargo.toml found, default to latest
                "latest".to_string()
            }
        }
    };

    // Fetch and search documentation
    let results = fetch_docs(&args.crate_spec.name, &version, &args.symbol)?;

    // Display results
    println!("\n=== Search Results ===\n");

    if results.is_empty() {
        println!("No items found matching '{}'", args.symbol);
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
