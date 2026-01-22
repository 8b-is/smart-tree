# CLAUDE.md

This Rust project uses Smart Tree for optimal AI context management.

## Project Stats
- Files: 594
- Directories: 81
- Total size: 203633625 bytes

## Essential Commands

```bash
# Build & Test
cargo build --release
cargo test -- --nocapture
cargo clippy -- -D warnings

# Smart Tree context
st -m context .          # Full context with git info
st -m quantum .           # Compressed for large contexts
st -m relations --focus main.rs  # Code relationships
```

## Key Patterns
- Always use `Result<T>` for error handling
- Prefer `&str` over `String` for function parameters
- Use `anyhow` for error context
- Run clippy before commits

## Smart Tree Integration
This project has hooks configured to automatically provide context.
The quantum-semantic mode is used for optimal token efficiency.
