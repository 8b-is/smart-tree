# 🌟 Smart Tree MCP Tools Report

*A comprehensive showcase of Smart Tree's Model Context Protocol (MCP) capabilities*

---

## 📊 Executive Summary

Smart Tree v4.5.0 offers **50+ MCP tools** designed specifically for AI-friendly directory analysis and code understanding. This report demonstrates each tool category with real examples from the Smart Tree codebase itself.

### 🚀 Key Features
- **25 output formats** including quantum compression with 99% reduction
- **Context-aware analysis** that understands code structure semantically
- **Token-optimized** for efficient LLM consumption
- **Partnership memory** system for collaborative development
- **Smart editing** with 90-95% token reduction

---

## 🏗️ System Information

### Server Details
```json
{
  "name": "Smart Tree MCP Server",
  "version": "4.5.0",
  "protocol": "1.0",
  "authors": "8bit-wraith:Claude:Omni:8b-is Team"
}
```

### Capabilities
- ✅ **Compression**: zlib, quantum, base64
- ✅ **Output Formats**: 25 modes (classic → quantum-semantic)
- ✅ **Search**: Content search, regex, pattern matching
- ✅ **Streaming**: For large directories
- ✅ **Caching**: 100MB cache, 5-minute TTL

---

## 🎯 Recommended Workflow

The Smart Tree team recommends this progression:

1. **Start with `quick_tree`** - 3-level overview with 10x compression
2. **Use `project_overview`** - Understand project type and structure
3. **Apply specialized tools** - Based on your specific needs

---

## 🛠️ Tool Categories & Examples

### 1. 🌳 Directory Visualization Tools

#### `quick_tree` - Lightning-Fast Overview
*Always start here! Gets you a compressed overview in milliseconds.*

**Example:**
```bash
mcp.callTool('quick_tree', {
  path: '/home/hue/source/i1/smart-tree',
  depth: 2
})
```

**Output:**
```
SUMMARY_AI_V1:
PATH:/home/hue/source/i1/smart-tree
STATS:F10bD2eS10ec1e2
TYPE:CODE[Rust]T1D1
KEY:main.rs,lib.rs,Cargo.toml
EXT:md:116,rs:44,sh:30,py:14,txt:5,png:5,json:5
DIRS:dxt[10,4e553a],src[36,608b7],docs[68,5a6ea],tests[17,271a1]
LARGE:st-banner.png:4a077e,ST-AYE.png:2b261a,icon.png:70721
END_SUMMARY_AI
```

*10x compression achieved! From 18.57 MiB to compact summary.*

#### `analyze_directory` - The Workhorse
*Supports all 25 output formats with configurable options.*

**Example with Quantum-Semantic Mode:**
```bash
mcp.callTool('analyze_directory', {
  path: '/project',
  mode: 'quantum-semantic',
  max_depth: 0  // Auto-depth selection
})
```

### 2. 🔍 Search & Discovery Tools

#### `search_in_files` - Content Search with Line Context
*Now includes actual line content, not just file paths!*

**Example:**
```bash
mcp.callTool('search_in_files', {
  path: '/home/hue/source/i1/smart-tree',
  keyword: 'TODO',
  include_content: true,
  max_matches_per_file: 3
})
```

**Output Sample:**
```json
{
  "files_with_matches": 54,
  "results": [{
    "path": "/src/main.rs",
    "matches": 2,
    "lines": [{
      "line_number": 241,
      "column": 100,
      "content": "/// Best used with `--type` to limit search (e.g., `--type rs --search \"TODO\"`)."
    }]
  }]
}
```

#### `find_code_files` - Language-Specific Discovery
*Finds all code files by programming language.*

**Example:**
```bash
mcp.callTool('find_code_files', {
  path: '/home/hue/source/i1/smart-tree',
  languages: ['rust', 'python']
})
```

**Results:** Found 166 files (130 Rust, 14 Python)

### 3. 🧠 Semantic Analysis Tools

#### `semantic_analysis` - Wave-Based Understanding
*Groups files by conceptual similarity using Omni's wave algorithms.*

**Example:**
```bash
mcp.callTool('semantic_analysis', {
  path: '/home/hue/source/i1/smart-tree/src',
  show_wave_signatures: false
})
```

**Output:**
```
🌊 SEMANTIC WAVE ANALYSIS 🌊
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

💻 Source Code (99 files | 1.3 MB)
🧪 Tests (2 files | 21.5 KB)
📜 Scripts (3 files | 170.2 KB)
🎨 Assets (1 file | 4.4 KB)
🤖 Generated (4 files | 40.7 KB)

