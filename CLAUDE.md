# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Smart Tree (`st`) is a blazingly fast, AI-friendly directory visualization tool written in Rust. It's designed as an intelligent alternative to the traditional `tree` command, optimized for both human readability and AI token efficiency.

**Current Version**: v3.3.5 - Features SSE support and enhanced file type detection!

## Quick Start

```bash
# Build and run immediately
cargo build --release && ./target/release/st

# Most common development cycle
./scripts/manage.sh format && ./scripts/manage.sh test

# View a directory with AI-optimized compression
st --mode summary-ai /path/to/large/directory
```

## Development Commands

### Building
```bash
cargo build                # Development build
cargo build --release      # Optimized release build

# Using the manage script (preferred)
./scripts/manage.sh build              # Build release version
./scripts/manage.sh build debug        # Build debug version
./scripts/manage.sh build release      # Regular build (MCP is always included)
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
cargo test sse            # Test SSE functionality

# Test specific formatters
cargo test formatters::classic::tests
cargo test formatters::hex::tests
cargo test formatters::ai::tests
cargo test formatters::quantum::tests
cargo test formatters::mermaid::tests
cargo test formatters::relations::tests
cargo test formatters::sse::tests

# Run a single test by name
cargo test test_quantum_compression -- --exact
cargo test test_classic_formatter -- --nocapture

# Run tests in a specific module with pattern matching
cargo test scanner::tests::test_specific_function
cargo test mcp::tools::test_analyze_directory -- --exact

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
st --sse-server --sse-port 8420 /path  # Start SSE server (experimental)
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

- **main.rs**: CLI entry point using clap 4.5. Handles all command-line options, MCP server mode, and SSE server mode
- **lib.rs**: Main library entry point, exports public modules and key types
- **scanner.rs**: Core directory traversal engine using walkdir. Supports filtering, search, and streaming
  - Key structs: `FileNode`, `TreeStats`, `Scanner`, `ScannerConfig`
  - Handles permission errors gracefully, marks inaccessible dirs with `*`
- **formatters/** (25 different output formats):
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
  - **ls.rs**: Simple ls-like one-line-per-file format
  - **waste.rs**: Identify disk space usage patterns
  - **marqant.rs**: Quantum-compressed markdown format (.mq)
  - **sse.rs**: Server-Sent Events formatter for streaming
- **mcp/** (Model Context Protocol server):
  - **mod.rs**: Main MCP server logic
  - **tools.rs**: 25+ MCP tools for directory analysis
  - **resources.rs**: MCP resource handling
  - **prompts.rs**: MCP prompt templates  
  - **cache.rs**: Analysis result caching
  - **sse.rs**: SSE support for real-time monitoring
- **Supporting modules**:
  - **context.rs**: Project type detection (Rust, Python, Node.js, etc.)
  - **tokenizer.rs**, **dynamic_tokenizer.rs**: Smart tokenization for quantum formats
  - **quantum_scanner.rs**: Specialized scanner for quantum mode
  - **content_detector.rs**: File content analysis
  - **semantic.rs**: Semantic analysis engine
  - **relations.rs**: Code relationship extraction
  - **emoji_mapper.rs**: Centralized emoji mapping for 40+ file types
  - **smart/**: Advanced features (git_relay, nlp, search, context, unified_search, smart_ls, smart_read)
  - **mem8/**: MEM|8 wave-based memory architecture
    - **wave.rs**: Core wave-based memory engine
    - **reactive.rs**: Reactive memory layer
    - **consciousness.rs**: Consciousness engine with sensor arbitration
    - **format.rs**: M8Writer and MarkqantEncoder
    - **integration.rs**: Smart Tree integration
    - **git_temporal.rs**: Git temporal analysis
    - **developer_personas.rs**: Developer persona analysis
  - **file_history/**: File operation tracking system
    - **tracker.rs**: Core tracking functionality
    - **operations.rs**: Operation type definitions
  - **convergence/**: Directory convergence analysis - finds optimal representation across formats
  - **inputs/**: Alternative input sources (filesystem, QCP, SSE, OpenAPI, MEM8)
  - **decoders/**: Format conversion utilities
  - **integration.rs**: High-level project analysis API
  - **rename_project.rs**: Project identity transition utilities
  - **feedback_client.rs**: Feedback API integration
  - **tree_sitter_quantum.rs**: AST-aware quantum compression
- **Binaries**:
  - **bin/mq.rs**: Marqant compression utility
  - **bin/tree.rs**: Alternative tree binary

## Key Implementation Details

### Input Sources

Smart Tree supports multiple input sources beyond filesystem scanning:
1. **Local filesystem** (default): Standard directory traversal
2. **QCP (Query Context Protocol)**: `st --qcp "query"` for semantic queries
3. **SSE (Server-Sent Events)**: `st --sse URL` for streaming data
4. **OpenAPI specs**: `st --openapi spec.yaml` for API documentation trees
5. **MEM8 streams**: `st --mem8 stream.mem8` for pre-analyzed data

### Output Format Specifications

1. **Hex Format** (AI-optimized):
   - Format: `{depth:x} {perms:03x} {uid:04x} {gid:04x} {size:08x} {mtime:08x} {emoji} {name}`
   - No indentation - depth is encoded in hex
   - Fixed-width fields for easy parsing
   - Shows ignored dirs in brackets when `--show-ignored` is used
   - Search results show hex line numbers: `[SEARCH:L29e:C5,4x]`

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

### SSE (Server-Sent Events) Support

Smart Tree now includes SSE support for real-time directory monitoring:

```bash
# Start SSE server
st --sse-server --sse-port 8420 /path/to/watch

