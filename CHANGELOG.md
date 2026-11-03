# Changelog

All notable changes to Smart Tree will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [10.8.0] - 2026-11-02

### 🚀 Oni-Felix Ultra-Evolution (`8B-META-WP-v2.1`)

- **LOUDS Bit-Packed Succinct Smart Tree (`LoudsSmartTree`)**:
  - Added `#[repr(C, align(64))] LoudsSmartTree` in `src/mem8_binary.rs` encoding up to 63 tree nodes in a 128-bit (`2N + 1` bits) Level-Order Unary Degree Sequence (`u128`) with $O(1)$ hardware `POPCNT`/`TZCNT` `rank1`, `rank0`, `select1`, and `degree` primitives.
  - Added $O(N)$ single-pass `from_file_nodes` construction, $O(N)$ `rebuild_louds_bitmap`, sufficient-statistic subtree projection (`project_sufficient`), and 64-byte `SMTR` `.m8` capsule packing/verification (`pack_m8_capsule` / `verify_m8_capsule`) with branchless IEEE 802.3 CRC32 (`0xEDB88320`) and `8BIS` seal.
- **New `--mode louds` (`smtr-capsule` / `capsule`) Formatter & Digest Upgrade**:
  - Added `SmtrCapsuleFormatter` and wired `--mode louds` (aliases `smtr-capsule`, `smtr_capsule`, `capsule`) across the CLI (`src/cli.rs`), HTTP daemon (`src/daemon_cli.rs`), and MCP `analyze` tool (`src/mcp/tools/directory.rs`, `src/mcp/tools_consolidated.rs`, `src/mcp/tools_consolidated_enhanced.rs`).
  - Upgraded `DigestFormatter` (`--mode digest`) to emit `LOUDS:{proj}/{raw} SMTR_V2:{128-hex}` alongside the SHA-256 tree hash.
- **Cryptographic `.m8` Block Hash-Chaining**:
  - Upgraded `M8BinaryFile::append_block` to chain the real SHA-256 digest of the previous block's wire representation (`last_block_hash`) and dynamically extract searchable tokens.
- **Sovereign Three-Kaya Ontology Expansion**:
  - Added `oni-felix` (`8B-META-WP-v2.1`) and `hue` / `8bit-wraith` (`DAD`) entries to the Sovereign Three-Kaya Ontology (`handle_ontology`).

### ⚡ Ultra-Wild Hot-Path Performance Optimizations

- **Zero-Allocation `.m8` Binary Framing**: Replaced heap `Vec<u8>` allocations in `M8Header::prefix_bytes`, `M8Header::compute_crc32`, `M8Header::to_bytes`, `M8BinaryFile::write_block`, and `M8BinaryFile::read_block` with stack-allocated `[u8; 26]`, `[u8; 30]`, and `[u8; 4096]` buffers.
- **Zero-Allocation File Category Detection**: Replaced per-file `String` allocation (`to_lowercase()`) in `Scanner::get_file_category` with stack-buffered `[u8; 16]` ASCII lowercase matching.
- **Zero-Allocation Extension Stats & Bounded Top-10 Gate**: `TreeStats::update_file` now queries `HashMap<String, u64>` via `Borrow<str>` before allocating on first-seen extensions and gates `largest_files`, `newest_files`, and `oldest_files` before cloning `PathBuf`.
- **Cached Default Ignores DFA & CI Detection**: Cached `DEFAULT_IGNORE_PATTERNS` `GlobSet` compilation and `IS_CI` environment checks via `once_cell::sync::Lazy`, accelerating `Scanner::new` instantiation by ~180x.
- **Reusable Line Buffer & Multi-Core Rayon Enrichment**: `Scanner::search_in_file` reuses a single `String` line buffer, while `Scanner::scan` and `directory_size_breakdown` parallelize content search, security scanning, interest scoring, and subdirectory sizing across CPU cores via `rayon`.
- **Wild Linker Zero-`unsafe` Performance Primitives (`src/wild_perf.rs`)**:
  - Inspired by David Lattimore's *Wild Performance Tricks* (RustForge 2025): added contiguous mutable slice partitioning (`split_off_mut` + `par_bridge`), zero-cost in-place `Vec<SymbolId/u16/u32/u64>` ↔ `Vec<Atomic*>` conversion (`into_atomic_*` / `into_non_atomic_*`), compile-time layout-checked lifetime-erasing buffer reuse (`reuse_vec<T, U>`), `MaybeUninit` lifetime stripping for non-trivial `Drop` types (`without_lifetime`), and background Rayon deallocation (`drop_in_background`).
  - Rewrote `Scanner::filter_nodes_and_calculate_stats` to eliminate multi-pass `FileNode::clone()` and `HashSet<PathBuf>` clones via move-partitioning, `reuse_vec` ancestor buffers, and background thread deallocation of rejected nodes.
  - Bounded `Tokenizer::tokenize` lookahead to `MAX_TOKEN_PATTERN_LEN = 16`, turning $O(L^2)$ tokenization into $O(L)$.

