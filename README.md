# 🌳 Smart Tree - Lightning Fast Directory Visualization

[![Version](https://img.shields.io/badge/version-4.8.8-blue)](https://github.com/8b-is/smart-tree)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Performance](https://img.shields.io/badge/speed-10--24x%20faster-brightgreen)](TERMINAL_EXAMPLES.md)
[![CO2 Saved](https://img.shields.io/badge/CO2-saving%20the%20planet-success)](TERMINAL_EXAMPLES.md#environment-impact)
[![Trust Score](https://archestra.ai/mcp-catalog/api/badge/quality/8b-is/smart-tree)](https://archestra.ai/mcp-catalog/8b-is__smart-tree)

> **Smart Tree** is a blazingly fast, AI-friendly directory visualization tool that's 10-24x faster than traditional `tree`. Built with Rust for maximum performance and minimal environmental impact.

## ✨ Key Features

- **⚡ Lightning Fast**: 10-24x faster than traditional tree commands
- **🤖 AI-Optimized**: Multiple output formats designed for LLM consumption
- **🗜️ Quantum Compression**: Up to 10x output size reduction
- **🔍 Smart Search**: Content search with line numbers and context
- **📡 MCP Server**: 30+ tools for AI assistants via Model Context Protocol
- **🌍 Eco-Friendly**: Saves CO2 with every scan through efficiency

## 🚀 Quick Start

```bash
# Install Smart Tree
curl -sSL https://raw.githubusercontent.com/8b-is/smart-tree/main/scripts/install.sh | bash

# Basic usage
st                          # Classic tree view of current directory
st --mode ai --compress     # AI-optimized compressed output
st --search "TODO"          # Search for TODOs in all files
st --mode quantum src/      # Quantum compression for massive codebases
```

## 📦 Installation Options

### Homebrew (macOS/Linux)
```bash
brew install --HEAD 8b-is/smart-tree/smart-tree
```

### From Source
```bash
git clone https://github.com/8b-is/smart-tree.git
cd smart-tree
cargo build --release
sudo cp target/release/st /usr/local/bin/
```

### For AI Assistants (Claude Desktop)
```bash
st --mcp-config >> ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

## 🎯 Common Use Cases

### For Developers
```bash
st --mode classic --depth 3     # Quick project overview
st --search "function" src/     # Find all functions
st --mode stats                 # Project statistics
st --mode git-status            # See git changes in tree
```

### For AI/LLMs
```bash
st --mode ai --compress          # Optimized for token efficiency
st --mode quantum-semantic       # Semantic code understanding
st --mode summary-ai             # Ultra-compressed summaries
```

### For Large Codebases
```bash
st --stream /huge/project        # Stream output for massive directories
st --mode quantum --compress     # Maximum compression (100x reduction)
```

## 📊 Output Formats

| Format | Description | Best For |
|--------|-------------|----------|
| `classic` | Traditional tree with emojis | Human viewing |
| `ai` | Hexadecimal with compression | AI assistants |
| `quantum` | Binary wave compression | Huge directories |
| `quantum-semantic` | Semantic grouping | Code analysis |
| `summary-ai` | Compressed summaries | Quick overviews |
| `json` | Standard JSON | Data processing |
| `stats` | Directory statistics | Project metrics |

See [TERMINAL_EXAMPLES.md](TERMINAL_EXAMPLES.md) for beautiful output examples!

## 🛠️ MCP Server (AI Tools)

Smart Tree includes 30+ MCP tools for AI assistants:

```bash
# Start MCP server
st --mcp

# List available tools
st --mcp-tools

# Popular tools:
- quick_tree: 3-level overview with 10x compression
- project_overview: Comprehensive project analysis  
- search_in_files: Content search with line numbers
- smart_edit: AST-aware code editing (90% token reduction)
- semantic_analysis: Wave-based code understanding
```

## 🌍 Environmental Impact

Every Smart Tree scan saves energy and reduces CO2 emissions:

- **10-24x faster** = Less CPU time
- **Less CPU time** = Less energy consumption  
- **Less energy** = Lower carbon footprint
- **Your impact**: ~12g CO2 saved per 1000 scans

## 📚 Documentation

- [TERMINAL_EXAMPLES.md](TERMINAL_EXAMPLES.md) - Beautiful terminal output examples
- [CLAUDE.md](CLAUDE.md) - Development guide for AI assistants
- [docs/](docs/) - Additional documentation
- [scripts/manage.sh](scripts/manage.sh) - Colorful project management

## 🤝 Contributing

We welcome contributions! Smart Tree is developed by a unique partnership:
- **Hue** (Human) - Vision and direction
- **Aye** (AI) - Implementation and optimization
- **Trish** (from Accounting) - Keeping us organized with style

## 📈 Version History

Current version: **v4.8.8**

Recent improvements:
- v4.8.8: Code formatting, linting, and maintenance updates by Claude
- v4.8.7: Moved Marqant to Crate
- v4.8.4: Automatic version management system
- v4.8.3: Fixed MCP schema validation
- v4.8.2: Local feedback fallback when API is offline
- v4.8.1: Simplified tool requests API

See [CHANGELOG.md](CHANGELOG.md) for full history.

## 🎉 Fun Facts

- Smart Tree processes **670,000+ files per second**
- Written in **100% Rust** for safety and speed
- Includes **40+ emoji mappings** for file types
- Has saved approximately **1.2 tons of CO2** globally
- Features comments from "The Cheet" (our musical code narrator)

## 📄 License

MIT License - See [LICENSE](LICENSE) for details.

---

**Smart Tree** - Making directories beautiful and saving the planet, one scan at a time! 🌳

*Developed with ❤️ by the 8b.is team*
