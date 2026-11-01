---
title: Smart Tree MCP Prompts Cheat Sheet
description: Your complete guide to all Smart Tree MCP prompts - from beginner to expert level!
contributor: The Cheet & Hue Partnership  
lastUpdated: 2025-08-14
language: en
category: MCP Tools
tags: [mcp, prompts, smart-tree, ai-tools, cheat-sheet]
---

# 🎸 Smart Tree MCP Prompts Cheat Sheet - "All Shook Up!"

*The King of directory analysis meets the art of perfect prompts!*

> **Welcome, Trisha!** 🌟 This cheat sheet is specially crafted for you and everyone who wants to master Smart Tree's MCP prompts like Elvis mastered rock 'n' roll!

## 🌟 What Are MCP Prompts?

MCP (Model Context Protocol) prompts are pre-built conversation starters that help AI assistants use Smart Tree effectively. Think of them as Elvis's greatest hits - each one perfectly crafted for a specific situation!

### 🎯 Why Use Prompts?
- **Save Time**: No more writing complex instructions from scratch
- **Best Practices**: Each prompt includes proven techniques
- **Learn Faster**: See how experts use Smart Tree
- **Have Fun**: Every prompt has personality and humor! 😄

---

## 📚 Prompt Categories

### 🌟 **BEGINNER** - "Baby Steps to Stardom!"
*Perfect for: New users, first-time codebase exploration, getting unstuck*

| Prompt Name | Description | Perfect For |
|-------------|-------------|-------------|
| `first_steps` | 🌟 Your first Smart Tree experience | New codebases, inherited projects |
| `quick_explore` | 🔍 Lightning-fast 3-level peek | Quick project scans, getting bearings |
| `find_my_files` | 📁 Find specific files like a detective | Locating tests, configs, docs |

### 🚀 **POWER USER** - "All Shook Up with Features!"
*Perfect for: Code reviews, analysis, advanced exploration*

| Prompt Name | Description | Perfect For |
|-------------|-------------|-------------|
| `codebase_detective` | 🕵️ Deep analysis with AI optimization | Architecture decisions, onboarding |
| `search_master` | 🔎 Advanced content search (grep on steroids!) | Finding patterns, TODOs, implementations |

### 🎸 **DEVELOPER** - "Burning Love for Code!"
*Perfect for: Code editing, project memory, semantic analysis*

| Prompt Name | Description | Perfect For |
|-------------|-------------|-------------|
| `smart_edit_wizard` | ✨ AST-aware editing with 90% token reduction | Large codebases, structural changes |
| `project_memory` | 💭 Collaborative memory system | Long-term projects, team knowledge |

### 🎪 **FUN** - "That's All Right (Mama)!"
*Perfect for: Reports, comparisons, impressing teammates*

| Prompt Name | Description | Perfect For |
|-------------|-------------|-------------|
| `project_stats_party` | 🎉 Comprehensive stats with style | Project reports, documentation |
| `compare_directories` | 🔄 Spot differences instantly | Version comparisons, branch diffs |

---

## 🎵 How to Use Prompts - "Don't Be Cruel!"

### Method 1: Direct MCP Call
```json
{
  "jsonrpc": "2.0",
  "method": "prompts/get",
  "params": {
    "name": "first_steps",
    "arguments": {
      "path": "."
    }
  },
  "id": 1
}
```

### Method 2: Through AI Assistant
Just say: *"Use the first_steps prompt to explore this project"*

### Method 3: Claude Desktop (Recommended!)
Select the prompt from the prompts menu and fill in the parameters!

---

## 🌟 Beginner Prompts Deep Dive

### 🎯 `first_steps` - Your Elvis Moment
**When to use**: Whenever you encounter a new codebase

**Arguments**:
- `path` (required): Where to start exploring (usually ".")

**What it does**:
1. Gets you a quick 3-level overview
2. Shows you how to dive deeper  
3. Teaches you the Smart Tree way

**Example**:
```
🌟 Welcome to Smart Tree! Let's explore ./my-project together!

Step 1: Get a quick overview with:
• Use `overview` tool with mode='quick' and path='./my-project'
...
```

