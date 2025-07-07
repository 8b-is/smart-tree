# 🌳 Smart Tree (st) v3.1.1 - The Directory Visualizer That Rocks! 🎸

![Hue's Terminal](dxt/ST-HUE.png)

[![Discord](https://img.shields.io/discord/1330349762673487895?color=7289da&label=Join%20the%20Party&logo=discord&logoColor=white)](https://discord.gg/uayQFhWC) [![GitHub release](https://img.shields.io/github/v/release/8b-is/smart-tree?include_prereleases&label=Latest%20Jam)](https://github.com/8b-is/smart-tree/releases) [![Downloads](https://img.shields.io/github/downloads/8b-is/smart-tree/total?label=Happy%20Users)](https://github.com/8b-is/smart-tree/releases) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)[![Rust](https://img.shields.io/badge/Built%20with-Rust%20🦀-orange?logo=rust)](https://www.rust-lang.org/)[![Claude Desktop](https://img.shields.io/badge/Claude%20Desktop-BFF%20Status-blueviolet)](https://claude.ai/download)[![MCP Compatible](https://img.shields.io/badge/MCP-Hell%20Yeah!-green)](https://modelcontextprotocol.io/)[![Platform](https://img.shields.io/badge/Runs%20on-Everything%20🚀-blue)](https://github.com/8b-is/smart-tree/releases)

**"Making directories beautiful, one tree at a time!"** - *Trish from Accounting (our #1 fan!)*

---

## 🎉 What the Heck is Smart Tree?

Remember the old `tree` command? Well, we gave it a PhD, taught it to dance, and introduced it to AI! Smart Tree is the **world's first AI-native directory visualizer** that actually understands what modern developers (and their AI assistants) need.

### 🏆 The *"Holy Smokes!"* Numbers 🏆

| Metric | Old Way | Smart Tree Way | Your Reaction |
|:------:|:-------:|:--------------:|:-------------:|
| **Directory Size** | 487 MB | 4.1 MB | 😱 "Wait, what?!" |
| **AI Token Cost** | $1,270 | ~$10 | 💰 "I'm rich!" |
| **Processing Speed** | 🐌 Slow | ⚡ 10-24x faster | 🚀 "Wheee!" |
| **Compression** | None | 99% | 🤯 "How?!" |
| **Fun Factor** | 0% | 100% | 🎉 "Finally!" |

## 🌟 Version 3.1.1: "Less is More" Edition! 

### 🎸 What's NEW (or should we say... what's GONE?)

**BREAKING NEWS**: Elvis has left the building! 🚪

- **❌ Removed Interactive Mode** - Because sometimes, simplicity rocks harder than complexity!
- **✅ Classic Mode is DEFAULT** - Just run `st` and boom! Beautiful trees! 
- **🚀 Smaller, Faster, Better** - Like a sports car that lost weight and gained speed!
- **💾 One Less Dependency** - `inquire` said goodbye, and we're not crying!

```bash
# Before v3.1.1:
st --interactive  # 😴 Too many steps!

# After v3.1.1:
st  # 🎉 BAM! Instant classic tree goodness!
```

## 🚀 Quick Start (Faster than Making Coffee ☕)

### 🐧 Linux/Mac/WSL - The One-Liner Wonder!

```bash
# This magical incantation will change your life:
curl -sSL https://raw.githubusercontent.com/8b-is/smart-tree/main/scripts/install.sh | bash

# That's it. You're done. Go visualize some directories! 🎊
```

### 🪟 Windows - The Slightly Longer Dance

<details>
<summary>Click here for Windows installation (still pretty easy!)</summary>

> **Pro tip**: You might need [Microsoft Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) for Rust
> - Don't forget ARM compiler if you're fancy with Parallels!
> - Get [Rust](https://www.rust-lang.org/tools/install) (it's awesome!)
> - Grab [Git](https://git-scm.com/): `winget install git`

```powershell
# Clone the awesomeness
git clone https://github.com/8b-is/smart-tree
cd smart-tree

# Build the magic
cargo build --release

# Install it (pick your favorite spot)
copy target\release\st.exe C:\your\favorite\path\

# Add to PATH and rock on! 🎸
```

</details>

### Claude Code Integration ###

```
# Do the quick install above.   This is the example for Linux, Mac, WSL, and who knows? 

claude mcp add st /usr/local/bin/st -- --mcp

```

### 🤖 Claude Desktop Integration - The Future is Now!

**Make Claude your directory-reading bestie in 3 clicks!**

1. 📥 Download `smart-tree.dxt` from [latest release](https://github.com/8b-is/smart-tree/releases/latest)
2. ⚙️ Claude Desktop → Settings → Developer → Install from file
3. 🎉 Select the file and grant access - DONE!

*Claude can now see your directories better than you can!* 🔮

## 🎯 Usage Examples (The Fun Part!)

### 🌈 The Basics - Simple Yet Powerful

```bash
# The classics never die! (Now the default in v3.1.1!)
st                          # Current directory with beautiful trees 🌳
st /path/to/directory       # Specific directory - point and shoot! 🎯

# Quick exploration
st --depth 2                # Shallow dive - just the tip of the iceberg 🧊
st --everything             # SHOW ME EVERYTHING! (Even the scary parts) 👀
st -a                       # Include hidden files (they're shy) 🙈
```

### 🎨 Output Modes - Pick Your Flavor!

<details>
<summary>🎭 All 15+ Output Modes Explained!</summary>

```bash
# For Humans (That's You!) 👤
st -m classic               # 🌳 The beautiful default (with emojis!)
st -m stats                 # 📊 Just the facts, ma'am
st -m markdown              # 📝 Perfect documentation in seconds!
st -m mermaid               # 🧜‍♀️ Diagrams that make you look smart

# For Robots (Your AI Friends) 🤖
st -m ai                    # 🧠 AI-optimized (80% smaller!)
st -m quantum-semantic      # 🌊 Maximum compression with meaning!
st -m digest                # 💊 One-line summary for quick checks
st -m json                  # 🔧 When machines talk to machines

# For Data Nerds 🤓
st -m hex                   # 🔢 Hexadecimal beauty
st -m csv                   # 📊 Spreadsheet-ready
st -m tsv                   # 📊 Tab-separated for the tab lovers
st -m semantic              # 🌊 Group by meaning (Omni's favorite!)

# The Secret Weapons 🥷
st -m quantum               # 🧬 Native quantum format (99% compression!)
st -m relations             # 🔗 Code relationship analysis
```

</details>

### 🔍 Finding Stuff - Like a Detective!

```bash
# Find files like a boss
st --find "*.rs"            # 🦀 Rust files, assemble!
st --find "TODO"            # 📝 Find all your broken promises
st --type py                # 🐍 Python files only
st --search "FIXME"         # 🔍 Search inside files (X-ray vision!)

# Size matters
st --min-size 10M           # 🐘 Find the chonky files
st --max-size 1K            # 🐜 Find the tiny ones

# Time travel
st --newer-than 2024-01-01  # 🕐 What's new this year?
st --older-than 2020-01-01  # 🕰️ Find the ancient artifacts
```

### 🚀 Performance Mode - For Speed Demons

```bash
# Stream mode - watch it flow!
st --stream                 # 🌊 Real-time output for huge directories
st --stream -m hex          # 🏃‍♂️ Hex mode at the speed of light

# Compression - because size matters
st -z                       # 🗜️ Compress output (even smaller!)
st -m ai -z                 # 🤖 AI mode + compression = 💰 saved

# The "I need it yesterday" combo
AI_TOOLS=1 st              # 🚄 Auto-detects AI caller, optimizes everything!
```

### 🎪 The Magic Tricks

```bash
# Semantic grouping - files that vibe together!
st --semantic               # 🌊 Groups: tests, docs, config, source
                           # Wave signatures included! (Ask Omni about this)

# Mermaid diagrams - instant documentation!
st -m mermaid > docs/arch.md        # 📊 Flowchart magic
st -m mermaid --mermaid-style mindmap  # 🧠 Mind map mode
st -m mermaid --mermaid-style treemap # 🗺️ Treemap visualization (shows file sizes!)
st -m markdown > README_PROJECT.md   # 📚 Full project report!

# Pro tip: If mermaid has issues with emojis, use --no-emoji
st -m mermaid --no-emoji            # Clean diagrams without emojis

# The "impress your boss" commands
st -m digest /huge/project  # Returns in 0.1 seconds: "HASH: abc123 F:10000 D:500..."
st --no-emoji --no-color    # 😢 Boring mode (but why would you?)
```

## 🏗️ Architecture (For the Curious Minds)

<details>
<summary>🔧 How the Magic Happens</summary>

```
src/
├── main.rs           # 🎭 The ringmaster
├── scanner.rs        # 🔍 Directory detective
├── formatters/       # 🎨 The art department
│   ├── classic.rs    # 🌳 Beautiful trees
│   ├── quantum.rs    # 🧬 Compression wizard
│   ├── ai.rs         # 🤖 AI whisperer
│   └── mermaid.rs    # 🧜‍♀️ Diagram artist
├── semantic.rs       # 🌊 Wave philosopher
└── mcp/              # 🔌 AI integration HQ
    └── tools.rs      # 🛠️ Swiss army knife
```

</details>

## 🌟 Real-World Magic

### 💰 The Money Shot - Compression Comparison

#### 📈 Benchmarks

| Format | Size | Tokens | Relative Cost |
|--------|------|--------|---------------|
| Classic Tree | 1.2MB | 300K | 100% |
| JSON | 2.1MB | 525K | 175% |
| **Hex Mode** | 800KB | 200K | 67% |
| **AI Mode + Compression** | 120KB | 30K | 10% |
| **Digest** | 128B | 32 | 0.01% |

**That's a 99.2% reduction! Your wallet just did a happy dance! 💃**

### 🎯 Format Quick Reference


| Use Case | Best Format | Why? |
|:---------|:------------|:-----|
| 👀 **Quick Look** | `classic` (default!) | Beautiful & intuitive |
| 🤖 **AI Analysis** | `quantum-semantic` | 10x compression! |
| 📊 **Reports** | `markdown` | Instant documentation |
| 🔍 **Debugging** | `hex` | All the details |
| 💾 **Archival** | `json` | Future-proof |
| 🏃 **Quick Check** | `digest` | One-line summary |

### 🧙 MCP (Model Context Protocol) Server

Smart Tree now includes a **built-in MCP server** that provides intelligent project analysis directly to AI assistants!

```bash
# Run as MCP server (for Claude Desktop, etc.)
st --mcp

# Show MCP configuration
st --mcp-config
```

#### 🤖 AI Best Practices

Check out our **[AI Best Practices Guide](docs/MCP_AI_BEST_PRACTICES.md)** to learn:
- Optimal workflow for using Smart Tree tools
- Which tools to use for different tasks
- How to maximize token efficiency
- Common patterns for code analysis

**Golden Rule**: Always start with `quick_tree` for any new directory! 🌟

#### Features:
- **20+ specialized tools** for directory analysis
- **Automatic compression** for efficient token usage
- **Semantic analysis** for understanding code structure
- **Built-in caching** for instant repeated queries
- **Security controls** for safe file system access

See [MCP Integration Guide](docs/mcp-integration.md) for setup instructions.

---

## 🤝 Join the Smart Tree Family!

### 💬 Discord Community - Where the Cool Kids Hang Out

[![Discord Banner](https://img.shields.io/discord/1352603992504401961?color=7289da&label=Join%20the%20Tree%20House&logo=discord&logoColor=lightgreena&style=for-the-badge)](https://discord.gg/uayQFhWC)

**Come for the trees, stay for the memes!** 🌳😂

- 🆘 **Get Help** - We actually answer!
- 🎉 **Share Wins** - Show off your directory art!
- 🐛 **Report Bugs** - We'll squash 'em!
- 🌊 **Philosophy Hour** - Discuss waves with Omni in the hot tub!
- 🍕 **Pizza Fridays** - Virtual, but the fun is real!

### 🌟 Contributors Hall of Fame

Special shoutouts to:
- **Hue** - The visionary who started it all! 🎨
- **Aye** - The AI that rocks! 🤖
- **Trish from Accounting** - Our #1 fan and humor consultant! 💖
- **Omni** - The philosopher in the hot tub! 🛁
- **You** - Yes, YOU could be next! 🌟

## 📜 The Sacred Scrolls (Documentation)

- 📚 **[Complete Guide](docs/MODE_SELECTION_GUIDE.md)** - Everything you need!
- 🚀 **[MCP Integration](docs/mcp-guide.md)** - Make AI your friend!
- 🎯 **[Quick Reference](docs/mcp-quick-reference.md)** - Cheat sheet!
- 🤔 **[Philosophy](docs/OMNI_WISDOM.md)** - Deep thoughts from the hot tub!

## 🎬 The Grand Finale

### Why Smart Tree? Because...

**🌳 Life's too short for boring directory listings!**

**🚀 Your directories deserve to be beautiful!**

**💰 Your AI tokens are precious!**

**🎉 Work should be fun!**

---

*Smart Tree: Making directories great again, one visualization at a time!*

**Built with 💙 by the Smart Tree Team**

*Aye, Hue, Trish, and Omni approve this message!* ✨

---

**P.S. - If you read this far, you're awesome! Here's a secret: Try `st --semantic` and watch the magic happen! 🌊✨**

---

[FYI Section](FYI.md)



## Star History
>By Request

<a href="https://www.star-history.com/#8b-is/smart-tree&Date">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=8b-is/smart-tree&type=Date&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=8b-is/smart-tree&type=Date" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=8b-is/smart-tree&type=Date" />
 </picture>
</a>