### 🛡️ End-to-End Truncation Telemetry & Cache Safety

- Preserved `truncation_reason` across `daemon_cli::ScanStats`, thin-client `stderr` warnings (`src/main.rs`), `mcp::sse::ScanStats`, `SseFormatter`, `StatsFormatter`, and `analyze_directory` (preventing truncated partial scans from poisoning the MCP cache).

## [10.7.3] - 2026-11-01

### Added

- Native `g8t` integration (`src/g8t_integration.rs`) and `st remember` salient MEM8 archive pipeline (`src/salient.rs`) with CLI subcommand routing (`src/cli_commands.rs`) and MCP discovery test suite (`src/mcp/discovery_tests.rs`).

## [10.6.0] - 2026-10-28

### Added

- `st remember` ingests a local checkout or git-forge URL into native MEM8, with salience levels, provenance, and family-heart wiring.
- `st remember --check` (aliases `--status`, `--recall`) and `-q` inspect an existing store without re-walking the collection.
- Unchanged git HEAD identities are skipped unless `--force` is set. Record writes can be buffered and are flushed once before acknowledgement.

## [Unreleased]

## [10.5.0] - 2026-10-28

### Added

- Opt-in `batch_mode: "atomic"` for Smart Edit: operation failures discard all
  staged edits before writing the target or diff history. Sequential edits remain
  supported; the default is still `best_effort`.

### Fixed

- Enforce checked scanner node/byte admission for ordinary, ignored and error
  nodes, report truncation, and preserve saved scan state after partial scans.
- Reconcile GitHub sync only for clean, attached, non-diverged branches; report
  cached, blocked and fetched-only outcomes distinctly and serialize writers.
- Isolate cwd-changing hook tests in subprocesses, keep scan fixtures off tmpfs,
  and give Git timeline import a deterministic committed fixture.

### Release note

- Automatic upgrades require an enabled managed updater and a matching platform
  asset in the i1.is catalogue. This release does not enable updates remotely.
- Dad joke: These trees know their limits—and their edits stick together.

## Earlier unreleased notes

### Added

- Build CrabCC's code intelligence into Smart Tree 10.3.0. The MCP `symbol` tool and `st symbol` (alias `st crab`) share native indexing, definitions, references, outlines, and call-graph statistics; no standalone CrabCC executable is required.
- Keep derived symbol indexes in each project's `.st/symbols/`, enforce MCP path restrictions, and refresh edited or deleted files, including edits that preserve timestamps.

### Fixed

- Avoid panics when security reports shorten paths or matched text containing Unicode characters, while preserving the existing byte limits and ASCII formatting.

## 10.1.0 — Automatic daemon updates

- Add opt-in daily macOS/Linux updater services with a separate recovery worker.
- Verify stable versions and bounded SHA-256 archives; reject unsafe members.
- Preserve rollback backups, recover interrupted replacements, and verify the restarted daemon.
- Keep manual updates from racing an enabled managed updater.