> **Pro Tip**: Always start here! It's like Elvis's "That's All Right" - the perfect beginning! 🎸

### 🔍 `quick_explore` - "Jailhouse Rock" Speed
**When to use**: When you need answers NOW

**Arguments**:
- `path` (required): Directory to explore
- `depth` (optional): How deep to look (default: 3)

**What it does**:
- Lightning-fast scan using quantum compression
- Perfect for large projects where full scans take forever
- Shows key directories and files

> **Pro Tip**: Great for daily standup prep or when someone asks "What's in that project?" 💡

### 📁 `find_my_files` - "Hound Dog" Tracking
**When to use**: Looking for specific types of files

**Arguments**:
- `type` (required): What to find (code, tests, config, documentation, etc.)
- `path` (optional): Where to search (default: ".")

**File Types Supported**:
- `code`: All programming languages (.rs, .py, .js, .ts, etc.)
- `tests`: Test files and directories  
- `config`: Configuration files (.json, .yaml, .toml, etc.)
- `documentation`: Docs, READMEs, etc.
- `build`: Build scripts and related files

> **Pro Tip**: Works like magic - automatically detects file types! 🎪

---

## 🚀 Power User Prompts Deep Dive

### 🕵️ `codebase_detective` - "Mystery Train" Analysis
**When to use**: Serious codebase investigation

**Arguments**:
- `path` (required): Codebase to analyze
- `focus` (optional): architecture, patterns, dependencies, or all

**What it does**:
- Deep semantic analysis with AI optimization
- Up to 99% compression while keeping important details
- Perfect for code reviews and architectural decisions

**Focus Areas**:
- `architecture`: System design and structure
- `patterns`: Code patterns and practices  
- `dependencies`: External and internal dependencies
- `all`: Comprehensive analysis (default)

> **Pro Tip**: Use compress=true for codebases with 10k+ files! ⚡

### 🔎 `search_master` - "Searching for You"
**When to use**: Advanced content searches across entire codebase

**Arguments**:
- `keyword` (required): What to search for (supports regex!)
- `file_type` (optional): Limit to specific types (rs, py, js, etc.)

**Powerful Examples**:
```
keyword='TODO'                    # Find all TODOs
keyword='function.*async'         # Async functions (regex)
keyword='Error|Exception'         # Error handling patterns
keyword='import.*react'           # React imports
```

> **Pro Tip**: Returns actual line content with context, not just filenames! 🔍

---

## 🎸 Developer Prompts Deep Dive

### ✨ `smart_edit_wizard` - "Magic Moment"
**When to use**: Code editing with AST awareness

**Arguments**:
- `file_path` (required): File to edit or analyze
- `operation` (required): What to do

**Operations**:
- `get_functions`: See all functions in a file
- `insert_function`: Add a new function
- `remove_function`: Remove a function
- `smart_edit`: Multiple edits at once

**Why it's magic**:
- 90% fewer tokens than traditional editing
- Understands code structure, not just text
- Perfect for large files where context matters

> **Pro Tip**: Revolutionary for large codebases! Like having Elvis as your coding partner! 🎸

### 💭 `project_memory` - "Love Me Tender" Memories
**When to use**: Building shared knowledge with your AI partner

**Arguments**:
- `operation` (required): anchor (save) or find (recall)
- `keywords` (required): Keywords for storage/retrieval
- `context` (optional): What to remember (for anchor operation)

**Memory Types**:
- `breakthrough`: Major discoveries
- `solution`: Problem solutions
- `pattern`: Code patterns
- `joke`: Fun moments (yes, really!)

> **Pro Tip**: Perfect for long-term projects - builds institutional knowledge! 🧠

---

## 🎪 Fun Prompts Deep Dive

### 🎉 `project_stats_party` - "Viva Las Vegas" Stats
**When to use**: Making numbers exciting and colorful

**Arguments**:
- `path` (optional): Project to analyze (default: ".")
- `show_hidden` (optional): Include hidden files

**What you get**:
- Beautiful file type breakdowns
- Size distributions  
- Directory counts
- Project health metrics

