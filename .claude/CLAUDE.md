# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Quick Commands

```bash
# Pre-commit (REQUIRED)
./scripts/manage.sh test          # cargo test + clippy -D warnings + fmt --check

# Build
cargo build --release             # Core binary
cargo build --release --features tui       # With Spicy TUI
cargo build --release --features full      # Everything

# Single test
cargo test test_name -- --exact --nocapture

# MCP server
cargo run --release -- --mcp
```

## Architecture

Smart Tree is a 100% Rust directory visualization tool with AI optimization.

**Core flow**: `CLI (main.rs) → Scanner (scanner.rs) → Formatters (formatters/)`

**Key modules**:
- `src/mcp/` - 30+ JSON-RPC tools for AI assistants
- `src/mem8/` - Wave-based memory persistence
- `src/formatters/` - 25+ output formats (classic, ai, quantum, marqant)
- `src/smart/` - NLP, relevance scoring, git relay
- `src/web_dashboard/` - Browser-based terminal

**Binaries**: `st` (main), `mq` (marqant), `m8` (mem8), `n8x` (nexus)

## Code Patterns

- Error handling: `anyhow::Result<T>` with `.context()`
- Prefer `&str` over `String` for parameters
- Async: `tokio` runtime, use `#[tokio::test]` for async tests
- Tests use real filesystem (no mocks)

## Feature Flags

```toml
tui = [...]      # Spicy TUI mode (ratatui)
candle = [...]   # Local LLM inference
full = ["tui", "candle"]
```