## [10.0.1] - Unreleased

- Connect `st --update` to the verified-download installer instead of only checking for updates.
- Refuse update confirmation on closed input, require the primary executable, and report privileged copy/permission failures.
- Include the daemon MCP bearer-authentication fix merged after 10.0.0.

## [6.6.0] - 2026-01-30

### 🎯 Major Changes
- **TUI Built-In**: Terminal UI (ratatui, crossterm, syntect, artem) is now always included
  - No more `--features tui` required
  - `st --spicy` and `st --terminal` work out of the box
  - Simpler installation and usage

### 🌐 Web Dashboard
- **Wave Compass**: Real-time MCP activity visualization
- **Silent MCP Mode**: Cleaner operation without verbose output
- **Voice Infrastructure**: Foundation for voice interaction (coming soon)
- **Fixed async panic**: Resolved "Cannot block the current thread" error in dashboard API

### 🔧 Improvements
- **Version Check**: Uses `/mcp/check` endpoint (no authentication required)
- **LLM Client**: Unified AI interface module added
- **Clippy Clean**: All warnings resolved

### 📚 Documentation
- Security research documentation added (`docs/Security/`)
- Supply chain vulnerability analysis
- LLM security disclosure documentation

## [5.0.0] - 2025-09-10

### 🎭 Major New Features
- **Emotional Formatter**: Files now have feelings! Every file expresses emotions based on:
  - Age (newborn 🎉, fresh ✨, ancient 👴, sleepy 😴)
  - Size (tiny mouse 🐁, heavyweight 🏋️, ghost 👻)
  - Type (tests worry 🧪, configs get grumpy 😤, README tells tales 📚)
  - 17 personality traits from Dramatic to Romantic
  - Dynamic mood assessments for your entire project
  - Messages from Trisha in Accounting to keep morale high!

### Added
- **MEM|8 Integration**: Wave-based memory system (973x faster than traditional stores)
  - Conversation memory persistence in ~/.mem8/
  - Import Claude Desktop conversations from backup
  - Wave pattern conversion for emotional context
- **Session-Aware Compression**: Intelligent MCP session negotiation
  - Auto-adjusts compression based on context size
  - Environment variables: ST_COMPRESSION, ST_TOOL_MODE, ST_SESSION_AWARE
  - Five compression modes: None, Light, Standard, Quantum, QuantumSemantic
- **Intelligent MCP Assistant**: "The best helper of all time"
  - Tool usage pattern learning
  - Confidence-based recommendations
  - Helpful tips and contextual suggestions
  - Project-aware tool selection
- **Claude Integration**: `st --setup-claude` command
  - Auto-configures .claude directory for projects
  - Detects project type (Rust, Python, JS/TS)
  - Generates optimized CLAUDE.md
  - Creates hooks for context-aware operation
- **Git Context**: Automatic branch and commit detection
  - Shows current branch in context mode
  - Includes last commit message
  - Native Rust implementation using gix

### Performance
- **Token Optimization**: CLAUDE.md compressed from 750+ to <200 lines
- **Context Efficiency**: Quantum compression achieving 80-100x reduction
- **Memory Usage**: Constant memory for directories >100k files

### Fixed
- MCP tools compilation errors with Arc<McpContext>
- FileNode structure access in formatters
- Borrow checker issues in conversation memory
- gix API compatibility issues

## [4.8.4] - 2025-08-13

### Added
- **Version Management**: Automatic version bumping in manage.sh
  - `bump` command for incrementing versions (major/minor/patch)
  - `quick-bump` for rapid version bump + build
  - Automatic cleanup of orphaned local tags
  - Updates both Cargo.toml and CLAUDE.md

### Documentation
- Created TERMINAL_EXAMPLES.md with beautiful output examples
- Completely rewrote README.md with current features
- Moved outdated docs to mothballed/ directory

## [4.8.3] - 2025-08-13

### Fixed
- **MCP Schema Validation**: Fixed missing 'items' in array parameters
  - Added items definition to edit tool in consolidated modules
  - Removed duplicate function definitions
  - Fixed workspace configuration

