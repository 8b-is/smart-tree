# Implementation Plan for CLAUDE-WISHLIST Top Priorities

## 1. Show Line Content in Search Results (Priority: ⭐⭐⭐⭐⭐)

### Current State
```rust
// search_in_files currently returns:
{
  "path": "/src/main.rs",
  "matches": 3
}
```

### Desired State
```rust
{
  "path": "/src/main.rs",
  "matches": 3,
  "lines": [
    {
      "line_number": 42,
      "content": "// TODO: Add better error handling",
      "column": 3
    },
    {
      "line_number": 156,
      "content": "fn process_todo_items() {",
      "column": 14
    },
    {
      "line_number": 203,
      "content": "// TODO: Optimize this function",
      "column": 3
    }
  ]
}
```

### Implementation Steps

1. **Update search_in_files in mcp/tools.rs**:
   - Add `include_content: bool` parameter (default: true for AI mode)
   - Add `context_lines: Option<usize>` for grep -C like behavior
   - Return line content with matches

2. **Modify the search logic**:
   - Currently uses ripgrep for finding matches
   - Need to capture the actual line content
   - Store line number, column, and text

3. **Add pagination for large results**:
   - Use existing PaginationParams
   - Limit to first N matches per file
   - Add continuation cursor

4. **Optimization considerations**:
   - Stream results for large files
   - Cache frequently searched patterns
   - Use the digest-first workflow

### Code Changes Needed

```rust
// In mcp/tools.rs
async fn search_in_files(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    // ... existing code ...
    
    let include_content = args["include_content"].as_bool().unwrap_or(true);
    let context_lines = args["context_lines"].as_u64().map(|n| n as usize);
    let max_matches_per_file = args["max_matches_per_file"].as_u64().unwrap_or(100);
    
    if include_content {
        // Use ripgrep with line output
        let output = Command::new("rg")
            .arg("--json")  // JSON output includes line content
            .arg("--max-count").arg(max_matches_per_file.to_string())
            .arg(pattern)
            .arg(path)
            .output()?;
        
        // Parse JSON lines from ripgrep
        // Each line is a JSON object with type, data, etc.
    }
}
```

## 2. Find and Replace Tool (Priority: ⭐⭐⭐⭐⭐)

### Tool Specification
```rust
tool_name: "find_and_replace"
parameters:
  - path: Directory to search
  - find_pattern: Text or regex to find
  - replace_with: Replacement text
  - file_pattern: Optional file filter (e.g., "*.rs")
  - preview: bool (show changes before applying)
  - dry_run: bool (don't actually make changes)
```

### Safety Features
- Always create backup before changes
- Show diff preview
- Require confirmation for >10 files
- Exclude binary files automatically
- Respect .gitignore

## 3. Symbol Search (Priority: ⭐⭐⭐⭐)

### Leveraging tree-sitter
We already have tree-sitter integration! We can use it for symbol search:

```rust
tool_name: "find_symbol"
parameters:
  - name: Symbol name to find
  - type: Optional - "function", "struct", "trait", "class", "method"
  - path: Directory to search
  - language: Optional - auto-detect from file extension
```

### Implementation
- Use existing tree-sitter parsers
- Build symbol index on first search
- Cache results with file modification times
- Return definition location + signature

## Timeline Estimate

1. **Week 1**: Show Line Content in Search Results
   - Day 1-2: Implement basic line content return
   - Day 3-4: Add context lines and pagination
   - Day 5: Testing and optimization

2. **Week 2**: Find and Replace Tool
   - Day 1-2: Basic find/replace with preview
   - Day 3-4: Safety features and confirmations
   - Day 5: Testing with various file types

3. **Week 3**: Symbol Search
   - Day 1-2: tree-sitter integration
   - Day 3-4: Caching and indexing
   - Day 5: Testing across languages

## Quick Wins We Can Do Today

1. **Add line content to search_in_files** - Just the basic version without context
2. **Create find_and_replace with dry_run** - Start with preview mode only
3. **Leverage existing tree-sitter for basic symbol search**

These three features would dramatically improve the AI coding experience!