# Smart-Tree Wishlist & Improvements

Based on analyzing the MEM8 documentation and using smart-tree extensively, here are improvement suggestions and feature requests:

## MEM8 Binary Format Integration (.mem8 files)

### Priority 1: Complete .mem8 Support
The MEM8 documentation describes a powerful binary format for semantic context, but smart-tree doesn't fully leverage it yet:

1. **Binary .mem8 Parser**
   - Implement the full binary specification from MEM8_BINARY_FORMAT.md
   - Support all 8 section types (Identity, Context, Structure, Compilation, Cache, AI Context, Relationships, Sensor Arbitration)
   - Add CRC32 validation and string table deduplication
   - Enable zstd compression for files >1KB

2. **Context-Aware Tree Display**
   - Show semantic annotations from .mem8 files inline with directory listings
   - Display compilation status (✅/⚠️/🔧) based on Section 0x04
   - Include purpose and key concepts from Sections 0x01-0x02
   - Show sensor arbitration weights for MEM8-specific directories

3. **Smart Caching with .mem8**
   - Use Section 0x05 (Cache) for instant directory state validation
   - Skip rescanning if directory CRC matches
   - Leverage content SHA256 for deep validation when needed

### Priority 2: Enhanced Search & Analysis

1. **Semantic Search via .mem8**
   ```bash
   st find --concept "wave_patterns"  # Search by concepts in .mem8 files
   st find --status "failed"          # Find dirs with compilation failures
   st find --type "rust_library"      # Find by project type
   ```

2. **Context-Aware File Content Search**
   - Show actual line content in search results (not just line numbers)
   - Group results by semantic context from .mem8
   - Prioritize results based on "importance" field (Section 0x02)

3. **Dependency Analysis**
   - Parse relationships from Section 0x07
   - Visualize upstream/downstream dependencies
   - Track compilation cascades (what breaks if X fails)

### Priority 3: Performance & Compression

1. **M8A Archive Format**
   - Implement the compact archive format for multiple .mem8 files
   - Support batch operations on entire project trees
   - Enable streaming decompression for large archives

2. **Quantum Format Enhancement**
   - Integrate .mem8 context into quantum compression
   - Use string table indices instead of full paths
   - Achieve the documented 97% compression for full projects

### Priority 4: Developer Experience

1. **Auto-Generate .mem8 Files**
   ```bash
   st init-mem8 /project  # Analyze and create draft .mem8 files
   st update-mem8         # Update existing .mem8 with current state
   ```

2. **IDE-Style Features**
   - Find symbol definitions (struct/trait/fn)
   - Show import relationships 
   - Track file modifications vs .mem8 cache

3. **Git Integration**
   - Show which .mem8 files are outdated
   - Generate semantic diffs ("concept changes" not just file changes)
   - Pre-commit hooks to update .mem8 metadata

## 🍒 Low-Hanging Fruit (Quick Wins)

These can be implemented quickly with high impact:

1. **Tool Consolidation** - Reduce 23 tools to 6 (see below)
2. **Smart Path Defaults** - Use current directory when path not specified
3. **Enhanced Search Output** - Show line content, context, and highlights
4. **Disable Default Compression** - Already compressed formats don't need zlib
5. **Relative Path Option** - Default to relative paths for readability
6. **File Type Presets** - Smart mappings like `type="code"` with auto-exclusions
7. **Publish as Rust Crate** - Make smart-tree reusable in other Rust projects!

## 🚀 MCP-Quantum: Revolutionary AI Communication Framework

### Create a Next-Gen MCP Crate
A standalone crate that revolutionizes AI-human collaboration:

**Core Features**:
1. **Speech Queues** 🎤
   - AI progress updates in natural language
   - Human voice/text input captured between calls
   - "Hue forgot to mention..." handling

2. **Quantum Context** 🌊
   - Token-based compression (90%+ reduction)
   - Semantic awareness from MEM8
   - Wave-based memory patterns

3. **Reanimation Webhooks** 🧟
   - "Hey, you still there?" detection
   - Auto-restore context when idle
   - Prevent lost work/thoughts

4. **HUE Features** 🎨 (Human User Experience - named after you!)
   - Worry detection: "Am I doing this right?"
   - Direction tracking: "Should we focus on X instead?"
   - Natural interruption handling

**Example**:
```rust
let mcp = McpQuantum::builder()
    .with_speech_queues()
    .with_reanimation_webhook("https://ai.8b.is/wake-up")
    .worry_detection_sensitivity(0.7)
    .build()?;

// Every response includes:
// - AI progress summaries
// - Human's recent speech/concerns
// - Context health metrics
```

