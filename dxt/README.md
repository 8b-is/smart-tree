# Smart Tree Installation Helper for Claude Desktop 🌳

This extension helps you install and configure Smart Tree, the world's smartest directory visualization tool!

## What This Extension Does

The Smart Tree Installation Helper is a friendly guide that:

1. **Checks Your System** - Detects if Smart Tree is already installed
2. **Provides Instructions** - Platform-specific installation steps
3. **Helps Configure** - Shows exactly how to set up Claude Desktop
4. **Verifies Success** - Confirms everything is working

## Why This Approach?

Rather than bundling the binary (which hits Electron trust issues), this helper guides you to install Smart Tree properly on your system. This gives you:

- ✅ **Full Permissions** - No sandbox restrictions
- ✅ **Better Performance** - Native system execution
- ✅ **Terminal Access** - Use `st` anywhere, not just Claude
- ✅ **Easy Updates** - Update Smart Tree independently

## Quick Start

### 1. Install This Extension
- Download `smart-tree.dxt` from [releases](https://github.com/8b-is/smart-tree/releases)
- Open Claude Desktop → Settings → Developer
- Click "Install from file" and select the DXT

### 2. Use the Helper
After installation, ask Claude:
```
Please help me install Smart Tree on my system
```

The helper will:
- Check if Smart Tree is already installed
- Provide the right installation command for your OS
- Show you the exact configuration to add

### 3. Install Smart Tree

**macOS/Linux/WSL:**
```bash
curl -sSL https://raw.githubusercontent.com/8b-is/smart-tree/main/scripts/install.sh | bash
```

**Windows:**
- Download from [GitHub Releases](https://github.com/8b-is/smart-tree/releases/latest)
- Extract and add to PATH

### 4. Configure Claude Desktop
The helper will show you the exact configuration. It looks like:
```json
{
  "smart-tree": {
    "command": "/usr/local/bin/st",
    "args": ["--mcp"],
    "env": {
      "AI_TOOLS": "1"
    }
  }
}
```

### 5. Restart Claude Desktop
And you're done! 🎉

## After Installation

Once Smart Tree is properly installed, you'll have access to 20+ powerful MCP tools:

- **`quick_tree`** - Lightning-fast 3-level directory overview
- **`analyze_directory`** - Comprehensive analysis with AI optimization
- **`search_in_files`** - Content search across your codebase
- **`semantic_analysis`** - Group files by conceptual similarity
- **`project_overview`** - Understand any codebase instantly
- And many more!

## Features You'll Love

### 🚀 Performance
- 10-24x faster than traditional tree commands
- 99% compression with quantum modes
- Constant memory usage for huge directories

### 💰 Cost Savings
- Reduce AI token costs by 98%
- $1,270 → ~$10 for large directory analysis
- Optimized formats for AI consumption

### 🎨 Beautiful Output
- Classic tree with emojis
- Mermaid diagrams
- Multiple formats for every use case

## Troubleshooting

**"Smart Tree not found"**
- Make sure you ran the installation command
- On Windows, ensure it's in your PATH
- Try restarting your terminal

**"Permission denied"**
- The installer needs write access to `/usr/local/bin`
- On macOS/Linux, the script will use `sudo` if needed

**Still having issues?**
- Check [GitHub Issues](https://github.com/8b-is/smart-tree/issues)
- Join our [Discord](https://discord.gg/uayQFhWC)

## How the Helper Works

This extension provides a minimal MCP server that:
1. Responds to installation check requests
2. Detects your operating system
3. Checks common installation paths
4. Provides platform-specific guidance
5. Generates the correct configuration

It's designed to be helpful without being intrusive!

## Building from Source

```bash
cd smart-tree/dxt
./build-dxt.sh
```

## Support

- **GitHub**: https://github.com/8b-is/smart-tree
- **Discord**: https://discord.gg/uayQFhWC
- **Issues**: https://github.com/8b-is/smart-tree/issues

---

Built with 💙 by the Smart Tree Team
*Making directories beautiful, one tree at a time!*