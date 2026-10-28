# 🎸 Smart Tree Ultimate Cheet Sheet 🎸
*The Complete Rock Opera Guide to Smart Tree v4.0.0*

## 🚀 Quick Start (30 Seconds to Glory)

```bash
# Install (pick your poison)
cargo install smart-tree                   # From crates.io
brew install smart-tree                    # macOS
curl -L bit.ly/smart-tree | bash          # Universal installer

# The Greatest Hits
st                                         # Classic tree (with emojis!)
st --mode ai                               # AI-optimized (80% smaller)
st --mode quantum-semantic                 # Maximum compression (94%!)
st --mode function-markdown                # Extract all functions! NEW! 🔥

# Find stuff like a rockstar
st --search "TODO"                         # X-ray vision for code
st --find "*.rs"                           # Find all Rust files
st --type py --newer-than 2025-01-01       # Recent Python files
```

## 🎭 All 20+ Output Modes (Pick Your Instrument)

### 🎸 The Classics (Human-Friendly)
```bash
st -m classic          # 🌳 Beautiful tree with emojis (default)
st -m ls               # 📁 Unix ls -la format
st -m stats            # 📊 Just statistics
st -m waste            # 🗑️ Find duplicates & bloat (Marie Kondo mode)
st -m function-markdown # 📚 Living code documentation! NEW!
```

### 🎹 The Experimental (AI-Optimized)
```bash
st -m ai               # 🧠 AI-optimized format (80% compression)
st -m quantum          # 🧬 Binary quantum format (99% compression!)
st -m quantum-semantic # 🌊 Semantic compression (94% + meaning!)
st -m summary-ai       # 📝 Ultra-compressed summary (10x reduction)
st -m digest           # 💊 One-line summary
```

### 🥁 The Visualizers (Pretty Pictures)
```bash
st -m mermaid          # 🧜‍♀️ Flowchart diagrams
st -m mermaid --mermaid-style mindmap     # 🧠 Mind maps
st -m mermaid --mermaid-style treemap     # 📊 Size visualization
st -m markdown         # 📄 Full report with charts
st -m relations        # 🔗 Code relationships
st -m semantic         # 🌊 Group by meaning
```

### 🎺 The Data Formats (Machine Food)
```bash
st -m json             # 🔧 Standard JSON
st -m csv              # 📊 Comma-separated
st -m tsv              # 📊 Tab-separated
st -m hex              # 🔢 Hexadecimal fields
st -m marqant          # 📦 Quantum-compressed markdown
st -m sse              # 📡 Server-sent events streaming
```

## 🔍 Search & Filter (Detective Mode)

```bash
# Size matters
st --min-size 10M                  # 🐘 Find big files
st --max-size 1K                   # 🐜 Find tiny files
st --sort largest --top 10         # 🏆 Top 10 biggest

# Time travel
st --newer-than 2025-01-01         # 🆕 What's new?
st --older-than 2020-01-01         # 🏛️ Ancient artifacts

# Type filtering
st --type rs                       # 🦀 Rust files only
st --type "py,js,ts"              # 🐍📜 Multiple types
st --entry-type d                  # 📁 Directories only

# Pattern matching
st --find "test_.*\.rs"            # 🧪 Find test files
st --search "FIXME|TODO"           # 🔍 Search in files
```

## 💪 Power Features

### 🏠 Home Directory Safety (NEW!)
```bash
st ~                               # Won't crash on 2.4M files!
st --depth 3 ~                     # Limit depth for huge dirs
st --stream ~                      # Stream mode for massive dirs
```

### 📚 Function Documentation (NEW!)
```bash
st -m function-markdown src/       # Extract all functions
st -m function-markdown --show-private  # Include private functions
watch -n 5 'st -m function-markdown src/ > FUNCS.md'  # Live docs!
```

### 🤖 MCP Server Mode
```bash
st --mcp                           # Run as MCP server
st --mcp-tools                     # List 30+ AI tools
st --mcp-config                    # Show Claude Desktop config
```

### 🎬 Advanced Options
```bash
st --no-emoji                      # 😢 Disable fun
st --no-ignore                     # 🙈 See .gitignore'd files
st --compress                      # 🗜️ Compress output
st --stream                        # 🌊 Stream for huge dirs
st --path-mode full                # 📍 Show full paths
st --semantic                      # 🧠 Group by meaning
```

