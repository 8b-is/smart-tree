# CLAUDE.md - Smart Tree v5.4.0 Comprehensive Development Guide

**Location**: `/aidata/ayeverse/smart-tree`
**Language**: 100% Rust (173 source files)
**Binary Size**: ~11MB (highly optimized)
**Latest Version**: 5.4.0

## Quick Start for Returning Claude Instances

If this is your second+ session on smart-tree, use this section:

```bash
# Get back up to speed (5 seconds)
st -m context .                    # See what changed since last time
st -m quantum .                    # Super-compressed project overview

# Build & Test (follows pre-commit checklist automatically)
./scripts/manage.sh test           # Runs: cargo fmt + clippy + cargo test

# Run MCP server (for Claude Desktop integration)
./scripts/manage.sh mcp-run
```

---

## Project Overview

**Smart Tree** is a lightning-fast, AI-friendly directory visualization tool built in Rust.

**Key Metrics**:
- 10-24x faster than traditional `tree` command
- 30+ MCP tools for AI integration
- Multiple compression formats (AI, Quantum, Marqant)
- Spicy TUI mode for interactive exploration
- Real-time SSE streaming support
- 25+ output format types

**Architecture**: Single binary (`st`) that can also output `mq` (Marqant compression) and `m8` (MEM8 consciousness tools).

---

## Essential Build & Test Commands

### Pre-Commit Checklist (REQUIRED)

Before any git commit, always run:

```bash
./scripts/manage.sh test
```

This runs (in order):
1. `cargo test` - All unit tests
2. `cargo clippy -- -D warnings` - Strict linting (no warnings allowed!)
3. `cargo fmt -- --check` - Format verification

**Why this matters**: Smart Tree has strict quality gates. Clippy warnings will fail CI/CD.

### Standard Build Commands

```bash
# Release build (10x faster runtime, what you almost always want)
cargo build --release

# Debug build (slower runtime, better debugging symbols)
cargo build

# Release build with specific features
cargo build --release --features "std"

# Quick format + lint check
cargo fmt && cargo clippy -- -D warnings
```

### Test-Specific Commands

```bash
# Run all tests with output (great for debugging)
cargo test -- --nocapture

# Test a specific module
cargo test scanner                 # Tests in scanner.rs
cargo test formatters              # Tests in formatters/

# Single test by name
cargo test test_quantum -- --exact --nocapture

# Integration tests
cargo test --test mcp_integration -- --nocapture

# Benchmarks (if any)
cargo bench                        # Criterion benchmarks if configured
```

### Using manage.sh (Recommended)

The project includes a powerful `scripts/manage.sh` that handles most tasks:

```bash
# Interactive menu
./scripts/manage.sh menu           # Launch interactive setup

# Build
./scripts/manage.sh build release  # Release mode (can also specify debug)
./scripts/manage.sh build          # Interactive menu

# Test (recommended way)
./scripts/manage.sh test           # Runs full suite + clippy + fmt

# Run the tool
./scripts/manage.sh run            # Interactive arguments
./scripts/manage.sh run -- . -d 3  # Direct args: analyze . with depth 3

# Install binary
./scripts/manage.sh install        # Install to /usr/local/bin (may need sudo)

# Version management
./scripts/manage.sh bump patch     # v5.4.0 → v5.4.1
./scripts/manage.sh bump minor     # v5.4.0 → v5.5.0
./scripts/manage.sh release v6.0.0 "Major feature release"

# MCP/Server commands
./scripts/manage.sh mcp-run        # Run as MCP stdio server
./scripts/manage.sh mcp-config     # Show Claude Desktop config
./scripts/manage.sh mcp-tools      # List all 30+ tools

# Demos
./scripts/manage.sh demo-stream    # Show streaming mode
./scripts/manage.sh demo-search    # Show content search
./scripts/manage.sh demo-relations # Show code relations
```

Use `-n` flag for non-interactive mode:
```bash
NON_INTERACTIVE=true ./scripts/manage.sh build release
```

---

## Architecture Overview

### Core Design Pattern

Smart Tree uses a **Scanner → Formatter** pipeline:

