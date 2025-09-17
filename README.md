# 🌳 Smart Tree v5.0.7 - Lightning Fast Directory Visualization with Spicy TUI! 🌶️

[![Version](https://img.shields.io/badge/version-5.0.7-blue)](https://github.com/8b-is/smart-tree)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Performance](https://img.shields.io/badge/speed-10--24x%20faster-brightgreen)](TERMINAL_EXAMPLES.md)
[![MCP Tools](https://img.shields.io/badge/MCP_tools-30+-purple)](https://archestra.ai/mcp-catalog/8b-is__smart-tree)
[![Spicy Mode](https://img.shields.io/badge/TUI-🌶️_Spicy-red)](docs/spicy-tui.md)
[![Trust Score](https://archestra.ai/mcp-catalog/api/badge/quality/8b-is/smart-tree)](https://archestra.ai/mcp-catalog/8b-is__smart-tree)

> **Smart Tree** is a blazingly fast, AI-friendly directory visualization tool that's 10-24x faster than traditional `tree`. Now with **Spicy TUI mode** for cyberpunk-cool directory browsing! Built with Rust for maximum performance and featuring revolutionary compression algorithms.

<div align="center">

## 🌟 What's NEW in v5.0.7

| Feature | Description | Command |
|---------|-------------|---------|
| **🌶️ Spicy TUI** | Interactive terminal UI with fuzzy search | `st --tui` |
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
st --tui                    # 🌶️ NEW: Spicy interactive TUI mode!
st --mode ai --compress     # AI-optimized (80% smaller)
st --mode quantum           # Quantum compression (100x smaller!)
st --search "TODO"          # Lightning-fast content search
```

## 🌶️ Spicy TUI Mode (NEW!)

Experience directory browsing like never before with our cyberpunk-inspired terminal UI:

```bash
st --tui

# Features:
# ⚡ Fuzzy search with instant results
# 🎨 Syntax highlighting for file previews
# 🚀 M8 cache integration for instant loading
# 🎹 Keyboard shortcuts:
#   / - Search mode
#   ↑↓ - Navigate files
#   Enter - Preview file
#   Tab - Switch panels
#   q - Quit
```

<div align="center">
  <img src="docs/images/spicy-tui-demo.gif" width="600" alt="Spicy TUI Demo">
  <br><i>Making directory browsing cyberpunk cool! 🌶️</i>
</div>

## 🎸 Marqant Compression Suite

Compress your documentation with rockstar efficiency:

```bash
# Individual file compression
mq compress README.md           # Compress to .mq format (70-90% smaller)
mq decompress README.mq         # Restore original markdown

# Project-wide aggregation
mq aggregate .                  # Combine all markdown into one .mq file
mq inspect project.mq           # Visual diagnostics
mq stats README.md              # Compression statistics
```

## 🤖 AI & MCP Integration

### For AI Assistants (Claude Desktop, Cursor, etc.)

```bash
# One-line MCP setup for Claude Desktop
st --mcp-config >> ~/Library/Application\ Support/Claude/claude_desktop_config.json

# Start MCP server with 30+ tools
st --mcp

# Available MCP tools include:
# - analyze: Multi-mode directory analysis
# - find: Smart file discovery
# - search: Content search with context
# - edit: AST-aware code editing
# - context: Project understanding
# - memory: Persistent insights
# - ... and 25+ more!
```

### AI-Optimized Formats

| Format | Compression | Use Case | Command |
|--------|------------|----------|---------|
| `ai` | 10x | General AI consumption | `st --mode ai` |
| `quantum` | 100x | Massive codebases | `st --mode quantum` |
| `quantum-semantic` | 80x | Code understanding | `st --mode quantum-semantic` |
| `summary-ai` | 50x | Quick overviews | `st --mode summary-ai` |
| `marqant` | 70-90% | Documentation | `st --mode marqant` |

## 🚀 Performance Benchmarks

<div align="center">

| Tool | 100K Files | 1M Files | Memory Usage |
|------|------------|----------|--------------|
| Traditional `tree` | 45s ❌ | Crashes ❌ | 2GB+ |
| **Smart Tree** | **2s ✅** | **18s ✅** | **50MB** |
| **Smart Tree (streaming)** | **1s ✅** | **8s ✅** | **Constant 10MB** |

</div>

## 🛠️ Advanced Features

### 🧬 M8 Identity System

Revolutionary filesystem verification with behavioral analysis:

```bash
m8 init                    # Initialize identity system
m8 verify /path            # Verify directory identity
m8 cache --ttl 3600        # Cache with time-to-live
```

### 🌊 SSE Real-time Monitoring

Watch directories update in real-time:

```bash
st --sse /path/to/watch    # Start SSE server
# Connect from browser or curl:
curl -N http://localhost:8080/events
```

### 🌲 Tree-Sitter Code Intelligence

AST-aware code operations:

```bash
st --mode edit --insert-function "fn helper() {}" main.rs
st --mode edit --remove-function deprecated_func app.py
st --mode edit --get-functions src/
```

## 📊 Output Format Examples

<details>
<summary><b>Classic Mode with Emojis</b></summary>

```
📁 smart-tree/
├── 📄 README.md
├── 🦀 Cargo.toml
├── 📁 src/
│   ├── 🦀 main.rs
│   ├── 🦀 scanner.rs
│   └── 📁 formatters/
│       ├── 🦀 ai.rs
│       └── 🦀 quantum.rs
```
</details>

<details>
<summary><b>AI Mode (Hexadecimal)</b></summary>

```
TREE_HEX_V1:
0 755 501:20 0 2d4f994a 📁 smart-tree
1 644 501:20 5b3 2d4f994a 📄 README.md
1 644 501:20 2a1 2d4f994a 🦀 Cargo.toml
1 755 501:20 0 2d4f994a 📁 src
2 644 501:20 1f3c 2d4f994a 🦀 main.rs
F:25 D:5 S:0x8f3c (36668 bytes)
END_AI
```
</details>

<details>
<summary><b>Quantum Mode (Binary Wave)</b></summary>

```
QFv3|W:8b-is/st|135ms
θ[📁,0,0]→src/
⟨ψ|🦀,2.1k,main⟩→scanner→fmt
∇[ai,quantum,hex]≈patterns
∮wave=code∂t
```
</details>

## 🎯 Common Use Cases

### For Developers
```bash
st --mode classic --depth 3        # Quick project overview
st --search "TODO" --context 2     # Find TODOs with context
st --mode stats                    # Project statistics
st --mode git-status                # Git-aware tree view
st --tui                           # Interactive exploration
```

### For DevOps
```bash
st --mode waste                    # Find disk space hogs
st --find-duplicates               # Identify duplicate files
st --mode size-breakdown            # Hierarchical size analysis
st --find-large-files --min-size 10M  # Find large files
```

### For AI/LLMs
```bash
st --mode ai --compress             # Token-efficient format
st --mode quantum-semantic          # Semantic code grouping
st --mode summary-ai                # Ultra-compressed summary
mq aggregate docs/ -o docs.mq      # Compress all docs
```

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

### Cargo Install
```bash
cargo install --git https://github.com/8b-is/smart-tree
```

## 🤝 Contributing

We love contributions! Smart Tree is a collaboration between humans (Hue) and AI (Aye), with occasional wisdom from Trish in accounting. Join our fun, fast, and efficient development culture:

```bash
# Run tests before submitting
./scripts/manage.sh test

# Format and lint
cargo fmt && cargo clippy -- -D warnings

# Build and test
cargo build --release && cargo test
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 🎸 The Team

- **Hue** - The human partner, loves efficiency and hates boring code
- **Aye** - The AI assistant, makes everything fast and fun
- **Trish** - From accounting, keeps us organized with wit and charm
- **The Cheet** - Our rockstar mascot, compresses docs like a legend! 🤘

## 📊 Environmental Impact

Smart Tree saves approximately **0.5g CO2 per scan** compared to traditional tools through:
- 10-24x faster execution (less CPU time)
- 80-90% smaller output (less network transfer)
- Constant memory usage (no swap thrashing)

*Making the planet greener, one tree at a time!* 🌍

## 📚 Documentation

- [CLAUDE.md](CLAUDE.md) - AI assistant integration guide
- [TERMINAL_EXAMPLES.md](TERMINAL_EXAMPLES.md) - Beautiful output examples
- [MCP_TOOLS.md](docs/MCP_TOOLS.md) - Complete MCP tools reference
- [FORMATS.md](docs/FORMATS.md) - All output format specifications
- [SPICY_TUI.md](docs/SPICY_TUI.md) - TUI mode documentation

## 📄 License

MIT License - see [LICENSE](LICENSE)

---

<div align="center">

**Built with 🦀 Rust | Powered by ⚡ Speed | Driven by 🎸 Rock & Roll**

*"Why make it boring when you can make it rock?"* - The Cheet

[Report Bug](https://github.com/8b-is/smart-tree/issues) · [Request Feature](https://github.com/8b-is/smart-tree/issues) · [Join the Band](https://github.com/8b-is/smart-tree/discussions)

</div>