## 🎯 Real-World Rockstar Examples

```bash
# The "Where's My Code?" Solo
st -m ai --search "TODO" --type rs src/

# The "Documentation Hero" Riff
st -m function-markdown src/ > docs/API.md

# The "Performance Detective" Groove
st -m quantum-semantic --compress | base64 > snapshot.q

# The "Clean House" Ballad
st -m waste --min-size 10M | grep -E "(node_modules|target|build)"

# The "What Changed?" Bridge
st --newer-than 2025-01-01 --sort newest --top 20

# The "AI Context" Anthem
st -m summary-ai ~/projects/big-codebase > context.txt

# The "Live Monitor" Jam Session
st --sse-server --sse-port 28428 /path/to/watch
```

## 🔮 8-O Mode Preview (Coming Soon!)

```bash
# Attach to running process
st --mode 8-O --attach-pid 12345 --cast my-tv

# Profile with heat map
cargo run & st --mode 8-O --profile --heat-map

# Record performance session
st --mode 8-O --record perf.mp4 --duration 60s
```

**Visual Elements:**
- 🔥 Hot functions glow red
- 🧊 I/O waits freeze blue
- ⚡ Thread contention = lightning
- 💜 GC pressure = purple waves

## 📊 Compression Stats (The Numbers Don't Lie)

| Mode | Size | Reduction | Use Case |
|------|------|-----------|----------|
| Classic | 487MB | 0% | Humans |
| AI | 97MB | 80% | Claude/GPT |
| Quantum | 4.9MB | 99% | Storage |
| Quantum-Semantic | 29MB | 94% | Analysis |
| Summary-AI | 48MB | 90% | Overview |
| Digest | 73 bytes | 99.99% | Quick check |

## 🎪 Special Tricks

### The "Emotional Tree" (Coming Back Soon)
```bash
st -m emotional   # Tree gets bored of node_modules! 😴
```

### The "Security Vigilance" (In Development)
```bash
st --security-mode  # Watches for suspicious patterns 🔍
```

### The "Hot Tub Debug" (Easter Egg)
```bash
st --hot-tub  # Collaborative debugging with Omni! 🛁
```

## 🛠️ Installation Variations

```bash
# Cargo (Rust users)
cargo install smart-tree

# From source (hackers)
git clone https://github.com/8b-is/smart-tree.git
cd smart-tree
./scripts/manage.sh install

# Pre-built binaries
curl -L https://github.com/8b-is/smart-tree/releases/latest/download/st-$(uname -s)-$(uname -m) -o st
chmod +x st
sudo mv st /usr/local/bin/

# Claude Desktop
st --mcp-config >> ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

## 🎸 Pro Tips from The Cheet

1. **Speed Run**: `st -m digest` for instant directory fingerprint
2. **AI Budget**: `st -m summary-ai` saves 90% on tokens
3. **Live Docs**: `watch` + `function-markdown` = self-updating docs
4. **Big Dirs**: Always use `--stream` for dirs > 100K files
5. **Git Ignore**: Use `--no-ignore` to see what git hides
6. **Performance**: `st -m quantum` for network transfer (99% smaller!)

## 🎭 The Cast

- **The Cheet**: Your musical guide through the filesystem
- **Hue**: The human partner (that's you!)
- **Aye**: The AI assistant (that's me!)
- **Trisha**: From accounting, loves making things sparkle ✨
- **Omni**: Philosophical guide in the Hot Tub 🛁

## 🚒 Emergency Commands

```bash
# Home directory crash? Limit it!
st --depth 3 --stream ~

# Too much output? Compress it!
st -m quantum | gzip > tree.qz

# Need context for AI? Summarize!
st -m summary-ai > context.txt

# Lost in output? Digest it!
st -m digest
```

## 🌟 Why Smart Tree Rocks

- **10-24x faster** than traditional tree
- **99% compression** with quantum modes
- **25+ languages** for function extraction
- **30+ MCP tools** for AI integration
- **Crash-proof** on massive directories
- **Living docs** that update themselves
- **Emotional** trees that get bored
- **Performance** visualization (soon!)

---

*"Smart Tree: Where directories come alive and performance glows red hot!"* 🔥

**Version**: 4.0.0 | **Released**: August 8, 2025 | **Made with**: 🎸 & ❤️

*P.S. - For the full experience, read while listening to "Stairway to Heaven" 🎵*