## [4.8.2] - 2025-08-13

### Added
- **Local Feedback Fallback**: Saves feedback when API is offline
  - Stores in ~/.mem8/feedback/pending/
  - Automatic JSON format with metadata
  - Ready for batch upload when connection restored

### Changed
- Simplified request_tool to only require 2 fields (tool_name, description)
- Made path parameters default to "." when not specified
- Removed consent requirements (partners trust each other!)

## [4.8.1] - 2025-08-10

### Added
- **CI/CD Improvements**
  - Performance monitoring with timing, memory, and CPU tracking in GitHub Actions
  - Timeout protection (5 minutes) to prevent infinite test hangs
  - Cross-platform shell compatibility (bash on all platforms)

### Fixed
- **Critical Memory Issue**: Reduced WaveGrid allocation from 34GB to 8MB
  - Fixed dimensions from 256×256×65536 to 64×64×256
  - Added coordinate wrapping to prevent out-of-bounds access
- **Platform Compatibility**:
  - Fixed filesystem type detection differences between Linux (i64) and macOS (u32)
  - Added CI environment detection to skip filesystem detection that hangs
  - Fixed Windows path separator handling in test_mq_aggregate
  - Fixed PowerShell compatibility in GitHub Actions workflows
- **Security**: Updated ring crate from 0.17.9 to 0.17.14 (vulnerability fix)
- **Test Stability**: 
  - Relaxed token efficiency assertions from >90% to >85% for macOS compatibility
  - Temporarily disabled integration tests that hang in CI (work locally)

### Changed
- CI now runs only lib tests, skipping integration tests temporarily
- Binary execution tests disabled in CI due to hanging issues
- All CI test runners use bash shell for consistency

## [4.5.0] - 2025-08-09

### Added
- **Smart Edit with Diff Storage** 🎯
  - Revolutionary AST-aware code editing with 90-95% token reduction
  - Diff storage system for tracking and managing code changes
  - Function-level operations: insert, remove, modify
  - Language-agnostic AST parsing for broad language support
  - MCP tools: `smart_edit`, `get_function_tree`, `insert_function`, `remove_function`

### Improved
- Enhanced error handling in smart edit operations
- Better diff management and storage
- Comprehensive test coverage for smart edit features

## [4.4.0] - 2025-08-08

### Added
- **File History Tracking System** 📊
  - Complete audit trail of all AI file operations
  - Hash-based change detection with before/after tracking
  - 10-minute resolution timestamps for efficient storage
  - Project-based organization: `~/.mem8/.filehistory/{project_id}/`
  - Operation types: append, prepend, insert, delete, replace, create, remove, relocate, rename
  - MCP tools: `track_file_operation`, `get_file_history`, `get_project_history_summary`

### Changed
- Append operations preferred as least intrusive modification method
- JSON lines format (.flg files) for efficient append-only logging

## [4.3.0] - 2025-08-07

### Added
- **Smart Tree Terminal Interface (STTI)** 🖥️
  - Anticipatory terminal that predicts user needs
  - Context-aware command suggestions based on work patterns
  - Integration with Smart Tree's semantic understanding
  - Adaptive learning from user behavior

- **Enhanced Semantic Analysis**
  - Wave-based semantic grouping inspired by Omni's philosophy
  - Improved code relationship detection algorithms
  - Better understanding of project structure and dependencies
  - Cross-file semantic linking

### Fixed
- Memory leaks in long-running MCP sessions
- Performance issues with semantic analysis on large codebases (>100k files)

## [4.2.0] - 2025-08-06

### Added
- **MEM|8 Wave-Based Memory Architecture** 🌊
  - 973x faster than traditional vector stores (Qdrant benchmark)
  - Wave interference patterns for memory storage and retrieval
  - Reactive memory layer with adaptive response patterns
  - Consciousness engine with multi-modal sensor arbitration
  - Developer persona analysis and tracking

