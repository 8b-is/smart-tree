# Enhanced `find` Tool Design

## Overview
The new consolidated `find` tool combines file discovery and content search with intelligent defaults and powerful options.

## Key Features

### 1. Smart Path Defaults
```json
{
  "path": {
    "type": "string",
    "description": "Path to search in. Defaults to current directory if not specified.",
    "default": "."
  }
}
```

### 2. Comprehensive Search Schema
```json
{
  "name": "find",
  "description": "Universal smart finder for files and content with AI-optimized output",
  "inputSchema": {
    "type": "object",
    "properties": {
      "path": {
        "type": "string",
        "description": "Path to search in (defaults to current directory)",
        "default": "."
      },
      "type": {
        "type": "string",
        "enum": ["all", "code", "config", "docs", "tests", "build", "large", "recent", "duplicates", "empty", "modified"],
        "description": "Preset file type filters",
        "default": "all"
      },
      "pattern": {
        "type": "string",
        "description": "Regex pattern for file/directory names"
      },
      "content": {
        "type": "string",
        "description": "Search within file contents (regex supported)"
      },
      "languages": {
        "type": "array",
        "items": {"type": "string"},
        "description": "Programming languages (for type=code): python, rust, js, etc."
      },
      "extensions": {
        "type": "array",
        "items": {"type": "string"},
        "description": "File extensions to include (e.g., ['rs', 'py'])"
      },
      "exclude": {
        "type": "array",
        "items": {"type": "string"},
        "description": "Patterns to exclude (e.g., ['*.lock', 'node_modules'])"
      },
      "size": {
        "type": "object",
        "properties": {
          "min": {"type": "string", "description": "Minimum size (e.g., '1K', '10M')"},
          "max": {"type": "string", "description": "Maximum size"}
        }
      },
      "date": {
        "type": "object",
        "properties": {
          "after": {"type": "string", "description": "Modified after (YYYY-MM-DD or relative: '7 days ago')"},
          "before": {"type": "string", "description": "Modified before"},
          "within": {"type": "integer", "description": "Within last N days"}
        }
      },
      "depth": {
        "type": "integer",
        "description": "Maximum directory depth",
        "default": 10
      },
      "limit": {
        "type": "integer",
        "description": "Maximum number of results",
        "default": 100
      },
      "output": {
        "type": "object",
        "properties": {
          "format": {
            "type": "string",
            "enum": ["simple", "detailed", "tree", "grouped"],
            "description": "Output format",
            "default": "simple"
          },
          "show_content": {
            "type": "boolean",
            "description": "Include matching lines for content searches",
            "default": true
          },
          "context_lines": {
            "type": "integer",
            "description": "Lines of context around matches",
            "default": 2
          },
          "group_by": {
            "type": "string",
            "enum": ["none", "directory", "extension", "type"],
            "description": "Group results by",
            "default": "none"
          },
          "sort": {
            "type": "string",
            "enum": ["path", "name", "size", "date", "relevance"],
            "description": "Sort results by",
            "default": "path"
          },
          "relative_paths": {
            "type": "boolean",
            "description": "Show paths relative to search path",
            "default": true
          }
        }
      },
      "smart": {
        "type": "object",
        "properties": {
          "follow_imports": {
            "type": "boolean",
            "description": "Follow import/require statements",
            "default": false
          },
          "semantic_ranking": {
            "type": "boolean",
            "description": "Rank by semantic relevance (uses .mem8 files)",
            "default": false
          },
          "auto_expand": {
            "type": "boolean",
            "description": "Auto-expand to related files",
            "default": false
          }
        }
      }
    },
    "required": []
  }
}
```

### 3. Intelligent File Type Presets

```javascript
const FILE_TYPE_PRESETS = {
  code: {
    extensions: ["rs", "py", "js", "ts", "go", "java", "cpp", "c", "rb", "php"],
    exclude: ["*.min.js", "*.lock", "vendor/", "node_modules/", "target/", "dist/"]
  },
  config: {
    patterns: ["*.toml", "*.yaml", "*.yml", "*.json", "*.ini", ".env*", ".*rc"],
    names: ["Cargo.toml", "package.json", "requirements.txt", "Gemfile", "Makefile"]
  },
  docs: {
    extensions: ["md", "txt", "rst", "adoc"],
    patterns: ["README*", "CHANGELOG*", "LICENSE*", "CONTRIBUTING*"]
  },
  tests: {
    patterns: ["test_*.py", "*_test.go", "*.test.js", "*.spec.ts"],
    paths: ["tests/", "test/", "__tests__/", "spec/"]
  },
  build: {
    names: ["Makefile", "CMakeLists.txt", "build.gradle", "pom.xml"],
    patterns: ["*.mk", "BUILD*", "*.build"]
  },
  large: {
    size: { min: "10M" }
  },
  recent: {
    date: { within: 7 }
  },
  modified: {
    // Git-aware: files with uncommitted changes
    git_status: ["modified", "new", "deleted"]
  }
};
```

