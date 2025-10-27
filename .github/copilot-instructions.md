# GitHub Copilot Instructions for Smart Tree MCP Tools

This document provides guidelines for GitHub Copilot (and other AI assistants) on how to effectively use Smart Tree's Model Context Protocol (MCP) tools.

## 🎯 Quick Start Philosophy

Smart Tree provides 40+ MCP tools organized in a **three-lane escalation pattern**:

1. **🔍 EXPLORE** - Start here! Discovery and overview tools (read-only, safe)
2. **🧪 ANALYZE** - Deep analysis and search tools (read-only, intensive)
3. **⚡ ACT** - Modification and write operations (requires caution)

**Always start with EXPLORE tools before moving to ANALYZE or ACT.**

## ⚠️ Critical: Required Parameters

**IMPORTANT**: Many tools REQUIRE the `path` parameter. Do NOT omit required parameters!

### Common Mistakes to Avoid

❌ **WRONG** - Missing required `path` parameter:
```json
{
  "keyword": "def ",
  "file_type": "py",
  "include_content": true
}
```

✅ **CORRECT** - Include required `path`:
```json
{
  "path": ".",
  "keyword": "def ",
  "file_type": "py",
  "include_content": true
}
```

## 🔍 EXPLORE Lane - Start Here!

These tools are perfect for initial exploration. They're read-only, fast, and safe.

### `quick_tree` - Your First Tool
**Always start here!** Get a 3-level directory overview.

```json
{
  "path": ".",
  "depth": 3
}
```

**When to use**:
- First time exploring any directory
- Getting a quick overview of project structure
- Before using other tools

**Required**: `path`
**Optional**: `depth` (default: 3)

### `project_overview` - Comprehensive Analysis
Get automatic project type detection and key files.

```json
{
  "path": "."
}
```

**When to use**:
- Understanding new codebases
- Identifying project type (Node.js, Rust, Python, etc.)
- Finding important configuration files

**Required**: `path`

### `server_info` - Discover Capabilities
Learn what Smart Tree can do.

```json
{}
```

**When to use**:
- First time using Smart Tree
- Checking available features
- Understanding compression options

**Required**: None

### `get_statistics` - Project Metrics
Get file counts, size distribution, and patterns.

```json
{
  "path": ".",
  "show_hidden": false
}
```

**When to use**:
- Understanding project composition
- Finding large files
- Analyzing file type distribution

**Required**: `path`
**Optional**: `show_hidden` (default: false)

## 🧪 ANALYZE Lane - Deep Dive

Use these after EXPLORE phase for detailed investigation.

### `search_in_files` - Content Search
**Most commonly used!** Search for keywords in files.

```json
{
  "path": ".",
  "keyword": "TODO",
  "file_type": "rs",
  "include_content": true,
  "max_matches_per_file": 20
}
```

**When to use**:
- Finding function implementations
- Searching for TODOs or FIXMEs
- Locating specific code patterns
- Finding where something is used

**Required**: `path`, `keyword`
**Optional**: `file_type`, `case_sensitive`, `include_content`, `context_lines`, `max_matches_per_file`

**Common patterns**:
```json
// Find all TODOs in Python files
{"path": ".", "keyword": "TODO", "file_type": "py"}

// Find function definition
{"path": "src", "keyword": "fn handle_request", "file_type": "rs"}

// Case-sensitive search
{"path": ".", "keyword": "ClassName", "case_sensitive": true}
```

### `find_files` - File Discovery
Find files by pattern, size, or date.

```json
{
  "path": ".",
  "pattern": "test_.*\\.rs$",
  "file_type": "rs",
  "max_depth": 5
}
```

**When to use**:
- Finding specific files by name pattern
- Locating files by extension
- Finding recent changes
- Size-based filtering

**Required**: `path`
**Optional**: `pattern`, `file_type`, `entry_type`, `min_size`, `max_size`, `newer_than`, `older_than`, `max_depth`

