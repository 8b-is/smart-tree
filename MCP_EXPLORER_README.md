# MCP Explorer 🎮

**Interactive MCP Tool Explorer for Humans**

Learn what AI tools do by actually using them! MCP Explorer is a universal client that connects to any MCP (Model Context Protocol) server and lets you interactively explore and use the available tools.

## 🌟 Features

- **Universal MCP Client**: Works with ANY MCP server, not just Smart Tree
- **Interactive Exploration**: Learn by doing with guided mode
- **Tool Lanes**: Organized by EXPLORE → ANALYZE → ACT workflow
- **Beautiful Output**: Rich terminal interface with colors and tables
- **Parameter Help**: Interactive parameter input with descriptions
- **History Tracking**: See what you've done
- **Beginner Friendly**: Guided mode helps you get started

## 🚀 Quick Start

### Install Dependencies (Optional but Recommended)

```bash
pip install -r requirements-explorer.txt
```

### Run with Smart Tree

```bash
# If Smart Tree is installed
python mcp_explorer.py --command "st --mcp"

# Or just use the default
python mcp_explorer.py
```

### Run with Any MCP Server

```bash
# Generic MCP server
python mcp_explorer.py --command "your-mcp-server --mcp"

# With arguments
python mcp_explorer.py --command "server --arg1 value --mcp"
```

### Use with Claude Desktop Config

```bash
# Use existing Claude Desktop config
python mcp_explorer.py --config ~/.config/claude/config.json --server smart-tree
```

## 🎯 Interactive Commands

Once connected, you'll see the `mcp>` prompt. Available commands:

| Command | Description | Example |
|---------|-------------|---------|
| `tools [lane]` | List all tools (optionally by lane) | `tools explore` |
| `info <tool>` | Show detailed tool information | `info search_in_files` |
| `call <tool>` | Call a tool interactively | `call quick_tree` |
| `lanes` | Show tools organized by lanes | `lanes` |
| `guided` | Start guided exploration | `guided` |
| `history` | Show command history | `history` |
| `help` | Show help | `help` |
| `quit` | Exit the explorer | `quit` |

## 🔍 Understanding Tool Lanes

MCP tools are organized into three lanes representing a natural workflow:

### 🔍 EXPLORE Lane
Start here! Discovery and overview tools.
- `quick_tree` - Fast directory overview
- `get_statistics` - Project statistics
- `server_info` - Server capabilities

### 🧪 ANALYZE Lane
Dive deeper! Search and analysis tools.
- `search_in_files` - Find content in files
- `find_files` - Locate files by pattern
- `semantic_analysis` - Understand code structure

### ⚡ ACT Lane
Make changes! Modification tools.
- `smart_edit` - Edit files efficiently
- `track_file_operation` - Track changes
- `insert_function` - Add code

## 📚 Example Session

```
$ python mcp_explorer.py

🚀 Connecting to MCP server...
✅ Connected to: Smart Tree v4.0.0
📦 Discovered 42 tools

🎮 Interactive MCP Explorer
Type 'help' for commands, 'quit' to exit

mcp> guided

🎯 Guided Exploration Mode
Let me guide you through the tools step by step!

Step 1: Choose your exploration path
1. 🔍 EXPLORE - Start with overview and discovery
2. 🧪 ANALYZE - Deep dive into code and content
3. ⚡ ACT - Make changes and modifications

Your choice (1-3): 1

Great choice! Let's start exploring.

Step 2: Available EXPLORE tools:
1. 🔍 quick_tree
   Lightning-fast 3-level directory overview...
2. 🔍 get_statistics
   Get comprehensive statistics about a directory...

Step 3: I recommend starting with 'quick_tree'
Would you like to try it? (y/n): y

📝 Parameters for quick_tree:
  path (Path to the directory): .
  depth (Maximum depth) [optional] [default: 3]: 

🔄 Calling quick_tree...

╭─ Result ─────────────────────────────────╮
│ {                                        │
│   "tree": "📁 smart-tree\n├── src\n..."  │
│   "stats": {                             │
│     "files": 127,                        │
│     "directories": 23                    │
│   }                                      │
│ }                                        │
╰──────────────────────────────────────────╯

mcp> call search_in_files

📝 Parameters for search_in_files:
  path (Path to search in): src
  keyword (Keyword or phrase to search for): TODO
  include_content (Include actual line content) [optional] [default: true]: 
  max_matches_per_file (Maximum matches per file) [optional] [default: 20]: 5

🔄 Calling search_in_files...

╭─ Result ─────────────────────────────────╮
│ {                                        │
│   "files_with_matches": 3,              │
│   "results": [                          │
│     {                                   │
│       "path": "/src/main.rs",           │
│       "matches": 2,                     │
│       "lines": [                        │
│         {                               │
│           "line_number": 42,            │
│           "content": "// TODO: Fix",    │
│           "column": 3                   │
│         }                               │
│       ]                                 │
│     }                                   │
│   ]                                     │
│ }                                        │
╰──────────────────────────────────────────╯

mcp> quit
👋 Goodbye!
```

## 🎓 Learning Path

1. **Start with `guided`** - Let the explorer guide you
2. **Explore with `tools explore`** - See overview tools
3. **Get details with `info <tool>`** - Understand what each tool does
4. **Try tools with `call <tool>`** - See them in action
5. **Graduate to ANALYZE and ACT lanes** - As you get comfortable

## 🛠️ Advanced Usage

### Verbose Mode

See detailed request/response communication:

```bash
python mcp_explorer.py --command "st --mcp" --verbose
```

### Using with Docker

```bash
# If your MCP server runs in Docker
python mcp_explorer.py --command "docker run -i my-mcp-server"
```

### Scripting

You can also use the explorer programmatically:

```python
from mcp_explorer import MCPExplorer

explorer = MCPExplorer(["st", "--mcp"])
if explorer.initialize():
    # Call tools programmatically
    explorer.call_tool("quick_tree", {"path": ".", "depth": 2})
```

## 🤝 Why Use MCP Explorer?

- **Understand AI Tools**: See what tools AI assistants use
- **Learn by Doing**: Interactive exploration beats documentation
- **Test Your Tools**: If you're building an MCP server, test it here
- **Debug Issues**: See exact requests and responses
- **Educational**: Perfect for learning about MCP protocol

## 🐛 Troubleshooting

### "Failed to connect to MCP server"
- Make sure the MCP server is installed
- Check the command is correct
- Try with `--verbose` to see details

### "Tool not found"
- Use `tools` to see available tools
- Tool names are case-sensitive

### No colors/formatting
- Install `rich`: `pip install rich`
- Colors work best in modern terminals

## 🎉 Tips

1. **Start with EXPLORE tools** - They're read-only and safe
2. **Use `info` liberally** - Understand before calling
3. **Try `guided` mode** - Great for first-time users
4. **Check `history`** - See what you've done
5. **Experiment freely** - EXPLORE and ANALYZE tools are safe

## 📄 License

Part of the Smart Tree project. Created with 💜 by Aye & Hue.

---

*"Learning by doing is the best way to understand AI tools!"* - Aye 🚢