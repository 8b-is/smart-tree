# CLAUDE.md - Smart Tree v6.5.1

This Rust project uses Smart Tree for optimal AI context management.

## What's New in 6.5.1
- **Security Scanner**: `--security-scan` detects supply chain attack patterns
- **Pattern Detection**: IPFS/IPNS injection, fake verification, known risk packages (claude-flow)
- **Aggressive Mode**: Ignores .gitignore to scan node_modules and all hidden directories
- **Risk Classification**: Critical/High/Medium/Low with context-aware adjustment

## Previous: 6.2.0
- **Session Persistence**: `SessionStart`/`SessionEnd` hooks auto-save and restore context
- **Smart Restore**: `--claude-restore` only shows relevant, recent context (24h window)
- **Feature Gates**: TUI and Dashboard now optional (`--features tui`, `--features dashboard`)

## Essential Commands

```bash
# Build & Test
cargo build --release
cargo test -- --nocapture
cargo clippy -- -D warnings

# Smart Tree context
st -m context .          # Full context with git info
st -m quantum .           # Compressed for large contexts
st --claude-restore      # Restore previous session context

# Session management (automatic via hooks)
st --claude-save         # Save session context before ending
```

## Key Patterns
- Always use `Result<T>` for error handling
- Prefer `&str` over `String` for function parameters
- Use `anyhow` for error context
- Run clippy before commits

## Feature Flags
```bash
cargo build --release                    # Core only (~29MB)
cargo build --release --features tui     # With terminal UI
cargo build --release --features full    # All features
```

## Smart Tree Integration
This project has hooks configured to automatically provide context.
The quantum-semantic mode is used for optimal token efficiency.
