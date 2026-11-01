# Smart Tree MCP (Model Context Protocol) Guide

Smart Tree's MCP server enables AI assistants to analyze directory structures programmatically. This guide covers everything you need to know about using Smart Tree with AI.

## Table of Contents
- [Quick Start](#quick-start)
- [Features](#features)
- [Tools Reference](#tools-reference)
- [Output Modes](#output-modes)
- [Advanced Features](#advanced-features)
- [Prompts](#prompts)
- [Examples](#examples)
- [Best Practices](#best-practices)

## Quick Start

### 1. Install Smart Tree with MCP enabled
```bash
# Build with MCP feature (now enabled by default)
cargo build --release

# Or install from source
cargo install --path .
```

### 2. Configure your AI assistant
Add to your Claude Desktop config (`claude_desktop_config.json`):
on
```json
{
  "mcpServers": {
    "smart-tree": {
      "command": "/path/to/st",
      "args": ["--mcp"],
      "env": {}
    }
  }
}
```

### 3. Test the connection
```bash
# List available tools
st --mcp-tools

# Show configuration snippet
st --mcp-config
```

## Features

### 🚀 Streaming Output
For large directories, get results as they're discovered:
```json
{
  "name": "analyze_directory",
  "arguments": {
    "path": "/large/codebase",
    "mode": "ai",
    "stream": true
  }
}
```

### 🗜️ Compression
Reduce token usage with zlib compression:
```json
{
  "name": "analyze_directory",
  "arguments": {
    "path": "/project",
    "compress": true
  }
}
```
Output: `COMPRESSED_V1:<hex-encoded-data>`

### 🔍 Content Search
Find files containing specific keywords:
```json
{
  "name": "analyze_directory",
  "arguments": {
    "path": "/src",
    "search": "TODO",
    "file_type": "rs"
  }
}
```

### 📊 Multiple Output Formats
- `ai`: Optimized for LLMs (default)
- `digest`: Ultra-compact SHA256 hash
- `json`: Structured data
- `stats`: Directory statistics only
- `hex`: Fixed-width format
- `classic`: Human-readable tree

## Tools Reference

### analyze_directory
The main workhorse - analyzes directory structures with extensive filtering options.

**Key Parameters:**
- `path`: Directory to analyze
- `mode`: Output format (default: "ai")
- `max_depth`: Traversal depth limit
- `compress`: Enable compression
- `stream`: Enable streaming (ai/hex modes only)
- `find`: Regex pattern for names
- `search`: Search file contents
- `file_type`: Filter by extension
- `min_size`/`max_size`: Size filters
- `newer_than`/`older_than`: Date filters

### find_files
Specialized file search tool.

**Example:**
```json
{
  "name": "find_files",
  "arguments": {
    "path": "/project",
    "pattern": "test_.*\\.rs$",
    "newer_than": "2024-01-01"
  }
}
```

### get_statistics
Get directory statistics without the full tree.

### get_digest
Get a compact SHA256 digest - perfect for:
- Change detection
- Caching keys
- Quick comparisons

## Output Modes

### AI Mode (Default)
Optimized for LLM consumption:
```
📁 project/ (15.2M)
├── 📄 README.md (2.3K)
├── 📁 src/ (8.1M)
│   ├── 📄 main.rs (28K) 🔍
│   └── 📄 lib.rs (1.2K)
└── [📁 target/] (7.1M)

Statistics:
Files: 234 | Dirs: 45 | Total: 15.2M
```

### Digest Mode
Ultra-compact for quick checks:
```
SHA256:a3f5... Files:234 Dirs:45 Size:15.2M
```

### JSON Mode
Structured data for programmatic use:
```json
{
  "path": "/project",
  "nodes": [...],
  "stats": {...}
}
```

## Advanced Features

### Path Display Modes
Control how paths are shown:
- `off`: Names only (default)
- `relative`: Relative to scan root
- `full`: Absolute paths

### Ignore Control
- `show_ignored`: Show ignored items in brackets
- `no_ignore`: Ignore .gitignore files
- `no_default_ignore`: Disable built-in patterns
- `show_hidden`: Include hidden files

### Filesystem Indicators
Enable with `show_filesystems: true`:
- `X`: XFS
- `4`: ext4
- `B`: Btrfs
- `N`: NTFS

## Prompts

Pre-defined prompts guide AI assistants:

### analyze_codebase
```json
{
  "name": "analyze_codebase",
  "arguments": {
    "path": "/my/project",
    "include_hidden": false
  }
}
```

### find_large_files
```json
{
  "name": "find_large_files",
  "arguments": {
    "path": "/project",
    "min_size": "10M",
    "limit": 20
  }
}
```

### recent_changes
```json
{
  "name": "recent_changes",
  "arguments": {
    "path": "/src",
    "days": 7
  }
}
```

### project_structure
```json
{
  "name": "project_structure",
  "arguments": {
    "path": "/project",
    "max_depth": 3
  }
}
```

## Examples

### Example 1: Analyze a Rust Project
```json
{
  "name": "analyze_directory",
  "arguments": {
    "path": "/rust/project",
    "mode": "ai",
    "file_type": "rs",
    "compress": true,
    "show_ignored": true
  }
}
```

### Example 2: Find Recent Large Files
```json
{
  "name": "find_files",
  "arguments": {
    "path": "/downloads",
    "min_size": "100M",
    "newer_than": "2024-01-15"
  }
}
```

### Example 3: Search for TODOs
```json
{
  "name": "analyze_directory",
  "arguments": {
    "path": "/src",
    "search": "TODO",
    "file_type": "py",
    "path_mode": "relative"
  }
}
```

### Example 4: Quick Directory Comparison
```json
{
  "name": "get_digest",
  "arguments": {
    "path": "/project/before"
  }
}
```
Then later:
```json
{
  "name": "get_digest",
  "arguments": {
    "path": "/project/after"
  }
}
```

## Best Practices

### 1. Use Compression for Large Trees
When analyzing large codebases, always enable compression:
```json
{"compress": true}
```

### 2. Stream for Real-time Feedback
For directories with many files, streaming provides immediate feedback:
```json
{"stream": true, "mode": "ai"}
```

### 3. Combine Filters for Precision
Use multiple filters together:
```json
{
  "file_type": "js",
  "min_size": "1K",
  "newer_than": "2024-01-01",
  "search": "import.*react"
}
```

### 4. Use Digest for Change Detection
Before deep analysis, check if anything changed:
```json
{"name": "get_digest", "arguments": {"path": "/project"}}
```

### 5. Leverage Prompts
Use pre-defined prompts for common tasks - they're optimized for AI understanding.

### 6. Path Modes for Context
- Use `relative` paths when showing search results
- Use `full` paths when you need exact locations
- Use `off` (default) for general structure viewing

## Security

Smart Tree includes security features:
- Path restrictions (configurable)
- Default blocked paths (/etc, /sys, /proc)
- Respects .gitignore by default
- No symlink following by default

## Performance Tips

1. **Use appropriate max_depth** - Default is 10, but 3-5 is often enough
2. **Enable caching** - Results are cached for 5 minutes by default
3. **Use file_type filters** - Dramatically reduces search space
4. **Compress large outputs** - Reduces token usage by 70-90%
5. **Stream for large directories** - Get results immediately

## Troubleshooting

### Output is too large
- Enable compression: `compress: true`
- Reduce depth: `max_depth: 3`
- Use digest mode for overview

### Can't find files
- Check if using .gitignore: `no_ignore: true`
- Include hidden files: `show_hidden: true`
- Verify path exists and is accessible

### Streaming not working
- Only works with `ai` and `hex` modes
- Cannot be used with compression
- Check your AI client supports streaming

---

Built with ❤️ by the Smart Tree team. Making directory analysis AI-friendly! 