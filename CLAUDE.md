# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Smart Tree (`st`) is a blazingly fast, AI-friendly directory visualization tool written in Rust. It's designed as an intelligent alternative to the traditional `tree` command, optimized for both human readability and AI token efficiency.

**Current Version**: v3.2.0 - Features revolutionary MEM|8 Quantum compression for 99% size reduction!

## Development Commands

### Building
```bash
cargo build                # Development build
cargo build --release      # Optimized release build

# Using the manage script (preferred)
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
cargo test mcp            # Test MCP server
cargo test quantum        # Test quantum compression
cargo test semantic       # Test semantic analysis

# Test specific formatters
cargo test formatters::classic::tests
cargo test formatters::hex::tests
cargo test formatters::ai::tests
cargo test formatters::quantum::tests
cargo test formatters::mermaid::tests
cargo test formatters::relations::tests

# Run a single test by name
cargo test test_quantum_compression -- --exact
cargo test test_classic_formatter -- --nocapture

# Using the manage script
./scripts/manage.sh test   # Runs tests, clippy, and format check
```

### Running
```bash
cargo run -- [args]            # Run in development mode
cargo run --release -- [args]  # Run optimized version
./target/release/st [args]     # Run compiled binary

# Using the manage script
./scripts/manage.sh run -- [args]    # Run with arguments

# Example commands
st                         # Default classic mode for current directory
st --mode quantum          # MEM|8 quantum format (8x compression)
st --mode summary-ai       # Maximum compression (10x reduction)
st --mode quantum-semantic # Semantic-aware quantum compression
st --search "TODO"         # Search for TODO in file contents
st --stream                # Stream output for large directories
st --mode mermaid treemap  # Generate mermaid treemap visualization
```

### Linting and Formatting
```bash
cargo fmt                  # Format code
cargo fmt -- --check       # Check formatting without modifying
cargo clippy              # Lint code
cargo clippy -- -D warnings  # Treat warnings as errors

# Using the manage script
./scripts/manage.sh format  # Format code
./scripts/manage.sh lint    # Run clippy
```

## Architecture

The codebase follows a modular structure:

