---
name: docsrs
description: Look up Rust crate documentation — struct fields, function signatures, trait methods, module contents — with automatic version resolution from Cargo.toml. Use this BEFORE grepping ~/.cargo/registry, fetching docs.rs, or guessing from training data. Triggers on Rust paths like `tokio::spawn`, `serde::Deserialize`, questions like "what fields does X have", "what methods does Y expose", "what's in module Z", or any mention of a Rust crate's API surface. Resolves versions from the current project's Cargo.toml automatically.
allowed-tools: Bash(docsrs:*)
---

# docsrs — Rust documentation lookup

Use the `docsrs` CLI for any question about Rust crate APIs: struct fields, function signatures, trait methods, module contents, available items. It resolves versions from the current `Cargo.toml` (or falls back to latest on docs.rs), supports local workspace crates, and returns rustdoc-quality output.

## Syntax

```
docsrs <crate>[@<version>][::<path>] [filter]
```

## When to use

- The user mentions a Rust crate (`tokio`, `serde`, `anyhow`, ...).
- The user writes a Rust path (`tokio::sync::mpsc`, `serde::Deserialize`).
- The user asks "what fields does X have", "what methods on Y", "what's in module Z".
- You need to verify a signature, return type, or available item before writing code that depends on it.

Prefer `docsrs` over reading `~/.cargo/registry/**` or fetching `docs.rs` directly — it's faster, version-aware, and produces cleaner output for LLM context.

## Examples

```
docsrs tokio                        # crate root + public items
docsrs tokio::spawn                 # full docs for a specific item
docsrs serde::Deserialize           # full docs for a trait
docsrs serde@1.0::Map               # explicit version
docsrs tokio::sync::mpsc channel    # search within a module
docsrs anyhow Error                 # search within a crate
```

## Version resolution

When you omit `@version`:

1. If the crate is a direct/transitive dependency in the current `Cargo.toml`, the locked version is used.
2. If it's a workspace member, docs are built locally with `cargo +nightly doc`.
3. Otherwise, latest from docs.rs.

## Output format

Output is plain text with rustdoc-style item descriptions. A leading `// dependency <crate>@<version>` comment shows the resolved version. Multiple matches return a sorted list; a single match returns full docs.
