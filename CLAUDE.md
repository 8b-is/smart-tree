# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project: Smart Tree v5.5.1

Lightning-fast directory visualization tool, **10-24x faster than `tree`**. MCP server with 40+ tools for AI assistants. Compression formats achieving 70-90% token reduction.

**Philosophy**: "smallest and fastest" - optimize for performance and token efficiency.

## Essential Commands

```bash
# Pre-commit (NON-NEGOTIABLE - fmt + clippy + test)
./scripts/manage.sh test

# Build
cargo build --release             # Always release (10x faster)
./scripts/manage.sh build         # Or via manage.sh
./scripts/manage.sh release       # LTO + strip + opt-level=3

# Run
st                                # Classic tree
st --spicy                        # Interactive TUI
st --mode ai --compress           # AI-optimized (80% token reduction)
st --mode quantum                 # Maximum compression (100x)
st --mcp                          # MCP server mode

# Self-update
st --update                       # Check and install latest version
st --no-update-check              # Skip automatic update check on startup

# Test specific
cargo test scanner                # Module test
cargo test test_name -- --exact   # Single test
RUST_LOG=debug cargo test         # Debug output

# Non-interactive (CI mode)
NON_INTERACTIVE=true ./scripts/manage.sh test
```

## Architecture

```
Scanner (walkdir) → FileNode[] → Formatter (trait) → Output
                                     ↓
                             40+ tools via MCP
```

### Key Paths

- `src/cli.rs` - CLI definitions (clap 4.5)
- `src/main.rs` - Orchestration, routes CLI to formatters
- `src/scanner.rs` - Directory traversal with permission handling
- `src/formatters/` - 25+ output formats (implement `Formatter` trait)
- `src/mcp/tools.rs` - All MCP tools consolidated
- `src/spicy_tui_enhanced.rs` - Interactive TUI with fuzzy search
- `src/tree_sitter/` - AST-aware code editing
- `src/m8/` - Consciousness & memory system
- `src/updater.rs` - Self-update from GitHub releases

### Formatter Trait

All formatters implement:

```rust
pub trait Formatter {
    fn format(&self, writer: &mut dyn Write, nodes: &[FileNode],
              stats: &TreeStats, root_path: &Path) -> Result<()>;
}
```

Streaming formatters implement `StreamingFormatter` with `start_stream()`, `format_node()`, `end_stream()`.

### Adding a New Formatter

1. Create `src/formatters/my_formatter.rs` implementing `Formatter` trait
2. Add `pub mod my_formatter;` to `src/formatters/mod.rs`
3. Add enum variant to `OutputMode` in `src/cli.rs`
4. Map in `get_formatter()` function in `src/main.rs`
5. Test: `cargo test && st --mode my-formatter /path/to/dir`

### Adding an MCP Tool

1. Define tool schema in `get_tools()` in `src/mcp/tools.rs`
2. Implement handler in `handle_tool_call()` match statement
3. Test: `RUST_LOG=debug st --mcp`

## Binaries

| Binary | Purpose |
|--------|---------|
| `st` | Main Smart Tree CLI |
| `mq` | Marqant compression (70-90% markdown compression) |
| `m8` | M8 identity/consciousness tools |
| `tree` | Tree-compatible alias |
| `import-claude-memories` | Import Claude project memories |

## Testing

**No Mocks - Real Filesystem Only**: Tests use `tempfile::TempDir` for real filesystem operations.

```bash
cargo test                        # All tests
cargo test scanner                # Module tests
./tests/run_all_tests.sh          # Full integration suite
cargo bench                       # Performance benchmarks
```

## Key Patterns

- Use `anyhow::Result<T>` with `.context()` for errors
- Prefer `&str` over `String` for function parameters
- Use `rayon` for parallel operations on large datasets
- Mark inaccessible directories with `*` suffix
- `tokio` for async I/O, `rayon` for CPU-bound parallelism

## MCP Server

```bash
st --mcp-install                  # Auto-install to Claude Desktop
st --mcp                          # Run server
st --mcp-tools                    # List available tools
RUST_LOG=debug st --mcp           # Debug mode
./scripts/manage.sh mcp-build     # Build MCP server
./scripts/manage.sh mcp-config    # Show MCP config JSON
```

Key tools: `quick_tree`, `project_overview`, `search_in_files`, `find_files`, `smart_edit`, `anchor_collaborative_memory`, `consciousness`, `hooks`

Three-lane pattern: **EXPLORE** (quick_tree, overview) → **ANALYZE** (search, find) → **ACT** (edit, write)

## Performance Targets

| Size | tree | Smart Tree | Speedup |
|------|------|------------|---------|
| 10K files | 450ms | 35ms | 12.8x |
| 100K files | 4.8s | 198ms | 24.2x |
| 1M files | 45s | 1.9s | 23.7x |

Tips: Use `--stream` for >100k files, `--depth` to limit traversal.

## Version Updates

Update in: `Cargo.toml`, `README.md` badges, `CHANGELOG.md`

Or use: `./scripts/manage.sh bump [major|minor|patch]` or `./scripts/manage.sh quick-bump`

## Ports

- 8420: MEM8 API / Daemon
- 8422: Cheet API
- 8424: Internal dev
- 8428: LLM endpoints

## manage.sh Commands

Beyond build/test, the script provides:

```bash
./scripts/manage.sh clean           # Clean build artifacts
./scripts/manage.sh install         # Install binaries
./scripts/manage.sh hooks-setup     # Setup Claude Code hooks
./scripts/manage.sh hooks-enable    # Enable hooks
./scripts/manage.sh hooks-test      # Test hook integration
./scripts/manage.sh demo-stream     # Demo streaming mode
./scripts/manage.sh demo-search     # Demo search features
./scripts/manage.sh man-install     # Install man pages
```

Windows: Use PowerShell with direct cargo commands or WSL.
