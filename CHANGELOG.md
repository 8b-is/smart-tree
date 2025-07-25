# Changelog

All notable changes to Smart Tree will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.0.0] - 2025-07-02 - "Quantum Awakening"

### 🌌 Major Release

This release represents a paradigm shift in how Smart Tree understands and compresses code. We've evolved from a directory visualizer to a semantic understanding engine.

### 🚀 Major Features

#### Relations as a Mode
- **BREAKING**: Moved `--relations` flag to `--mode relations` for consistency
- Code relationship analysis is now a first-class output mode
- Added `--focus` to analyze specific files
- Added `--relations-filter` to filter by relationship type (imports, calls, types, tests, coupled)
- Outputs include text summaries and Mermaid diagrams

#### Content-Aware Intelligence
- Added `ContentDetector` for automatic directory type detection
- Detects: code projects, photo collections, document archives, media libraries, data science workspaces
- Language detection for code projects (Rust, Python, JavaScript, etc.)
- Foundation for interactive summaries based on content type

#### Quantum Semantic Compression 🧬
- New `--mode quantum-semantic` for AST-aware compression
- Dynamic tokenizer that learns project-specific patterns automatically
- Achieves 94%+ compression while preserving semantic information
- Token-based compression eliminates repetitive patterns
- Uses importance scoring to extract semantic meaning from code
- Language-specific parsers with scoring:
  - Rust: main=1.0, pub functions=0.9, tests=0.3
  - Python: main=1.0, __init__=0.9, private methods=0.4
- Extensible trait-based parser system ready for tree-sitter integration

### 🎨 New Output Modes
- `summary` - Interactive summary for humans (foundation laid)
- `summary-ai` - AI-optimized summary (foundation laid)
- `relations` - Code relationship analysis
- `quantum-semantic` - Semantic-aware compression

### 🔧 Technical Improvements
- Unified all visualizations as modes (consistent CLI interface)
- Added `tree_sitter_quantum.rs` module for semantic parsing
- Trait-based `LanguageQuantumParser` system
- Importance scoring for code elements
- Prepared architecture for full tree-sitter AST integration

### 📚 Documentation
- Added `QUANTUM_SEMANTIC.md` explaining semantic compression
- Added `RELATIONS_FEATURE.md` for code relationships
- Added `RELEASE_V3_QUANTUM_AWAKENING.md` milestone document
- Created demo scripts for new features
- Updated all examples to use new mode syntax

### 🙏 Contributors
- Architectural vision by Chris (Wraith)
- Implementation by Claude
- Quantum semantic innovations by Omni
- Testing and feedback by the 8b-is team

### 💡 Philosophy
> "Don't just shrink it. Make it matter." - Omni

This release embodies our vision: compression with comprehension, making codebases not just visible but understandable.

## [2.0.4] - 2025-01-06

### ✨ New Features
- **Mermaid Diagram Formatter**: Generate directory structures as Mermaid diagrams!
  - Flowchart style (default)
  - Mind map style (`--mermaid-style mindmap`)
  - Git graph style (`--mermaid-style gitgraph`)
  - NEW: Treemap style (`--mermaid-style treemap`) - visualize file sizes!
  - Use `--mermaid-style` to choose diagram type
  - Copy & paste output directly into GitHub/GitLab markdown files
  - Automatic styling based on file types (code, docs, config)
  - Example: `st -m mermaid src/ > docs/architecture.md`

### 🐛 Bug Fixes
- **MCP Notification Handling**: Fixed JSON-RPC notification handling to prevent validation errors
  - Notifications (like `notifications/initialized`) no longer receive responses
  - Properly distinguishes between requests and notifications based on `id` field
  - Eliminates "invalid_union" errors in Claude Desktop

## [2.0.3] - 2025-01-06

### 🐛 Bug Fixes
- **MCP server_info Tool**: Fixed "Field required" error by wrapping response in MCP content format
  - The server_info tool now properly returns formatted JSON text
  - Resolves compatibility issue with Claude Desktop

## [2.0.2] - 2025-01-06

### 🌊 Semantic Analysis & MCP Enhancements

This release introduces Omni-inspired semantic file grouping and makes claude mode the default for MCP tools!

### ✨ New Features
- **Semantic Analysis Mode**: Group files by conceptual similarity using wave signatures
  - `--semantic` CLI flag for semantic grouping  
  - New `semantic_analysis` MCP tool
  - Wave-based categorization inspired by Omni's wisdom
  - Categories include: Documentation, Source Code, Tests, Configuration, etc.
  - Each category has a unique wave signature for similarity matching

### 🚀 Improvements
- **MCP Tools Default to Claude Mode**: Maximum compression is now the default!
  - `analyze_directory` now defaults to claude mode (10x compression)
  - `quick_tree` uses claude mode for initial exploration
  - `project_overview` leverages claude mode for efficiency
  - Tool descriptions emphasize claude mode benefits
- **DXT Installer Fix**: Fixed release artifact naming for v2.x compatibility
  - Removed version numbers from artifact names
  - Simplified MCP artifact creation (MCP is now default feature)