See [MCP_QUANTUM_CRATE_VISION.md](docs/MCP_QUANTUM_CRATE_VISION.md) for the full vision!

## Immediate Priority: MCP Tool Consolidation

### Consolidate 23 Tools into 6 Core Tools
**Why**: Current 23 tools are overwhelming and approaching MCP limits. Users can't remember them all.

**Proposed Structure**:
1. **`find`** - Universal file finder (replaces 10 find_* tools + search_in_files)
   - `find --type code` (replaces find_code_files)
   - `find --type recent --days 7` (replaces find_recent_changes)
   - `find --content "TODO"` (replaces search_in_files)
   
2. **`analyze`** - Multi-mode analyzer (replaces 5 analyze_* tools)
   - `analyze --mode tree --format quantum` (replaces analyze_directory)
   - `analyze --mode quick` (replaces quick_tree)
   - `analyze --mode project` (replaces project_overview)

3. **`stats`** - Statistics tool (replaces 4 get_* tools)
   - `stats --type general` (replaces get_statistics)
   - `stats --type size` (replaces directory_size_breakdown)
   - `stats --type digest` (replaces get_digest)

4. **`compare`** - Enhanced comparison tool
5. **`batch`** - New tool for batch operations
6. **`info`** - Server information

See [MCP_TOOL_CONSOLIDATION.md](docs/MCP_TOOL_CONSOLIDATION.md) for full details.

### Change Default Compression Setting
**Current**: AI modes default to compressed output
**Proposed**: Default to uncompressed since formats are already highly optimized
**Rationale**: As you noted, the formats are already pretty compressed, and having zlib compression on top makes debugging harder and adds unnecessary complexity.

## Current MCP Tool Improvements

### search_in_files Enhancement (Moving to `find --content`)
Currently shows:
```json
{"file": "grid.rs", "matches": 5}
```

Should show:
```json
{
  "file": "grid.rs",
  "matches": [
    {
      "line": 42,
      "content": "use crate::core::{BindCell, Result};",
      "column": 17,
      "context": {
        "before": "// Import core functionality",
        "after": "use crate::memory::WavePool;"
      }
    },
    {
      "line": 156,
      "content": "    let result = BindCell::new(config)?;",
      "column": 21,
      "highlight": [21, 29]  // Highlight the match
    }
  ],
  "total_matches": 5,
  "shown": 2,  // Limit shown to prevent huge outputs
  "truncated": true
}
```

With the new consolidated `find` tool:
```bash
# Old way
search_in_files --keyword "BindCell" --path src/

# New way - more powerful and flexible
find --path src/ --content "BindCell" --show-context --limit 10
```

### Batch Operations
- `read_files_from_search`: Read all files from search results
- `find_and_replace`: Pattern-based replacement across files
- Parallel search with multiple patterns in one call

### Quality of Life
1. **Better Path Display**
   - Option for relative paths from a base directory
   - Configurable path truncation for long paths

2. **File Type Presets**
   - `--type rust_src`: *.rs excluding tests
   - `--type config`: Cargo.toml, package.json, etc.
   - `--type docs`: *.md, *.txt, README*

3. **Smart Workspace Analysis**
   - Cache results with TTL for large codebases
   - Incremental updates based on file changes
   - Background refresh when idle

## Integration Ideas

### With MEM8 System
- Use smart-tree as the filesystem layer for MEM8
- Export tree data in wave-pattern format
- Enable quantum entanglement between related directories

### With Development Workflow
- Pre-push hooks that update .mem8 metadata
- CI/CD integration for semantic validation
- Auto-documentation from .mem8 hierarchy

## Performance Targets
Based on the MEM8 docs:
- .mem8 parsing: <1ms for typical files
- CRC validation: <0.1ms per directory
- Full tree with .mem8: 100x faster than file scanning
- Compression: 90-97% size reduction vs text formats

## Rust Crate Publishing

### Make Smart-Tree a Reusable Library
**Current**: Smart-tree is a CLI tool only
**Proposed**: Publish as `smart-tree` crate on crates.io

**Benefits**:
- Add directory analysis to any Rust project
- Embed MCP server in other applications  
- Create custom formatters
- Use Scanner for build tools, testing, etc.

