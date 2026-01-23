# CLAUDE.md - Smart Tree v6.5.1 Comprehensive Development Guide

**Location**: `/aidata/aye/smart-tree`
**Language**: 100% Rust (174 source files)
**Binary Size**: ~29MB core (TUI/Dashboard optional)
**Latest Version**: 6.5.1

## What's New in 6.5.1

- **Security Scanner**: `--security-scan` detects supply chain attack patterns (IPFS/IPNS injection, fake verification, claude-flow references)
- **Aggressive Scanning**: Security scan ignores .gitignore to scan node_modules and hidden directories
- **Risk Classification**: Findings categorized as Critical/High/Medium/Low with context-aware adjustment
- **Actionable Recommendations**: Scanner provides cleanup steps and remediation guidance

## Previous: 6.2.0

- **Session Persistence**: `SessionStart`/`SessionEnd` hooks auto-save and restore context
- **Smart Restore**: `--claude-restore` only shows relevant, recent context (24h window)
- **Feature Gates**: TUI and Dashboard now optional (`--features tui`, `--features dashboard`)

## Quick Start for Returning Claude Instances

If this is your second+ session on smart-tree, use this section:

```bash
# Pre-commit (REQUIRED - zero clippy warnings enforced)
./scripts/manage.sh test          # Runs: cargo test + clippy -D warnings + fmt --check

# Build
cargo build --release             # Core (~29MB)
cargo build --release --features tui       # With Spicy TUI
cargo build --release --features dashboard # With egui dashboard
cargo build --release --features full      # Everything

# Run single test
cargo test test_name -- --exact --nocapture

# Run MCP server
cargo run --release -- --mcp
```

## Architecture

**Smart Tree** is a fast, AI-optimized directory visualization tool. Version 6.5.2.

### Core Pipeline: Scanner → Formatter

1. **Scanner** (`src/scanner.rs`): Directory traversal engine
   - Respects `.gitignore`, handles symlinks, marks inaccessible dirs with `*`

2. **Formatters** (`src/formatters/`): 25+ output formats
   - `classic` - Traditional tree (baseline)
   - `ai` - 80% token reduction
   - `quantum` - 100x+ compression via wave functions
   - `marqant` - Markdown compression

3. **MCP Server** (`src/mcp/`): 30+ JSON-RPC tools for AI assistants
   - Main tools: `src/mcp/tools_consolidated.rs`
   - Smart edit: `src/mcp/smart_edit.rs` (AST-aware editing)
   - Memory persistence: `~/.mem8/`

### Key Modules

| Module | Purpose |
|--------|---------|
| `src/main.rs` | CLI entry (clap 4.5) |
| `src/scanner.rs` | Core traversal logic |
| `src/formatters/mod.rs` | Formatter trait & registry |
| `src/mcp/mod.rs` | MCP server entry |
| `src/mem8/` | Wave-based consciousness/memory |
| `src/spicy_tui_enhanced.rs` | Interactive TUI (feature-gated) |

### Adding a New Formatter

1. Create `src/formatters/my_format.rs` implementing `Formatter` trait
2. Add module to `src/formatters/mod.rs`
3. Add CLI variant in `src/main.rs`
4. Wire in format dispatch

## Code Patterns

**Error handling**: Use `anyhow::Result<T>` with `.context()` for all fallible operations.

```rust
use anyhow::{Context, Result};

fn process(path: &str) -> Result<String> {
    std::fs::read_to_string(path)
        .context("Failed to read file")?
}
```

**Function parameters**: Prefer `&str` over `String`.

**Async**: Uses `tokio` runtime. Async tests use `#[tokio::test]`.

**Comments**: The codebase uses a fun "Cheet" persona style - keep it when extending existing commented sections.

## Feature Flags

```toml
default = ["std"]
tui = ["ratatui", "crossterm", "syntect", "artem"]      # Spicy TUI
dashboard = ["egui", "eframe", "egui_extras", "winit"]  # egui dashboard
candle = ["candle-core", "candle-transformers", ...]    # Local LLM
full = ["tui", "dashboard", "candle"]
```

## Testing

```bash
cargo test                              # All tests
cargo test scanner                      # Module tests
cargo test --test mcp_integration       # Integration tests
RUST_LOG=debug cargo test -- --nocapture  # With logging
```

Tests use real filesystem operations (no mocks). Async tests require `#[tokio::test]`.

## Recent Features (6.5.x)

- **Security Scanner**: `--security-scan` detects supply chain attacks (IPFS injection, claude-flow patterns)
- **Session Persistence**: `SessionStart`/`SessionEnd` hooks auto-save context
- **Claude Code Hooks**: `st --hooks-install` for automatic context injection
- **Code Review**: AI-powered review via `--review` flag (Grok/OpenRouter)

## Binaries

- `st` - Main CLI
- `mq` - Marqant markdown compressor
- `m8` - MEM8 consciousness tools
- `tree` - Drop-in tree replacement

## Scripts

`./scripts/manage.sh` handles most tasks:
- `test` - Full pre-commit checks
- `build release` - Release build
- `mcp-run` - Start MCP server
- `bump patch|minor|major` - Version management
- `release v6.x.x "notes"` - GitHub release
