# 🌳 Smart Tree (stree) - The Intelligent Directory Visualizer

A blazingly fast, AI-friendly directory tree visualization tool written in Rust. Smart Tree goes beyond traditional tree commands by providing intelligent filtering, multiple output formats, and optimization for both human and AI consumption.

**Status**: ✅ **Fully Implemented** - All core features are now working!

## 🚀 Features

### Core Features
- **Multiple Output Formats**: Classic tree, hex, JSON, CSV, TSV, digest, and AI-optimized formats
- **Intelligent Filtering**: By file type, size, date, with automatic `.gitignore` respect
- **Permission Handling**: Shows inaccessible directories with `*` indicator
- **Built-in Search**: `--find` for quick file location during traversal
- **Compression**: Built-in zlib compression for any output format
- **Statistics**: Directory summaries with file counts, sizes, and type distributions
- **Show Ignored**: `--show-ignored` flag displays ignored directories in brackets [dirname]
- **Hex Statistics**: Stats mode shows file counts, directory counts, and sizes in hexadecimal

### AI Optimization
- **Compact Hex Format**: Fixed-width fields for easy parsing
- **AI Mode**: Combines hex tree with statistics for optimal token usage
- **Digest Mode**: Ultra-compact single-line summary (hash + stats) for AI pre-checks
- **Project Context Detection**: Automatically detects and includes project type/description
- **SHA256 Hash**: Provides consistency verification for caching and change detection
- **AI JSON Mode**: Optional JSON-wrapped output for programmatic consumption (--ai-json)
- **Compression**: ~10x reduction in output size
- **No Wasted Tokens**: Every byte counts for AI consumption

### Performance
- Written in Rust for maximum speed and efficiency
- SIMD optimizations where applicable
- Minimal memory footprint
- Handles massive directory trees with ease

## 📦 Installation

```bash
# From source
cargo install --path .

# Build from source
git clone https://github.com/8bit-wraith/smart-tree
cd smart-tree
cargo build --release
sudo cp target/release/stree /usr/local/bin/

# Using the manage script
./scripts/manage.sh install
```

## 🎯 Usage

```bash
# Basic usage
stree                          # Current directory
stree /path/to/directory       # Specific directory

# Output modes
stree -m hex                   # Hex format with file metadata
stree -m json                  # JSON output
stree -m ai                    # AI-optimized format
stree -m digest                # Ultra-compact digest (one line)
stree -m stats                 # Statistics only
stree -m csv                   # CSV format
stree -m tsv                   # TSV format

# Filtering
stree --find "config"          # Find files/dirs matching pattern
stree --type rs                # Only Rust files
stree --min-size 1M            # Files larger than 1MB
stree --newer-than 2024-01-01  # Recently modified files

# Options
stree --no-emoji               # Plain text output
stree --no-ignore              # Don't respect .gitignore
stree --show-ignored           # Show ignored dirs in brackets
stree --depth 3                # Limit depth
stree -z                       # Compress output

# AI usage
AI_TOOLS=1 stree               # Auto AI mode + compression
stree -m ai -z                 # Manual AI mode + compression
stree -m ai --ai-json          # AI mode with JSON wrapper
stree -m digest                # Quick digest for AI pre-check

# Digest mode - perfect for AI to quickly check directories
stree -m digest /large/project # Returns: HASH: 9b3b00cbcc1e8503 F:1623 D:89 S:ac39e7 TYPES: js:523 py:412...
```

## 🏗️ Architecture

```
src/
├── main.rs           # CLI entry point
├── scanner.rs        # Directory traversal engine
├── filters.rs        # File filtering logic
├── formatters/       # Output format implementations
│   ├── classic.rs
│   ├── hex.rs
│   ├── json.rs
│   ├── ai.rs
│   ├── digest.rs
│   └── stats.rs
├── compression.rs    # Zlib compression
└── utils.rs          # Helper functions
```

## 🎨 Output Examples

### Classic Mode
```
📁 my-project (2.3 MB, 45 files)
├── 📄 Cargo.toml (1.2 KB)
├── 📁 src (15 files)
│   ├── 📄 main.rs (5.6 KB)
│   └── 📄 lib.rs (3.2 KB)
├── 📁 [.git] (ignored)
└── 📁 tests (8 files)
    └── 📄 integration.rs (2.1 KB)
```

### Hex Mode (No indentation, fixed-width)
```
0 1fd 03e8 03e8 00000924 68538d0d d my-project
1 1b4 03e8 03e8 000004b0 68538f4c f Cargo.toml
1 1fd 03e8 03e8 00000200 68538e12 d src
2 1b4 03e8 03e8 000015e0 68538a21 f main.rs
2 1b4 03e8 03e8 00000c80 68538b15 f lib.rs
1 1fd 03e8 03e8 00001000 68538a00 d [.git]
```