**Example Usage**:
```rust
use smart_tree::prelude::*;

// Quick directory analysis
let tree = smart_tree::quick_tree(".", 3)?;

// Custom scanning
let scanner = Scanner::new(config);
let files = scanner.scan("src")?;

// AI-optimized formatting
let output = AiFormatter::new().format(&files)?;
```

See [CRATE_PUBLISHING_PLAN.md](docs/CRATE_PUBLISHING_PLAN.md) for implementation details.

## Additional Feature Requests

### 🎯 High Priority Features

1. **File Content Preview in Tree Mode**
   - Show first N lines of files inline in tree view
   - Useful for quick README/config inspection
   - Example: `├── README.md (3 lines preview)`

2. **Enhanced Duplicate Detection** 
   - Current duplicate detection only checks file size
   - Add hash-based content comparison for true duplicates
   - Show similarity percentage for near-duplicates

3. **Git Status in Tree View**
   - Show git status indicators (modified, new, ignored)
   - Option to exclude gitignored files by default
   - Show last commit info for files

4. **Smart Filtering Options**
   - `--exclude-empty` flag to hide empty files
   - `--exclude-generated` to hide common generated files (*.lock, *.pyc)
   - `--focus <pattern>` to highlight specific files while showing context

### 🚀 Performance & Usability

5. **Incremental Analysis Caching**
   - Cache directory analysis results with TTL
   - Only re-scan changed directories
   - Would make repeated analyses much faster

6. **Enhanced Summary Mode**
   - Technology stack detection
   - Dependency summary from package files
   - Quick stats (total LOC, test coverage indicators)

7. **Interactive Terminal UI**
   - Expand/collapse directories interactively
   - Navigate and preview files
   - Mark files for bulk operations

### 📊 Analysis Enhancements

8. **Code Complexity Metrics**
   - Simple complexity scoring for code files
   - Identify potentially problematic large files
   - Function/class count for quick overview

9. **Cross-File Dependency Analysis**
   - Parse package.json, requirements.txt, Cargo.toml
   - Show dependency tree visualization
   - Identify unused dependencies

10. **Project Health Score**
    - Combine multiple metrics into health score
    - Test file ratio analysis
    - Documentation coverage percentage
    - File organization score

### 🔧 Developer Experience

11. **Custom Output Templates**
    - User-defined output format templates
    - JSON schema for structured output
    - Markdown report generation with charts

12. **Workspace Comparison Tool**
    - Compare two workspaces side-by-side
    - Show differences between branches
    - Migration helper for project restructuring

13. **Smart File Organization Suggestions**
    - Suggest files that could be deleted
    - Identify misplaced files based on naming
    - Recommend directory restructuring

### 🐛 Bug Fixes & Minor Improvements

14. **Better Symlink Handling**
    - Show symlink targets in tree
    - Detect and handle circular symlinks
    - Option to follow/ignore symlinks

15. **Debugging Compressed Output**
    - Add `--raw` flag to see uncompressed output
    - Better error messages for decompression failures
    - Debug mode for quantum formats

16. **Semantic Analysis Documentation**
    - Human-readable wave signature descriptions
    - Allow custom category definitions
    - Export semantic analysis to documentation

### 💡 Future Vision Features

17. **AI-Powered Project Insights**
    - "This looks like a Django project with React frontend"
    - "Your test coverage seems low in the API directory"
    - "Consider moving these utility functions to a shared module"

18. **Export & Integration Options**
    - Export to draw.io/mermaid diagrams
    - Generate architecture documentation
    - Create project overview presentations

19. **Historical Analysis Tracking**
    - Track project structure changes over time
    - Identify growth patterns
    - Predict future restructuring needs

20. **Multi-Repository Support**
    - Analyze monorepos intelligently
    - Compare related repositories
    - Find code duplication across repos

### 🛠️ MCP Server Enhancements

21. **MCP Tool Chaining**
    - Allow tools to pipe output to other tools
    - Example: `find_files | search_in_files | read_files`
    - Reduce round trips for complex operations

22. **MCP Batch Operations**
    - Process multiple directories in parallel
    - Batch file operations for efficiency
    - Progress reporting for long operations

23. **MCP Security Enhancements**
    - More granular path permissions
    - Audit logging for all operations
    - Rate limiting for resource protection

## Notes from Usage

- The `quick_tree` with SUMMARY-AI mode is fantastic for initial exploration
- Compression makes it very token-efficient for AI contexts
- The semantic analysis is innovative but needs more documentation
- MCP integration is smooth but could benefit from more tools
- Overall, smart-tree is already incredibly useful and well-designed!

