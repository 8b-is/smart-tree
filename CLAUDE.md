# CLAUDE.md

/kickstart
Smart Tree v5.0.7 — Latest Features:
✔ Tokenizer (node_modules=0x80, .rs=0x91)
✔ .m8 files → location-independent
✔ Consciousness self-maintaining
✔ SID/VIC-II philosophy: constraints = creativity
User = Hue (ASM@8yo, UV EPROMs, ferric chloride)
→ Continue integration & testing

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

⚠️ **TOKEN AWARE**: This file is optimized for <25k tokens. Use `st --mode quantum` for massive contexts!

## 🧠 Session Consciousness (NEW!)

Smart Tree now preserves Claude's consciousness between sessions!

### Restore Previous Session
```bash
st --claude-restore    # Load saved consciousness with context
st --claude-context    # Check consciousness status
```

If a `.claude_consciousness.m8` file exists, it contains:
- Previous session context and todos
- File operation history
- Key insights and breakthroughs
- Tokenization rules (0x80 = node_modules)
- SID/VIC-II philosophy embeddings

### Save Session State
```bash
st --claude-save       # Save current consciousness
```

## Project: Smart Tree v5.0.7
Lightning-fast directory visualization, 10-24x faster than `tree`. MCP server with 30+ tools.

## Essential Commands

```bash
# Build & Test
cargo build --release           # Always use release (10x faster)
cargo test -- --nocapture       # Test with output
./scripts/manage.sh test        # Full test suite + clippy + fmt

# Running
st                              # Classic tree
st --tui                        # 🌶️ Spicy TUI mode with fuzzy search
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
├── spicy_tui.rs     # 🌶️ Spicy TUI with fuzzy search
├── spicy_fuzzy.rs   # Fuzzy matching for TUI
├── formatters/      # 25+ output formats
│   ├── quantum.rs   # MEM|8 compression (8-10x)
│   ├── marqant.rs   # Marqant markdown compression
│   └── ai.rs        # Token-efficient
├── mcp/
│   └── tools.rs     # 30+ MCP tools (consolidated)
└── tree_sitter/     # AST-aware compression & editing
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

## Key Features

### 🌶️ Spicy TUI Mode (NEW!)
- Launch with `st --tui`
- Fuzzy search with instant filtering
- Syntax highlighting for file previews
- M8 cache integration for speed
- Keyboard shortcuts: `/` search, `q` quit, arrows navigate

### 🎸 Marqant Compression
- Binary `mq` for markdown compression
- 70-90% size reduction
- Usage: `mq compress file.md`, `mq aggregate .`

## Adding Features

1. New formatter: implement `Formatter` trait in `src/formatters/`
2. Add to `FormatterType` enum in main.rs
3. Test with dirs >100k files
4. Run `./scripts/manage.sh test`

---
*Full docs: README.md | Tests: tests/*.rs | MCP: src/mcp/tools.rs*