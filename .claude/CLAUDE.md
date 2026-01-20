# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Quick Reference

See the root `CLAUDE.md` for full documentation.

## Commands

```bash
./scripts/manage.sh test          # Pre-commit: fmt + clippy + test
cargo build --release             # Build (always release)
st --spicy                        # Interactive TUI
st --mcp                          # MCP server mode
cargo test test_name -- --exact   # Single test
```

## Architecture

```
Scanner → FileNode[] → Formatter (trait) → Output
                            ↓
                      30+ MCP tools
```

- `src/cli.rs` - CLI definitions
- `src/scanner.rs` - Directory traversal
- `src/formatters/` - Output formats (implement `Formatter`)
- `src/mcp/tools.rs` - MCP tools

## Patterns

- `anyhow::Result<T>` with `.context()` for errors
- `rayon` for parallel, `tokio` for async I/O
- Real filesystem tests only (no mocks)