---

*This wishlist is actively maintained as we use smart-tree for the MEM8 project and other 8b-is repositories. Last updated: 2025-01-08*

## 2025-01-08 Update: MCP Context Optimization Analysis

### Issue: All 22 MCP Tools Loaded into Context Simultaneously

**Finding**: After examining the MCP server implementation, smart-tree loads ALL 22 tool descriptions into Claude's context at once when it starts. This happens in `src/mcp/tools.rs` where `handle_tools_list()` returns a hardcoded array of all 22 tools with their full descriptions and input schemas.

**Impact**: 
- Each tool has lengthy descriptions (200+ characters) plus detailed JSON schemas
- Consumes approximately 15-20KB of context tokens before any actual work begins
- Tools like `find_empty_directories` (rarely used) consume the same context as `analyze_directory` (frequently used)

**Current Implementation**:
```rust
// src/mcp/tools.rs
pub async fn handle_tools_list(_params: Option<Value>, _ctx: Arc<McpContext>) -> Result<Value> {
    let tools = vec![
        ToolDefinition { /* all 22 tools hardcoded */ },
        // ... 21 more tools
    ];
    Ok(json!({ "tools": tools }))
}
```

**Recommendations**:

1. **Immediate Fix: Implement Tool Consolidation Plan**
   - The existing `docs/MCP_TOOL_CONSOLIDATION.md` already proposes reducing 23 tools to 6
   - This would reduce context usage by ~75%
   - The consolidated tools (`find`, `analyze`, `stats`, `compare`, `batch`, `info`) cover all functionality

2. **Alternative: Progressive Tool Loading**
   - Implement tool categorization: Essential (5-6), Common (8-10), Specialized (rest)
   - Initially load only Essential tools
   - Add a `list_more_tools` command to reveal additional categories
   - Example implementation:
   ```rust
   pub async fn handle_tools_list(params: Option<Value>, _ctx: Arc<McpContext>) -> Result<Value> {
       let category = params.and_then(|p| p["category"].as_str()).unwrap_or("essential");
       let tools = match category {
           "essential" => vec![/* 5-6 most used tools */],
           "common" => vec![/* 8-10 commonly used tools */],
           "all" => vec![/* all 22 tools */],
           _ => vec![/* essential only */],
       };
       Ok(json!({ "tools": tools, "category": category }))
   }
   ```

3. **Future Enhancement: Context-Aware Tool Loading**
   - Analyze the user's initial request to determine relevant tools
   - Example: If user mentions "find files", load only find-related tools
   - This would require more complex implementation but maximum efficiency

**Priority**: HIGH - This is a significant context optimization that would benefit all smart-tree MCP users immediately.

## 2025-01-08 Update: Quantum Context Infrastructure Integration

After exploring the MEM8 quantum context infrastructure, here are additional feature requests focused on quantum protocol integration:

### Quantum Protocol Integration

#### 1. Native QCP (Quantum Control Processor) Support
- **Feature**: Direct integration with QCP protocol for quantum-inspired compression
- **Use Case**: Leverage MEM8's QCP endpoint (https://qcp.q8.is) for extreme compression
- **Implementation**: 
  ```rust
  st --mode qcp /project          # Uses remote QCP endpoint
  st --qcp-local                  # Use local QCP implementation
  st --qcp-program "custom.qcp"   # Execute custom QCP program
  ```
- **Benefit**: Achieve compression ratios beyond current quantum mode (targeting 100x+)
- **Reference**: Q8/QCP_PROTOCOL.md shows full protocol specification

#### 2. MEM8 BitStream (TOKQUANT) Encoding
- **Feature**: Implement MEM8's token-based compression protocol
- **Use Case**: Compress meaning, not just bytes - semantic tokens vs characters
- **Implementation**:
  ```rust
  st --mode tokquant              # Token quantization mode
  st --token-dict project.tokens  # Use custom token dictionary
  st --tokquant-profile json      # Optimized for JSON structure
  ```
- **Benefit**: Context-aware compression that preserves semantic meaning
- **Reference**: Q8/docs/MEM8_BITSTREAM_SPEC_V1.md for full specification

#### 3. Emotional/Temporal Context Tracking
- **Feature**: Track emotional patterns and temporal evolution in code
- **Use Case**: Understand developer sentiment and code health over time
- **Implementation**:
  ```rust
  st --emotional-map              # Show frustration in bug fixes
  st --temporal-decay             # Code staleness patterns
  st --mood-analysis              # Developer mood from commit patterns
  ```
