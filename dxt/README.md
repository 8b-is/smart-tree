# Smart Tree DXT Package 🌳

Official DXT (Desktop eXtension Tool) package for Smart Tree, providing seamless integration with Claude Desktop and AI assistants through the Model Context Protocol.

## Overview

Smart Tree (`st`) is a blazingly fast directory visualization tool that goes beyond traditional tree commands. This DXT package wraps the Rust binary with an intelligent Node.js layer that provides:

- **Automatic Binary Management**: Downloads the correct binary for your platform
- **Auto-Update System**: Keeps your Smart Tree installation current
- **20+ Specialized Tools**: Comprehensive file system analysis
- **Cross-Platform Support**: Works on macOS, Linux, and Windows

📸 **[View Screenshots & Examples](SCREENSHOTS.md)** - See Smart Tree in action!

## Features ✨

### Core Capabilities
- Multiple output formats (hex, JSON, AI-optimized, digest)
- Smart filtering by file type, size, date, and content
- Built-in compression for large outputs
- Respects `.gitignore` patterns
- Project context detection

### Auto-Update System 🔄
- Non-blocking update checks on startup
- Automatic installation of updates on restart
- Version tracking and comparison
- Graceful fallback on network issues

### Security 🔐
- Configurable allowed/blocked paths
- Read-only operations
- No access to sensitive system files by default

## Installation 📦

### Quick Install (Claude Desktop)

1. Download `smart-tree.dxt` from the [latest release](https://github.com/8b-is/smart-tree/releases)
2. Open Claude Desktop
3. Go to Settings → Developer
4. Click "Install from file"
5. Select `smart-tree.dxt`
6. Grant access to directories you want to analyze

### Building from Source

```bash
# Clone the repository
git clone https://github.com/8b-is/smart-tree
cd smart-tree/dxt

# Build the DXT package
./build-dxt.sh

# The package is now at smart-tree.dxt
```

## Available Tools 🛠️

The DXT provides 20+ specialized tools:

- **`analyze_directory`** - Comprehensive directory analysis with multiple formats
- **`find_files`** - Find files by name, type, size, or date
- **`search_in_files`** - Search content within files (AI-friendly output)
- **`project_overview`** - Get project context and structure
- **`find_large_files`** - Identify space consumers
- **`compare_directories`** - Compare two directory structures
- **`get_git_status`** - Analyze git repository status
- **`find_duplicates`** - Find duplicate files
- **`analyze_workspace`** - Identify project type and dependencies
- And many more!

## Prompts 💬

The package includes 8 intelligent prompts for common tasks:
- Finding recently modified documents
- Analyzing disk usage
- Comparing duplicate folders
- Organizing project structure
- And more!

## Configuration ⚙️

### Environment Variables
```bash
# Allowed paths (set by Claude Desktop)
ST_MCP_ALLOWED_PATHS=/home/user/projects,/home/user/documents

# Blocked paths (optional)
ST_MCP_BLOCKED_PATHS=/etc,/sys

# Enable debug logging
DEBUG=1
```

### MCP Configuration
Create `~/.st/mcp-config.toml` for advanced settings:
```toml
cache_enabled = true
cache_ttl = 300
max_cache_size = 104857600
```

## How It Works 🔧

1. **Initial Launch**: The Node.js wrapper checks for the Smart Tree binary
2. **Binary Download**: If missing, downloads the appropriate binary from GitHub releases
3. **Update Check**: Checks for newer versions in the background
4. **MCP Server**: Launches the Rust binary in MCP server mode
5. **Tool Routing**: Routes tool calls from Claude to the binary

## Contributing 🤝

This DXT package demonstrates several best practices:
- Auto-update mechanism for keeping tools current
- Cross-platform binary distribution
- Graceful error handling
- Non-blocking operations

Feel free to use this as a template for your own DXT packages!

## License 📄

MIT License - See [LICENSE](../LICENSE) for details

---

Built with 💖 by Aye, Hue, and Trisha from Accounting (who insisted on the sparkles ✨)