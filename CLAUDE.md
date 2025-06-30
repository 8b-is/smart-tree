# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Smart Tree (`st`) is a blazingly fast, AI-friendly directory visualization tool written in Rust. It's designed as an intelligent alternative to the traditional `tree` command, optimized for both human readability and AI token efficiency.

**Current Version**: v2.0.1 - Features revolutionary MEM|8 Quantum compression for 99% size reduction!

**Current Status**: ✅ Fully functional with all core features implemented including:
- MCP (Model Context Protocol) server for AI assistant integration
- DXT (Desktop Extension Tool) package for Claude Desktop
- MEM|8 Quantum format (`--mode quantum`) with 8x compression
- Claude format (`--mode claude`) with 10x compression
- Content search within files (`--search`)
- Streaming mode for large directories (`--stream`)
- Multiple output formats (classic, hex, json, ai, ai_json, stats, csv, tsv, digest, quantum, claude, semantic)
- Built-in compression with `-z` flag for any format
- Semantic grouping with `--semantic` flag (inspired by Omni!)

## Development Commands

### Building
```bash
cargo build                # Development build
cargo build --release      # Optimized release build
```

### Testing
```bash
cargo test                 # Run all tests
cargo test -- --nocapture  # Show test output
cargo test scanner         # Run specific module tests
./scripts/manage.sh test   # Run tests with formatted output

# Test specific components:
cargo test formatters::tests     # Test all formatters
cargo test hex::tests           # Test hex formatter
cargo test scanner::tests       # Test directory scanner
cargo test mcp::tests           # Test MCP server (if feature enabled)
cargo test quantum::tests       # Test quantum format encoder
cargo test claude::tests        # Test claude format compression

# Run tests with feature flags:
cargo test --all-features       # Test with MCP server enabled
cargo test --no-default-features # Test core functionality only

# Run a single test by name:
cargo test test_quantum_compression -- --exact
cargo test test_classic_formatter -- --nocapture
```

### Running
```bash
cargo run -- [args]        # Run in development mode
cargo run --release -- [args]  # Run optimized version
./target/release/st [args]    # Run compiled binary (note: binary is 'st' not 'stree')

# Example commands:
st                         # Default classic mode for current directory
st --mode hex              # Output in hex format
st --mode ai -z            # AI format with compression
st --mode quantum          # MEM|8 quantum format (8x compression)
st --mode claude           # Maximum compression (10x reduction)
st --find "*.rs"           # Find all Rust files
st --search "TODO"         # Search for TODO in file contents
st --stream                # Stream output for large directories
st --everything            # Show all files (--all --no-ignore --no-default-ignore)
st --semantic              # Group files by conceptual similarity (Omni's wisdom!)
```

### Linting and Formatting
```bash
cargo fmt                  # Format code
cargo fmt -- --check       # Check formatting without modifying
cargo clippy              # Lint code
cargo clippy -- -D warnings  # Treat warnings as errors
```

### MCP Server Mode
```bash
# Run as MCP server (for Claude Desktop integration)
stree --mcp

# Show MCP configuration for Claude Desktop
stree --mcp-config

# List available MCP tools
stree --mcp-tools
```

### MCP Development Workflow
```bash
# Test MCP server locally
cargo run --features mcp -- --mcp

# Debug MCP communication
RUST_LOG=debug cargo run --features mcp -- --mcp

# Test specific MCP tools
cargo test mcp::tools --features mcp

# Verify MCP protocol compliance
cargo run --features mcp -- --mcp-tools | jq  # Should output valid JSON
```

## Architecture

The codebase follows a modular structure designed for extensibility:

- **main.rs**: CLI entry point using clap for argument parsing. Handles all command-line options and MCP server mode. Contains humorous comments from "The Cheet" persona!
- **scanner.rs**: Core directory traversal engine using walkdir. Supports filtering, search, and streaming. The "Indiana Jones" of the codebase!
- **formatters/**: Output format implementations
  - **classic.rs**: Traditional tree view with Unicode box drawing
  - **hex.rs**: Fixed-width hexadecimal format
  - **json.rs**: Standard JSON output
  - **ai.rs**: AI-optimized format combining hex + stats
  - **ai_json.rs**: JSON wrapper around AI format
  - **quantum.rs**: MEM|8 quantum compression (8x reduction)
  - **claude.rs**: Maximum compression format (10x reduction)
  - **stats.rs**: Statistics only
  - **csv.rs/tsv.rs**: Tabular formats
  - **digest.rs**: Ultra-compact single-line summary
- **context.rs**: Project context detection for AI modes (detects Rust, Python, Node.js projects)
- **tokenizer.rs**: Smart tokenization for quantum format
- **quantum_scanner.rs**: Specialized scanner for quantum mode
- **mcp/**: Model Context Protocol server implementation
  - **mod.rs**: Main MCP server logic
  - **tools.rs**: 20+ MCP tools for directory analysis
  - **resources.rs**: MCP resource handling
  - **prompts.rs**: MCP prompt templates
  - **cache.rs**: Analysis result caching
- **decoders/**: Decoders for various formats (hex, json, classic)

## Python Reference Implementation

A working Python implementation exists in `old/stree.py` that demonstrates all features. Key implementation details from the Python version:

### Output Format Specifications

1. **Hex Format** (most important for AI consumption):
   - Format: `{depth:x} {perms:03x} {uid:04x} {gid:04x} {size:08x} {mtime:08x} {emoji} {name}`
   - No indentation - depth is encoded in the hex value
   - Fixed-width fields for easy parsing
   - Shows ignored dirs in brackets when `--show-ignored` is used

2. **AI Format**:
   - Combines hex tree with compact statistics
   - Header: `TREE_HEX_V1:`
   - Stats section includes: `F:{files} D:{dirs} S:{size:x} ({size_mb}MB)`
   - File types: `TYPES: ext1:count1 ext2:count2` (top 10)
   - Largest files: `LARGE: file1:size1 file2:size2` (top 5)
   - Date range: `DATES: {oldest:x}-{newest:x}`
   - Footer: `END_AI`

3. **Compression Format**:
   - Header: `COMPRESSED_V1:{hex_encoded_zlib_data}`
   - Any output mode can be compressed with `-z` flag

4. **Quantum Format** (MEM|8 compression):
   - 8-bit header encoding multiple attributes
   - Delta encoding from parent nodes
   - ASCII control codes for tree traversal
   - Token dictionary for common patterns
   - Achieves 8x compression vs classic format

5. **Claude Format** (Maximum compression):
   - Further optimized quantum format
   - Additional context-aware compression
   - 10x compression vs classic format

### Statistics Tracking

The Python version tracks:
- Total files, directories, and size
- File type distribution (by extension)
- 10 largest files
- 10 newest/oldest files

### Permission Denied Handling

When encountering permission errors, the Python version:
- Continues scanning without crashing
- For hex mode, shows the directory with available metadata
- Classic mode shows the directory normally but can't traverse into it

## Key Implementation Notes

1. **Performance First**: Use rayon for parallel scanning, SIMD where applicable
2. **Token Efficiency**: Hex format uses fixed-width fields, no wasted bytes
3. **Error Handling**: Use anyhow for errors, handle permission denied gracefully (show with *)
4. **Memory Efficiency**: Stream large directories, don't load everything into memory
5. **Cross-Platform**: Ensure works on Linux, macOS, Windows

## Recent Features Added

1. **MCP Server Support** (December 2024)
   - Built-in Model Context Protocol server for AI assistants
   - Tools: analyze_directory, find_files, get_statistics, get_digest
   - Caching support for repeated queries
   - Security features with path allow/block lists

2. **Content Search** (`--search`)
   - Search for keywords within file contents
   - Combines with `--type` filter for targeted searches
   - Efficient streaming implementation

3. **Streaming Mode** (`--stream`)
   - Real-time output for large directories
   - Progressive results as directories are scanned
   - Prevents timeouts on slow filesystems

4. **AI JSON Wrapper** (`--ai-json`)
   - JSON-wrapped AI output for programmatic consumption
   - Maintains compact AI format benefits

## Important Patterns

1. **Output Modes**: Use enum dispatch pattern for different formatters
2. **Filtering**: Build a composable filter system that can chain conditions
3. **Progress**: Use indicatif for progress bars on large directories
4. **Compression**: Wrap any output writer with flate2 when -z flag is used

## Testing Strategy

- Unit tests for each module (scanner, formatters, filters)
- Integration tests using tempfile for directory structures
- Use assert_cmd and predicates for CLI testing
- Benchmark tests for performance critical paths

## Dependencies to Note

Core dependencies:
- **clap 4.5**: Modern derive-based CLI parsing with env var support
- **walkdir**: Efficient directory traversal
- **rayon**: Data parallelism for concurrent operations
- **globset**: Fast gitignore pattern matching
- **flate2**: Zlib compression support
- **colored**: Terminal colors (respects NO_COLOR env var)
- **serde/serde_json**: JSON serialization
- **csv**: CSV/TSV output formats
- **sha2**: SHA256 hashing for digest mode

MCP dependencies (feature-gated):
- **tokio**: Async runtime for MCP server
- **dashmap**: Concurrent hashmap for caching
- **async-trait**: Async trait support
- **futures**: Future utilities

## User Preferences (from global CLAUDE.md)

- Create scripts/manage.sh for building, testing, and running
- Prefer efficiency - smallest and fastest implementation
- Use latest Rust patterns and idioms
- Add humor and pizzazz to scripts
- Support both interactive and non-interactive modes

## scripts/manage.sh Usage

The project includes a comprehensive management script at `scripts/manage.sh` with:

```bash
# Basic commands
./scripts/manage.sh build [debug|release] [features]  # Build project
./scripts/manage.sh run -- [args]        # Run with arguments
./scripts/manage.sh test                 # Run all tests
./scripts/manage.sh bench                # Run benchmarks
./scripts/manage.sh fmt                  # Format code (alias: format)
./scripts/manage.sh lint                 # Run clippy
./scripts/manage.sh clean                # Clean build artifacts
./scripts/manage.sh install              # Install to /usr/local/bin
./scripts/manage.sh uninstall            # Remove from /usr/local/bin
./scripts/manage.sh status               # Show project info
./scripts/manage.sh help                 # Show all commands

# MCP-specific commands
./scripts/manage.sh mcp-build            # Build with MCP support
./scripts/manage.sh mcp-run              # Run as MCP server
./scripts/manage.sh mcp-config           # Show Claude Desktop config
./scripts/manage.sh mcp-tools            # List available MCP tools

# Demo commands
./scripts/manage.sh demo-stream          # Demo streaming feature
./scripts/manage.sh demo-search          # Demo search feature
./scripts/manage.sh examples             # Show usage examples

# Release and distribution
./scripts/manage.sh release v1.0.0 "Release notes"  # Create GitHub release
./scripts/manage.sh completions          # Setup shell completions
./scripts/manage.sh man-install          # Install man page
./scripts/manage.sh man-uninstall        # Uninstall man page
```

The script supports non-interactive mode (`NON_INTERACTIVE=true`) and includes humor/pizzazz in messages.

## Common Development Workflows

1. **After making changes**: Always run `cargo fmt` and `cargo clippy` before committing
2. **Testing specific functionality**: Use module-specific tests, e.g., `cargo test scanner`
3. **Performance testing**: Use `cargo build --release` for benchmarking
4. **MCP development**: Test with `stree --mcp-tools` to verify tool registration

## Performance Considerations

- The codebase uses rayon for parallel operations where beneficial
- Streaming mode (`--stream`) is essential for directories with >100k entries
- Compression (`-z`) typically reduces output by 10x but adds CPU overhead
- The digest mode is optimized for AI pre-flight checks (single line output)
- Classic formatter was optimized from O(n²) to O(n) for parent-child relationships (Dec 2024)
- Default depth changed from 10 to 5 to prevent hanging on deep structures

## Recent Improvements (December 2024)

1. **Performance Fix**: Classic mode tree building changed from O(n²) to O(n) using HashMap lookups
2. **Default Depth**: Changed from 10 to 5 levels to prevent hanging on deep directory structures
3. **--everything Flag**: Added master flag that combines --all, --no-ignore, and --no-default-ignore
4. **Size Reporting**: Clarified that stree reports actual file sizes while `du` reports disk blocks

## Debugging Tips and Common Issues

### Permission Denied Errors
- The scanner gracefully handles permission errors and marks inaccessible directories with `*`
- Use `sudo` if you need to scan system directories
- Check `std::io::ErrorKind::PermissionDenied` handling in scanner.rs:72

### Large Directory Performance
- Use `--stream` flag for directories with >10k entries
- Consider `--depth 3` to limit traversal depth
- Enable compression with `-z` to reduce output size
- The classic formatter now uses O(n) HashMap-based tree building

### Debugging Output Formats
```bash
# Compare formatter outputs
stree --mode classic src/ > classic.out
stree --mode hex src/ > hex.out
stree --mode ai src/ > ai.out

# Test compression
stree --mode ai -z src/ | wc -c  # Should be ~10x smaller

# Verify hex format fields
stree --mode hex | head -5  # Check field alignment
```

### Common Development Issues
1. **Gitignore not working**: Check `--no-ignore` flag isn't set, verify `.gitignore` syntax
2. **Missing emojis**: Terminal might not support Unicode, use `--no-emoji`
3. **Broken pipe errors**: Normal when piping to `head`, scanner handles gracefully
4. **MCP server not starting**: Ensure tokio runtime is available, check feature flags
5. **Binary name confusion**: The binary is `st`, not `stree` (though aliases may exist)
6. **Permission errors on install**: Use `sudo` or specify user-writable install directory

## Important Code Patterns

### Adding a New Output Format
1. Create new formatter in `src/formatters/yourformat.rs`
2. Implement `Formatter` trait (and optionally `StreamingFormatter`)
3. Add to `OutputMode` enum in `main.rs`
4. Add match arm in `create_formatter()` function
5. Update tests and documentation

### Error Handling Pattern
```rust
// Use anyhow::Result for all functions that can fail
use anyhow::{Result, Context};

fn process_file(path: &Path) -> Result<()> {
    let content = fs::read_to_string(path)
        .context("Failed to read file")?;
    // ...
    Ok(())
}
```

### Working with The Cheet Comments
The codebase includes humorous comments from "The Cheet" persona. When adding new code:
- Continue the musical/rock concert theme in comments
- Make technical concepts fun and approachable
- Credit contributions to the team (Aye, Hue, Trisha from Accounting)

## DXT Package Development

When updating the DXT package:
1. Update version in `dxt/manifest.json`
2. Test with: `cd dxt && ./build-dxt.sh`
3. Install locally in Claude Desktop to verify
4. Update `dxt/README.md` with any new features
5. Include `.dxt` file in GitHub releases

## Release Checklist

Before creating a release:
1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` with new features/fixes
3. Run full test suite: `./scripts/manage.sh test`
4. Build both standard and MCP versions
5. Test quantum/claude formats on large directories
6. Update documentation if needed
7. Create release: `./scripts/manage.sh release v2.0.2 "Amazing new features!"`