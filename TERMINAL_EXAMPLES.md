# 🌳 Smart Tree Terminal Examples

> Beautiful, colorful terminal output examples showing Smart Tree in action!

## Table of Contents
- [Classic Tree View](#classic-tree-view)
- [AI-Optimized Mode](#ai-optimized-mode)
- [Quantum Compression](#quantum-compression)
- [Search Features](#search-features)
- [MCP Server Tools](#mcp-server-tools)
- [Version Management](#version-management)

---

## Classic Tree View

```bash
$ st --mode classic --depth 2
```

```
📁 smart-tree
├── 📁 docs
│   ├── 📝 AI_OPTIMIZATION.md (4.95 KiB)
│   ├── 📝 COMPRESSION_GUIDE.md (6.16 KiB)
│   └── 📝 SSE_USAGE.md (5.21 KiB)
├── 📁 src
│   ├── 🦀 main.rs (59.19 KiB)
│   ├── 🦀 scanner.rs (98.00 KiB)
│   └── 🦀 lib.rs (3.41 KiB)
├── 🔧 Cargo.toml (2.25 KiB)
├── 📝 README.md (3.41 KiB)
└── 📜 LICENSE (1.04 KiB)

5 directories, 8 files, 183.62 KiB total
```

---

## AI-Optimized Mode

```bash
$ st --mode ai --compress
```

```
TREE_HEX_V1:
0 755 1000 1000 00000000 66B12345 📁 smart-tree
1 755 1000 1000 00000000 66B12345 📁 docs
2 644 1000 1000 000013A7 66B12345 📝 AI_OPTIMIZATION.md
2 644 1000 1000 000018C5 66B12345 📝 COMPRESSION_GUIDE.md
1 755 1000 1000 00000000 66B12345 📁 src
2 644 1000 1000 0000E67B 66B12345 🦀 main.rs
2 644 1000 1000 00017F00 66B12345 🦀 scanner.rs

STATS: F:8 D:3 S:2BCE7 (183.62 KiB)
TYPES: rs:3 md:4 toml:1
END_AI
```

---

## Quantum Compression

```bash
$ st --mode quantum-semantic src/
```

```
QUANTUM_V3:973X_FASTER
[WAVE:8B:IS:MEM8]
╔══════════════════════════════════════╗
║  Semantic Wave Groups Detected:      ║
║  • Core Logic: 45% coherence         ║
║  • Formatters: 30% coherence         ║
║  • MCP Tools: 25% coherence          ║
╚══════════════════════════════════════╝
Σ 183.62 KiB → 18.4 KiB (10x compression)
🌊 Wave signature: ∿∿∿∿∿∿∿∿
```

---

## Search Features

```bash
$ st --search "TODO" --include-line-content
```

```ansi
📁 smart-tree
├── 🦀 src/main.rs
│   └── [38;5;196mL142[0m: // TODO: Implement quantum entanglement
│   └── [38;5;196mL256[0m: // TODO: Add wave collapse detection
├── 🦀 src/scanner.rs
│   └── [38;5;196mL89[0m: // TODO: Optimize for large directories
└── 📝 README.md
    └── [38;5;196mL45[0m: - TODO: Add benchmarks

Found 4 matches in 3 files
```

---

## MCP Server Tools

```bash
$ st --mcp-tools | jq '.tools[0:3]'
```

```json
[
  {
    "name": "quick_tree",
    "description": "🚀 Lightning-fast 3-level overview"
  },
  {
    "name": "project_overview",
    "description": "📊 Comprehensive project analysis"
  },
  {
    "name": "search_in_files",
    "description": "🔍 Content search with line numbers"
  }
]
```

---

## Version Management

```bash
$ ./scripts/manage.sh bump
```

```ansi
[38;5;51m🌳 Version Management 🔢 🌳[0m

[38;5;135m📊[0m Current version: v4.8.4
[38;5;135m📊[0m Bumping to: v4.8.5
[38;5;46m✅[0m Updated CLAUDE.md
[38;5;135m📊[0m Cleaning orphaned tags...
[38;5;46m✅[0m Version bumped to v4.8.5! [38;5;46m✅[0m

[38;5;226mNext steps:[0m
  1. Build: ./manage.sh build
  2. Test: ./manage.sh test
  3. Commit: git add -A && git commit -m 'chore: bump version to v4.8.5'
  4. Tag: git tag -a v4.8.5 -m 'Version 4.8.5'
  5. Push: git push origin main && git push origin v4.8.5
```

---

## File History Tracking

```bash
$ st --mode classic ~/.mem8/feedback/pending/
```

```
📁 pending
├── 📄 feedback_bug_20250813_081644.json (1.2 KiB)
├── 📄 tool_request_smart-tree-dev_20250813_081644.json (2.3 KiB)
└── 📄 feedback_feature_20250813_090122.json (890 B)

3 files pending upload
```

---

## Performance Metrics

```bash
$ time st --mode classic /large/codebase --stream
```

```
real    0m0.234s  ← 10-24x faster than tree!
user    0m0.189s
sys     0m0.045s

🌳 Processed 156,789 files in 0.234 seconds
⚡ Performance: 670,466 files/second
🌍 CO2 saved: ~2.3g (vs traditional tree command)
```

---

## Beautiful Git Status Integration

```bash
$ st --mode git-status
```

```
📁 smart-tree
├── 📁 src
│   ├── 🦀 main.rs [M]
│   ├── 🦀 scanner.rs [M]
│   └── 🦀 new_feature.rs [A]
├── 🔧 Cargo.toml [M]
├── 📝 README.md
└── 📝 CHANGELOG.md [M]

Modified: 4, Added: 1, Unchanged: 3
```

---

## Semantic Analysis

```bash
$ st --mode semantic src/
```

```
🧠 Semantic Code Groups:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📦 Core Engine (Wave: ∿∿∿)
  ├── scanner.rs - Directory traversal
  ├── tokenizer.rs - Pattern recognition
  └── quantum_scanner.rs - Wave mechanics

🎨 Formatters (Wave: ≈≈≈)
  ├── classic.rs - Traditional output
  ├── ai.rs - AI-optimized
  └── quantum.rs - Compressed binary

🔧 Tools (Wave: ~~~)
  ├── mcp/tools.rs - MCP endpoints
  └── smart_edit.rs - AST operations
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## Live SSE Monitoring

```bash
$ st --sse-server --sse-port 8420 /project
```

```
🌐 SSE Server Started on http://localhost:8420
📡 Monitoring: /project
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[08:16:44] 📝 Modified: src/main.rs
[08:16:45] ➕ Created: test.rs
[08:16:47] 🗑️  Deleted: old_file.rs
[08:16:50] 📊 Stats: 234 files, 45 dirs
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Clients connected: 3
Events sent: 127
Uptime: 5m 23s
```

---

## Error Handling

```bash
$ st /root/protected
```

```ansi
[38;5;196m⚠️  Permission denied:[0m /root/protected
[38;5;226m📁[0m /root/protected [38;5;196m*[0m
[38;5;244m└── (inaccessible)[0m

[38;5;244mNote: Directories marked with * require elevated permissions[0m
```

---

## Quick Stats

```bash
$ st --mode stats
```

```
📊 Directory Statistics
═══════════════════════════════════
Total Size:       1.23 GiB
Total Files:      12,456
Total Dirs:       1,234
Avg File Size:    103.4 KiB

📈 File Types (Top 5):
  .rs   4,567 files (36.7%)
  .md   2,345 files (18.8%)
  .json 1,234 files (9.9%)
  .toml   987 files (7.9%)
  .txt    654 files (5.2%)

⏰ Recent Activity:
  Last hour:    23 modifications
  Last 24h:    156 modifications
  Last week:   892 modifications
═══════════════════════════════════
```

---

## Fun with Emojis

```bash
$ st --mode classic test_files/
```

```
📁 test_files
├── 🦀 test.rs
├── 🐍 test.py
├── 📜 test.js
├── 🎨 style.scss
├── 📊 data.csv
├── 🖼️ image.jpg
├── 🎵 audio.mp3
├── 🎬 video.mp4
├── 📦 test.zip
├── 🔒 test.gpg
├── 🌐 test.wasm
└── 🧠 test.mem8

12 files, each with its perfect emoji! 
```

---

## Environment Impact

```bash
$ st --show-carbon-savings
```

```
🌍 Environmental Impact Report
═══════════════════════════════════════
Session Statistics:
  • Commands run: 42
  • Files scanned: 523,456
  • Time saved: 8.3 seconds
  • CPU cycles saved: ~2.1M

Carbon Footprint:
  • Traditional tree: ~12.4g CO2
  • Smart Tree: ~0.5g CO2
  • SAVED: 11.9g CO2 ✅

Cumulative Impact (This Month):
  • Commands: 1,234
  • CO2 Saved: 348g 🌳
  • Equivalent to: 1 tree planted! 🌲
═══════════════════════════════════════
```

---

> **Note**: Colors are rendered using ANSI escape codes in actual terminal output.
> Smart Tree is 10-24x faster than traditional tree, saving energy with every scan! 🌳

## Configuration

Create `~/.config/smart-tree/config.toml`:

```toml
[display]
emoji = true
colors = true
max_depth = 10

[performance]
stream_threshold = 10000
cache_enabled = true

[mcp]
compression = true
no_emoji = false
```

---

*Smart Tree v4.8.4 - Making directories beautiful and saving the planet, one scan at a time!* 🌍