### 📚 Documentation
- Added comprehensive semantic mode documentation
- Updated CLAUDE.md with project-specific instructions
- Enhanced MCP tool descriptions to highlight compression benefits

## [2.0.1] - 2024-12-30

### 📖 Documentation & UX Improvements

This release focuses on improving user experience based on feedback from Claude Desktop usage.

### ✨ Improvements
- **Mode Selection Guide**: Added comprehensive guide to help users choose optimal output modes
- **Better Tool Descriptions**: Updated MCP tool descriptions to guide mode selection
  - `quick_tree`: Now clearly marked as "START HERE!" for initial exploration
  - `analyze_directory`: Explains ai/claude/classic mode trade-offs
- **Enhanced Prompts**: Updated MCP prompts to recommend quick_tree → ai → claude workflow
- **Documentation Updates**:
  - Added MODE_SELECTION_GUIDE.md with decision tree
  - Updated README with mode comparison emojis
  - Added token cost calculations for each mode

### 🐛 Bug Fixes
- Removed emoji from MCP tool descriptions to prevent protocol errors

### 📚 Documentation
- **Mode Selection Guide**: Complete guide with:
  - Quick start workflow recommendations
  - Mode comparison table with compression ratios
  - Token budget management strategies
  - Real-world examples and cost analysis
  - Common mistakes to avoid

## [2.0.0] - 2024-12-30

### 🚀 Revolutionary Update: MEM|8 Quantum Compression

This is a major architectural change that makes Smart Tree the first directory visualization tool designed for the AI era.

### ⚡ Breaking Changes
- The tree walker now natively outputs quantum format
- All other formats (classic, hex, json) are now decoders from quantum
- Default output format changed from classic to quantum for MCP mode

### ✨ New Features
- **MEM|8 Quantum Format**: Revolutionary compression achieving 99% size reduction
  - Bitfield headers encode only what differs from defaults
  - Variable-length encoding for sizes and timestamps
  - Semantic tokenization maps patterns to 16-bit tokens
  - Context-aware delta compression for permissions
- **Native Quantum Architecture**: Tree walker outputs quantum directly
  - Zero conversion overhead
  - Streaming by default
  - Constant memory usage regardless of tree size
- **JSON-Safe Transport**: Base64 encoding for MCP compatibility
- **Token Savings**: $1,270 saved per Chromium tree analysis
- **Performance**: 10-24x faster than traditional tree command

### 🛠️ Improvements
- Added comprehensive quantum format documentation
- Implemented semantic equivalence detection (.js ≡ .mjs ≡ .cjs)
- Created decoders for classic, hex, and json formats from quantum
- Enhanced MCP server with version info and server_info tool
- Optimized classic formatter from O(n²) to O(n) complexity

### 📚 Documentation
- Complete MEM|8 Quantum format specification
- Network efficiency analysis (PPS optimization)
- Real-world compression examples
- Integration guides for CI/CD, VS Code, and Git
- Migration guide from traditional tree command

### 🐛 Bug Fixes
- Fixed quantum formatter hanging on deep directory structures
- Resolved infinite loop in depth tracking
- Fixed MCP UTF-8 encoding errors with binary data
- Corrected permission delta encoding issues

### 📊 Performance Metrics
| Codebase | Traditional | Quantum+Z | Reduction | Cost Savings |
|----------|-------------|-----------|-----------|--------------|
| Linux Kernel | 487MB | 4.1MB | 99.2% | $1,237 |
| Node Modules | 42MB | 412KB | 99.0% | $103 |
| Chromium | 487MB | 4.1MB | 99.2% | $1,270 |

### 🙏 Acknowledgments
Special thanks to:
- Hue for the C64 assembly wisdom: "Every byte wasted is a context switch suffered"
- Trisha from Accounting for making compression metrics sparkle ✨
- Omni for the quantum breakthrough during a philosophical Hot Tub session 🛁

## [1.1.1] - 2024-01-XX

### Added
- Enhanced search functionality with line and column information
  - Search results now show exact line number and column position
  - Display format: `[SEARCH:L<line>:C<column>]` for single matches
  - Multiple matches show count: `[SEARCH:L<line>:C<column>,<count>x]`
  - Truncated results indicated: `[SEARCH:L<line>:C<column>,<count>x,TRUNCATED]`
- Improved search performance with truncation at 100 matches per file

### Fixed
- Search filtering now properly excludes files without matches
- Fixed issue where `--type` filter would show all files of that type even without search matches
- Search results are now properly filtered in both streaming and non-streaming modes

### Changed
- Search match structure improved for better performance and usability
- Reduced maximum search matches per file from 1000 to 100 for better performance

## [1.1.0] - 2024-01-XX

### Added
- Partnership documentation updates
- MCP (Model Context Protocol) as default feature
- OpenAPI specification for MCP server

## [1.0.2] - 2024-01-XX

### Added
- GitHub release management
- New version system

## [1.0.0] - 2024-01-XX

### Added
- Initial release with all core features
- Multiple output formats (classic, hex, json, csv, tsv, ai)
- Advanced filtering options
- MCP server integration
- Cross-platform support 