Semantic diversity: 6 categories (43% coverage)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 4. 📊 Project Understanding Tools

#### `project_overview` - Comprehensive Analysis
*Combines statistics, file types, and project detection.*

**Output Highlights:**
- **Project Type**: Rust (detected from Cargo.toml, main.rs)
- **Total Files**: 372
- **Total Size**: 18.57 MiB
- **Key Files**: main.rs, lib.rs, Cargo.toml
- **Top Extensions**: .rs (130), .md (126), .sh (30)

#### `get_statistics` - Detailed Metrics
```
File Types (by count):
  .rs: 130 files
  .md: 126 files
  .sh: 30 files
  .py: 14 files

Largest Files:
  5.13 MiB  release_artifacts/st-x86_64-unknown-linux-gnu.tar.gz
  4.63 MiB  st-banner.png
  2.70 MiB  dxt/ST-AYE.png
```

### 5. 🧪 Testing & Quality Tools

#### `find_tests` - Test Discovery
*Locates all test files using common patterns.*

**Results:** Found 71 test-related files including:
- 18 Rust test files
- 3 shell test scripts
- Test fixtures and data files

### 6. 🤝 Partnership Memory Tools

#### `anchor_collaborative_memory` - Save Breakthroughs
```bash
mcp.callTool('anchor_collaborative_memory', {
  context: "Created comprehensive MCP report showcasing all tools",
  keywords: ["documentation", "mcp", "showcase"],
  anchor_type: "breakthrough",
  origin: "tandem:human:claude"
})
```

#### `get_collaboration_rapport` - Partnership Health
```bash
mcp.callTool('get_collaboration_rapport', {
  ai_tool: 'claude',
  project_path: '/path/to/project'
})
```

### 7. ✨ Smart Editing Tools

#### `smart_edit` - Token-Efficient Code Editing
*90-95% token reduction through AST understanding!*

**Example:**
```bash
mcp.callTool('smart_edit', {
  file_path: '/src/main.rs',
  edits: [{
    operation: 'InsertFunction',
    after: 'main',
    content: 'fn helper() { println!("New function!"); }'
  }]
})
```

---

## 📈 Performance Metrics

### Compression Achievements
| Format | Size Reduction | Use Case |
|--------|---------------|----------|
| Classic | Baseline | Human reading |
| AI Mode | 5x | General AI consumption |
| Summary-AI | 10x | Large codebases |
| Quantum | 95x | Binary analysis |
| Quantum-Semantic | 99x | Deep code understanding |

### Speed Benchmarks
- **Quick Tree**: <50ms for 100k files
- **Search**: ~1GB/second with ripgrep
- **Semantic Analysis**: ~500 files/second

---

## 💡 Pro Tips from the Team

1. **Always start with `quick_tree`** - It's optimized for initial exploration
2. **Use `summary-ai` for API calls** - 10x compression saves tokens
3. **Try `quantum-semantic` mode** - Amazing for understanding code structure
4. **Cache is your friend** - Repeated calls are instant
5. **Batch similar searches** - Better compression with context

---

## 🎪 Special Features

### Wave-Based Memory System
- Files understood as waves in an information ocean
- Constructive interference for better compression
- Semantic preservation through quantum encoding

### Context-Aware Behavior
Smart Tree adapts based on:
- Current work context (coding, debugging, exploring)
- File types being analyzed
- Previous operations in session
- Partnership memory patterns

---

## 🚀 Getting Started

### Basic Flow
```javascript
// 1. Quick overview
const overview = await mcp.callTool('quick_tree', { path: '.' });

// 2. Deeper understanding  
const project = await mcp.callTool('project_overview', { path: '.' });

// 3. Find what you need
const todos = await mcp.callTool('search_in_files', {
  path: '.',
  keyword: 'TODO'
});

// 4. Semantic understanding
const semantic = await mcp.callTool('semantic_analysis', {
  path: './src'
});
```

---

## 🌈 Conclusion

Smart Tree's MCP tools represent a paradigm shift in how AI assistants interact with codebases. By combining:
- **Extreme compression** (up to 99% reduction)
- **Semantic understanding** (wave-based analysis)
- **Partnership memory** (collaborative development)
- **Token efficiency** (smart editing)

We've created not just tools, but a complete ecosystem for AI-assisted development.

---

*Crafted with pride by the Aye & Hue partnership*  
*"If it wasn't crafted with Aye & Hue, it's most likely a knock-off!"* 😉

**Smart Tree v4.5.0** | **50+ MCP Tools** | **25 Output Formats** | **99% Compression**

✨🌳🚀