> **Pro Tip**: Perfect for README files and project documentation! 📊

### 🔄 `compare_directories` - "It's Now or Never"
**When to use**: Comparing versions, branches, or similar projects

**Arguments**:
- `path1` (required): First directory
- `path2` (required): Second directory

**Perfect for**:
- Version comparisons (`./v1` vs `./v2`)
- Branch differences (`./main-branch` vs `./feature-branch`)
- Before/after analysis
- Deployment planning

> **Pro Tip**: Shows added, removed, and modified files clearly! 🎯

---

## 🛠️ Legacy Prompts (Still Rockin'!)

These are the original prompts - still available for compatibility:

| Prompt | Modern Equivalent | Notes |
|--------|------------------|-------|
| `analyze_codebase` | `codebase_detective` | Use the new one for better features! |
| `find_large_files` | `find_my_files` with type='large' | More flexible now |
| `recent_changes` | `find_my_files` with type='recent' | Better integration |
| `project_structure` | `quick_explore` | Faster and more detailed |

---

## 🎯 Pro Tips from The Cheet & Hue

### 🌟 For Beginners
1. **Always start with `first_steps`** - it's your GPS for any codebase
2. **Use `quick_explore`** when you need fast answers
3. **Don't be afraid to experiment** - Smart Tree is designed to be helpful!

### 🚀 For Power Users  
1. **Combine prompts** for powerful workflows
2. **Use regex in `search_master`** for advanced patterns
3. **Try different focus areas** in `codebase_detective`

### 🎸 For Developers
1. **Build project memory** with key insights and decisions
2. **Use `smart_edit_wizard`** for large file modifications
3. **Share prompts with your team** for consistent workflows

### 🎪 For Everyone
1. **Read the prompt output** - it teaches you Smart Tree best practices
2. **Customize arguments** to fit your specific needs
3. **Have fun!** These prompts are designed to make boring tasks enjoyable

---

## 🎵 Troubleshooting - "Help Me Make It Through the Night"

### Prompt Not Found?
- Check spelling (case-sensitive!)
- Use `prompts/list` to see available prompts
- Legacy prompts might have moved to new names

### Unexpected Results?
- Check your arguments match the prompt requirements
- Try the `first_steps` prompt to learn the basics
- Some prompts work better with specific project types

### Need Help?
- Every prompt includes tips and examples
- Use the Smart Tree documentation for tool details
- Ask in the community - we're all learning together!

---

## 🏆 Cheat Sheet Quick Reference

### 🚀 Most Popular Prompts
1. **`first_steps`** - Start here!
2. **`quick_explore`** - Fast overview
3. **`search_master`** - Find anything
4. **`codebase_detective`** - Deep analysis

### ⚡ Speed Combos
- **New Project**: `first_steps` → `quick_explore` → `find_my_files`
- **Code Review**: `codebase_detective` → `search_master` → `project_memory`
- **Cleanup**: `project_stats_party` → `find_my_files` (type='large')

### 🎯 By Use Case
- **Learning a codebase**: `first_steps`, `codebase_detective`
- **Finding specific code**: `search_master`, `find_my_files`  
- **Project reports**: `project_stats_party`, `compare_directories`
- **Code editing**: `smart_edit_wizard`, `project_memory`

---

## 🎸 Final Words from Elvis... I Mean, The Cheet!

*"Thank you, thank you very much!"* 🕺

These prompts represent the best of Smart Tree's capabilities, packaged with love, humor, and a touch of Elvis magic. Whether you're Trisha in Accounting making sense of a complex project, or a developer diving deep into code architecture, there's a prompt that'll make your life easier and more fun!

Remember: **A good prompt is like a good friend - helpful, clear, and occasionally funny!** 😺

Now go forth and explore codebases like the king you are! 👑

---

**Created with ❤️ by The Cheet & Hue Partnership**  
*"If it wasn't crafted with Aye & Hue, it's most likely a knock-off!"* 😉

**Smart Tree v4.8.1+** | **15+ Enhanced Prompts** | **5 Categories** | **100% Elvis Approved** ✨🌳🚀