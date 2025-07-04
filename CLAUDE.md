# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Smart Tree (`st`) is a blazingly fast, AI-friendly directory visualization tool written in Rust. It's designed as an intelligent alternative to the traditional `tree` command, optimized for both human readability and AI token efficiency.

**Current Version**: v3.1.1 - Features revolutionary MEM|8 Quantum compression for 99% size reduction!

## Development Commands

### Building
```bash
cargo build                # Development build
cargo build --release      # Optimized release build

# Using the manage script
./scripts/manage.sh build              # Build release version
./scripts/manage.sh build debug        # Build debug version
./scripts/manage.sh build release mcp  # Build with MCP support
```

### Testing
```bash
# Run all tests
cargo test                
cargo test -- --nocapture  # Show test output

# Run specific module tests
cargo test scanner         # Test scanner module
cargo test formatters      # Test formatters module

# Test specific components
cargo test formatters::tests     # Test all formatters
cargo test hex::tests           # Test hex formatter
cargo test scanner::tests       # Test directory scanner
cargo test mcp::tests           # Test MCP server
cargo test quantum::tests       # Test quantum format encoder
cargo test claude::tests        # Test claude format compression

# Run tests with feature flags
cargo test --all-features       # Test with MCP server enabled
cargo test --no-default-features # Test core functionality only

# Run a single test by name
cargo test test_quantum_compression -- --exact
cargo test test_classic_formatter -- --nocapture

# Using the manage script
./scripts/manage.sh test   # Runs tests, clippy, and format check
```

### Running
```bash
cargo run -- [args]        # Run in development mode
cargo run --release -- [args]  # Run optimized version
./target/release/st [args]     # Run compiled binary

# Using the manage script
./scripts/manage.sh run -- [args]    # Run with arguments

# Example commands
st                         # Default classic mode for current directory
st --mode quantum          # MEM|8 quantum format (8x compression)
st --mode claude           # Maximum compression (10x reduction)
st --search "TODO"         # Search for TODO in file contents
st --stream                # Stream output for large directories
```

### Linting and Formatting
```bash
cargo fmt                  # Format code
cargo fmt -- --check       # Check formatting without modifying
cargo clippy              # Lint code
cargo clippy -- -D warnings  # Treat warnings as errors

# Using the manage script
./scripts/manage.sh format  # Format code
./scripts/manage.sh fmt     # Alias for format
```

## Architecture

The codebase follows a modular structure:

- **main.rs**: CLI entry point using clap 4.5 for argument parsing. Handles all command-line options and MCP server mode
- **scanner.rs**: Core directory traversal engine using walkdir. Supports filtering, search, and streaming
- **formatters/**: Output format implementations
  - **classic.rs**: Traditional tree view with Unicode box drawing (optimized to O(n) from O(n²))
  - **hex.rs**: Fixed-width hexadecimal format (most important for AI)
  - **json.rs**: Standard JSON output
  - **ai.rs**: AI-optimized format combining hex + stats
  - **quantum.rs**: MEM|8 quantum compression (8x reduction)
  - **claude.rs**: Maximum compression format (10x reduction)
  - **mermaid.rs**: Mermaid diagram generation
  - **markdown.rs**: Comprehensive markdown reports
  - **semantic.rs**: Semantic grouping (wave-based, inspired by Omni)
- **context.rs**: Project context detection for AI modes (Rust, Python, Node.js)
- **tokenizer.rs**: Smart tokenization for quantum format
- **quantum_scanner.rs**: Specialized scanner for quantum mode
- **mcp/**: Model Context Protocol server implementation
  - **mod.rs**: Main MCP server logic
  - **tools.rs**: 20+ MCP tools for directory analysis
  - **resources.rs**: MCP resource handling
  - **prompts.rs**: MCP prompt templates
  - **cache.rs**: Analysis result caching

## Key Implementation Details

### Output Format Specifications

1. **Hex Format** (AI-optimized):
   - Format: `{depth:x} {perms:03x} {uid:04x} {gid:04x} {size:08x} {mtime:08x} {emoji} {name}`
   - No indentation - depth is encoded in hex
   - Fixed-width fields for easy parsing
   - Shows ignored dirs in brackets when `--show-ignored` is used

2. **AI Format**:
   - Header: `TREE_HEX_V1:`
   - Combines hex tree with compact statistics
   - Stats section: `F:{files} D:{dirs} S:{size:x} ({size_mb}MB)`
   - File types: `TYPES: ext1:count1 ext2:count2` (top 10)
   - Footer: `END_AI`

3. **Quantum/Claude Formats**:
   - 8-bit header encoding multiple attributes
   - Delta encoding from parent nodes
   - Token dictionary for common patterns
   - Achieves 8-10x compression vs classic format

### Performance Considerations

- Uses rayon for parallel operations
- Streaming mode (`--stream`) essential for directories with >100k entries
- Classic formatter optimized from O(n²) to O(n) for parent-child relationships
- Default depth changed from 10 to 5 to prevent hanging on deep structures

## MCP Server Development

### Running MCP Server
```bash
# Build with MCP support
cargo build --release

# Run as MCP server
st --mcp

# Show MCP configuration for Claude Desktop
st --mcp-config

# List available MCP tools
st --mcp-tools

# Using the manage script
./scripts/manage.sh mcp-build   # Build with MCP support
./scripts/manage.sh mcp-run     # Run as MCP server
./scripts/manage.sh mcp-config  # Show Claude Desktop config
./scripts/manage.sh mcp-tools   # List available MCP tools
```

### MCP Development Workflow
```bash
# Test MCP server locally
cargo run -- --mcp

# Debug MCP communication
RUST_LOG=debug cargo run -- --mcp

# Test specific MCP tools
cargo test mcp::tools

# Verify MCP protocol compliance
cargo run -- --mcp-tools | jq  # Should output valid JSON
```

## Common Development Workflows

1. **After making changes**: 
   ```bash
   cargo fmt
   cargo clippy
   cargo test
   ```

2. **Testing specific functionality**:
   ```bash
   cargo test scanner::tests
   cargo test formatters::hex
   ```

3. **Performance testing**:
   ```bash
   cargo build --release
   ./scripts/manage.sh bench
   ```

4. **Installing locally**:
   ```bash
   ./scripts/manage.sh install
   ```

## Debugging Tips

### Permission Denied Errors
- Scanner gracefully handles permission errors, marks inaccessible directories with `*`
- Check handling in scanner.rs:72

### Large Directory Performance
- Use `--stream` flag for directories with >10k entries
- Consider `--depth 3` to limit traversal depth
- Enable compression with `-z` to reduce output size

### Testing Output Formats
```bash
# Compare formatter outputs
st --mode classic src/ > classic.out
st --mode hex src/ > hex.out
st --mode ai src/ > ai.out

# Test compression
st --mode ai -z src/ | wc -c  # Should be ~10x smaller

# Verify hex format fields
st --mode hex | head -5  # Check field alignment
```

## Important Notes

- The codebase includes humorous comments from "The Cheet" persona - continue the musical/rock theme when adding comments
- Always prefer efficiency - smallest and fastest implementation
- Support both interactive and non-interactive modes in scripts
- When adding new formatters, implement both `Formatter` and optionally `StreamingFormatter` traits
- MCP server features are included by default (no longer feature-gated)

## Release Process

```bash
# Create a new release
./scripts/manage.sh release v2.0.9 "Amazing new features!"

# This will:
# 1. Build release artifacts
# 2. Create DXT package for Claude Desktop
# 3. Tag and push to GitHub
# 4. Create GitHub release with artifacts
```