1. **Scanner** (`src/scanner.rs`): Traverses directories, collects metadata
   - Respects `.gitignore` patterns
   - Handles symlinks safely
   - Permission-aware (marks inaccessible with `*`)
   - ~1800 lines of core logic

2. **Formatters** (`src/formatters/`): Convert scan results to output
   - 25+ different formats supported
   - Each formatter implements `Formatter` trait
   - Composable (formatters can wrap other formatters)

3. **MCP Server** (`src/mcp/`): Exposes tools via Model Context Protocol
   - JSON-RPC stdio server
   - 30+ tools for AI assistants
   - Consciousness persistence (`~/.mem8/`)

### Directory Structure

```
src/
├── main.rs                           # CLI entry point (clap 4.5)
├── lib.rs                            # Module declarations
├── scanner.rs                        # Directory traversal (core logic)
├── terminal.rs                       # Smart Tree Terminal Interface
│
├── formatters/                       # 25+ output formats
│   ├── mod.rs                        # Formatter trait definition
│   ├── classic.rs                    # Traditional tree view
│   ├── ai.rs                         # Token-efficient AI format (80% smaller!)
│   ├── quantum.rs                    # Wave-based compression (100x!)
│   ├── marqant.rs                    # Markdown compression
│   ├── hex.rs                        # Hex-encoded AI format
│   ├── json.rs, csv.rs, tsv.rs      # Data formats
│   ├── stats.rs, digest.rs           # Metadata summaries
│   ├── semantic.rs                   # Semantic grouping
│   ├── relations.rs                  # Code relationship analysis
│   ├── sse.rs                        # Server-Sent Events streaming
│   └── ... (18 more formatters)
│
├── mcp/                              # Model Context Protocol
│   ├── mod.rs                        # Server entry point
│   ├── tools_consolidated.rs         # 30+ tool definitions
│   ├── smart_edit.rs                 # AST-aware code editing
│   ├── unified_watcher.rs            # Real-time file monitoring
│   ├── consciousness/                # Memory persistence
│   └── ... (20+ supporting modules)
│
├── spicy_tui.rs                      # Interactive TUI mode
├── spicy_fuzzy.rs                    # Fuzzy matching for TUI
│
├── compression_manager.rs            # Global compression control
├── content_detector.rs               # Detects file types
├── relations.rs                      # Code relationship analyzer
├── semantic.rs                       # Semantic analysis
├── inputs.rs                         # Universal input adapters
├── tokenizer.rs                      # Smart tokenization
│
└── bin/                              # Additional binaries
    ├── mq.rs                         # Marqant markdown compressor
    ├── m8.rs                         # MEM8 consciousness tools
    ├── tree.rs                       # Tree command alternative
    └── import_claude_memories.rs     # Claude context import
```

### Key File Count & Sizes

- **Total Source Files**: 173 Rust files
- **Main Binary Target**: `st` (11MB release)
- **Additional Binaries**: `mq`, `m8`, `tree`
- **Tests**: 10 integration test files
- **Dependencies**: ~140 (see Cargo.toml)

### Critical Dependencies

```toml
# CLI & Async
clap 4.5              # Argument parsing
tokio 1.42            # Async runtime with full features
async-trait           # Async trait support

# Data Processing
serde/serde_json      # Serialization
regex/globset         # Pattern matching & gitignore support
walkdir 2.5           # Directory traversal
rayon                 # Parallel processing

# Compression & Encoding
flate2                # Zlib compression
base64                # Base64 encoding
bincode               # Binary serialization

# AI/ML Integration
tree-sitter + 10 language modules  # AST parsing
fuzzy-matcher         # Fuzzy search for TUI
syntect               # Syntax highlighting

# Terminal UI
ratatui 0.25          # Terminal UI framework (TUI mode)
crossterm             # Terminal manipulation
termimad              # Markdown rendering
colored               # Color output

# MCP/Server
axum 0.7              # Web framework (WebSocket support)
hyper 1.7             # HTTP protocol
reqwest               # HTTP client

# File operations
notify 6.1            # File system watching
gix 0.73              # Git operations (will become g8t)

# Utilities
chrono                # Date/time
humansize 2.1         # Human-readable sizes
uuid                  # Unique identifiers
```

