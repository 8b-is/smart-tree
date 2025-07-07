# MCP Tool Consolidation Proposal

## Current State: 23 Individual Tools
Currently smart-tree exposes 23 separate MCP tools, which is overwhelming and approaching tool limits.

## Proposed Consolidation: 6 Core Tools

### 1. `find` - Universal File Finder
Consolidates 10 current tools into one powerful finder with a `type` parameter:
```json
{
  "name": "find",
  "description": "Universal file finder with filters for type, size, date, and patterns",
  "inputSchema": {
    "properties": {
      "path": {"type": "string", "description": "Path to search in (defaults to current directory)", "default": "."},
      "type": {
        "type": "string",
        "enum": ["all", "code", "config", "docs", "tests", "build", "large", "recent", "duplicates", "empty"],
        "description": "Type of files to find",
        "default": "all"
      },
      "languages": {
        "type": "array",
        "items": {"type": "string"},
        "description": "For type=code: languages to search (python, rust, js, etc.)"
      },
      "pattern": {"type": "string", "description": "Regex pattern for names"},
      "extension": {"type": "string", "description": "File extension filter"},
      "min_size": {"type": "string", "description": "Minimum size (e.g., '1M')"},
      "max_size": {"type": "string", "description": "Maximum size"},
      "newer_than": {"type": "string", "description": "Date filter (YYYY-MM-DD)"},
      "older_than": {"type": "string", "description": "Date filter (YYYY-MM-DD)"},
      "days": {"type": "integer", "description": "For type=recent: files from last N days"},
      "content": {"type": "string", "description": "Search within file contents"},
      "show_content": {"type": "boolean", "description": "Show matching lines for content search", "default": true},
      "context_lines": {"type": "integer", "description": "Lines of context around matches", "default": 2},
      "limit": {"type": "integer", "description": "Max results to show", "default": 100}
    }
  }
}
```

**Replaces:**
- find_files → `find` (with pattern)
- find_code_files → `find --type code`
- find_config_files → `find --type config`
- find_documentation → `find --type docs`
- find_tests → `find --type tests`
- find_build_files → `find --type build`
- find_large_files → `find --type large` or `find --min-size 10M`
- find_recent_changes → `find --type recent` or `find --newer-than`
- find_duplicates → `find --type duplicates`
- find_empty_directories → `find --type empty`
- search_in_files → `find --content "pattern"`

### 2. `analyze` - Multi-Mode Directory Analyzer
Consolidates analysis tools with mode selection:
```json
{
  "name": "analyze",
  "description": "Analyze directories with various output formats and insights",
  "inputSchema": {
    "properties": {
      "path": {"type": "string", "description": "Path to analyze"},
      "mode": {
        "type": "string",
        "enum": ["tree", "quick", "project", "workspace", "semantic", "stats", "digest"],
        "description": "Analysis mode",
        "default": "tree"
      },
      "format": {
        "type": "string",
        "enum": ["classic", "ai", "quantum", "summary", "json", "csv"],
        "description": "Output format (for mode=tree)",
        "default": "ai"
      },
      "depth": {"type": "integer", "description": "Max depth", "default": 3},
      "compress": {"type": "boolean", "description": "Compress output", "default": true}
    }
  }
}
```

**Replaces:**
- analyze_directory → `analyze --mode tree`
- quick_tree → `analyze --mode quick`
- project_overview → `analyze --mode project`
- analyze_workspace → `analyze --mode workspace`
- semantic_analysis → `analyze --mode semantic`

### 3. `stats` - Statistics and Metrics
Consolidates statistical analysis:
```json
{
  "name": "stats",
  "description": "Get statistics, metrics, and insights about directories",
  "inputSchema": {
    "properties": {
      "path": {"type": "string", "description": "Path to analyze"},
      "type": {
        "type": "string",
        "enum": ["general", "size", "digest", "git"],
        "description": "Type of statistics",
        "default": "general"
      },
      "show_hidden": {"type": "boolean", "default": false}
    }
  }
}
```

**Replaces:**
- get_statistics → `stats --type general`
- directory_size_breakdown → `stats --type size`
- get_digest → `stats --type digest`
- get_git_status → `stats --type git`

### 4. `compare` - Directory Comparison
Remains as a standalone tool but enhanced:
```json
{
  "name": "compare",
  "description": "Compare directories, branches, or track changes",
  "inputSchema": {
    "properties": {
      "path1": {"type": "string", "description": "First path"},
      "path2": {"type": "string", "description": "Second path (optional for git)"},
      "mode": {
        "type": "string",
        "enum": ["directories", "git-changes", "branch-diff"],
        "default": "directories"
      }
    }
  }
}
```

### 5. `batch` - Batch Operations
New tool for complex workflows:
```json
{
  "name": "batch",
  "description": "Perform batch operations on multiple files",
  "inputSchema": {
    "properties": {
      "operation": {
        "type": "string",
        "enum": ["read-files", "replace", "analyze-all"],
        "description": "Operation to perform"
      },
      "files": {
        "type": "array",
        "items": {"type": "string"},
        "description": "Files to operate on (from find results)"
      },
      "pattern": {"type": "string", "description": "For replace: pattern to find"},
      "replacement": {"type": "string", "description": "For replace: replacement text"}
    }
  }
}
```

### 6. `info` - Server Information
Simplified server info:
```json
{
  "name": "info",
  "description": "Get Smart Tree server information and capabilities",
  "inputSchema": {
    "properties": {
      "topic": {
        "type": "string",
        "enum": ["server", "formats", "examples", "limits"],
        "default": "server"
      }
    }
  }
}
```

## Benefits of Consolidation

1. **Easier Discovery**: 6 tools instead of 23 - users can actually remember them
2. **Powerful Parameters**: Each tool is more flexible with options
3. **Consistent Interface**: Similar patterns across all tools
4. **Future-Proof**: Easy to add new types/modes without new tools
5. **Under Tool Limits**: Well under the 100 tool limit
6. **Logical Grouping**: Tools are grouped by what they do, not how they do it

## Migration Examples

```bash
# Old way
find_code_files --languages python,rust
find_recent_changes --days 7
search_in_files --keyword "TODO"

# New way
find --type code --languages python,rust
find --type recent --days 7
find --content "TODO"

# Old way
analyze_directory --mode quantum-semantic
quick_tree --depth 3
project_overview

# New way
analyze --mode tree --format quantum
analyze --mode quick --depth 3
analyze --mode project
```

## Implementation Notes

1. Keep backward compatibility by aliasing old tool names for a transition period
2. Use subcommand-style parsing internally to route to appropriate handlers
3. Share common functionality (path validation, caching, etc.)
4. Better error messages with suggestions for common operations
5. Include examples in tool descriptions

This consolidation would make smart-tree's MCP interface much more intuitive and powerful!