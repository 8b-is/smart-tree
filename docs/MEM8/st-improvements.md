# Smart-Tree Improvements Wishlist

## Feature Requests

### 1. Show Line Content in Search Results
**Current**: `search_in_files` shows line numbers and match counts
**Desired**: Option to show the actual line content (like `grep -n`)
**Use Case**: When fixing imports, I need to see the exact import statement without opening each file

Example:
```json
{
  "file": "grid.rs",
  "matches": [
    {
      "line": 1,
      "content": "use crate::core::{BindCell, Result};",
      "column": 1
    }
  ]
}
```

### 2. Batch File Read Tool
**Current**: Need to use multiple tools to read content from search results
**Desired**: `read_files_from_search` that takes search results and returns content
**Use Case**: After finding files with specific patterns, often need to read them all

### 3. Find and Replace Tool
**Current**: Need to search, then read, then use external edit tools
**Desired**: `find_and_replace` with pattern matching across files
**Use Case**: Updating import paths across many files after refactoring

Example:
```bash
find_and_replace --path /crates --pattern "use crate::core" --replacement "use mem8_core"
```

### 4. Dependency Graph Analysis
**Current**: No way to analyze crate dependencies
**Desired**: `analyze_dependencies` for Rust projects showing crate relationships
**Use Case**: Understanding which crates depend on which during reorganization

### 5. Import Analysis Tool
**Current**: Can search for imports but no semantic understanding
**Desired**: `analyze_imports` showing what's imported from where
**Use Case**: Refactoring module structure and updating import paths

## Performance Suggestions

### 1. Cached Workspace Analysis
For large codebases, cache the workspace analysis results with a TTL

### 2. Parallel Search Operations
Allow multiple search patterns in a single call for better performance

## Quality of Life

### 1. Relative Path Option
Option to show paths relative to a base directory (not just filename)

### 2. File Type Groups
Predefined groups like "rust_src" (*.rs but not tests), "config" (Cargo.toml, etc.)

### 3. Type/Symbol Search
Search for type definitions, struct/trait/fn declarations
Example: `find_symbol --type "struct" --name "StoredVector"`
Would be super helpful for finding where types are defined during refactoring

## Major Feature: .mem8 Contextual Metadata System

### Overview
Add support for `.mem8` files that provide semantic context to directories, creating a fast contextual understanding layer for AI agents.

### How It Works

1. **Directory Context Files**
   - Each directory can have a `.mem8` file
   - Contains semantic metadata about the directory's purpose and contents
   - Example `.mem8` content:
   ```yaml
   type: rust_library
   purpose: Core memory wave processing
   key_concepts:
     - wave_patterns
     - temporal_navigation
     - sensor_arbitration
   dependencies:
     - nalgebra: "Linear algebra for wave calculations"
     - tokio: "Async runtime"
   subdirs:
     src/wave: "Wave mathematics implementation"
     src/sensor: "Sensor input processing"
   ```

2. **Context Inheritance**
   - Start from root directory `.mem8` (if permissions allow)
   - Each subdirectory inherits parent context
   - Child `.mem8` files can override or extend parent context
   - Creates a semantic tree that parallels the file tree

3. **Performance Optimization**
   - **Quick Check**: CRC based on file modification dates
     ```
     directory_crc = CRC32(
       dir_mtime + 
       sum(file_mtimes) + 
       .mem8_mtime
     )
     ```
   - **Cache Hit**: If CRC matches, use cached context
   - **Full Verification**: Optional SHA256 hash of actual content
   - **Incremental Updates**: Only reprocess changed directories

4. **Context Queue System**
   - If no `.mem8` exists, queue from nearest parent
   - AI can suggest `.mem8` content based on file analysis
   - Auto-generate draft `.mem8` files for review

### Example Use Cases

1. **Project Understanding**
   ```bash
   smart-tree analyze /project --with-context
   ```
   Returns tree with semantic annotations from `.mem8` files

2. **Context-Aware Search**
   ```bash
   smart-tree find --context "rust_library" --concept "wave_patterns"
   ```
   Finds directories tagged with specific concepts

3. **AI-Friendly Navigation**
   - "Show me all test directories" 
   - "Find the audio processing modules"
   - "What directories handle user authentication?"

### .mem8 File Schema


### Implementation Benefits

1. **Speed**: CRC checks are near-instant
2. **Context**: Rich semantic understanding without parsing files
3. **Scalability**: Works with massive codebases
4. **AI-Friendly**: Provides exactly what LLMs need to understand structure
5. **Version Control**: `.mem8` files can be tracked in git
6. **Flexible**: Can be hand-written or AI-generated

### Future Extensions

1. **Cross-Reference System**: `.mem8` files can reference other directories
2. **Semantic Diff**: Show what changed conceptually, not just files
3. **Project Templates**: Standard `.mem8` templates for common project types
4. **Integration**: IDEs could use `.mem8` for better project navigation

### Integration with Existing Smart-Tree Features

1. **Enhanced analyze_directory**
   ```bash
   smart-tree analyze /project --mode=quantum-semantic --use-mem8
   ```
   Would incorporate `.mem8` context into the semantic analysis

2. **Context-Aware quick_tree**
   ```bash
   smart-tree quick /project --with-context
   ```
   Shows tree with inline semantic annotations from `.mem8`

3. **Smart project_overview**
   - Auto-detects project type from root `.mem8`
   - Uses subdirectory purposes for better summaries
   - Highlights important files from `.mem8` metadata

4. **Semantic Code Search**
   ```bash
   smart-tree search --concept "wave_patterns" --in-context
   ```
   Searches within directories tagged with specific concepts

### Example Output with .mem8 Integration
```
/home/hue/source/MEM8 [rust_workspace: Wave-based memory system]
├── crates/ [modular components]
│   ├── mem8-core/ [foundation: wave math, traits] ✓ compiles
│   ├── mem8-grid/ [spatial storage] ✓ compiles
│   └── mem8-vector/ [SIMD vectors] ⚠️ needs fixes
├── docs/ [documentation & research]
└── scripts/ [build & management]
```

---

*Last Updated: 2025-01-04*
*For: MEM8 Project Reorganization*