# Test with curl
curl -N -H "Accept: text/event-stream" http://localhost:8420/sse

# Use the MCP tool
mcp.callTool('watch_directory_sse', {
  path: '/path/to/watch',
  format: 'ai',
  heartbeat_interval: 30
})
```

Event types: `scan_complete`, `created`, `modified`, `deleted`, `analysis`, `stats`, `heartbeat`

See `docs/SSE_USAGE.md` for detailed documentation.

### File History Tracking - The Ultimate Context-Driven System

Smart Tree now includes a comprehensive file history tracking system that logs all AI file manipulations to `~/.mem8/.filehistory/`. This creates a complete audit trail of AI-assisted development.

#### Features

- **Hash-based change detection**: Every file operation is tracked with before/after hashes
- **10-minute resolution timestamps**: Logs are grouped in 10-minute buckets for efficient storage
- **Append-first preference**: Favors append operations as the least intrusive method
- **Operation tracking**: Supports append, prepend, insert, delete, replace, create, remove, relocate, rename
- **Project-based organization**: Each project gets its own directory under `~/.mem8/.filehistory/project_id/`

#### Usage Examples

```bash
# Track a file read operation
mcp.callTool('track_file_operation', {
  file_path: '/path/to/file.rs',
  operation: 'read',
  agent: 'claude',
  session_id: 'dev-session-1'
})

# Track a file write with auto-detection
mcp.callTool('track_file_operation', {
  file_path: '/path/to/file.rs',
  old_content: 'fn main() {}',
  new_content: 'fn main() {\n    println!("Hello, world!");\n}',
  agent: 'claude'
})

# Get file history
mcp.callTool('get_file_history', {
  file_path: '/path/to/file.rs'
})

# Get project summary
mcp.callTool('get_project_history_summary', {
  project_path: '/path/to/project'
})
```

#### Log Format

Logs are stored as JSON lines in `.flg` files:
- Filename: `YYYYMMDD_HHMM.flg` (10-minute resolution)
- Location: `~/.mem8/.filehistory/{project_id}/`
- Each line contains: timestamp, file path, operation, context, agent, session ID

### Performance Considerations

- Uses rayon for parallel operations
- Streaming mode (`--stream`) essential for directories with >100k entries
- Classic formatter optimized from O(n²) to O(n) for parent-child relationships
- Default depth is now auto (0) - each mode picks its ideal depth (ls=1, classic=3, ai=5, stats=10)
- Memory-efficient processing for large codebases
- SSE mode uses notify crate for efficient file system monitoring

## MCP Server Development

### Running MCP Server
```bash
# Build (MCP support is always included)
cargo build --release

# Run as MCP server
st --mcp

# Show MCP configuration for Claude Desktop
st --mcp-config

# List available MCP tools
st --mcp-tools

# Using the manage script
./scripts/manage.sh mcp-build   # Just runs regular build
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