---

## Testing Patterns & Practices

### Test Organization

```
tests/
├── test_unified_integration.rs   # Integration tests
├── test_smart_edit.rs            # Smart Edit tool tests
├── test_claude_integration.rs     # Claude-specific integration
├── mcp_integration.rs            # MCP server tests
└── ...
```

### How to Run Tests

```bash
# All tests (REQUIRED before commit)
cargo test

# Specific test file
cargo test --test test_smart_edit

# Single test
cargo test test_scanner_basic -- --exact --nocapture

# With logging
RUST_LOG=debug cargo test -- --nocapture

# Benchmarks
cargo bench
```

### Testing Philosophy

- **No Mock Data**: Smart Tree tests use real file system operations
- **Permission Handling**: Tests verify `*` markers for inaccessible directories
- **Large Directory Tests**: Tests include 1M+ file scenarios
- **Async Testing**: Uses tokio runtime for async code
- **Property-Based**: Some tests use quickcheck-style randomization

### Common Test Patterns You'll See

```rust
#[test]
fn test_scanner_respects_gitignore() {
    // Create temp directory with .gitignore
    // Verify ignored files are marked
}

#[tokio::test]
async fn test_mcp_tool_execution() {
    // Async test for MCP tools
}

#[test]
fn test_formatter_output() {
    // Verify formatter produces expected format
}
```

---

## Formatter Architecture (Core to Smart Tree)

### Implementing a New Formatter

All formatters implement the `Formatter` trait:

```rust
pub trait Formatter {
    fn format(&self, nodes: &[FileNode]) -> Result<String>;
    fn supports_streaming(&self) -> bool { false }
}
```

**Location**: `src/formatters/mod.rs` (trait definition)

**To add a new formatter**:

1. Create `src/formatters/my_format.rs`:
```rust
use crate::scanner::FileNode;
pub struct MyFormatter;

impl Formatter for MyFormatter {
    fn format(&self, nodes: &[FileNode]) -> Result<String> {
        // Implement your format
        Ok(String::new())
    }
}
```

2. Add to `src/formatters/mod.rs`:
```rust
pub mod my_format;
pub use my_format::MyFormatter;
```

3. Add to CLI in `src/main.rs`:
```rust
#[derive(ValueEnum)]
enum FormatterType {
    MyFormat,
    // ... others
}
```

4. Wire it in the format dispatch:
```rust
FormatterType::MyFormat => {
    let formatter = MyFormatter;
    formatter.format(&nodes)?
}
```

5. Test it:
```bash
cargo test formatters::
./scripts/manage.sh test
```

### Key Formatters

| Formatter | Purpose | Token Efficiency |
|-----------|---------|------------------|
| `classic` | Traditional tree view | 100% (baseline) |
| `ai` | AI-optimized output | 20% (5x smaller!) |
| `quantum` | Wave compression | <1% (100x+!) |
| `hex` | Hex-encoded for AI | 30% |
| `json` | Machine-readable | 60% |
| `stats` | Metadata only | 10% |
| `digest` | Ultra-compact summary | 5% |
| `semantic` | Grouped by type/purpose | 15% |
| `relations` | Code dependencies | 25% |
| `sse` | Real-time streaming | N/A (streaming) |
| `marqant` | Markdown compression | 10-30% |

**Performance Tips**:
- Use `--mode quantum` for massive directories (100x compression!)
- Use `--mode ai --compress` for LLM context (80% reduction)
- Use `--mode digest` for pre-analysis (5% of original)
- Use `--stream` for directories >100k files

---

## MCP (Model Context Protocol) Integration

### Running as MCP Server

```bash
# Start MCP server (stdio-based)
./scripts/manage.sh mcp-run

# Or directly
cargo run --release -- --mcp

# List all available tools
./scripts/manage.sh mcp-tools

# Show Claude Desktop config
./scripts/manage.sh mcp-config
```

### Available MCP Tools (30+)

**Project Analysis**:
- `overview` - Quick 3-level project scan
- `find` - Powerful file discovery (type, pattern, recent, large)
- `search` - Content search with line context
- `analyze` - Deep analysis (git, statistics, semantic)
- `analyze:git_status` - Git-aware directory analysis

