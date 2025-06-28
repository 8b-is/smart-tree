# Smart Tree MCP Quick Reference

## 🚀 Essential Commands

```bash
st --mcp              # Run as MCP server
st --mcp-tools        # List available tools
st --mcp-config       # Show config snippet
```

## 🛠️ Available Tools

### analyze_directory
```json
{
  "name": "analyze_directory",
  "arguments": {
    "path": "/path/to/analyze",      // Required
    "mode": "ai",                    // ai|hex|json|stats|csv|tsv|digest
    "compress": true,                // Token-efficient output
    "stream": true,                  // Real-time results (ai/hex only)
    "max_depth": 5,                  // Traversal limit
    "search": "TODO",                // Search in files
    "find": "test.*\\.rs",           // Regex for names
    "file_type": "rs",               // Filter by extension
    "min_size": "1M",                // Size filters
    "newer_than": "2024-01-01"       // Date filters
  }
}
```

### find_files
```json
{
  "name": "find_files",
  "arguments": {
    "path": "/search/root",          // Required
    "pattern": ".*\\.test\\.js$",    // Regex pattern
    "min_size": "100K",              // Size threshold
    "newer_than": "2024-01-15"       // Date filter
  }
}
```

### get_statistics
```json
{
  "name": "get_statistics",
  "arguments": {
    "path": "/project",              // Required
    "show_hidden": true              // Include hidden files
  }
}
```

### get_digest
```json
{
  "name": "get_digest",
  "arguments": {
    "path": "/project"               // Required
  }
}
```

## 📋 Output Modes

| Mode | Description | Best For |
|------|-------------|----------|
| `ai` | LLM-optimized with emojis | AI analysis (default) |
| `digest` | SHA256 + minimal stats | Quick comparisons |
| `hex` | Fixed-width format | Parsing/streaming |
| `json` | Structured data | Programmatic use |
| `stats` | Statistics only | Directory overview |
| `classic` | Human-readable tree | Manual review |

## 🎯 Common Patterns

### Analyze Codebase with Compression
```json
{
  "name": "analyze_directory",
  "arguments": {
    "path": "/project",
    "mode": "ai",
    "compress": true,
    "show_ignored": true
  }
}
```

### Find Large Recent Files
```json
{
  "name": "find_files",
  "arguments": {
    "path": "/downloads",
    "min_size": "50M",
    "newer_than": "2024-01-01"
  }
}
```

### Search Code for TODOs
```json
{
  "name": "analyze_directory",
  "arguments": {
    "path": "/src",
    "search": "TODO|FIXME",
    "file_type": "py",
    "path_mode": "relative"
  }
}
```

### Stream Large Directory
```json
{
  "name": "analyze_directory",
  "arguments": {
    "path": "/huge/repo",
    "mode": "ai",
    "stream": true,
    "max_depth": 3
  }
}
```

## 🔧 Key Parameters

### Visibility Control
- `show_hidden`: Include .files
- `show_ignored`: Show [ignored] items
- `no_ignore`: Bypass .gitignore
- `no_default_ignore`: Disable built-in ignores

### Path Display
- `off`: Names only (default)
- `relative`: From scan root
- `full`: Absolute paths

### Performance
- `compress`: ~80% smaller output
- `stream`: Immediate results
- `max_depth`: Limit traversal
- `file_type`: Reduce search space

## 💡 Pro Tips

1. **Always compress** for large trees: `compress: true`
2. **Stream** for immediate feedback: `stream: true`
3. **Use digest** for change detection
4. **Combine filters** for precision
5. **Cache** lasts 5 minutes by default

## 🔐 Security

- Blocked: `/etc`, `/sys`, `/proc`
- No symlink following
- Configurable path restrictions
- Respects .gitignore by default

## 📦 Output Examples

### Compressed Output
```
COMPRESSED_V1:789c4d8fc10a...
```

### AI Mode
```
📁 project/ (15.2M)
├── 📄 README.md (2.3K)
└── 📁 src/ (8.1M)
    └── 📄 main.rs (28K) 🔍
```

### Digest Mode
```
SHA256:a3f5b2c1... Files:234 Dirs:45 Size:15.2M
```

---
**Smart Tree MCP** - Making directories AI-friendly! 🌳✨ 