### 4. Enhanced Content Search Output

```json
{
  "summary": {
    "total_files_searched": 1523,
    "files_with_matches": 12,
    "total_matches": 47,
    "search_time_ms": 234
  },
  "results": [
    {
      "file": "src/memory/grid.rs",
      "relative_path": "src/memory/grid.rs",
      "matches": [
        {
          "line": 42,
          "column": 17,
          "content": "    let cell = BindCell::new(config)?;",
          "highlight": [17, 25],  // Start and end of match
          "context": {
            "before": [
              "    // Initialize the binding cell",
              "    let config = CellConfig::default();"
            ],
            "after": [
              "    cell.connect(wave_pool)?;",
              "    Ok(cell)"
            ]
          },
          "semantic": {
            "function": "initialize_cell",
            "class": "GridManager",
            "importance": "high"  // From .mem8 metadata
          }
        }
      ],
      "file_info": {
        "size": 4523,
        "modified": "2025-01-05T10:30:00Z",
        "language": "rust",
        "encoding": "utf-8"
      }
    }
  ],
  "grouped": {
    "by_directory": {
      "src/memory": 3,
      "src/core": 2,
      "tests": 7
    }
  }
}
```

### 5. Smart Features

#### Auto-Detection Examples:
```python
# Searching for a function? Automatically search in code files
find(content="def process_data")  # Auto sets type="code"

# Searching for TODO? Include context
find(content="TODO|FIXME|XXX")  # Auto sets show_content=true, context_lines=3

# Looking for a class? Use semantic search
find(content="class StorageManager")  # Auto enables semantic_ranking

# Path looks like project root? Default to current directory
find()  # Uses path="." automatically
```

#### Import Following:
```python
# Find all files that import a specific module
find(content="from mymodule import", smart={"follow_imports": true})

# Results include:
# - Direct imports
# - Indirect imports (files that import files that import mymodule)
# - Dependency graph visualization
```

#### Semantic Ranking (using .mem8 files):
```python
find(content="wave", smart={"semantic_ranking": true})

# Results prioritized by:
# 1. Files marked as "important" in .mem8
# 2. Files in directories with matching concepts
# 3. Files with high coupling to search term
```

### 6. Performance Optimizations

1. **Parallel Search**: Use rayon for concurrent file processing
2. **Smart Caching**: Cache file metadata and content indices
3. **Progressive Results**: Stream results as found for large searches
4. **Index Support**: Use .mem8 indices when available
5. **Git-Aware**: Skip .git directories automatically

### 7. Example Usage

```python
# Simple search with smart defaults
find()  # Lists all files in current directory

# Find Python files modified this week
find(type="code", languages=["python"], date={"within": 7})

# Search for a specific function with context
find(content="async def fetch_data", output={"show_content": true})

# Find large log files
find(pattern="*.log", size={"min": "100M"})

# Complex search with grouping
find(
    type="code",
    content="TODO|FIXME",
    output={
        "format": "detailed",
        "group_by": "directory",
        "sort": "date"
    }
)

# Find duplicate files (by content hash)
find(type="duplicates", output={"format": "grouped"})

# Semantic code search
find(
    content="memory allocation",
    smart={"semantic_ranking": true},
    output={"show_content": true}
)
```

### 8. Migration Path

Old tools → New `find` equivalents:

```bash
# find_files
find_files(pattern="*.rs") → find(pattern="*.rs")

# find_code_files  
find_code_files(languages=["python"]) → find(type="code", languages=["python"])

# search_in_files
search_in_files(keyword="TODO") → find(content="TODO")

# find_large_files
find_large_files(min_size="10M") → find(type="large") or find(size={"min": "10M"})

# find_recent_changes
find_recent_changes(days=7) → find(type="recent") or find(date={"within": 7})
```

## Benefits

1. **One Tool to Rule Them All**: Single, powerful interface
2. **Smart Defaults**: Intelligent behavior based on input
3. **Rich Output**: Detailed, contextual results
4. **Performance**: Optimized for large codebases
5. **AI-Friendly**: Output designed for LLM consumption
6. **Extensible**: Easy to add new file types and smart features

This design makes the `find` tool the Swiss Army knife of file discovery and search!