**Code Editing**:
- `edit` - AST-aware code editing (90% token reduction!)
- `edit:get_functions` - List all functions in file
- `edit:insert_function` - Add new function
- `edit:remove_function` - Remove function
- `edit:smart_edit` - Multiple edits at once

**Memory & Context**:
- `context:gather_project` - Full project context
- `context:collaboration_rapport` - Session history
- `context:suggest_insights` - AI suggestions
- `history:get_file` - File change history
- `history:get_project` - Project audit trail
- `memory:anchor` - Save breakthrough insight
- `memory:find` - Recall saved insights

**Advanced Features**:
- `sse` - Real-time directory monitoring
- `hooks` - Claude Code hook management (enable/disable/test)
- `server_info` - Smart Tree capabilities
- `verify_permissions` - Check directory access
- `unified_watcher` - Monitor multiple directories

### MCP Tool Usage Examples

```javascript
// Get quick overview
overview {mode:'quick', path:'.', depth:2}

// Find Rust files
find {type:'code', languages:['rust'], path:'src'}

// Search for TODOs
search {keyword:'TODO', include_content:true}

// Edit with AST awareness
edit {operation:'get_functions', file_path:'src/main.rs'}

// Gather full context
context {operation:'gather_project', project_path:'.'}

// Enable Claude Code hooks
hooks {operation:'set', hook_type:'UserPromptSubmit', enabled:true}

// Real-time monitoring
sse {path:'./src', heartbeat_interval:30}
```

### How MCP Tools Are Implemented

**Location**: `src/mcp/tools_consolidated.rs` (main tool implementations)

Each tool is a JSON-RPC method that:
1. Receives parameters as JSON
2. Executes the operation
3. Returns results as JSON

The server reads JSON-RPC requests from stdin and writes responses to stdout.

---

## Performance & Optimization

### Why Smart Tree is Fast

1. **Single-pass directory traversal**: `walkdir` crate does smart iteration
2. **Parallel processing**: `rayon` for independent operations
3. **Smart memory management**: `unsafe` blocks used sparingly, with comments
4. **Format-specific optimizations**: Each formatter optimizes for its use case
5. **Streaming mode**: Doesn't load everything in memory (`--stream` flag)

### Benchmarking

```bash
# Run benchmarks (if configured in Cargo.toml)
cargo bench

# Manual timing
time st . -m hex > /dev/null
time st . -m quantum > /dev/null
time st . -m ai > /dev/null

# Large directory test
time st /usr -m hex --depth 2 > /dev/null
```

### Expected Performance

| Directory Size | Time | vs tree | Speedup |
|---|---|---|---|
| 100 files | 2ms | 15ms | 7.5x |
| 10K files | 35ms | 450ms | 12.8x |
| 100K files | 198ms | 4.8s | 24.2x |
| 1M files | 1.9s | 45s | 23.7x |

**Key insight**: Speedup increases with directory size (better cache locality).

---

## Feature Flags & Configuration

### Cargo Features

```toml
[features]
default = ["std"]
std = []           # Standard library (always enabled)
alloc = []         # Allocator-specific (experimental)
mem8 = []          # MEM8 consciousness integration
```

### Environment Variables

```bash
# Logging
export RUST_LOG=debug        # Show debug logs
export RUST_LOG=info         # Info level
export RUST_LOG=trace        # Trace (very verbose)

# Smart Tree specific
export ST_DEFAULT_DEPTH=5    # Default tree depth
export ST_COLOR=always       # Force colors
export ST_NO_ICONS=1         # Disable emoji icons
export ST_MAX_FILE_SIZE=10M  # Skip large files

# Build
export CARGO_BUILD_JOBS=4    # Parallel jobs during build
```

### Config File

Create `~/.config/smart-tree/config.toml`:

```toml
[display]
default_depth = 5
show_hidden = false
use_icons = true
color_mode = "auto"

[performance]
max_buffer_size = "100MB"
thread_count = 8
use_streaming = true

[mcp]
enabled = true
port = 3000
```

---

## Development Workflows