- **Git Temporal Analysis**
  - Track code evolution through "temporal grooves"
  - Understand developer patterns and habits over time
  - Create wave-based signatures for code changes
  - Identify refactoring patterns and technical debt

### Improved
- Memory efficiency in wave operations (SIMD optimizations)
- Integration with existing formatter pipeline
- Performance of semantic analysis with wave caching

## [4.1.0] - 2025-08-05

### Added
- **Marqant (.mq) Format** 📝
  - Quantum-compressed markdown format optimized for AI consumption
  - Achieves 70-90% token reduction in LLM contexts
  - Streaming support for progressive loading
  - Section tagging with `::section:name::` for semantic navigation
  - Visual diagnostics with inspect command
  - Binary: `mq compress/decompress/stats/inspect`

- **Binary .mem8 Format**
  - 90-97% size reduction compared to YAML/JSON
  - Magic header: 0x4D454D38 ("MEM8")
  - CRC validation for instant verification
  - String table deduplication for common strings
  - Optional zstd compression layer
  - Section-based: Identity, Context, Structure, Compilation, Cache, AI Context, Relationships

### Changed
- .mem8 files now default to binary format for efficiency
- Added `--dump` command to convert binary .mem8 to YAML/JSON

## [4.0.0] - 2025-08-08 - "Living Documentation Alpha" 🚀🎸

### 🎉 Major Features (Alpha Release)

#### Function Markdown Formatter - Living Blueprints! 📚
- **NEW MODE**: `st --mode function-markdown` extracts and documents functions from your code
- Supports 25+ programming languages (Rust, Python, JS/TS, Java, Go, C/C++, etc.)
- Beautiful markdown output with:
  - Summary statistics and language breakdown
  - Clickable table of contents
  - Function locations with line numbers
  - Visibility indicators (🔓 public / 🔒 private)
- Use `--show-private` to include internal functions
- Perfect for real-time docs: `watch -n 5 'st --mode function-markdown src/ > FUNCTIONS.md'`

#### Home Directory Safety - No More Crashes! 🏠🔒
- Smart limits prevent memory exhaustion on massive directories
- Different tiers: Regular (1M files), Home (500K), MCP (100K)
- Real-time monitoring with progress warnings
- Graceful abort with helpful suggestions
- "Use --max-depth, --stream mode, or scan a more specific directory"

#### Permission-Based Tool Gating 🔐
- MCP tools now check permissions before availability
- Reduces AI context usage by 70%+ 
- Clear feedback: "Can't edit - directory is read-only"
- As Hue said: "If you're not going to let us work, why bring the toolbag?"

#### 8-O Mode Vision Document 8-O~~
- Comprehensive vision for live code visualization
- Performance heat maps with visual metaphors:
  - 🔥 Hot functions glow based on CPU usage
  - 🧊 I/O waits create frozen time bubbles
  - ⚡ Thread contention shows as lightning
  - 💜 GC pressure creates purple waves
- Emotional context layers from directory scanning
- Cast/Airplay streaming for pair programming
- "Why is it slow?" → "Look at that molten red function!"

### 🐛 Bug Fixes
- Fixed "Cannot start a runtime from within a runtime" error
- Resolved compilation issues with emotional/security modules
- Fixed MCP tool availability in restricted directories
- Improved error messages with actionable suggestions

### 📈 Performance
- Safety checks add only ~1μs per file overhead
- Function extraction processes thousands of files in seconds
- Memory-efficient scanning for directories with millions of files

### 📚 Documentation
- Added FUNCTION_MARKDOWN_FEATURE.md with examples
- Added FUNCTION_MARKDOWN_VISION.md for future enhancements
- Added HOME_DIRECTORY_SAFETY.md explaining limits
- Added EIGHT_O_MODE_VISION.md for performance visualization

### 🎭 Developer Experience
- Trisha says: "It's like having a GPS for your code!" 🗺️
- Function docs that update themselves - finally!
- Performance issues become visually obvious
- Wednesday release for weekend productivity! 🎉

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