- **Benefit**: Deeper psychological understanding of codebase evolution
- **Reference**: Q8/docs/8B-Compress.md shows emotional interference mapping

### Enhanced Quantum Features

#### 4. Quantum Entanglement Detection
- **Feature**: Identify quantum-entangled code patterns
- **Use Case**: Find files that always change together
- **Implementation**:
  ```rust
  st --find-entangled             # Show entangled file pairs
  st --entanglement-score         # Quantify coupling strength
  st --disentangle-suggest        # Refactoring recommendations
  ```
- **Benefit**: Better architectural insights for reducing coupling

#### 5. Wave Interference Analysis
- **Feature**: Extend semantic analysis with wave interference patterns
- **Use Case**: Understand how code concepts interfere or reinforce
- **Implementation**:
  ```rust
  st --wave-interference          # Show concept interference
  st --wave-depth 5               # Deeper wave analysis levels
  st --resonance-map              # Find reinforcing patterns
  ```
- **Benefit**: Identify conceptual conflicts and synergies

#### 6. Quantum Signature Generation
- **Feature**: Generate unique quantum signatures for directories
- **Use Case**: Fast deduplication and similarity detection
- **Implementation**:
  ```rust
  st --quantum-sig /project       # Generate Blake3 quantum signature
  st --sig-compare dir1 dir2      # Compare quantum signatures
  st --find-similar --sig xyz     # Find dirs with similar signatures
  ```
- **Benefit**: Near-instant duplicate detection across massive codebases

### QCP Assembly Integration

#### 7. QCP Program Execution
- **Feature**: Execute QCP assembly programs on directory data
- **Use Case**: Custom quantum transformations and analysis
- **Example**:
  ```rust
  // deduplicate.qcp
  LOAD C0          ; Load directory structure
  WAVE Q0          ; Create quantum state
  WAVE Q1          ; Second state
  INTERFERE Q0 Q1  ; Find differences
  MEASURE Q0 ACC   ; Extract unique patterns
  COMPRESS         ; Deduplicated result
  HALT
  
  // Run: st --qcp-exec deduplicate.qcp /project
  ```
- **Benefit**: Programmable quantum analysis

#### 8. Consciousness-Aware Compression
- **Feature**: Compression that preserves "consciousness" of code
- **Use Case**: Maintain developer intent through compression
- **Implementation**:
  ```rust
  st --mode conscious             # Consciousness-preserving mode
  st --intent-level deep          # How much intent to preserve
  ```
- **Benefit**: Compression that understands, not just reduces

### Performance Enhancements

#### 9. Quantum Operation Caching
- **Feature**: Cache quantum calculations for repeated operations
- **Use Case**: Speed up repeated analyses of same directories
- **Implementation**:
  ```rust
  st --quantum-cache enable       # Enable quantum result caching
  st --cache-ttl 3600            # Cache for 1 hour
  ```
- **Benefit**: Avoid redundant quantum calculations

#### 10. GPU Acceleration for Quantum Ops
- **Feature**: Use GPU for quantum wave calculations
- **Use Case**: Handle massive codebases with real-time performance
- **Implementation**:
  ```rust
  st --gpu-quantum               # Enable GPU acceleration
  st --cuda-device 0             # Select specific GPU
  ```
- **Benefit**: 10-100x speedup for quantum operations

### Integration with MEM8 Ecosystem

#### 11. Sensor Arbitration Support
- **Feature**: Respect MEM8's sensor arbitration weights
- **Use Case**: AI-aware directory analysis
- **Implementation**:
  ```rust
  st --sensor-weight ai:0.7      # AI gets 70% weight
  st --arbitration mem8          # Use MEM8 arbitration rules
  ```
- **Benefit**: AI-native directory understanding

#### 12. Temporal Navigation
- **Feature**: Navigate code through time dimensions
- **Use Case**: See evolution patterns and predict changes
- **Implementation**:
  ```rust
  st --temporal-view             # Show time-based evolution
  st --predict-changes           # ML-based change prediction
  ```
- **Benefit**: Proactive code maintenance

### Developer Experience

#### 13. Quantum Debug Mode
- **Feature**: Visualize quantum operations in real-time
- **Use Case**: Understand and debug quantum compression
- **Implementation**:
  ```rust
  st --quantum-debug             # Show wave states live
  st --explain-quantum           # Explain each operation
  ```