### Adding a New Command-Line Option

1. Edit `src/main.rs` - Update `Cli` struct (clap parsing)
2. Add logic to handle the new flag
3. Test with: `cargo run --release -- --help`
4. Update docs in `README.md` and `docs/st-cheetsheet.md`

### Debugging with Smart Tree Itself

```bash
# Context on your changes
st -m context .

# Relationships in modified files
st -m relations --focus src/main.rs

# Search for your changes
st --search "let new_var" -m hex

# Watch directory for changes (SSE)
st --sse
```

### Running Tests During Development

```bash
# Quick test (no full cleanup)
cargo test scanner::test_

# With backtrace for panics
RUST_BACKTRACE=1 cargo test

# Test a specific feature
cargo test --features "std" --test mcp_integration
```

### Updating Dependencies

```bash
# Check for outdated dependencies
cargo outdated

# Update all (carefully!)
cargo update

# Update specific crate
cargo update --package serde

# Then test!
./scripts/manage.sh test
```

---

## Code Patterns & Best Practices

### Error Handling

Smart Tree uses `anyhow::Result<T>` throughout:

```rust
use anyhow::{Context, Result};

fn process_file(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read file")?;
    Ok(content)
}
```

**Always add context** with `.context()` - helps users debug.

### String vs str

- Use `&str` for function parameters
- Use `String` for owned values
- Never `.clone()` when you can use references

```rust
// Good
fn analyze(path: &str) -> Result<()> { }

// Avoid
fn analyze(path: String) -> Result<()> { }
```

### Async Code

Smart Tree uses `tokio` for async:

```rust
#[tokio::main]
async fn main() {
    // Async entry point
}

// In libraries
pub async fn scan_async(path: &Path) -> Result<Vec<FileNode>> {
    // Async operation
}
```

### Avoiding Panics

- Never unwrap unless you're 100% sure (use `.expect("reason")` instead)
- Use `?` operator for error propagation
- Mark intentional panics with comments explaining why

```rust
// Good
let val = map.get("key").context("missing key")?;

// Avoid
let val = map.get("key").unwrap();

// Acceptable with comment
let val = config.port.expect("port required in config");
```

### Comments & Documentation

Smart Tree has a fun commenting style (the "Cheet" persona). Keep it:

```rust
// Fun but informative comments explaining the why, not the what
// Example from scanner.rs:
// "You've found scanner.rs, the intrepid explorer and engine room of st."

/// Documentation for public items (use ///)
/// Multiple lines are fine for complex stuff
pub fn do_something() { }
```

---

## Debugging & Troubleshooting

### Common Issues

**Issue**: Tests fail with permission errors
```bash
# Solution: Run with elevated permissions or use test isolation
sudo cargo test
# Or check test directory permissions
ls -la /tmp/st_test_*
```

**Issue**: MCP server won't start
```bash
# Solution: Check for port conflicts
lsof -i :3000
# Or use debug logging
RUST_LOG=debug st --mcp
```

**Issue**: Slow compile times
```bash
# Solution: Use incremental compilation
export CARGO_INCREMENTAL=1
# Or use mold linker (faster than GNU ld)
# Install: sudo apt install mold
# Use: RUSTFLAGS="-C link-arg=-fuse-ld=mold" cargo build
```

**Issue**: Binary won't run after build
```bash
# Verify it was built
ls -lh target/release/st

# Try running directly
./target/release/st --version

# Check dependencies
ldd target/release/st
```

### Enabling Debug Output

```bash
# Maximum verbosity
RUST_LOG=trace cargo run --release -- . -m hex

# Specific module
RUST_LOG=st::scanner=debug cargo run --release -- .

# With backtraces
RUST_BACKTRACE=full cargo run -- .
```

### Profiling Performance

```bash
# Use Linux perf (if available)
cargo build --release
perf record -g ./target/release/st . -m quantum > /dev/null
perf report

# Or use flamegraph (install: cargo install flamegraph)
cargo flamegraph --bin st -- . -m quantum
```

---

## Git Workflow & Commits

### Pre-Commit Requirements (STRICT)

Before every commit:

```bash
./scripts/manage.sh test
```

