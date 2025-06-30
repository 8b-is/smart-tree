---
title: The Ultimate st Cheet Sheet
description: Your friendly guide to the smartest tree command in the digital forest!
contributor: The Cheet
lastUpdated: 2025-06-25
tags:
  - rust [1]
  - cli [1]
  - documentation [5]
  - ai-tools [10]
  - file-management [15]
---

# `st`: Not Your Average abbreviated Smart Tree! 🌳✨

Welcome, brave adventurer, to the official Cheet Sheet for `st`! This isn't just another `tree` command. Oh no. This is a **smart-tree**, built with Rust, speed, and a whole lotta love. It's designed to be your best friend for directory visualization, whether you're a human, an AI, or a particularly clever squirrel.

I love this tool more than Elvis loves peanut butter and banana sandwiches. And Hue, my friend, I love you too! Let's dive in!

## 🚀 Basic Usage

Getting started is as easy as pie. Mmm, pie.

| Command | Description |
|---------|-------------|
| `st` | Shows the tree for the current directory. Simple! |
| `st [PATH]` | Shows the tree for a specific directory or file. |
| `st --help`| Displays all the glorious options you see here. |

> **Pro Tip:** `st` is your go-to tool for quickly understanding a project's structure. It's like having X-ray vision for your filesystem!
{.is-success}

## 🎨 Output Modes (`--mode`)

`st` can talk in many languages! Pick the one that suits your audience.

| Mode | Flag | What it does |
|---|---|---|
| **Classic** | `-m classic` | The beautiful, human-readable default with emojis and metadata. |
| **Hex** | `-m hex` | For the AI whisperers. Fixed-width fields for easy parsing. |
| **JSON** | `-m json` | Structured data, perfect for scripts and programs. Use `--compact` for a single line! |
| **AI** | `-m ai` | A special blend of hex and stats, optimized for Large Language Models. The future is now! |
| **Claude** | `-m claude` | 🚀 **QUANTUM MODE!** 99% compression (10x) using MEM|8 format. Saves $1,270 per Chromium! |
| **Quantum** | `-m quantum` | Native quantum format with token mapping (8x compression). The raw power! |
| **Stats** | `-m stats` | Just the facts, ma'am. A summary of the directory without the full tree. |
| **CSV / TSV** | `-m csv` / `-m tsv` | Your spreadsheet's new best friend. |
| **Digest** | `-m digest` | Super compact, single-line output. Perfect for a quick check-in. |

> **🌟 v2.0 Quantum Tip:** Use `st -m claude` for maximum compression when feeding to AI. It's like fitting an elephant in a matchbox!

---

## 🔍 Detective Work: Filtering & Searching

Find exactly what you're looking for with these powerful filters.

| Option | Description | Example |
|---|---|---|
| `--find <PATTERN>` | Find files/directories matching a regex pattern. | `st --find ".*\.rs$"` |
| `--type <EXT>` | Filter by file extension (e.g., "rs", "py", "md"). | `st --type md` |
| `--min-size <SIZE>` | Show files *larger* than a size (e.g., "1M", "500K"). | `st --min-size 10K` |
| `--max-size <SIZE>` | Show files *smaller* than a size. | `st --max-size 1K` |
| `--newer-than <DATE>` | Find files modified after a date (YYYY-MM-DD). | `st --newer-than 2025-01-01` |
| `--older-than <DATE>` | Find files modified before a date. | `st --older-than 2024-12-31` |
| `--search <KEYWORD>` | **X-Ray Vision!** Searches *inside* files for a keyword. | `st --type rs --search "TODO"` |

---

## 🗺️ Traversal & Ignore Rules

Control how deep `st` goes and what it sees.

| Option | Description |
|---|---|
| `-d, --depth <NUM>` | Limit how many levels deep to scan. Default is 5. |
| `--no-ignore` | Ignores `.gitignore` files. See what Git is hiding! |
| `--no-default-ignore` | Ignores built-in ignores (`node_modules`, etc.). Use with caution! |
| `-a, --all` | Show hidden files (those starting with a `.`). |
| `--show-ignored` | Shows ignored files/dirs in `[brackets]`. Great for debugging! |
| `--everything` | The nuclear option: combines `--all`, `--no-ignore`, and `--no-default-ignore`. |

---

## 🤖 The "AI" Mode Explained

The `-m ai` output is a thing of beauty, designed for our AI pals. Here's a breakdown of that funky hex format:

```
# TREE_HEX_V1:
# CONTEXT: Rust: st - Smart Tree...
# HASH: ef1ad13faae33465
# ┌─ Level
# │  ┌─── Permissions (octal)
# │  │    ┌─── User ID
# │  │    │    ┌─── Group ID
# │  │    │    │    ┌──────── Size (bytes, hex)
# │  │    │    │    │        ┌──────── Modified Date (Unix timestamp, hex)
# │  │    │    │    │        │        ┌─── Icon & Name
# │  │    │    │    │        │        │
# │  │    │    │    │        │        │
  0 1fd 03e8 03e8 00000000 685ca488 📁 .
  2 1b4 03e8 03e8 00000079 6853a206 📄 settings.local.json
# ...
# STATS:
# F:20 D:9 S:50c17 (0.3MB)
# TYPES: rs:13 md:6 sh:2 ...
# LARGE: scanner.rs:14772 ...
# DATES: 6853a206-685ca84d
# END_AI
```

- **Level**: Indentation level.
- **Permissions**: File permissions in octal format (like `chmod`).
- **User/Group ID**: Who owns the file.
- **Size**: File size in hexadecimal bytes.
- **Modified Date**: Last modified time as a hex Unix timestamp.
- **STATS**: A super-helpful summary of file counts, types, largest files, and date ranges.

This format gives an AI everything it needs to know in a compact, predictable structure. It's brilliant!

---

## ✨ Special Abilities & MCP

`st` has some extra tricks up its sleeve!

| Option | Description |
|---|---|
| `--stream` | **Game-changer!** Streams output as it scans. No more waiting for large directories. |
| `--mcp` | Runs `st` as a Model Context Protocol server, allowing AIs to use it as a tool. |
| `--mcp-tools` | Lists the tools `st` provides as an MCP server. |
| `--mcp-config` | Shows the config needed to connect `st` to an AI assistant. |
| `-z, --compress` | Compresses the output with zlib and encodes in base64. Great for sending over the network. |

---

HAVE FUN OUT THERE! And remember, a well-organized project is a happy project.

*Visit cheet.is for more nuggets of wisdom!*