### AI Mode (Optimal for LLMs)
```
TREE_HEX_V1:
CONTEXT: Rust: my-project - A blazingly fast web server
HASH: 3d9a2f1cae6af9a5
0 1fd 03e8 03e8 00000924 68538d0d d my-project
1 1b4 03e8 03e8 000004b0 68538f4c f Cargo.toml
...

STATS:
F:45 D:12 S:23fc00 (2.3MB)
TYPES: rs:35 toml:3 md:2 txt:5
LARGE: main.rs:15e0 lib.rs:c80 README.md:960
DATES: 68536122-6853981a
END_AI
```

### Digest Mode (Ultra-compact for AI pre-checks)
```
HASH: 9b3b00cbcc1e8503 F:45 D:12 S:23fc00 TYPES: rs:35 toml:3 md:2 txt:5 json:1
```

### AI JSON Mode (--ai-json flag)
```json
{
  "version": "AI_JSON_V1",
  "context": "Rust: my-project - A blazingly fast web server",
  "hash": "3d9a2f1cae6af9a5",
  "hex_tree": [
    "0 1fd 03e8 03e8 00000924 68538d0d d my-project",
    "1 1b4 03e8 03e8 000004b0 68538f4c f Cargo.toml"
  ],
  "statistics": {
    "files": 45,
    "directories": 12,
    "total_size": 2358272,
    "total_size_mb": "2.3",
    "file_types": [
      {"extension": "rs", "count": 35},
      {"extension": "toml", "count": 3}
    ],
    "largest_files": [
      {"name": "main.rs", "size": 5600}
    ],
    "date_range": "68536122-6853981a"
  }
}
```

### Stats Mode (Hex values for counts and sizes)
```
Directory Statistics:
  Files: 2d (45)
  Directories: c (12)
  Total size: 23fc00 (2.3MB)
File Types (by extension):
  rs: 23 (35)
  toml: 3 (3)
  md: 2 (2)
  txt: 5 (5)
Largest Files:
  src/main.rs: 15e0 (5.6KB)
  src/lib.rs: c80 (3.2KB)
  README.md: 960 (2.4KB)
Date Range:
  Oldest: 2024-01-15 10:00:00
  Newest: 2024-12-19 15:30:00
```

### JSON Mode
```json
{
  "path": "/home/user/my-project",
  "name": "my-project",
  "type": "directory",
  "size": 2358272,
  "permissions": "rwxr-xr-x",
  "modified": "2024-12-19T15:30:00Z",
  "children": [
    {
      "name": "Cargo.toml",
      "type": "file",
      "size": 1200,
      "permissions": "rw-r--r--"
    }
  ]
}
```

### CSV Mode
```csv
path,name,type,size,permissions,modified,uid,gid
/home/user/my-project,my-project,directory,2358272,rwxr-xr-x,2024-12-19T15:30:00Z,1000,1000
/home/user/my-project/Cargo.toml,Cargo.toml,file,1200,rw-r--r--,2024-12-19T14:00:00Z,1000,1000
```

## 🔧 Configuration

Environment variables:
- `STREE_DEFAULT_MODE`: Default output mode
- `STREE_DEFAULT_DEPTH`: Default depth limit
- `AI_TOOLS=1`: Auto-enable AI mode with compression

## 🤝 Contributing

Contributions welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) first.

### Development Setup
```bash
git clone https://github.com/8bit-wraith/smart-tree
cd smart-tree
cargo build --release
cargo test

# Using the manage script
./scripts/manage.sh build
./scripts/manage.sh test
./scripts/manage.sh run -- --mode hex /path/to/dir
```

### Key Implementation Features
- **Parallel Scanning**: Uses rayon for fast directory traversal
- **Efficient Gitignore**: Cached gitignore parsing with globset
- **Memory Efficient**: Streams output without loading entire tree
- **Permission Handling**: Gracefully handles permission denied errors
- **Compression**: Optional zlib compression for any output format

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## ✅ Implementation Status

All core features are now implemented:
- ✅ Classic tree output with emojis and file sizes
- ✅ Hex mode with fixed-width fields
- ✅ JSON output with full metadata
- ✅ CSV/TSV export formats
- ✅ AI-optimized mode with compression
- ✅ Project context auto-detection (Cargo.toml, package.json, etc.)
- ✅ SHA256 hash for consistency verification
- ✅ AI JSON mode with --ai-json flag
- ✅ Statistics mode with hex values
- ✅ File filtering (type, size, date)
- ✅ Pattern matching with --find
- ✅ Gitignore support with --show-ignored option
- ✅ Permission error handling
- ✅ Compression for all output modes
- ✅ Parallel directory scanning
- ✅ Management script with humor

## 🙏 Acknowledgments

- Inspired by the classic `tree` command
- Built for the AI era where every token counts
- Optimized for both human readability and AI consumption
- Special thanks to all contributors

---

**Smart Tree**: Because directory visualization should be intelligent, fast, and token-efficient! 🌳✨