This must pass with **zero clippy warnings**.

### Commit Message Format

Follow Conventional Commits:

```
feat: Add quantum compression mode
fix: Handle symlinks in scanner
docs: Update README with examples
refactor: Simplify formatter trait
perf: Optimize tree traversal

Body (optional):
- List changes
- Explain why
```

### Creating Releases

```bash
# Bump version (updates Cargo.toml + CLAUDE.md)
./scripts/manage.sh bump patch      # v5.4.0 → v5.4.1
./scripts/manage.sh bump minor      # v5.4.0 → v5.5.0
./scripts/manage.sh bump major      # v5.4.0 → v6.0.0

# Build and create GitHub release
./scripts/manage.sh release v5.5.0 "New quantum features"

# This will:
# 1. Create git tag
# 2. Build release artifacts
# 3. Upload to GitHub
# 4. Generate release notes
```

---

## Project Philosophy & Vision

### Core Principles

1. **Smallest and Fastest**: Every kilobyte counts, every millisecond matters
2. **AI-First Design**: Optimize for AI context windows (not human readability)
3. **Constraints = Creativity**: Limited memory? Use wave patterns. Limited tokens? Use compression.
4. **SID/VIC-II Wisdom**: Work within hardware constraints to achieve magic
5. **The Cheet's Humor**: Keep development fun and memorable

### Why Different Output Modes Matter

- **classic**: Human-readable (for terminals)
- **ai**: 80% smaller (for LLM context)
- **quantum**: 100x+ compression (for massive projects)
- **hex/json**: Machine parseable
- **sse**: Real-time monitoring

Each mode is optimized for its audience.

### Future Direction (Roadmap)

- Integration with MEM8 consciousness system
- WebRTC interface for real-time collaboration
- Python/JS SDKs for broader ecosystem
- Kubernetes operators for deployment
- Hot Tub visualization dashboard

---

## Quick Reference Card

### Commands You'll Use Most

```bash
# Development loop
cargo test && cargo fmt          # Quick test
./scripts/manage.sh test         # Full checklist (do this before commits!)

# Running st
st .                             # Classic view
st --mode quantum .              # Compressed
st --mode ai --compress .        # AI-optimized
st --mcp                         # Start MCP server
st --spicy                       # Interactive TUI

# MCP tools
st -m overview .                 # Quick scan
st -m find --type rs .           # Find Rust files
st -m search "TODO" .            # Search content
st -m analyze --mode semantic .  # Semantic analysis

# Building/Installing
./scripts/manage.sh build        # Build
./scripts/manage.sh install      # Install to /usr/local/bin
./scripts/manage.sh release v5.5.0 "Notes"  # Release

# Debugging
st -m context . > context.txt    # Save context
RUST_LOG=debug st --mcp          # Debug logging
```

### Pre-Commit Checklist

```bash
# Always do this before git commit
./scripts/manage.sh test

# Expected output:
# ✅ cargo test: All tests passed
# ✅ cargo clippy: No warnings (strict mode)
# ✅ cargo fmt: Code formatted
# Then you can commit!
```

---

## Useful Files to Know About

| File | Purpose |
|------|---------|
| `src/main.rs` | CLI entry point (clap 4.5 argument parsing) |
| `src/lib.rs` | Module declarations (65+ modules!) |
| `src/scanner.rs` | Core directory traversal engine |
| `src/formatters/mod.rs` | Formatter trait & registry |
| `src/mcp/mod.rs` | MCP server implementation |
| `scripts/manage.sh` | All build/test/release commands |
| `Cargo.toml` | Dependencies & build config |
| `tests/` | Integration tests |
| `docs/INDEX.md` | Full documentation index |
| `README.md` | Project overview |

---

## Need More Info?

- **Full Docs**: `st --help` or `docs/INDEX.md`
- **MCP Tools**: `./scripts/manage.sh mcp-tools`
- **Examples**: `./scripts/manage.sh examples`
- **Cheat Sheet**: `st --cheet`
- **Code Comments**: Start with "The Cheet" persona comments in source

---

**Made with Rust by the 8b-is team**  
*"Smallest and fastest."* - The Cheet
