# docsrs

Fast Rust documentation lookup for CLI and AI assistants.

Query [docs.rs](https://docs.rs) and local crate documentation with automatic version resolution, syntax-highlighted output, and MCP server integration for Claude.

## Features

- **Unified syntax**: `crate[@version][::path]` - one format for all queries
- **Smart version resolution**: Automatically detects versions from your `Cargo.toml`
- **Local crate support**: Auto-builds documentation for workspace crates
- **Syntax highlighting**: Color-coded terminal output with markdown formatting
- **MCP server**: Integrates with Claude Code and other MCP-compatible AI tools
- **Caching**: Fast repeated queries with local documentation cache

## Installation

### From source

```bash
git clone https://github.com/human-solutions/mx-docsrs
cd mx-docsrs
cargo install --path crates/docsrs
```

### Requirements

- Rust nightly toolchain (for local crate documentation):
  ```bash
  rustup toolchain install nightly
  ```

## Usage

### Basic queries

```bash
# View crate root documentation
docsrs tokio

# Navigate to specific item
docsrs tokio::spawn
docsrs serde::Deserialize

# Deep path navigation
docsrs tokio::sync::mpsc::channel
```

### Version specification

```bash
# Explicit version
docsrs tokio@1.35.0

# Version requirement
docsrs serde@^1.0

# From your Cargo.toml (automatic)
docsrs tokio  # uses version from your project's dependencies
```

### Filtering

```bash
# Search within a crate
docsrs tokio spawn         # items containing "spawn"

# Search within a module
docsrs tokio::sync mutex   # items in tokio::sync containing "mutex"
```

### Options

```bash
# Clear documentation cache
docsrs --clear-cache

# Skip cache for fresh fetch
docsrs --no-cache tokio

# Control color output
docsrs --color=always tokio
docsrs --color=never tokio
```

## MCP Server

docsrs includes an MCP (Model Context Protocol) server for integration with Claude Code and other AI assistants.

### Setup with Claude Code

Add to your Claude Code MCP configuration:

```json
{
  "mcpServers": {
    "docsrs": {
      "type": "stdio",
      "command": "docsrs",
      "args": ["--mcp"]
    }
  }
}
```

### Tool

The MCP server exposes a single tool:

**`lookup_docs`**
- `crate_spec` (required): Crate path like `tokio`, `serde@1.0`, or `tokio::spawn`
- `filter` (optional): Search term to filter results

## How It Works

### Version Resolution

When you query a crate without an explicit version:

1. **Direct dependency**: Uses the version from your `Cargo.toml`
2. **Transitive dependency**: Resolves through the dependency chain
3. **Local/workspace crate**: Builds documentation with `cargo +nightly doc`
4. **Not found**: Falls back to latest version on docs.rs

### Documentation Sources

- **Published crates**: Fetches pre-built JSON from docs.rs
- **Local crates**: Builds documentation using `cargo +nightly doc` with JSON output
- **Cached**: Stores downloaded documentation for fast subsequent queries

### Error Handling

- **Missing nightly**: Clear error message with installation instructions
- **Build failures**: Uses cached documentation with a warning if available

## Project Structure

| Crate | Description |
|-------|-------------|
| `docsrs` | Main binary (CLI and MCP server) |
| `docsrs-core` | Core CLI logic and documentation fetching |
| `docsrs-mcp` | MCP server implementation |
| `jsondoc` | Rustdoc JSON processing |
| `rustdoc-fmt` | Terminal markdown formatting with syntax highlighting |

## License

MIT OR Apache-2.0