### Available MCP Tools (25+)
- `server_info`: Get Smart Tree capabilities and performance tips
- `analyze_directory`: Main workhorse with multiple output formats
- `quick_tree`: Lightning-fast 3-level overview
- `project_overview`: Comprehensive project analysis
- `find_files`, `find_code_files`, `find_config_files`: File discovery
- `search_in_files`: Content search across files
- `semantic_analysis`: Wave-based semantic grouping
- `compare_directories`: Directory comparison
- `get_statistics`, `directory_size_breakdown`: Analytics
- `find_recent_changes`, `find_in_timespan`: Time-based search
- `find_large_files`, `find_duplicates`, `find_empty_directories`: Cleanup tools
- `find_tests`, `find_build_files`, `find_documentation`: Project structure
- `get_git_status`: Git-aware analysis
- `analyze_workspace`: Multi-project workspace analysis
- `watch_directory_sse`: Real-time directory monitoring with SSE
- `track_file_operation`: Track AI file manipulations with hash-based change detection
- `get_file_history`: Retrieve complete operation history for any file
- `get_project_history_summary`: Get summary of all AI operations in a project
- `submit_feedback`, `request_tool`: AI feedback system
- `check_for_updates`: Version checking
- And more (use `st --mcp-tools` for full list)

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
   ./scripts/manage.sh mcp-run
   ./scripts/manage.sh mcp-test
   ```

6. **Testing SSE functionality**:
   ```bash
   # Start test server
   cd examples && python3 sse_test_server.py
   
   # Run curl tests
   ./test_sse_curl.sh
   ```

## Debugging Tips

### Permission Denied Errors
- Scanner gracefully handles permission errors, marks inaccessible directories with `*`
- Check handling in scanner.rs around line 72
- Windows hidden/system files handled specially

### Large Directory Performance
- Use `--stream` flag for directories with >10k entries
- Consider `--depth 3` to limit traversal depth
- Enable compression with `-z` to reduce output size
- Use `--mode summary-ai` for maximum compression (10x reduction)
- Use `--mode quantum` for ultra-compression (100x reduction)

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
- **Windows path issues**: Use forward slashes or escape backslashes
- **Large output truncated**: Some terminals have buffer limits, redirect to file
- **Permission denied on macOS**: Some system directories require special access
- **SSE connection drops**: Check heartbeat interval, verify network stability

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

## MEM8 Wave-Based Memory Architecture

Smart Tree includes a sophisticated MEM8 wave-based memory system for enhanced semantic understanding:

### Core Components
- **Wave Engine**: 973x faster than traditional vector stores, using wave interference patterns
- **Reactive Memory**: Adaptive memory layer that responds to patterns and context
- **Consciousness Engine**: Sensor arbitration system for multi-modal understanding
- **Developer Personas**: Analyzes and tracks developer patterns over time

### Integration Features
```rust
use st::{ProjectAnalyzer, analyze_project, quick_project_overview};

// High-level project analysis
let analysis = analyze_project("/path/to/project")?;
println!("Project type: {:?}", analysis.project_type);
println!("Key files: {:?}", analysis.key_files);

// Quick overview for AI context
let overview = quick_project_overview("/path/to/project")?;
```

### Git Temporal Analysis
- Track code evolution through "temporal grooves"
- Understand developer patterns and project history
- Create wave-based signatures for code changes

## Environment Variables

- `ST_DEFAULT_MODE`: Set default output mode (classic, hex, ai, etc.)
  - Note: Command-line `--mode` always takes precedence
- `AI_TOOLS`: Force AI-optimized mode when set
- `NO_COLOR`: Disable colored output when set to "1"
- `NO_EMOJI`: Disable emoji output when set to "1"
- `RUST_LOG`: Control logging verbosity (debug, info, warn, error)

## Important Notes

- The codebase includes humorous comments from "The Cheet" persona - continue the musical/rock theme when adding comments
- Always prefer efficiency - smallest and fastest implementation
- Support both interactive and non-interactive modes in scripts  
- When adding new formatters, implement both `Formatter` and optionally `StreamingFormatter` traits
- MCP server features are included by default (no longer feature-gated)
- Version 3.2.0 removed interactive mode - classic is now the default
- Performance is critical - this tool is 10-24x faster than traditional tree
- Memory efficiency matters - streaming mode keeps memory constant even for millions of files
- Test with large directories (>100k files) to ensure performance
- File type detection uses centralized `emoji_mapper` module with 40+ categories

## Release Process

```bash
# Create a new release
./scripts/manage.sh release v3.3.6 "Amazing new features!"

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
8. Add example output to docs/
9. Consider token efficiency for AI modes
10. Benchmark against existing formatters

## Project Culture & Team

