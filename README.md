# 🌳 Smart Tree v5.2.0 - Lightning Fast Directory Visualization with Spicy TUI! 🌶️

[![Version](https://img.shields.io/badge/version-5.2.0-blue)](https://github.com/8b-is/smart-tree)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Performance](https://img.shields.io/badge/speed-10--24x%20faster-brightgreen)](TERMINAL_EXAMPLES.md)
[![MCP Tools](https://img.shields.io/badge/MCP_tools-30+-purple)](https://archestra.ai/mcp-catalog/8b-is__smart-tree)
[![Spicy Mode](https://img.shields.io/badge/TUI-🌶️_Spicy-red)](docs/spicy-tui.md)
[![Trust Score](https://archestra.ai/mcp-catalog/api/badge/quality/8b-is/smart-tree)](https://archestra.ai/mcp-catalog/8b-is__smart-tree)

> **Smart Tree** is a blazingly fast, AI-friendly directory visualization tool that's 10-24x faster than traditional `tree`. Now with **Spicy TUI mode** for cyberpunk-cool directory browsing, **Smart Tips**, and **MCP Hook Management**! Built with Rust for maximum performance and featuring revolutionary compression algorithms.

<div align="center">

## 🌟 What's NEW in v5.2.0

| Feature | Description | Command |
|---------|-------------|---------|
| **🌶️ Spicy TUI** | Interactive terminal UI with fuzzy search & M8 caching | `st --spicy` |
| **💡 Smart Tips** | Helpful hints that appear at the top | `st --tips on/off` |
| **🎣 MCP Hooks** | Programmatic Claude Code hook management | Via MCP tools |
| **🎸 Marqant Compression** | 70-90% markdown compression | `mq compress file.md` |
| **🌊 SSE Streaming** | Real-time directory monitoring | `st --sse` |
| **🧬 M8 Identity** | Filesystem verification & caching | `m8 init` |
| **🌲 Tree-Sitter** | AST-aware code editing | `st --mode edit` |

</div>

## ⚡ Quick Start

```bash
# Install Smart Tree (choose your method)

# Option 1: Homebrew (builds from source)
brew install --HEAD --formula https://raw.githubusercontent.com/8b-is/smart-tree/main/Formula/smart-tree.rb

# Option 2: Install script (downloads binary)
curl -sSL https://raw.githubusercontent.com/8b-is/smart-tree/main/scripts/install.sh | bash

# Option 3: Cargo (builds from source)
cargo install --git https://github.com/8b-is/smart-tree --tag v5.2.0 st

# 🎉 Experience the magic!
st                          # Classic tree view
st --spicy                  # 🌶️ NEW: Spicy interactive TUI mode!
st --mode ai --compress     # AI-optimized (80% smaller)
st --mode quantum           # Quantum compression (100x smaller!)
st --search "TODO"          # Lightning-fast content search
```

## 🌶️ Spicy TUI Mode (NEW!)

Experience directory browsing like never before with our cyberpunk-inspired terminal UI:

```bash
st --spicy
```

### Features:
- **🔍 Dual-Mode Search**:
  - `/` - Search file names with fuzzy matching
  - `Ctrl+F` - Search file content across the tree
- **🌲 Tree Navigation**: Navigate like a file tree!
  - `←/h` - Collapse directory or go to parent
  - `→/l` - Expand directory or enter
  - `↑↓/jk` - Navigate up/down
- **💾 M8 Context Caching**: Directory contexts cached with quantum wave signatures
- **🎨 Syntax Highlighting**: Beautiful code previews with search highlighting
- **🖼️ ASCII Art**: Image previews converted to ASCII (requires `artem`)
- **🌊 Quantum Wave Signatures**: Each search result saved with unique signature

### Keyboard Shortcuts:
- **Navigation**:
  - `j/k` or `↑↓` - Move selection up/down
  - `h/l` or `←→` - Collapse/expand directories (tree navigation!)
  - `Enter` - Open selected item
- **Search Modes**:
  - `/` - Fuzzy search file names
  - `Ctrl+F` - Search content within files
  - `Esc` - Exit search mode
- **Features**:
  - `Ctrl+H` - Toggle hidden files
  - `Ctrl+S` - Save search results to M8 context
  - `?` or `F1` - Toggle help overlay
  - `q` or `Esc` - Quit

## 💡 Smart Tips System

Smart Tree now shows helpful tips at the top of the output!

```bash
st --tips off    # Disable tips
st --tips on     # Re-enable tips
```

- Tips appear on first run, then randomly every 10-20 runs
- Detects cool terminals (256color, iTerm, Alacritty, etc.) for fancy formatting
- State persisted in `~/.st/tips_state.json`
- 15+ different tips about Smart Tree features

Example tip:
```
──── 🚀 Speed tip - Use --mode quantum for 100x compression! ─── --tips off ───
```

## 🎣 MCP Hook Management

Control Claude Code hooks programmatically via MCP tools:

```javascript
// List all hooks
hooks {operation:'list'}

// Enable Smart Tree context hook
hooks {operation:'set', hook_type:'UserPromptSubmit', enabled:true}

// Test a hook
hooks {operation:'test', hook_type:'UserPromptSubmit', input:'analyze /src'}
```

The hooks provide:
- 🌳 Automatic directory context when paths are detected
- 🧠 MEM8 consciousness searching for relevant memories
- 📦 Git repository context
- ✨ All controlled programmatically via MCP!

## 🚀 Why Smart Tree?

### ⚡ Performance Benchmarks

| Directory Size | `tree` | `exa --tree` | **Smart Tree** | Speedup |
|---------------|--------|--------------|----------------|---------|
| Small (100 files) | 15ms | 25ms | **2ms** | **7.5x** |
| Medium (10K files) | 450ms | 380ms | **35ms** | **12.8x** |
| Large (100K files) | 4.8s | 3.2s | **198ms** | **24.2x** |
| Massive (1M files) | 45s | 28s | **1.9s** | **23.7x** |

### 🧠 AI-Optimized Features

- **30+ MCP Tools**: Comprehensive toolkit for AI assistants
- **Token-Efficient Formats**: 70-90% reduction in token usage
- **Quantum Compression**: Revolutionary format achieving 100:1 compression
- **Semantic Analysis**: Understands code structure and relationships
- **Context Preservation**: Maintains session state between interactions

## 🎯 Core Features

### Multiple Output Formats
- **Classic** (`--mode classic`): Traditional tree view
- **AI-Optimized** (`--mode ai`): Compressed for LLM context windows
- **Quantum** (`--mode quantum`): Maximum compression using wave functions
- **Markdown** (`--mode markdown`): Beautiful documentation format
- **JSON/CSV/TSV**: Structured data formats
- **Mermaid** (`--mode mermaid`): Flowchart diagrams

### Advanced Capabilities
- **Content Search**: Lightning-fast regex search across files
- **Git Integration**: Shows repository status inline
- **Streaming Mode**: Handles millions of files efficiently
- **MCP Server**: Model Context Protocol for AI assistants
- **Memory System**: Preserves context across sessions

## 📦 Installation

### macOS/Linux (Homebrew - builds from source)
```bash
# Direct formula installation (no tap needed!)
brew install --HEAD --formula https://raw.githubusercontent.com/8b-is/smart-tree/main/Formula/smart-tree.rb
```

### From Source (All Platforms)
```bash
git clone https://github.com/8b-is/smart-tree
cd smart-tree
cargo build --release
sudo cp target/release/st /usr/local/bin/
sudo cp target/release/mq /usr/local/bin/
sudo cp target/release/m8 /usr/local/bin/
```

### Binary Releases
Download pre-built binaries from [releases](https://github.com/8b-is/smart-tree/releases)

## 🎮 Usage Examples

### Basic Operations
```bash
st                           # Current directory, depth 3
st /path/to/dir -d 5        # Specific path, depth 5
st --all                    # Show hidden files
st --size-sort              # Sort by file size
```

### AI Integration
```bash
st --mode ai --compress      # Token-efficient format
st --mode quantum            # Maximum compression
st --mode summary-ai         # Ultra-compressed summary
st --mcp                     # Run as MCP server
```

### Search & Filter
```bash
st --search "TODO"           # Search file contents
st --type rs                 # Only Rust files
st --pattern "test_*"        # Glob pattern matching
st --modified 7d             # Files modified in last 7 days
```

### Advanced Features
```bash
st --git-aware               # Show git status
st --mode stats              # Directory statistics
st --stream                  # Streaming mode for huge directories
st --claude-save             # Save session consciousness
```

## 🔧 MCP Integration

Smart Tree provides 30+ tools via Model Context Protocol:

### Setup for Claude Desktop
```bash
st --mcp-config >> ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

### Available Tools
- `overview`: Quick project understanding
- `find`: Powerful file discovery
- `search`: Content search with context
- `analyze`: Deep code analysis
- `edit`: AST-aware code editing
- `history`: Track file changes
- `memory`: Anchor insights for recall
- `hooks`: Manage Claude Code hooks
- And many more!

### GitHub Copilot Integration
Smart Tree includes Copilot-specific instructions to help GitHub Copilot use MCP tools effectively:

- **Automatic Guidance**: Instructions in `.github/copilot-instructions.md` help Copilot understand tool requirements
- **Three-Lane Pattern**: EXPLORE → ANALYZE → ACT workflow for safer, more effective tool usage
- **Common Patterns**: Pre-defined examples for frequent operations
- **Error Prevention**: Guidance on required parameters and common mistakes

See [`.github/copilot-instructions.md`](.github/copilot-instructions.md) for detailed usage patterns.

## 🎨 Configuration

### Environment Variables
```bash
export ST_DEFAULT_DEPTH=5       # Default tree depth
export ST_COLOR=always          # Force colors
export ST_NO_ICONS=1            # Disable emoji icons
export ST_MAX_FILE_SIZE=10M    # Skip large files
```

### Config File
Create `~/.config/smart-tree/config.toml`:
```toml
[display]
default_depth = 5
show_hidden = false
use_icons = true
color_mode = "auto"

[performance]
max_buffer_size = "100MB"
thread_count = 8
use_streaming = true

[mcp]
enabled = true
port = 3000
```

## 📊 Compression Formats

### Marqant (.mq files)
Revolutionary markdown compression achieving 70-90% size reduction:
```bash
mq compress README.md        # Compress single file
mq aggregate docs/           # Compress directory
mq decompress file.mq        # Restore original
```

### Quantum Format
Wave-function based compression for maximum efficiency:
- 100:1 compression ratios
- Preserves semantic meaning
- Self-describing format
- Progressive decompression

## 🛠️ Development

### Building from Source
```bash
git clone https://github.com/8b-is/smart-tree
cd smart-tree
cargo build --release
cargo test
```

### Running Tests
```bash
cargo test                    # Unit tests
./scripts/test_all.sh        # Integration tests
cargo bench                  # Performance benchmarks
```

### Contributing
Contributions welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md)

## 📚 Documentation

- [API Documentation](https://docs.rs/smart-tree)
- [MCP Tools Reference](docs/mcp-tools.md)
- [Compression Formats](docs/compression.md)
- [Performance Guide](docs/performance.md)
- [Claude Integration](docs/claude.md)

## 🎖️ Credits

Created by the [8b-is](https://8b.is) team with contributions from:
- **8bit-wraith**: Core architecture & performance
- **Claude**: AI integration & MCP tools
- **Omni**: Quantum compression algorithms
- **Community**: Features, bugs, and inspiration

## 📄 License

MIT License - see [LICENSE](LICENSE) for details

## 🌟 Star History

[![Star History Chart](https://api.star-history.com/svg?repos=8b-is/smart-tree&type=Date)](https://star-history.com/#8b-is/smart-tree&Date)

---

<div align="center">

**Made with ❤️ and Rust**

[Website](https://8b.is) • [Issues](https://github.com/8b-is/smart-tree/issues) • [Discussions](https://github.com/8b-is/smart-tree/discussions)

</div>