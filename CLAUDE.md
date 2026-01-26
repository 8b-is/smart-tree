# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
# Pre-commit (REQUIRED - zero clippy warnings enforced)
./scripts/manage.sh test          # cargo test + clippy -D warnings + fmt --check

# Build variants
cargo build --release                        # Core CLI (~29MB)
cargo build --release --features tui         # With Spicy TUI
cargo build --release --features full        # Everything (tui + candle)

# Run single test
cargo test test_name -- --exact --nocapture

# Run with logging
RUST_LOG=debug cargo test -- --nocapture

# Run MCP server
cargo run --release -- --mcp

# Smoke test CLI
cargo run --bin st -- --help
```

## Architecture

**Smart Tree** is a fast, AI-optimized directory visualization tool with MCP server capabilities.

### Core Pipeline: Scanner → Formatter

1. **Scanner** (`src/scanner.rs`): Directory traversal engine. Respects `.gitignore`, handles symlinks, marks inaccessible dirs with `*`.

2. **Formatters** (`src/formatters/`): 25+ output formats implementing the `Formatter` trait.
   - `classic` - Traditional tree
   - `ai` - 80% token reduction
   - `quantum` - 100x+ compression via wave functions
   - `marqant` - Markdown compression

3. **MCP Server** (`src/mcp/`): 30+ JSON-RPC tools for AI assistants.
   - Main tools: `src/mcp/tools/` (modular tool implementations)
   - Smart edit: `src/mcp/smart_edit.rs` (AST-aware editing via tree-sitter)
   - Memory persistence: `~/.mem8/`

### Key Modules

| Module | Purpose |
|--------|---------|
| `src/main.rs` | CLI entry point |
| `src/cli.rs` | Clap argument definitions |
| `src/scanner.rs` | Core directory traversal |
| `src/formatters/mod.rs` | Formatter trait & registry |
| `src/mcp/mod.rs` | MCP server entry |
| `src/mem8/` | Wave-based consciousness/memory system |
| `src/web_dashboard/` | Web UI with PTY terminal |
| `src/spicy_tui_enhanced.rs` | Interactive TUI (feature-gated) |

### Binaries

- `st` - Main CLI (thin, auto-starts daemon)
- `std` - ST Daemon (binary protocol, Unix socket, always-on brain)
- `mq` - Marqant markdown compressor
- `m8` - MEM8 consciousness tools
- `n8x` - Nexus Agent for AI-human orchestration

### ST Daemon Architecture

Two-product system:
- `st` routes through `std` daemon when available (auto-starts if not running)
- `std` listens on Unix socket (`/run/user/$UID/st.sock`) with binary protocol
- Use `--no-daemon` to run `st` standalone

**Binary Protocol** (`st-protocol` crate):
- Control ASCII (0x00-0x1F) as opcodes
- Frame format: `[verb 1B][payload N bytes][0x00 END]`
- Verbs: PING, SCAN, FORMAT, SEARCH, REMEMBER, RECALL, FORGET, M8_WAVE

### Adding a New Formatter

1. Create `src/formatters/my_format.rs` implementing `Formatter` trait
2. Add module to `src/formatters/mod.rs`
3. Add CLI variant in `src/cli.rs`
4. Wire in format dispatch in `src/main.rs`

## Code Patterns

**Error handling**: Use `anyhow::Result<T>` with `.context()`:
```rust
use anyhow::{Context, Result};

fn process(path: &str) -> Result<String> {
    std::fs::read_to_string(path)
        .context("Failed to read file")?
}
```

**Function parameters**: Prefer `&str` over `String`.

**Async**: Uses `tokio` runtime. Async tests use `#[tokio::test]`.

**Logging**: Use `tracing` macros.

**Naming**: snake_case for files/modules, PascalCase for types, kebab-case for CLI flags.

**Comments**: The codebase uses a fun "Cheet" persona style - keep it when extending existing commented sections.

## Feature Flags

```toml
default = []
tui = ["ratatui", "crossterm", "syntect", "artem"]  # Spicy TUI
candle = ["candle-core", "candle-transformers", ...] # Local LLM
full = ["tui", "candle"]
```

Web dashboard is always included (no feature flag).

## Testing

```bash
cargo test                              # All tests
cargo test scanner                      # Module tests
cargo test --test mcp_integration       # Integration tests
```

Tests use real filesystem operations (no mocks). Test fixtures live in `test/`, `test_files/`, and `test-dirs/`.

## Scripts

`./scripts/manage.sh` handles most tasks:
- `test` - Full pre-commit checks
- `build release` - Release build
- `mcp-run` - Start MCP server
- `bump patch|minor|major` - Version management

## Security

Run `st --cleanup` after installs and `st --security-scan .` before releases to detect risky MCP hooks or supply-chain patterns.
