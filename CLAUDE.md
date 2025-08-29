# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Smart Tree (`st`) is a blazingly fast, AI-friendly directory visualization tool written in Rust. It's designed as an intelligent alternative to the traditional `tree` command, optimized for both human readability and AI token efficiency.

**Current Version**: v4.8.8

### Key Features
- **10-24x faster** than traditional tree commands
- **30+ output formats** including AI-optimized and quantum compression modes
- **MCP server** with 30+ tools for AI assistants
- **File history tracking** for complete audit trail of AI operations
- **SSE support** for real-time directory monitoring

## Development Commands

### Building
```bash
cargo build --release      # Optimized release build (recommended)
cargo build               # Debug build

# Using manage script (preferred - includes format and lint checks)
./scripts/manage.sh build
```

### Testing
```bash
# Run all tests with output
cargo test -- --nocapture

# Run specific module tests
cargo test scanner
cargo test formatters
cargo test mcp
cargo test quantum
cargo test semantic

# Run single test by exact name
cargo test test_quantum_compression -- --exact

# Using manage script (runs tests + clippy + format check)
./scripts/manage.sh test

# Run comprehensive test suite
cd tests && ./run_all_tests.sh
```

### Linting and Formatting
```bash
# Format code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check

# Run clippy linter
cargo clippy -- -D warnings

# Using manage script
./scripts/manage.sh format
./scripts/manage.sh lint
```

### Running
```bash
# Run compiled binary
./target/release/st

# Run with cargo
cargo run --release -- [args]

# Common usage examples
st                               # Classic tree view
st --mode ai --compress          # AI-optimized compressed
st --mode quantum src/           # Quantum compression
st --search "TODO"               # Search in files
st --mcp                         # Run as MCP server
```

## Architecture Overview

### Core Modules
- **main.rs**: CLI entry point using clap 4.5
- **lib.rs**: Library entry point, exports public API
- **scanner.rs**: Directory traversal engine using walkdir
  - Handles permission errors gracefully (marks with `*`)
  - Supports streaming for large directories (>100k files)

### Formatters (src/formatters/)
25+ output formats including:
- **classic.rs**: Traditional tree view (O(n) optimized)
- **hex.rs**: Fixed-width hexadecimal (AI-optimized)
- **quantum.rs**: MEM|8 quantum compression (8-10x reduction)
- **ai.rs**, **summary_ai.rs**: Token-efficient formats
- **mermaid.rs**: Diagram generation
- **marqant.rs**: Quantum-compressed markdown (.mq files)

### MCP Server (src/mcp/)
- **mod.rs**: Main server logic
- **tools.rs**: 30+ MCP tools implementation
- **cache.rs**: Result caching
- **sse.rs**: Real-time monitoring

### Advanced Features (src/)
- **mem8/**: Wave-based memory architecture
- **file_history/**: AI operation tracking
- **smart/**: Git integration, NLP, unified search
- **tree_sitter_quantum.rs**: AST-aware compression
- **convergence/**: Optimal format detection

### Binary Tools (src/bin/)
- **mq.rs**: Marqant compression utility
- **tree.rs**: Alternative tree binary
- **m8.rs**: MEM8 memory tool

## Testing Strategy

### Unit Tests
- Test files alongside source in module directories
- Use `#[cfg(test)]` modules
- Run with `cargo test module_name`

### Integration Tests
```bash
# MCP protocol tests
./tests/test_mcp_integration.sh

# Core functionality
./tests/test_integration.sh

# Feature-specific tests
./tests/test_v3_features.sh
```

### CI/CD Requirements
Tests must pass:
1. `cargo fmt -- --check` (formatting)
2. `cargo clippy -- -D warnings` (linting)
3. `cargo test` (all unit tests)

## Performance Considerations

- Uses **rayon** for parallel operations
- **Streaming mode** (`--stream`) for directories >100k files
- Classic formatter optimized from O(n²) to O(n)
- Default depth auto-selected per format (ls=1, classic=3, ai=5)
- Binary formats (quantum) use compression for 100x reduction

## MCP Server Development

### Setup for Claude Desktop
```bash
# Show configuration
st --mcp-config

# Add to Claude Desktop config
st --mcp-config >> ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

### Testing MCP Tools
```bash
# Run MCP server
st --mcp

# Debug mode
RUST_LOG=debug st --mcp

# List available tools
st --mcp-tools

# Test specific MCP functionality
cargo test mcp::tools
```

## Common Workflows

### After Making Changes
```bash
./scripts/manage.sh format && ./scripts/manage.sh test
```

### Before Committing
```bash
# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy -- -D warnings

# Run tests
cargo test
```

### Release Process
```bash
./scripts/manage.sh release v4.8.8 "Release notes here"
```

## Environment Variables

- `ST_DEFAULT_MODE`: Default output format
- `NO_COLOR=1`: Disable colored output
- `NO_EMOJI=1`: Disable emoji output
- `RUST_LOG`: Logging verbosity (debug, info, warn, error)
- `SMART_TREE_NO_UPDATE_CHECK=1`: Disable update checks

## Project-Specific Patterns

### Error Handling
- Uses `anyhow` for error propagation
- Scanner gracefully handles permission errors
- Marks inaccessible directories with `*`

### Memory Management
- Streaming mode keeps memory constant for large dirs
- Quantum formats use token dictionaries for compression
- File history stored in `~/.mem8/.filehistory/`

### Code Style Notes
- Includes humorous "The Cheet" persona comments
- Rock/musical theme in comments
- Focus on efficiency: "smallest and fastest"
- Performance critical - benchmark large directories

## Debugging Tips

### Large Directory Issues
```bash
st --stream --depth 3 /large/dir
st --mode summary-ai /large/dir
```

### Permission Errors
Check scanner.rs:72 for permission handling

### MCP Communication
```bash
RUST_LOG=debug cargo run -- --mcp
```

### Binary Output Issues
Quantum modes output binary - use redirection:
```bash
st --mode quantum > output.mem8
```

## manage.sh Commands

Core operations:
- `build [debug|release]` - Build project
- `test` - Run tests, clippy, format check
- `run [args]` - Run with arguments
- `clean` - Clean artifacts
- `format`/`fmt` - Format code
- `lint` - Run clippy
- `bench` - Run benchmarks

MCP operations:
- `mcp-run` - Run as MCP server
- `mcp-config` - Show Claude Desktop config
- `mcp-tools` - List available tools

Installation:
- `install [dir]` - Install binary
- `release <version> [notes]` - Create GitHub release

Use `-n` or `--non-interactive` for automation.

## Contributing New Features

### Adding a Formatter
1. Create file in `src/formatters/`
2. Implement `Formatter` trait (required)
3. Implement `StreamingFormatter` (optional)
4. Add to `FormatterType` enum in main.rs
5. Add tests in module
6. Update CLI help text
7. Benchmark against existing formatters

### Testing Requirements
- Include unit tests with `#[test]`
- Test with directories >100k files
- Verify streaming mode works
- Check token efficiency for AI modes
- Run `./scripts/manage.sh test` before committing