**Common patterns**:
```json
// Find all test files
{"path": ".", "pattern": "test_.*"}

// Find large files
{"path": ".", "min_size": "10M"}

// Find recent changes
{"path": ".", "newer_than": "2024-01-01"}

// Only directories
{"path": ".", "entry_type": "d"}
```

### `analyze_directory` - Detailed Analysis
The main workhorse for detailed directory analysis.

```json
{
  "path": ".",
  "mode": "quantum-semantic",
  "max_depth": 5,
  "compress": true
}
```

**Modes** (choose based on need):
- `classic` - Human-readable tree view
- `ai` - AI-optimized format (default, 80% token reduction)
- `quantum-semantic` - **RECOMMENDED** for code analysis (includes tokens)
- `summary-ai` - Maximum compression (10x reduction, perfect for large codebases)
- `quantum` - Ultra-compressed binary
- `digest` - Minimal hash

**When to use**:
- After `quick_tree` when you need more details
- Analyzing specific subdirectories
- Getting semantic code understanding

**Required**: `path`
**Optional**: `mode`, `max_depth`, `compress`, `show_hidden`, `respect_gitignore`

### `semantic_analysis` - Code Structure
Deep code understanding with AST analysis.

```json
{
  "path": "src/main.rs",
  "include_imports": true
}
```

**When to use**:
- Understanding code structure
- Finding function definitions
- Analyzing imports and dependencies

**Required**: `path`
**Optional**: `include_imports`, `include_comments`

### `find_tests` - Test Discovery
Locate test files and test functions.

```json
{
  "path": "."
}
```

**When to use**:
- Finding test coverage
- Locating specific tests
- Understanding test structure

**Required**: `path`

### `find_code_files` - Source Code Discovery
Find programming language source files.

```json
{
  "path": ".",
  "language": "rust"
}
```

**When to use**:
- Focusing on implementation files
- Excluding generated code
- Language-specific analysis

**Required**: `path`
**Optional**: `language`

### `find_config_files` - Configuration Discovery
Find config files (package.json, Cargo.toml, etc.).

```json
{
  "path": "."
}
```

**When to use**:
- Understanding project setup
- Finding dependencies
- Locating build configuration

**Required**: `path`

### `find_documentation` - Doc Discovery
Find README, docs, and documentation files.

```json
{
  "path": "."
}
```

**When to use**:
- Finding project documentation
- Locating guides and tutorials
- Understanding documentation structure

**Required**: `path`

## ⚡ ACT Lane - Modifications

**USE WITH CAUTION!** These tools modify files.

### `smart_edit` - File Editing
Edit files with AST-aware changes.

```json
{
  "path": "src/main.rs",
  "operation": "replace",
  "old_content": "fn old_function() {}",
  "new_content": "fn new_function() {}"
}
```

**When to use**:
- Making precise code changes
- Refactoring
- Fixing bugs

**Required**: `path`, `operation`
**Optional**: Depends on operation type

### `track_file_operation` - Change Tracking
Track file modifications for history.

```json
{
  "path": "src/main.rs",
  "operation": "modified",
  "description": "Updated function signature"
}
```

**When to use**:
- After making changes
- Building change history
- Documenting modifications

**Required**: `path`, `operation`
**Optional**: `description`

## 🎯 Best Practices

### 1. Always Start with EXPLORE
```
quick_tree → project_overview → Detailed analysis
```

### 2. Use Specific Paths
❌ Bad: Omitting path
✅ Good: `{"path": "."}`
✅ Better: `{"path": "src/components"}`

### 3. Filter Early
Use `file_type`, `pattern`, and `max_depth` to narrow results:
```json
{
  "path": ".",
  "file_type": "rs",
  "max_depth": 3
}
```

### 4. Choose the Right Mode
- Quick overview? → `quick_tree`
- Code analysis? → `analyze_directory` with `mode: "quantum-semantic"`
- Large codebase? → Use `mode: "summary-ai"` for compression
- Need exact content? → `search_in_files`

