# Smart Tree Zsh Completion Examples

## What You'll See When Using Completions

### Basic Tab Completion
```bash
$ st <TAB>
Completing files and directories
.git/     Cargo.lock     README.md      completions/  scripts/      tests/
.github/  Cargo.toml     build/         dxt/          src/          tools/

$ st --<TAB>
Completing option
--cheet              -- Show the cheatsheet with all formatting modes and options
--completions        -- Generate shell completion scripts
--compress       -z  -- Compress output (recommended for AI modes)
--depth          -d  -- Maximum depth to traverse (default: 5)
--entry-type         -- Filter by entry type
--everything     -a  -- Show all files including hidden and ignored
--file-type      -t  -- Filter by file extension
--find           -f  -- Search for files/dirs matching regex pattern
--help           -h  -- Show help information
--man                -- Generate the man page
--max-size           -- Maximum file size filter
--mcp                -- Run as MCP (Model Context Protocol) server for AI assistants
--mcp-config         -- Show MCP configuration for Claude Desktop
--mcp-tools          -- List all MCP tools (20+ tools for AI agents)
--min-size           -- Minimum file size filter
--mode           -m  -- Output format mode
--newer-than         -- Show files newer than date (YYYY-MM-DD)
--no-default-ignores -- Disable default ignore patterns
--no-emoji       -e  -- Disable emoji in output (cleaner for scripts)
--older-than         -- Show files older than date (YYYY-MM-DD)
--path-mode      -p  -- Path display mode
--search         -s  -- Search for keyword in file contents
--show-filesystems -F -- Show filesystem type indicators
--show-ignored   -i  -- Show ignored directories in brackets
--stream             -- Enable streaming output for large directories (>10k files)
--version        -V  -- Show version and check for updates
```

### Mode Completion with Descriptions
```bash
$ st --mode <TAB>
Completing output mode
ai               -- AI-optimized format with compression (default for MCP)
ai_json          -- AI-friendly JSON with metadata
classic          -- Traditional tree view with Unicode box drawing
csv              -- Comma-separated values
digest           -- SHA256 hash only (minimal output)
hex              -- AI-optimized hexadecimal format
json             -- Standard JSON format
ls               -- Unix ls -Alh format with detailed file info
markdown         -- Comprehensive markdown report
mermaid          -- Mermaid diagrams (flowchart/mindmap/treemap)
quantum          -- MEM|8 quantum format (8x compression)
quantum-semantic -- Semantic-aware quantum compression
semantic         -- Wave-based semantic grouping
stats            -- Statistics only
summary          -- Human-readable summary
summary-ai       -- AI-optimized summary (10x compression)
tsv              -- Tab-separated values
waste            -- Show space wasters and large files
```

### Contextual Tips
```bash
$ st . --find <TAB>
💡 TIP: Use --mode ls with --find to see full match context

$ st . --mode summary-ai <TAB>
💡 TIP: Use 'summary-ai' for 10x compression when working with LLMs

$ st . --stream <TAB>
💡 TIP: Streaming mode essential for dirs with >100k files
```

### Using Aliases (after setup)
```bash
$ stai
# Runs: st . --mode summary-ai -z

$ stfind "test"
# Runs: st . --find "test"

$ stsearch "TODO"
# Runs: st . --search "TODO"

$ stwaste
# Runs: st . --mode waste

$ strecent
# Runs: st . --newer-than 2025-07-06  (7 days ago)

$ sttips
🌳 Smart Tree Tips & Tricks 🌳

QUICK COMMANDS:
  st                           # Classic tree view of current directory
  st /path --mode summary-ai   # AI-optimized summary with 10x compression
  st . --find "test" --mode ls # Find files and show with ls format
  st . --search "TODO"         # Search file contents for keyword
  st . --mode waste            # Find large files and space wasters
[... more tips ...]
```

### Auto-suggestions (with zsh-autosuggestions)
```bash
# As you type, you'll see grayed-out suggestions based on history:
$ st . --mode sum
                 mary-ai -z  # <- This appears in gray

# Press → to accept the suggestion
```

### File Type Filtering
```bash
$ st . --file-type <TAB>
# Your cursor here, type any extension like:
rs py js ts go java cpp c rb php swift kt

$ st . --entry-type <TAB>
Completing type
d  -- directories only
f  -- files only
```

### Date Filtering
```bash
$ st . --newer-than <TAB>
# Shows tip: Use YYYY-MM-DD format

$ st . --newer-than 2025-07-<TAB>
# You can continue typing the date
```

### Size Filtering
```bash
$ st . --min-size <TAB>
Completing size
1G   1K   1M   10M  100M

$ st . --max-size <TAB>
Completing size
1G   1K   1M   10M  100M
```

### Path Mode Options
```bash
$ st . --path-mode <TAB>
Completing path mode
full     -- Show full absolute paths
off      -- Don't show paths (default)
relative -- Show relative paths from root
```

## Advanced Usage

### Combining Options
```bash
# Find large Rust files modified recently
$ st . --file-type rs --min-size 10K --newer-than 2025-07-01 --mode ls

# Search for TODOs in Python files, excluding tests
$ st src/ --search "TODO" --file-type py --mode ls

# Get AI-optimized summary of recent changes
$ st . --newer-than 2025-07-10 --mode summary-ai -z
```

### MCP Integration
```bash
# Show MCP config for Claude Desktop
$ st --mcp-config

# List all MCP tools
$ st --mcp-tools

# Run as MCP server
$ st --mcp
```

## Tips for Power Users

1. **Use `sttips` frequently** - It shows all the tips and common patterns

2. **Combine with other tools**:
   ```bash
   st . --mode json | jq '.files[] | select(.size > 1000000)'
   st . --find "\.rs$" --mode csv | csvlook
   ```

3. **Create your own aliases** in `~/.config/st/config.zsh`:
   ```bash
   alias stbig="st . --min-size 10M --mode ls"
   alias sttodo="st . --search 'TODO|FIXME' --mode ls"
   ```

4. **Use completion to explore**: Even if you know the command, tab through options to discover new features

5. **Check for updates**:
   ```bash
   st --version  # Shows current version and checks for updates
   ```