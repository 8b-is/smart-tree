# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
# Build
cargo build --release          # Production build (binaries: st, mq, m8)
cargo build                    # Debug build

# Test
cargo test --lib               # Library tests only
cargo test scanner::tests      # Run specific test module
cargo test --lib -- --nocapture  # With stdout

# Lint & Format
cargo clippy -- -D warnings    # Must pass before commits
cargo fmt --check              # Check formatting
cargo fmt                      # Auto-format

# Run
./target/release/st --help     # CLI help
./target/release/st --mcp      # Run as MCP server
./target/release/st --spicy    # Interactive TUI mode
```

## Architecture Overview

Smart Tree is a Rust-based directory visualization tool with AI/LLM integration via Model Context Protocol (MCP).

### Core Data Flow

```
CLI (src/cli.rs) → Scanner (src/scanner.rs) → Formatter (src/formatters/*.rs) → Output
                                    ↓
                            FileNode tree
                                    ↓
                    MCP Server (src/mcp/) ← AI Assistants
```

### Key Modules

| Module | Purpose |
|--------|---------|
| `src/cli.rs` | 80+ CLI arguments via clap |
| `src/scanner.rs` | Directory traversal, `FileNode` tree building |
| `src/formatters/` | 22 output modes (ai, quantum, json, classic, etc.) |
| `src/mcp/` | MCP server with 30+ tools for AI assistants |
| `src/daemon.rs` | Always-on HTTP API service (port 8420) |
| `src/proxy/` | LLM proxy for OpenAI/Anthropic/Google APIs |
| `src/mem8/` | Binary consciousness format (.m8 files) |

### Output Modes (Formatters)

Each formatter implements the same trait pattern in `src/formatters/`:
- `ai.rs` - Hex-encoded for LLM token efficiency
- `quantum.rs` - Maximum compression (MEM8_QUANTUM format)
- `json.rs` - Structured JSON output
- `classic.rs` - Traditional tree view
- `digest.rs` - Hash + stats only (minimal tokens)

### MCP Tools Architecture

Tools are defined in `src/mcp/tools.rs` and `src/mcp/tools_consolidated.rs`:
- Discovery: `quick_tree`, `analyze_directory`, `project_overview`
- Search: `search_in_files`, `find_files`, `find_code_files`
- Memory: `anchor_collaborative_memory`, `find_collaborative_memories`
- Edit: `smart_edit`, `read` (AST-aware in `src/mcp/smart_edit.rs`)

### Binary Targets

- `st` - Main CLI (src/main.rs)
- `mq` - Marqant compression tool (src/bin/mq.rs)
- `m8` - MEM8 consciousness tool (src/bin/m8.rs)

## Code Patterns

### Error Handling
Use `anyhow::Result` with context:
```rust
.context("Failed to read directory")?
```

### Async
Tokio runtime for MCP server and daemon. Scanner is sync (walkdir).

### Adding a New Output Mode
1. Create `src/formatters/newmode.rs`
2. Implement the formatter function
3. Add variant to `OutputMode` enum in `src/cli.rs`
4. Wire up in `src/main.rs` match statement

### Adding a New MCP Tool
1. Add tool definition in `src/mcp/tools.rs`
2. Implement handler logic
3. Register in tool list
4. Tool appears automatically to AI clients

## Testing

```bash
# Quick functional test
./target/release/st --mode json --depth 1 . | python3 -m json.tool

# Compression comparison
./target/release/st --mode classic --depth 3 src/  # baseline
./target/release/st --mode quantum --depth 3 src/  # compressed

# MCP tools listing
./target/release/st --mcp-tools
```

## Key Files for Common Tasks

| Task | Files |
|------|-------|
| Add CLI flag | `src/cli.rs` |
| Change tree output | `src/formatters/*.rs` |
| Modify scanning | `src/scanner.rs` |
| Add MCP tool | `src/mcp/tools.rs`, `src/mcp/tools_consolidated.rs` |
| Change daemon API | `src/daemon.rs` |
| Modify compression | `src/compression_manager.rs`, `src/formatters/quantum.rs` |
