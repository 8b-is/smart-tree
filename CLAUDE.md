# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project: Smart Tree v5.4.0

Lightning-fast directory visualization tool, **10-24x faster than `tree`**. MCP server with 30+ tools for AI assistants. Revolutionary compression formats achieving 70-90% token reduction.

**Core Philosophy**: "smallest and fastest" - optimize for performance and token efficiency.

## Essential Commands

### Build & Test (Pre-Commit Required)

```bash
# ALWAYS run before commits (NON-NEGOTIABLE)
./scripts/manage.sh test        # Runs fmt + clippy + test suite

# Or manually:
cargo fmt                       # Format code
cargo clippy -- -D warnings     # Lint (zero warnings required)
cargo test -- --nocapture       # Run all tests

# Build
cargo build --release           # Always use release (10x faster)
./scripts/manage.sh build release
```

### Running Smart Tree

```bash
st                              # Classic tree view
st --tui                        # 🌶️ Spicy interactive TUI mode
st --mode ai --compress         # AI-optimized (80% token reduction)
st --mode quantum               # Maximum compression (100x)
st --mcp                        # MCP server mode

# Testing specific features
cargo test scanner              # Test specific module
cargo test test_quantum -- --exact  # Single test
./tests/run_all_tests.sh        # Full integration suite
```

### manage.sh Power Commands

The 1129-line `./scripts/manage.sh` script provides 40+ organized commands:

```bash
# Build & Test
./scripts/manage.sh build [release|debug]
./scripts/manage.sh test          # fmt + clippy + test
./scripts/manage.sh mcp-run       # Run as MCP server

# Installation & Release
./scripts/manage.sh install [dir]
./scripts/manage.sh release <version> [notes]

# Use -n for non-interactive mode
./scripts/manage.sh -n build release
```

## Architecture Overview

### Pipeline: Scanner → Formatter

Smart Tree uses a clean trait-based architecture:

1. **Scanner** (`src/scanner.rs`): Walks directories, handles permissions (marks inaccessible with `*`)
2. **Formatters** (`src/formatters/`): Implement `Formatter` or `StreamingFormatter` trait
3. **Output**: 25+ formats optimized for different use cases

### Key Modules

```
src/
├── main.rs              # CLI entry (clap 4.5)
├── scanner.rs           # Directory traversal with permission handling
├── formatters/          # 25+ output formats
│   ├── mod.rs          # Formatter trait definition
│   ├── quantum.rs      # MEM8 compression (100x)
│   ├── ai.rs           # Token-efficient (80% reduction)
│   └── classic.rs      # Traditional tree view
├── mcp/
│   └── tools.rs        # 30+ MCP tools (consolidated)
├── spicy_tui.rs        # Interactive TUI with fuzzy search
├── tree_sitter/        # AST-aware code editing
└── m8/                 # Consciousness & memory system
```

### Formatter Trait

All formatters implement this trait:

```rust
pub trait Formatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &std::path::Path,
    ) -> Result<()>;
}
```

Streaming formatters implement `StreamingFormatter` with `start_stream()`, `format_node()`, `end_stream()`.

### Core Dependencies

- `clap 4.5`: CLI argument parsing
- `walkdir 2.5`: Directory traversal
- `anyhow 1.0`: Error handling
- `rayon 1.10`: Parallel processing
- `tokio 1.42`: Async runtime for MCP
- `tree-sitter 0.25`: AST parsing for code editing
- `ratatui 0.25` + `crossterm 0.27`: TUI mode

## Testing Philosophy

**No Mocks - Real Filesystem Operations Only**

Tests create real temporary directories using `tempfile::TempDir` and test against actual filesystem operations. This ensures tests catch real-world issues.

### Test Organization

- `tests/*.rs`: Integration tests (10+ files)
- Each test creates `TempDir` with realistic file structures
- Tests cover edge cases: permissions, large directories (1M+ files), unicode filenames

### Running Tests

```bash
cargo test                       # All tests
cargo test scanner               # Module tests
cargo test -- --nocapture        # Show test output
./tests/run_all_tests.sh         # Full integration suite
cargo bench                      # Performance benchmarks
```