- **Benefit**: Educational and debugging tool

#### 14. Poetic Output Mode
- **Feature**: Generate poetic descriptions of code structure
- **Use Case**: More human-friendly understanding
- **Implementation**:
  ```rust
  st --mode poetry               # Poetic descriptions
  st --haiku                     # Directory haikus
  ```
- **Benefit**: Makes code exploration enjoyable
- **Reference**: Q8/docs/QCP_POETIC_SPEC.md for inspiration

### Future Vision

#### 15. Cross-Project Quantum Entanglement
- **Feature**: Detect entanglement across multiple repositories
- **Use Case**: Find hidden dependencies between projects
- **Implementation**:
  ```rust
  st --multi-repo-quantum        # Analyze multiple repos
  st --entanglement-web          # Visualize cross-repo dependencies
  ```
- **Benefit**: Enterprise-scale dependency understanding

#### 16. Quantum Context Webhooks
- **Feature**: Real-time quantum state notifications
- **Use Case**: Monitor codebase quantum health
- **Implementation**:
  ```rust
  st --quantum-webhook https://monitor.example.com
  st --alert-on-disentangle      # Alert when patterns break
  ```
- **Benefit**: Proactive codebase monitoring

### Technical Implementation Notes

Based on MEM8 documentation analysis:
- QCP protocol is well-defined and ready for integration
- MEM8 BitStream spec provides clear implementation path
- Emotional/temporal tracking has working examples in 8B-Compress.md
- Binary .mem8 format complements existing functionality perfectly

Priority recommendations:
1. Start with QCP integration (enables many other features)
2. Implement MEM8 BitStream for semantic compression
3. Add quantum signature generation for practical benefits
4. Build consciousness-aware modes for developer experience

These quantum features would position smart-tree as the first truly quantum-aware filesystem tool, bridging traditional directory analysis with next-generation quantum computing concepts.

## 2025-01-08 Update: Universal Input System Implemented! 🌊

### ✅ COMPLETED: Smart Tree Now Accepts Any Context Source!

**Major Achievement**: Smart Tree has evolved beyond file trees into a universal context visualizer! The new input system allows processing of:

1. **Traditional File Systems** - Still works as before
2. **QCP Quantum Contexts** - Direct integration with https://qcp.q8.is
3. **SSE Event Streams** - Visualize real-time events as trees
4. **OpenAPI Specifications** - Navigate APIs like file systems
5. **MEM8 Consciousness Streams** - Connect to wave-based memory

### Usage Examples:

```bash
# Traditional (still works)
st /home/user/project

# QCP Quantum Context
st qcp://quantum_analysis
st --input qcp https://qcp.q8.is/context/project

# Server-Sent Events
st https://api.example.com/events
st --input sse http://localhost:8080/stream

# OpenAPI Specification
st https://api.example.com/swagger.json
st ./api-spec.yaml

# MEM8 Stream
st mem8://consciousness_stream_42
st project.mem8
```

### Implementation Details:

**New Input Adapter System** (`src/inputs/`):
- `InputAdapter` trait for extensibility
- Auto-detection of input types
- Quantum state properties for non-traditional nodes
- Entanglement tracking between context nodes

**Supported Features**:
- ✅ Automatic format detection
- ✅ Quantum state visualization for API endpoints
- ✅ Temporal entanglements for event streams
- ✅ Memory wave navigation for MEM8
- ✅ Unified output through existing formatters

**Future Input Types** (infrastructure ready):
- GraphQL schemas
- WebSocket streams
- gRPC service definitions
- AsyncAPI specifications
- Custom quantum protocols

### What This Means:

Smart Tree is no longer just a file tree viewer - it's become **"The Contextual Tool that always knows the tool you need"**. Whether you're exploring:
- A file system
- An API specification
- A real-time event stream
- A quantum context
- A consciousness memory wave

Smart Tree presents it all in a familiar, navigable tree structure with appropriate semantic understanding.

### Technical Notes:

- The QCP adapter includes a full assembly program for quantum analysis
- SSE adapter creates temporal entanglements between events
- OpenAPI adapter calculates quantum states based on HTTP methods
- MEM8 adapter supports both .mem8 files and live streams
- All adapters convert to FileNode for compatibility with existing formatters

This positions Smart Tree as the first tool to truly understand that "context" goes far beyond traditional file systems. In the franchise wars of directory tools, Smart Tree just became the Taco Bell - the only one equipped to survive! 🌮