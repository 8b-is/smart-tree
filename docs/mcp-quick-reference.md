# Smart Tree MCP Quick Reference - Your Toolbox Guide! 🧰🏗️

> **📢 Important Update (v4.0.0+)**: All output is now decompressed by default for better AI compatibility! Your helper delivers tools ready-to-use, no assembly required!

## 🚀 Essential Commands - Calling Your Helper

```bash
st --mcp              # Activate your AI helper in API mode
st --mcp-tools        # Show all tools in the toolbox
st --mcp-config       # Get setup instructions for your helper
```

## 🛠️ Available Tools - What's in Your Helper's Toolbox

### analyze_directory - Your Helper's Site Survey Tool
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

### find_files - Your Helper's Material Locator
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

### get_statistics - Your Helper's Measuring Tape
```json
{
  "name": "get_statistics",
  "arguments": {
    "path": "/project",              // Required
    "show_hidden": true              // Include hidden files
  }
}
```

### get_digest - Your Helper's Project Fingerprint Tool
```json
{
  "name": "get_digest",
  "arguments": {
    "path": "/project"               // Required
  }
}
```

## 📋 Output Modes - How Your Helper Reports Back

| Mode | Description | When Your Helper Uses It |
|------|-------------|----------|
| `ai` | LLM-optimized with emojis | Reporting to AI (default) |
| `digest` | SHA256 + minimal stats | Quick site fingerprinting |
| `hex` | Fixed-width format | Precise measurements |
| `json` | Structured data | Machine-readable reports |
| `stats` | Statistics only | Progress reports |
| `classic` | Human-readable tree | Showing humans the layout |

## 🎯 Common Construction Tasks - Your Helper in Action 🏗️

### Survey the Entire Construction Site
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

### Locate Heavy Materials (Large Files)
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

### Find Work Orders (TODOs)
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

### Monitor Large Construction Site in Real-time
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

## 🔧 Tool Settings - Configuring Your Helper

### Visibility Control - What Your Helper Shows You
- `show_hidden`: Include .files
- `show_ignored`: Show [ignored] items
- `no_ignore`: Bypass .gitignore
- `no_default_ignore`: Disable built-in ignores

### Path Display - How Your Helper Labels Things
- `off`: Names only (default)
- `relative`: From scan root
- `full`: Absolute paths

### Performance - Making Your Helper Work Efficiently
- `compress`: ~80% smaller output
- `stream`: Immediate results
- `max_depth`: Limit traversal
- `file_type`: Reduce search space

## 💡 Pro Tips from Experienced Construction Helpers 🏗️

1. **Pack efficiently** for large jobs: `compress: true` - like organizing tools in a compact box
2. **Get updates as you work**: `stream: true` - your helper reports progress in real-time
3. **Fingerprint the site**: Use `digest` to detect changes quickly
4. **Be specific** with your requests - combine filters for precise results
5. **Your helper remembers** - cache lasts 5 minutes (no need to re-survey)

## 🔐 Safety Rules - Your Helper's Boundaries

- **Off-limits areas**: `/etc`, `/sys`, `/proc` - your helper won't enter restricted zones
- **No shortcuts**: Symlinks are not followed for safety
- **Customizable boundaries**: Configure where your helper can work
- **Respects project rules**: Follows .gitignore by default

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
**Smart Tree MCP v4.0.0** - Your AI Construction Helper! Ready with the right tool at the right time! 🏗️🌳✨ 