### 5. Verify Before ACT
Always use EXPLORE and ANALYZE tools to verify before using ACT tools.

## 🔧 Common Workflows

### Workflow 1: Understanding a New Project
```
1. quick_tree {"path": "."}
2. project_overview {"path": "."}
3. get_statistics {"path": "."}
4. find_documentation {"path": "."}
```

### Workflow 2: Finding Implementation
```
1. quick_tree {"path": "."}
2. search_in_files {"path": ".", "keyword": "function_name"}
3. semantic_analysis {"path": "path/to/file.rs"}
```

### Workflow 3: Finding Configuration
```
1. project_overview {"path": "."}
2. find_config_files {"path": "."}
3. analyze_directory {"path": ".", "mode": "ai", "max_depth": 2}
```

### Workflow 4: Searching for TODOs
```
1. quick_tree {"path": "."}
2. search_in_files {"path": ".", "keyword": "TODO", "include_content": true}
```

### Workflow 5: Analyzing Tests
```
1. find_tests {"path": "."}
2. search_in_files {"path": "tests", "keyword": "test_"}
```

## 📊 Tool Selection Guide

| Goal | Tool | Example |
|------|------|---------|
| First look at project | `quick_tree` | `{"path": "."}` |
| Understand project type | `project_overview` | `{"path": "."}` |
| Find specific text | `search_in_files` | `{"path": ".", "keyword": "TODO"}` |
| Find files by name | `find_files` | `{"path": ".", "pattern": "test_.*"}` |
| Get project stats | `get_statistics` | `{"path": "."}` |
| Detailed analysis | `analyze_directory` | `{"path": ".", "mode": "ai"}` |
| Code structure | `semantic_analysis` | `{"path": "src/main.rs"}` |
| Find tests | `find_tests` | `{"path": "."}` |
| Edit files | `smart_edit` | Use with caution! |

## 🚫 Common Errors and Solutions

### Error: "Missing path"
**Cause**: Required `path` parameter not provided
**Solution**: Always include `{"path": "."}` at minimum

### Error: "Access denied"
**Cause**: Trying to access restricted path
**Solution**: Use `verify_permissions` first: `{"path": "/some/path"}`

### Error: Tool returns empty results
**Cause**: Too restrictive filters or wrong path
**Solution**: 
- Check path exists
- Try without filters first
- Use `show_hidden: true` if looking for hidden files

### Error: Too much output
**Cause**: Not using filters or compression
**Solution**: 
- Use `max_depth` to limit traversal
- Use `file_type` to filter
- Use `mode: "summary-ai"` for compression
- Use `max_matches_per_file` for search results

## 💡 Pro Tips

1. **Token Efficiency**: Use `mode: "quantum-semantic"` or `mode: "summary-ai"` for large codebases
2. **Speed**: `quick_tree` is faster than `analyze_directory` for initial exploration
3. **Precision**: Combine `file_type` with `pattern` for precise file discovery
4. **Context**: Use `include_content: true` in searches to get actual code snippets
5. **Depth Control**: Set `max_depth: 3` for overview, `max_depth: 10` for deep analysis
6. **Git Awareness**: Most tools respect `.gitignore` automatically
7. **Safety**: EXPLORE and ANALYZE tools are read-only and always safe to use

## 🎓 Learning Path

1. **Beginner**: Start with `quick_tree` and `project_overview`
2. **Intermediate**: Use `search_in_files` and `find_files` for targeted discovery
3. **Advanced**: Combine `analyze_directory` with semantic modes for deep analysis
4. **Expert**: Use ACT lane tools for modifications with proper verification

## 📚 Additional Resources

- Full tool list: Use `server_info` tool
- MCP Explorer: Interactive tool for exploring MCP capabilities
- Documentation: Check project README for detailed examples
- Examples: See TERMINAL_EXAMPLES.md for CLI usage patterns

---

**Remember**: When in doubt, start with `quick_tree {"path": "."}` - it's fast, safe, and gives you the lay of the land!
