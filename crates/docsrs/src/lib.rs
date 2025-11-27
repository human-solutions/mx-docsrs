mod cli;
mod color;
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

    // Check if this is a local workspace crate
    let local_doc_path = if let Ok(resolver) = VersionResolver::new() {
        if resolver.is_local_crate(&crate_spec.name) {
            resolver.get_local_crate_doc_path(&crate_spec.name)
        } else {
            None
        }
    } else {
        None
    };

    // Load documentation
    let krate = if let Some(doc_path) = local_doc_path {
        output.push_str(&format!("Local crate found at: {}\n", doc_path.display()));
        load_local_docs(&doc_path)?
    } else {
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

        // Fetch documentation from docs.rs
        fetch_docs(&crate_spec.name, &version, use_cache)?
    };

    let item_processor = ItemProcessor::process(&krate);
    let mut list = list_items(&item_processor);

    // First filter by path prefix (if provided)
    if let Some(prefix) = path_prefix.as_deref() {
        filter_by_path_prefix(&mut list, prefix);
    }

    // Then filter by text filter (if provided)
    if let Some(filter) = filter.as_deref() {
        filter_list(&mut list, filter);
    }

    list.sort_by(|item1, item2| item1.path.cmp(&item2.path));

    let result = if list.len() != 1 {
        list.iter()
            .map(|entry| {
                if parsed_args.color.is_active() {
                    entry.as_output().to_colored_string()
                } else {
                    entry.as_output().to_string()
                }
            })
            .collect::<Vec<String>>()
            .join("\n")
    } else {
        let id = list[0].id;
        doc::signature_for_id(&krate, &item_processor, &id, parsed_args.color)?
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
/// Keeps items where path starts with `crate::{prefix}` (matching all descendants).
fn filter_by_path_prefix<'c>(list: &mut Vec<ListItem<'c>>, prefix: &str) {
    let full_prefix = format!("crate::{prefix}");
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
