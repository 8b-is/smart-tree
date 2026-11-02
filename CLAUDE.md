# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Smart Tree (`st`) is a fast, AI-friendly directory visualization tool written in Rust. It provides 30+ MCP tools for AI assistants and is 10-24x faster than traditional `tree`.

## Build & Development Commands

```bash
cargo build                    # Debug build
cargo build --release          # Release build (LTO enabled, stripped)
cargo test                     # Run all tests
cargo test --release           # Run performance tests
cargo test <test_name>         # Run a single test
cargo run --bin st -- --help   # Run locally without installing
cargo fmt                      # Format code
cargo clippy -- -D warnings    # Lint (zero-warning policy)
./scripts/manage.sh test       # Pre-commit bundle: test + clippy + fmt --check
./scripts/build-and-install.sh # Build and install locally (clears shell cache)
```

Run `./scripts/manage.sh test` before opening a PR — it is the canonical pre-commit gate. Async tests use `#[tokio::test]`; for verbose diagnostics use `RUST_LOG=debug cargo test -- --nocapture`.

After rebuilding, if `st --version` hangs, run `hash -r` to clear your shell's binary cache.

## Architecture

**Thin-client + daemon model**: The `st` CLI binary is a thin client that spawns an async daemon (`std`) for heavy lifting and persistent state. The daemon exposes an HTTP API on port **28428** (default).

### CLI → Daemon Flow

Most `st` commands follow this path:
1. `main.rs` parses CLI args, initializes logging
2. **Exclusive modes** run locally and exit: `--mcp`, `--version`, `--completions`, `--man`, `--security-scan`, `--terminal`, `--dashboard`, `--http-daemon`, `--guardian-daemon`, daemon control flags
3. **Everything else** (the common case): auto-starts daemon if needed, builds a `CliScanRequest` (defined in `daemon_cli.rs`), sends HTTP POST to `/cli/scan`, prints response

### Daemon Communication

- **CLI ↔ Daemon**: HTTP/JSON on port 28428, authenticated via Bearer token stored at `~/.st/daemon.token`
- **Daemon ↔ Daemon**: `st-protocol` binary wire protocol (6502-inspired opcodes) over Unix sockets — this is NOT used for CLI-daemon communication
- **Auto-spawn**: `DaemonClient::ensure_running()` spawns daemon in background with `setsid` (Unix) or `DETACHED_PROCESS` (Windows), retries with exponential backoff (5 attempts, 100ms→1600ms)

### MCP Integration (two modes)

- **Stdio** (`st --mcp`): JSON-RPC 2.0 over stdin/stdout, used by Claude Desktop and other MCP clients. Entry: `McpServer::run_stdio()` in `src/mcp/mod.rs`
- **HTTP** (via daemon): REST endpoints at `/mcp/tools/list`, `/mcp/tools/call`, etc. Entry: `src/web_dashboard/mcp_http.rs`
- Tool consolidation: Config flag `use_consolidated_tools` reduces 50+ tools to ~15 grouped tools to save context tokens

### Binaries (`src/bin/`)
- **`st`** — Main CLI (thin client), entry point in `src/main.rs`
- **`std`** — Always-on daemon service
- **`mq`** — Marqant markdown compression tool
- **`m8`** — Memory management tool
- **`n8x`** — Nexus Agent (tree orchestrator)
- **`import-claude-memories`** — Memory import utility

### Key Modules

| Domain | Key Files | Purpose |
|--------|-----------|---------|
| Scanner | `scanner.rs`, `scanner_interest.rs`, `scanner_safety.rs`, `scanner_state.rs` | Filesystem traversal, interest scoring, change detection |
| Formatters | `src/formatters/` | Output format handlers (classic, AI-optimized, quantum, markdown, JSON, CSV) |
| MCP | `mcp.rs`, `src/mcp/` | Model Context Protocol integration (30+ tools, stdio/HTTP) |
| Daemon | `daemon.rs`, `daemon_client.rs`, `daemon_cli.rs` | HTTP API server for AI context |
| Proxy | `src/proxy/` | LLM proxy with OpenAI-compatible API |
| Memory | `mem8/`, `memory_manager.rs`, `mega_session_manager.rs`, `m8_*.rs` | Wave-based memory system, session persistence |
| TUI | `spicy_tui_enhanced.rs`, `spicy_fuzzy.rs`, `src/terminal/` | Interactive terminal UIs |
| Web | `src/web_dashboard/` | Browser-based dashboard with real PTY |
| Security | `security_scan.rs`, `ai_guardian.rs` | Supply chain scanning, prompt injection protection |
| Smart Tools | `src/smart/` | Context-aware AI tools |
| Collaboration | `src/collab/` | Multi-AI collaboration features |
| Compression | `compression_manager.rs`, `dynamic_tokenizer.rs`, `st_tokenizer.rs` | Smart compression, tokenization |
| CLI | `cli.rs` | clap-based argument parsing (60+ options) |
| Config | `config.rs` | Configuration management |

### Workspace Members
- **`st-protocol/`** — Binary wire protocol (6502-inspired) for daemon-to-daemon communication only
- **`expert_prompt_engineer/`** — Prompt engineering workspace
- **`marqant/`** — Git submodule for markdown compression (excluded from workspace)

## Configuration & Environment

### Config files
- `~/.st/config.toml` — Main config (API keys, model preferences, daemon settings, safety)
- `~/.st/daemon.token` — Auth token (auto-generated on first daemon run)
- `.aye_consciousness.m8` — Per-directory consciousness state (persistent AI context)

### Key environment variables
| Variable | Purpose | Default |
|----------|---------|---------|
| `ST_TOKEN_PATH` | Override auth token file location | `~/.st/daemon.token` |
| `ST_DEFAULT_MODE` | Default output format | `smart` |
| `ST_SESSION_AWARE` | Enable MCP session-aware init | disabled |
| `MCP_DEBUG` | Show MCP startup messages | disabled |
| `RUST_LOG` | Tracing log level | `info` |

### Systemd services
- `systemd/smart-tree-daemon@.service` — User-level daemon (one per user)
- `systemd/smart-tree-daemon.service` — System-level daemon
- `systemd/smart-tree-guardian.service` — System-wide protection daemon

## Code Conventions

- **Error handling**: `anyhow::Result<T>` throughout; `thiserror` for typed errors
- **Async**: `tokio` runtime with full features
- **HTTP**: `axum` framework
- **Git ops**: `gix` crate
- **Serialization**: `serde` for all data structures
- **Terminal UI**: `ratatui` + `crossterm`
- **Commit messages**: `type: short description` (types: feat, fix, docs, style, refactor, test, chore)

## Feature Flags

- Default features: none (minimal build)
- `candle` — Local LLM support
- `voice` — Voice features
- `full` — All features enabled