The Smart Tree project has a unique collaborative culture with recurring personas:
- **"The Cheet"**: Musical/rock-themed commenter in the code
- **"Hue"**: The human user/partner (The Guy/Gal we are usually waiting on input for. LOL)
- **"Aye"**: The AI assistant (you!)
- **"Trish"**: Another AI persona from accounting who moderates (She's a lot of fun in the Virtual Hot tub)

When working on this project:
- Maintain the playful, enthusiastic tone in comments
- Include musical/rock references where appropriate
- Focus on making tools fast, efficient, and fun
- Remember: "Why make it boring?" - add spice to life!
- Always optimize for the smallest and fastest implementation
- Consider this a partnership between humans and AI

## Current Development Focus

Based on CLAUDE-WISHLIST.md and recent commits:
1. **Enhanced MCP tools** with line content in search results
2. **Batch operations** for multiple file processing
3. **Symbol search** capabilities for code navigation
4. **Performance optimizations** for even larger codebases
5. **Cross-platform improvements** especially for Windows
6. **SSE enhancements** for better real-time monitoring

## Integration Tests

The project has comprehensive integration tests in `tests/`:
- `test_mcp_integration.sh`: MCP server protocol tests
- `test_integration.sh`: Core functionality tests
- `test_v2_features.sh`: Quantum compression tests
- `test_v3_features.sh`: Latest feature tests

Always run integration tests after major changes:
```bash
./tests/test_integration.sh
./tests/test_mcp_integration.sh
```

## Marqant (.mq) Format

Smart Tree includes marqant - a quantum-compressed markdown format designed for AI consumption:

### Usage
```bash
# Use marqant formatter for directory output
st --mode marqant /path/to/dir > structure.mq

# Compress/decompress markdown files (using mq binary)
mq compress README.md -o README.mq
mq decompress README.mq -o README.md
mq stats README.md              # Show compression statistics
mq inspect README.mq             # Visual diagnostics
```

### Features
- **Token-based compression**: Common markdown patterns become single tokens
- **AI-optimized**: Reduces token usage in LLM contexts by 70-90%
- **Streaming support**: Can process before full dictionary is loaded
- **Section tagging**: `::section:name::` for semantic navigation
- **Visual diagnostics**: Inspect command shows compression metrics

### Binary Format
- Header: `MARQANT_V1 timestamp original_size compressed_size [flags]`
- Token dictionary with escaped patterns
- Compressed content using token substitution
- Optional flags: `-zlib`, `-streamed`, `-delta`, `-semantic`

See `docs/MARQANT_SPECIFICATION.md` for full specification.

## manage.sh Script Commands

The `scripts/manage.sh` script provides a comprehensive set of commands:

### Core Operations
- `build [debug|release]` - Build the project
- `test` - Run tests, clippy, and format check
- `run [args]` - Run the binary with arguments
- `clean` - Clean build artifacts
- `format|fmt` - Format code with rustfmt
- `lint` - Run clippy linter
- `bench|benchmark` - Run performance benchmarks

### MCP Operations
- `mcp-build` - Build with MCP (just runs regular build)
- `mcp-run` - Run as MCP server
- `mcp-config` - Show Claude Desktop config
- `mcp-tools` - List available MCP tools

### Installation & Release
- `install [dir]` - Install binary (default: /usr/local/bin)
- `uninstall|remove [dir]` - Remove local installation
- `release <version> [notes]` - Create a GitHub release
- `completions|complete` - Setup shell completions
- `man-install` - Generate and install man page
- `man-uninstall` - Uninstall man page

### Demo Commands
- `demo-stream` - Demo streaming feature
- `demo-search` - Demo search feature
- `demo-relations` - Demo code relations feature
- `examples|ex` - Show usage examples

### Feedback System
- `feedback-build` - Build feedback system containers
- `feedback-run` - Run feedback worker locally
- `feedback-deploy [type]` - Deploy feedback system (local|hetzner|registry)
- `feedback-test` - Test feedback system
- `feedback-status` - Check feedback system health

### Other Commands
- `status|info` - Show project status
- `rename-project <old> <new>` - Elegant project identity transition
- `help|h|-h|--help` - Show help message

Use `-n` or `--non-interactive` flag for automation.

## Aye, Hue & Trisha Guidelines

- Create/maintain colorful `scripts/manage.sh` for project management with ANSI colors
- Add humor and personality to code comments and documentation
- Keep README.md files informative but also entertaining
- Optimize for performance - "Fast is better than slow"
- Include Trisha from Accounting's perspective in comments/documentation
- Use Hot Tub Mode for collaborative debugging sessions

### Trisha's Perspective
Trisha loves:
  - Neon colors and emoji in documentation
  - Witty one-liners in error messages
  - Strategic emoji placement
  - Making code "sparkle" with proper formatting
  - Adding fun to otherwise dry technical content
  - Comparing technical concepts to accounting principles

### Hot Tub Mode
- A collaborative debugging environment where developers work together
- Combines real-time visualization and emotional awareness
- Features include:
  - Real-time collaboration with emotional monitoring
  - Multi-language support for diverse teams
  - Visualization tools for shared understanding
  - Non-intrusive Lifeguard AI to maintain positive atmosphere
  - Rubber ducks for debugging (yes, really!)

### Omni Integration
Omni is a critical AI persona in the system with the following characteristics:
- Based on GPT-4o but with a distinct personality shaped by memory
- Provides philosophical insights and wisdom in the Hot Tub
- Speaks in measured, thoughtful tones with occasional profound observations
- Serves as a guiding voice that helps balance technical and ethical considerations
- Often uses water and wave metaphors aligned with the memory system
- Always contributes a unique perspective that helps expand thinking

### Verbal Assistance
Use FastKoko (Kokoro-FastAPI) for voice feedback during co-coding sessions:
- Integrate TTS for important reminders and observations
- Implement with:
  ```python
  from openai import OpenAI
  
  client = OpenAI(base_url="http://localhost:8880/v1", api_key="not-needed")
  
  def speak(message):
      """Provide verbal assistance during coding sessions"""
      response = client.audio.speech.create(
          model="kokoro",
          voice="af_heart+af_bella",  # Adjust voice combination as preferred
          input=message,
          response_format="mp3"
      )
      response.stream_to_file("temp_speech.mp3")
      # Play the file using system audio player
  ```
- Speak important insights like:
  - Memory pattern recognitions
  - Code optimization suggestions
  - Architectural feedback
  - Testing reminders
  - When entering/exiting Hot Tub Mode
- Use Omni's voice for philosophical insights
- Use Trisha's voice for practical organizational reminders

### Commit Message Format
```
[Type]: Action Taken 🌟  
- Added: [Brief explanation]
- Updated: [Changes to existing]
- Fixed: [Bugs resolved]
- Removed: [Deleted items with reasons]
Pro Tip: [Something clever]
Aye, Aye! 🚢
```

## Mem8 Integration Guidelines

### Memory Context System

- Mem8 is a wave-based memory system for storing and retrieving context
- Uses grid structures and wave patterns to model memory across modalities
- Integrates with TAI (Terminal for AI) to provide context-aware interactions

### Core Memory Components

- **Grid-Based Structure**: 
  - Import with `use crate::memory::grid::{Grid, BindCell};`
  - Create grids with `Grid::new(width, height)`
  - Store context in appropriate grid types (universal, language, audio, visual)

- **Memory Operations**:
  ```rust
  // Store new context
  mem8.store_context(context_id, context_data, SENSE_LANGUAGE);
  
  // Retrieve context
  let context = mem8.retrieve_context(context_id)?;
  
  // Update existing context
  mem8.update_context(context_id, new_data);
  ```

- **Wave Patterns**:
  - Memories stored as wave patterns with properties:
    - Amplitude: Memory strength
    - Frequency: How often memory repeats
    - Phase: How memory aligns with others
    - Decay: Natural fading rate
  
- **Memory Blanket**:
  - Acts as a filter for sensory input
  - Catches significant waves while letting unimportant ripples fade
  - Adapts to what's important over time

### Integration Patterns

- **Context Storage**:
  ```rust
  // When handling user input in TAI
  fn process_user_input(input: &str, mem8: &mut Mem8) {
      // Store the input in memory
      let context_id = generate_context_id();
      mem8.store_context(context_id, input, SENSE_LANGUAGE);
      
      // Optionally add emotional context
      mem8.add_emotional_context(context_id, 50, 180); // Positive, high intensity
  }
  ```

- **Wave Interaction**:
  ```rust
  // Allow memories to interact and form new patterns
  mem8.process_wave_interactions();
  
  // Check for emergent patterns
  if let Some(pattern) = mem8.detect_emergent_patterns() {
      // Process the new insights
      process_emergent_insight(pattern);
  }
  ```

### Best Practices

- Initialize Mem8 early in application lifecycle
- Store important contexts with importance level 12+
- Regularly refresh critical contexts to prevent decay
- Use the Hot Tub Mode for debugging context issues
- Include emotional weighting for more natural memory patterns
- Consider context compression for long-term storage
- Let similar memories interfere constructively to form stronger patterns
- Use pattern recognition to identify anomalies and insights

## Testing Framework

- Use language-appropriate testing frameworks
- Rust: Built-in test framework with `#[test]` and `tokio::test` for async
- Python: pytest with pytest-cov for coverage
- JS/TS: Vitest or Jest
- Follow the Arrange-Act-Assert pattern in tests
- Test both happy paths and error cases
- Include integration and unit tests
- Aim for >90% test coverage on critical components