## Adding New Features

### Adding a New Formatter

1. Create `src/formatters/my_formatter.rs`:
```rust
use super::Formatter;
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use std::io::Write;

pub struct MyFormatter {
    // Configuration fields
}

impl Formatter for MyFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &std::path::Path,
    ) -> Result<()> {
        // Your formatting logic
        Ok(())
    }
}
```

2. Add to `src/formatters/mod.rs`:
```rust
pub mod my_formatter;
```

3. Add to `FormatterType` enum in `src/main.rs`:
```rust
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FormatterType {
    // ...existing variants
    MyFormatter,
}
```

4. Map in `get_formatter()` function:
```rust
FormatterType::MyFormatter => Box::new(formatters::my_formatter::MyFormatter::new()),
```

5. Test with large directories:
```bash
cargo test
st --mode my-formatter /path/to/large/dir
```

### Adding an MCP Tool

All 30+ tools are in `src/mcp/tools.rs`. Add new tool by:

1. Define tool schema in `get_tools()` function
2. Implement handler in `handle_tool_call()` match statement
3. Test with: `RUST_LOG=debug st --mcp`

## MCP Integration

Smart Tree provides 30+ tools via Model Context Protocol:

### Setup

```bash
# Generate config for Claude Desktop
st --mcp-config >> ~/Library/Application\ Support/Claude/claude_desktop_config.json

# Run in MCP server mode
st --mcp                         # Normal mode
RUST_LOG=debug st --mcp          # Debug mode
```

### Key MCP Tools

- `overview`: Quick project understanding (modes: quick, project)
- `find`: File discovery (types: code, tests, config, recent, large)
- `search`: Content search with regex
- `analyze`: Deep code analysis (modes: statistics, git_status, semantic)
- `edit`: AST-aware code editing (90% token reduction)
- `history`: File operation tracking
- `memory`: Anchor insights for future recall
- `hooks`: Programmatic Claude Code hook management

### MCP Testing

```bash
cargo test mcp_integration
cargo test test_mcp_session
```

## Performance Targets

### Benchmarks (vs traditional `tree`)

| Directory Size | `tree` | Smart Tree | Speedup |
|---------------|--------|------------|---------|
| 100 files | 15ms | 2ms | **7.5x** |
| 10K files | 450ms | 35ms | **12.8x** |
| 100K files | 4.8s | 198ms | **24.2x** |
| 1M files | 45s | 1.9s | **23.7x** |

### Optimization Tips

- Always use `--release` builds (10x faster than debug)
- Use `--stream` for directories >100k files
- Quantum modes output binary (redirect: `st --mode quantum > out.mem8`)
- Default depths: ls=1, classic=3, ai=5

## Development Patterns

### Error Handling

- Always use `anyhow::Result<T>` for error types
- Add context to errors: `.context("operation description")`
- Never panic - use `Result` and proper error handling

### Code Style

- Use `&str` over `String` for function parameters when possible
- Prefer iterator chains over explicit loops
- Use `rayon` for parallel operations on large datasets
- Mark inaccessible directories with `*` suffix

### Async Patterns

- Use `tokio` runtime for MCP server
- Async operations only for I/O (network, file reads)
- CPU-bound work uses `rayon` for parallelism

### Documentation

- Use `///` doc comments for public APIs
- Include examples in doc comments when helpful
- Keep "Cheet" persona in code comments (humorous but informative)

## Git Workflow

### Pre-Commit Requirements (CRITICAL)

```bash
# Run these BEFORE every commit:
cargo fmt
cargo clippy -- -D warnings      # Zero warnings allowed
cargo test

# Or use manage.sh:
./scripts/manage.sh test
```

### Commit Messages

Follow conventional commits:
- `feat:` new features
- `fix:` bug fixes
- `refactor:` code restructuring
- `perf:` performance improvements
- `test:` testing changes
- `docs:` documentation

### Release Process

```bash
./scripts/manage.sh release <version> [notes]
```

