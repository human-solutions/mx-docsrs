mod cli;
mod color;
pub mod colorizer;
mod crate_spec;
mod doc;
mod docfetch;
mod ext;
mod fmt;
mod list;
mod proc;
mod util;
mod version_resolver;

use clap::Parser;
use cli::Cli;
use docfetch::{clear_cache, fetch_docs, load_local_docs};
use version_resolver::VersionResolver;

use crate::{
    list::{ListItem, list_items},
    proc::ItemProcessor,
};

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

    // Apply global color override based on --color flag
    match parsed_args.color {
        color::Color::Never => colored::control::set_override(false),
        color::Color::Always => colored::control::set_override(true),
        color::Color::Auto => {} // colored handles auto-detection
    }

    // Handle --clear-cache flag
    if parsed_args.clear_cache {
        clear_cache()?;
        output.push_str("Cache cleared successfully\n");
        return Ok(output);
    }

    // Require crate_spec if not clearing cache
    let crate_spec = parsed_args
        .crate_spec
        .ok_or_else(|| anyhow::anyhow!("Missing required argument: CRATE_SPEC"))?;

    // Filter is optional - if not provided, we'll list all items
    let filter = parsed_args.filter;
    let path_prefix = crate_spec.path_prefix.clone();

    // Resolve the crate version and load documentation
    let krate = if let Some(explicit_version) = crate_spec.version.clone() {
        // User provided explicit version - skip resolution, just fetch
        let use_cache = !parsed_args.no_cache;
        fetch_docs(&crate_spec.name, &explicit_version, use_cache)?
    } else {
        // Try to resolve from Cargo.toml
        match VersionResolver::new() {
            Ok(resolver) => {
                if let Some(resolved) = resolver.resolve_crate(&crate_spec.name) {
                    // Print resolution message
                    output.push_str(&format!("{}\n", resolved.format_message()));

                    if resolved.is_local {
                        // Load local docs if available
                        if let Some(doc_path) = resolver.get_local_crate_doc_path(&crate_spec.name)
                        {
                            load_local_docs(&doc_path)?
                        } else {
                            // Local crate but no docs built yet - fetch from docs.rs
                            let use_cache = !parsed_args.no_cache;
                            fetch_docs(&crate_spec.name, &resolved.version, use_cache)?
                        }
                    } else {
                        // External dependency - fetch from docs.rs
                        let use_cache = !parsed_args.no_cache;
                        fetch_docs(&resolved.name, &resolved.version, use_cache)?
                    }
                } else {
                    // Not found in project, use latest
                    output.push_str(&format!("Using {}@latest\n", crate_spec.name));
                    let use_cache = !parsed_args.no_cache;
                    fetch_docs(&crate_spec.name, "latest", use_cache)?
                }
            }
            Err(_) => {
                // No Cargo.toml found, default to latest
                output.push_str(&format!("Using {}@latest\n", crate_spec.name));
                let use_cache = !parsed_args.no_cache;
                fetch_docs(&crate_spec.name, "latest", use_cache)?
            }
        }
    };

    let item_processor = ItemProcessor::process(&krate);

    // Determine the output based on path and filter
    let result = match (path_prefix.as_deref(), filter.as_deref()) {
        // Pure navigation: show doc for exact path
        (Some(prefix), None) => {
            let full_path = format!("{}::{}", crate_spec.name, prefix);
            let id = item_processor
                .find_item_by_path(&full_path)
                .ok_or_else(|| anyhow::anyhow!("No item found at {}", full_path))?;
            doc::signature_for_id(&krate, &item_processor, &id)?
        }
        // Search mode: filter items and show list or single doc
        (path_prefix, Some(filter)) => {
            let mut list = list_items(&item_processor);

            // Filter by path prefix if provided
            if let Some(prefix) = path_prefix {
                filter_by_path_prefix(&mut list, &crate_spec.name, prefix);
            }

            // Filter by text filter
            filter_list(&mut list, filter);

            list.sort_by(|item1, item2| item1.path.cmp(&item2.path));

            if list.len() == 1 {
                doc::signature_for_id(&krate, &item_processor, &list[0].id)?
            } else {
                let colorizer = colorizer::Colorizer::get();
                list.iter()
                    .map(|entry| colorizer.tokens(&entry.as_output().into_tokens()))
                    .collect::<Vec<String>>()
                    .join("\n")
            }
        }
        // No path, no filter: show crate root doc
        (None, None) => {
            let id = item_processor.crate_root_id();
            doc::signature_for_id(&krate, &item_processor, &id)?
        }
    };

    // Prepend any accumulated output (e.g., local crate banner)
    if output.is_empty() {
        Ok(result)
    } else {
        output.push_str(&result);
        Ok(output)
    }
}

/// Filter items by path prefix.
/// Keeps items where path starts with `{crate_name}::{prefix}` (matching all descendants).
fn filter_by_path_prefix<'c>(list: &mut Vec<ListItem<'c>>, crate_name: &str, prefix: &str) {
    let full_prefix = format!("{crate_name}::{prefix}");
    list.retain(|item| {
        // Match exact prefix or prefix followed by ::
        item.path == full_prefix || item.path.starts_with(&format!("{full_prefix}::"))
    });
}

fn filter_list<'c>(list: &mut Vec<ListItem<'c>>, filter: &str) {
    // First try exact suffix match
    let matching_end: Vec<_> = list
        .iter()
        .filter(|item| item.path.ends_with(filter))
        .cloned()
        .collect();

    if matching_end.len() == 1 {
        *list = matching_end;
        return;
    }

    // Then try substring match
    let matching_sub: Vec<_> = list
        .iter()
        .filter(|item| item.path.contains(filter))
        .cloned()
        .collect();

    if !matching_sub.is_empty() {
        *list = matching_sub;
    }
}
