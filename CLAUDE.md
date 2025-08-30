# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

⚠️ **TOKEN AWARE**: This file is optimized for <25k tokens. Use `st --mode quantum` for massive contexts!

## Project: Smart Tree v4.8.8
Lightning-fast directory visualization, 10-24x faster than `tree`. MCP server with 30+ tools.

## Essential Commands

```bash
# Build & Test
cargo build --release           # Always use release (10x faster)
cargo test -- --nocapture       # Test with output
./scripts/manage.sh test        # Full test suite + clippy + fmt

# Running
st                              # Classic tree
st --mode ai --compress         # AI-optimized (80% smaller!)
st --mode quantum src/          # Maximum compression (100x)
st --mcp                        # MCP server mode

# Before commits
cargo fmt && cargo clippy -- -D warnings && cargo test
```

## Architecture (Key Files Only)

```
src/
├── main.rs          # CLI entry (clap 4.5)
├── scanner.rs       # Directory traversal (handles permissions with *)
├── formatters/      # 25+ output formats
│   ├── quantum.rs   # MEM|8 compression (8-10x)
│   └── ai.rs        # Token-efficient
├── mcp/            
│   └── tools.rs     # 30+ MCP tools (139KB!)
└── tree_sitter_quantum.rs  # AST-aware compression
```

## Testing

```bash
cargo test scanner              # Test specific module
cargo test test_quantum -- --exact  # Single test
./tests/run_all_tests.sh       # Full suite
```

## MCP Setup

```bash
st --mcp-config >> ~/Library/Application\ Support/Claude/claude_desktop_config.json
RUST_LOG=debug st --mcp        # Debug mode
```

## Performance Tips

- Use `--stream` for dirs >100k files
- Quantum modes output binary (redirect: `st --mode quantum > out.mem8`)
- Default depths: ls=1, classic=3, ai=5

## Project Patterns

- Uses `anyhow` for errors
- Marks inaccessible dirs with `*`
- File history in `~/.mem8/.filehistory/`
- Humorous "Cheet" persona in comments
- Focus: "smallest and fastest"

## manage.sh Commands

```bash
build [release|debug]   # Build project
test                    # Test + clippy + fmt
mcp-run                 # Run as MCP server
install [dir]           # Install binary
release <ver> [notes]   # GitHub release
```

Use `-n` for non-interactive mode.

## Adding Features

1. New formatter: implement `Formatter` trait in `src/formatters/`
2. Add to `FormatterType` enum in main.rs
3. Test with dirs >100k files
4. Run `./scripts/manage.sh test`

---
*Full docs: README.md | Tests: tests/*.rs | MCP: src/mcp/tools.rs*