- **main.rs**: CLI entry point using clap 4.5. Handles all command-line options and MCP server mode
- **scanner.rs**: Core directory traversal engine using walkdir. Supports filtering, search, and streaming
- **formatters/** (21 different output formats):
  - **classic.rs**: Traditional tree view with Unicode box drawing (O(n) optimized)
  - **hex.rs**: Fixed-width hexadecimal format (AI-optimized)
  - **json.rs**, **csv.rs**, **tsv.rs**: Standard data formats
  - **ai.rs**, **ai_json.rs**: AI-optimized formats with compression
  - **quantum.rs**, **quantum_semantic.rs**: MEM|8 quantum compression (8-10x reduction)
  - **summary.rs**, **summary_ai.rs**: Compressed summaries for large directories
  - **mermaid.rs**: Mermaid diagrams (flowchart, mindmap, treemap)
  - **markdown.rs**: Comprehensive markdown reports
  - **semantic.rs**: Wave-based semantic grouping (inspired by Omni)
  - **relations.rs**: Code relationship analysis
  - **stats.rs**, **digest.rs**: Statistics and hashing
- **mcp/** (Model Context Protocol server):
  - **mod.rs**: Main MCP server logic
  - **tools.rs**: 20+ MCP tools for directory analysis
  - **resources.rs**: MCP resource handling
  - **prompts.rs**: MCP prompt templates  
  - **cache.rs**: Analysis result caching
- **Supporting modules**:
  - **context.rs**: Project type detection (Rust, Python, Node.js, etc.)
  - **tokenizer.rs**, **dynamic_tokenizer.rs**: Smart tokenization for quantum formats
  - **quantum_scanner.rs**: Specialized scanner for quantum mode
  - **content_detector.rs**: File content analysis
  - **semantic.rs**: Semantic analysis engine
  - **relations.rs**: Code relationship extraction
  - **mem8/** (planned): .mem8 binary format support
    - **reader.rs**: Binary format parser with CRC validation
    - **writer.rs**: YAML to binary compiler
    - **dumper.rs**: Binary to YAML/JSON converter
    - **cache.rs**: Context caching with TTL

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

3. **Quantum/Semantic Formats**:
   - 8-bit header encoding multiple attributes
   - Delta encoding from parent nodes
   - Token dictionary for common patterns
   - Wave-based semantic grouping for code understanding
   - Achieves 8-10x compression vs classic format

### Performance Considerations

- Uses rayon for parallel operations
- Streaming mode (`--stream`) essential for directories with >100k entries
- Classic formatter optimized from O(n²) to O(n) for parent-child relationships
- Default depth changed from 10 to 5 to prevent hanging on deep structures
- Memory-efficient processing for large codebases

## MCP Server Development

### Running MCP Server
```bash
# Build with MCP support (included by default)
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

### Available MCP Tools (20+)
- `analyze_directory`: Main workhorse with multiple output formats
- `quick_tree`: Lightning-fast 3-level overview
- `project_overview`: Comprehensive project analysis
- `find_files`, `find_code_files`, `find_config_files`: File discovery
- `search_in_files`: Content search across files
- `semantic_analysis`: Wave-based semantic grouping
- `compare_directories`: Directory comparison
- `get_statistics`, `directory_size_breakdown`: Analytics
- And many more (use `st --mcp-tools` for full list)

## Common Development Workflows

1. **After making changes**: 
   ```bash
   ./scripts/manage.sh format
   ./scripts/manage.sh test
   ```

2. **Testing specific functionality**:
   ```bash
   cargo test scanner::tests
   cargo test formatters::hex
   cargo test mcp::tools::test_analyze_directory
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

5. **Working with MCP**:
   ```bash
   ./scripts/manage.sh mcp-build
   ./scripts/manage.sh mcp-test
   ```

## Debugging Tips

### Permission Denied Errors
- Scanner gracefully handles permission errors, marks inaccessible directories with `*`
- Check handling in scanner.rs:72

### Large Directory Performance
- Use `--stream` flag for directories with >10k entries
- Consider `--depth 3` to limit traversal depth
- Enable compression with `-z` to reduce output size
- Use `--mode summary-ai` for maximum compression

### Testing Output Formats
```bash
# Compare formatter outputs
st --mode classic src/ > classic.out
st --mode hex src/ > hex.out
st --mode ai src/ > ai.out
st --mode quantum src/ > quantum.out

# Test compression ratios
st --mode ai -z src/ | wc -c  # Should be ~10x smaller
st --mode quantum src/ | wc -c # Should be ~100x smaller

# Verify hex format fields
st --mode hex | head -5  # Check field alignment
```

### Common Issues and Solutions
- **Binary output appears garbled**: Quantum modes output binary data, use redirection
- **MCP server not responding**: Check RUST_LOG=debug output
- **Slow on network drives**: Use `--stream` and reduce `--depth`

## .mem8 Contextual Metadata System

### Overview
Smart Tree supports `.mem8` files that provide semantic context to directories, creating a fast contextual understanding layer for AI agents. These files can be stored in binary format for efficiency and dumped to YAML for human readability.

### Binary Format
.mem8 files use a compact binary format with 90-97% size reduction compared to YAML/JSON:
- **Magic Header**: 0x4D454D38 ("MEM8")
- **CRC Validation**: Instant verification without full parse
- **Section-based**: Identity, Context, Structure, Compilation, Cache, AI Context, Relationships
- **String Table**: Deduplication for common strings
- **Compression**: Optional zstd for further reduction

### Dumping .mem8 Files
```bash
# Dump binary .mem8 to YAML
st --dump path/to/cache.mem8 > output.yaml

# Dump with pretty formatting
st --dump path/to/cache.mem8 --format yaml

# Dump as JSON
st --dump path/to/cache.mem8 --format json
```

### Creating .mem8 Files
```bash
# Generate .mem8 from directory analysis
st --mode mem8 /project > project.mem8

# Create from YAML template
st --compile template.yaml -o cache.mem8
```

### Integration with Existing Features
```bash
# Use .mem8 context in analysis
st --mode quantum-semantic --use-mem8 /project

# Show tree with semantic annotations
st --mode classic --with-context /project

# Search within semantically tagged directories
st --search "TODO" --concept "wave_patterns"
```

### .mem8 Schema Example
```yaml
type: rust_library
purpose: Core memory wave processing
key_concepts:
  - wave_patterns
  - temporal_navigation
  - sensor_arbitration
dependencies:
  - nalgebra: "Linear algebra"
  - tokio: "Async runtime"
subdirs:
  src/wave: "Wave mathematics"
  src/sensor: "Sensor processing"
```

## Wishlist & Future Improvements

### Feature Requests (from CLAUDE-WISHLIST.md)

1. **Show Line Content in Search Results**
   - Add actual line content to `search_in_files` output
   - Include column position for precise matches
   - Use case: Fixing imports without opening files

2. **Batch File Read Tool**
   - `read_files_from_search` for reading multiple files
   - Takes search results as input
   - Returns consolidated content

3. **Find and Replace Tool**
   - Pattern-based replacement across files
   - Support for regex patterns
   - Preview mode before applying changes

4. **Dependency Graph Analysis**
   - Analyze Rust crate dependencies
   - Show module relationships
   - Visualize with mermaid diagrams

5. **Import Analysis Tool**
   - Semantic understanding of imports
   - Track what's imported from where
   - Help with refactoring import paths

### Performance Improvements

1. **Cached Workspace Analysis**
   - TTL-based caching for large codebases
   - Incremental updates on file changes

2. **Parallel Search Operations**
   - Multiple patterns in single call
   - Concurrent file processing

### Quality of Life

1. **Relative Path Options**
   - Show paths relative to base directory
   - Configurable path display modes

2. **File Type Groups**
   - Predefined groups: `rust_src`, `config`, `tests`
   - Custom group definitions

3. **Symbol Search**
   - Find type definitions
   - Search for struct/trait/fn declarations
   - Example: `st --find-symbol "struct StoredVector"`

## Important Notes

- The codebase includes humorous comments from "The Cheet" persona - continue the musical/rock theme when adding comments
- Always prefer efficiency - smallest and fastest implementation
- Support both interactive and non-interactive modes in scripts  
- When adding new formatters, implement both `Formatter` and optionally `StreamingFormatter` traits
- MCP server features are included by default (no longer feature-gated)
- Version 3.2.0 removed interactive mode - classic is now the default

## Release Process

```bash
# Create a new release
./scripts/manage.sh release v3.1.2 "Amazing new features!"

# This will:
# 1. Build release artifacts for multiple platforms
# 2. Create DXT package for Claude Desktop
# 3. Update version in Cargo.toml
# 4. Tag and push to GitHub
# 5. Create GitHub release with artifacts
# 6. Generate release notes
```

## Contributing New Formatters

When adding a new formatter:
1. Create new file in `src/formatters/`
2. Implement `Formatter` trait (required)
3. Implement `StreamingFormatter` trait (optional, for large dirs)
4. Add to `FormatterType` enum in `main.rs`
5. Update CLI help text
6. Add tests in your formatter module
7. Update this CLAUDE.md with the new format specification

## .mem8 Binary Format Implementation

### Quick Implementation Guide
```rust
fn dump_to_yaml(binary_path: &str) -> Result<(), Box<dyn Error>> {
    let binary_data = std::fs::read(binary_path)?;
    let parsed_mem8 = Mem8Reader::parse(&binary_data)?;
    
    let yaml_output = serde_yaml::to_string(&parsed_mem8)?;
    println!("{}", yaml_output);
    
    Ok(())
}
```

### Binary Structure Overview
```
Header (16 bytes):
- Magic: 0x4D454D38 ("MEM8")
- Version: u16
- Flags: u16 (compressed, encrypted, has_parent, etc.)
- CRC32: u32
- Index offset: u32

Sections:
- 0x01: Identity (path, type, purpose, version)
- 0x02: Context (concepts with importance)
- 0x03: Structure (subdirectories and files)
- 0x04: Compilation (status, errors, warnings)
- 0x05: Cache (CRC, SHA256, expiry)
- 0x06: AI Context (understanding level, hints)
- 0x07: Relationships (upstream/downstream deps)
- 0x08: Sensor Arbitration (MEM8-specific)
```

### Size Efficiency
- YAML: ~4KB typical
- Binary: ~400 bytes (90% reduction)
- Compressed: ~200 bytes (95% reduction)

### Performance Benefits
- Instant CRC validation
- O(1) section access via index
- Memory-mapped file support
- Streaming parse capability