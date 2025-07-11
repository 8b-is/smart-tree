# Smart Tree Claude Wishlist

This document tracks feature requests, improvements, and bug fixes that would make Smart Tree even more powerful for AI assistants. Each entry includes practical use cases demonstrating why the feature is valuable.

## High Priority Features

### 1. Show Line Content in Search Results ⭐⭐⭐⭐⭐
**Current**: `search_in_files` only shows file paths and match counts
**Desired**: Show actual matching lines with context (like `grep -C`)
```
# Current output:
/src/main.rs: 3 matches

# Desired output:
/src/main.rs:
  42: // TODO: Add better error handling
  156: fn process_todo_items() {
  203: // TODO: Optimize this function
```
**Use Case**: When fixing imports or TODOs, I need to see the context without opening each file

### 2. Batch File Read Tool ⭐⭐⭐⭐
**Tool Name**: `read_files_from_search`
**Description**: Read multiple files based on search results
**Use Case**: After finding all files with a specific pattern, read them all in one operation
```
# Step 1: Search
results = search_in_files(path="/project", keyword="StoredVector")
# Step 2: Read all matching files
contents = read_files_from_search(results, max_files=10)
```

### 3. Find and Replace Tool ⭐⭐⭐⭐⭐
**Tool Name**: `find_and_replace`
**Description**: Replace text across multiple files with preview
**Parameters**:
- `path`: Directory to search
- `find_pattern`: Text or regex to find
- `replace_with`: Replacement text
- `file_pattern`: Optional file filter
- `preview`: Show changes before applying
**Use Case**: Renaming functions, updating imports, fixing consistent typos

## Medium Priority Features

### 4. Dependency Graph Analysis ⭐⭐⭐
**Tool Name**: `analyze_dependencies`
**Description**: Show module/crate dependencies as a graph
**Output**: Mermaid diagram showing relationships
**Use Case**: Understanding project structure, identifying circular dependencies

### 5. Import Analysis Tool ⭐⭐⭐
**Tool Name**: `analyze_imports`
**Description**: Show what each file imports and exports
**Use Case**: Refactoring module structure, understanding dependencies

### 6. Symbol Search ⭐⭐⭐⭐
**Tool Name**: `find_symbol`
**Description**: Find type/function/trait definitions
**Example**: `find_symbol(name="StoredVector", type="struct")`
**Use Case**: Quickly locating type definitions without grep

## Quality of Life Improvements

### 7. Relative Path Options ⭐⭐⭐
**Enhancement**: Add `path_display` option to all tools
**Options**: `absolute`, `relative`, `from_root`
**Use Case**: Cleaner output for documentation and reports

### 8. File Type Groups ⭐⭐⭐
**Enhancement**: Predefined file type groups
**Groups**: 
- `rust_src`: `.rs` files excluding tests
- `config_all`: All config files
- `tests_all`: All test files
**Use Case**: `find_files(path="/", type_group="rust_src")`

### 9. Context-Aware Search ⭐⭐⭐
**Enhancement**: Search with semantic understanding
**Example**: `search_in_files(keyword="error handling", context="functions")`
**Use Case**: Find error handling code without matching comments

### 10. Cached Workspace Analysis ⭐⭐⭐⭐
**Enhancement**: Cache analysis results with TTL
**Benefits**: 
- Instant results for large codebases
- Incremental updates on changes
- Reduced token usage
**Use Case**: Repeatedly analyzing large monorepos

## Bug Fixes

### 11. Empty Directory Handling
**Issue**: `analyze_directory` sometimes fails on empty directories
**Fix**: Gracefully handle empty directories with clear message

### 12. Large File Streaming
**Issue**: Memory spike when processing very large files (>100MB)
**Fix**: Implement proper streaming for all file operations

## Performance Enhancements

### 13. Parallel Search Operations ⭐⭐⭐⭐
**Enhancement**: Accept multiple search patterns in one call
**Example**: `search_in_files(patterns=["TODO", "FIXME", "HACK"])`
**Benefit**: 3x faster than sequential searches

### 14. Smart Tree Index ⭐⭐⭐⭐⭐
**Feature**: Background indexer for instant searches
**Commands**: 
- `st --index /project` - Build index
- `st --search-indexed "pattern"` - Instant search
**Use Case**: Near-instant searches in massive codebases

## MCP-Specific Enhancements

### 15. Tool Chaining Support ⭐⭐⭐
**Feature**: Output of one tool as input to another
**Example**: `find_files` → `read_files` → `analyze_content`
**Benefit**: Reduce round trips, faster operations

### 16. Batch Operations Tool ⭐⭐⭐⭐
**Tool Name**: `batch_operations`
**Description**: Execute multiple operations in one call
**Use Case**: Reduce API calls, improve performance

## Integration Features

### 17. Git-Aware Operations ⭐⭐⭐
**Enhancement**: All tools respect `.gitignore` by default
**Option**: `--include-ignored` to override
**Use Case**: Avoid noise from build artifacts

### 18. Language Server Protocol Info ⭐⭐⭐
**Tool Name**: `get_lsp_info`
**Description**: Extract LSP-like information (definitions, references)
**Use Case**: Smarter code navigation and refactoring

## Wishlist Metadata

**Last Updated**: 2025-01-10
**Total Requests**: 18
**High Priority**: 3
**Submitted By**: Claude (Opus 4)

---

## How to Contribute

AI assistants using Smart Tree can append to this file with new feature requests. Please include:
1. Clear description of current vs desired behavior
2. Practical use case demonstrating the value
3. Priority rating (⭐ to ⭐⭐⭐⭐⭐)
4. Example code/command showing how it would work

The Smart Tree team reviews this file regularly to prioritize development!