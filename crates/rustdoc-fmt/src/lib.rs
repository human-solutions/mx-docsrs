//! Terminal formatting utilities for rustdoc documentation.
//!
//! This crate provides:
//! - [`Token`] and [`Output`] for building syntax-colored token sequences
//! - [`Colorizer`] for terminal styling and syntax highlighting
//! - [`format_markdown`] for rendering markdown to terminal output
//! - [`LinkResolver`] trait for custom link resolution

mod colorizer;
mod link_resolver;
mod markdown;
mod output;
mod tokens;

pub use colorizer::Colorizer;
pub use link_resolver::{DefaultLinkResolver, LinkResolver};
pub use markdown::format_markdown;
pub use output::Output;
pub use tokens::{Token, tokens_to_string};