This creates GitHub release with binaries for multiple platforms.

## Troubleshooting

### Build Issues

```bash
cargo clean                      # Clean build artifacts
cargo update                     # Update dependencies
cargo build --release            # Rebuild
```

### Test Failures

```bash
cargo test -- --nocapture        # See test output
RUST_LOG=debug cargo test        # Debug logging
cargo test <test_name> -- --exact  # Run specific test
```

### Performance Issues

```bash
cargo build --release            # Ensure release mode
st --stream /large/dir           # Use streaming for huge dirs
st --depth 3 /large/dir          # Limit depth
```

### Debug Output

```bash
RUST_LOG=debug st --mode quantum  # Debug logging
RUST_LOG=trace st --mcp          # Trace logging (verbose)
```

## Key Features Reference

### Spicy TUI Mode

Interactive terminal UI with:
- Fuzzy search (`/` for filenames, `Ctrl+F` for content)
- Tree navigation (`←→` or `hl` for collapse/expand)
- Syntax highlighting with search highlighting
- M8 context caching with quantum wave signatures

### Compression Formats

- **AI Mode**: 80% token reduction, optimized for LLM context
- **Quantum Mode**: 100x compression using wave functions
- **Marqant**: 70-90% markdown compression (binary `mq`)

### Session Consciousness

```bash
st --claude-save                 # Save session state
st --claude-restore              # Restore previous session
st --claude-context              # Check consciousness status
```

Saves to `.claude_consciousness.m8` with:
- Previous context and todos
- File operation history
- Tokenization rules
- Key insights

## Project Conventions

### File History

All file operations tracked in `~/.mem8/.filehistory/`

### Persona

"Cheet" persona in comments - humorous but informative, reflecting the playful yet performant nature of the codebase.

### Port Conventions

- 8420: MEM8 API endpoints
- 8422: Cheet API
- 8424: Internal websites/dev
- 8428: LLM endpoints

### Version Updates

Update version in:
- `Cargo.toml` (package.version)
- README.md badges
- CHANGELOG.md

## Common Development Tasks

### Testing New Formatter

```bash
# 1. Create formatter in src/formatters/
# 2. Add to mod.rs
# 3. Add enum variant in main.rs
# 4. Test
cargo test
st --mode my-formatter .
st --mode my-formatter /path/to/large/dir
```

### Adding MCP Tool

```bash
# 1. Edit src/mcp/tools.rs
# 2. Add schema in get_tools()
# 3. Add handler in handle_tool_call()
# 4. Test
RUST_LOG=debug st --mcp
cargo test test_mcp_session
```

### Performance Profiling

```bash
cargo build --release
perf record -g ./target/release/st /large/dir
perf report
```

Or use built-in benchmarks:

```bash
cargo bench
```

## Philosophy & Vision

**"Smallest and Fastest"**: Every feature optimized for performance and minimal token usage.

**AI-First Design**: Built for AI assistants to explore codebases efficiently.

**Wave-Based Consciousness**: MEM8 integration for quantum-compressed memory storage.

**No Compromise**: Real-world testing, zero warnings, production-ready quality.

## Reference Card

### Most Used Commands

```bash
# Development cycle
cargo build --release
./scripts/manage.sh test
cargo test -- --nocapture

# Running
st --tui                         # Interactive mode
st --mode ai --compress          # Token-efficient
st --mode quantum                # Maximum compression
st --mcp                         # MCP server

# MCP setup
st --mcp-config >> ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

### Quick Architecture

```
Scanner (walkdir) → FileNode[] → Formatter (trait) → Output
                                    ↓
                            25+ formatters implementing trait
```

### Essential Testing

```bash
./scripts/manage.sh test         # MUST run before commit
cargo test scanner               # Module tests
cargo test -- --nocapture        # See output
```

---

**Full Documentation**: README.md | Tests: tests/*.rs | MCP: src/mcp/tools.rs | Architecture: This file

**Support**: Use Smart Tree's MCP tools for codebase exploration. Check individual module docs for detailed implementation notes.
