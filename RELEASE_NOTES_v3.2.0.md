# Smart Tree v3.2.0 - "Less is More!" 🎸

## 🎉 What's Changed

### 🔥 Major Changes
- **Removed Interactive Mode** - Elvis has left the building! We've removed the TUI interactive mode to keep things lean and focused.
- **Classic is Back as Default!** - Running `st` now gives you the beautiful classic tree format by default! 🌳
- **Enhanced MCP Integration** - Comprehensive Model Context Protocol support with 25+ specialized tools

### 📦 Dependencies  
- Removed `inquire` dependency (no longer needed without interactive mode)
- Streamlined dependency tree for faster builds and smaller binary size

### 🚀 Improvements
- **Cleaner, more focused codebase** - Removed complexity while maintaining power
- **Faster build times** - Fewer dependencies mean quicker compilation
- **Smaller binary size** - Optimized for efficiency
- **Enhanced MCP Tool Descriptions** - Made tools irresistible to AIs with clear, enticing descriptions! 🤖
- **Comprehensive MCP Tool Suite** - 25+ specialized tools for directory analysis, file search, and project understanding
- **Smart Installer Enhancements** - Now detects if releases have binaries and offers alternatives
- **Improved Waste Detection** - Marie Kondo mode for finding duplicates and optimizing disk usage
- **Advanced Semantic Analysis** - Wave-based file grouping inspired by Omni's wisdom

### 🤖 MCP Enhancements
- **25+ Specialized Tools** including:
  - `quick_tree` - Lightning-fast 3-level overview (START HERE!)
  - `project_overview` - Comprehensive project analysis with auto-detection
  - `find_code_files` - Multi-language source code discovery
  - `semantic_analysis` - Advanced file grouping by conceptual similarity
  - `analyze_workspace` - Complete development environment analysis
  - `submit_feedback` - Direct feedback channel to developers
  - `check_for_updates` - Automatic update detection and notifications
- **Intelligent Compression** - Automatic mode selection for optimal AI token usage
- **Enhanced Security** - Path validation and access controls for safe operation
- **Built-in Caching** - Instant repeated queries with intelligent cache management
- **Token Optimization** - 10x compression with summary-ai mode for large codebases

### 🌊 Universal Input Processing
- **Multi-format Support** - Handle filesystem, QCP queries, SSE streams, and MEM8 streams
- **Intelligent Detection** - Automatic input type recognition and processing
- **Seamless Integration** - Unified interface regardless of input source

### 🎨 Output Format Enhancements
- **Waste Detection Mode** - Find duplicates, build artifacts, and optimization opportunities
- **Enhanced Mermaid Support** - Multiple diagram styles including treemap visualization
- **Improved Markdown Reports** - Comprehensive project documentation generation
- **Quantum Semantic Compression** - Ultimate compression with semantic understanding

### 🔧 Technical Improvements
- **Streaming Support** - Real-time output for large directory scans
- **Advanced Filtering** - Regex patterns, size filters, date ranges, and content search
- **Filesystem Awareness** - Display filesystem types and mount point information
- **Performance Optimization** - 10-24x faster than traditional tree commands

## 🛠️ Installation

### Using the install script:
```bash
curl -sSL https://raw.githubusercontent.com/8b-is/smart-tree/main/scripts/install.sh | bash
```

### Manual download:
Download the appropriate binary for your platform from the assets below.

### MCP Integration:
```bash
# For Claude Code
claude mcp add st /usr/local/bin/st -- --mcp

# For other MCP clients, add to configuration:
{
  "mcpServers": {
    "smart-tree": {
      "command": "/usr/local/bin/st",
      "args": ["--mcp"],
      "env": {"AI_TOOLS": "1"}
    }
  }
}
```

## 💡 Philosophy
Sometimes less is more! By removing the interactive mode, we've made Smart Tree more focused on what it does best - providing lightning-fast, beautiful directory visualizations that work perfectly with AI tools and human users alike.

## 🎸 A Message from Elvis
"Thank you, thank you very much! The classic tree is back, baby! And those AI tools? They're all shook up with excitement!" 

## 🌟 Key Features for AI Integration

### 🚀 Recommended Workflow for AI Assistants:
1. **Start with `quick_tree`** - Get instant 3-level overview with 10x compression
2. **Use `project_overview`** - Understand project type, dependencies, and structure  
3. **Dive deeper with specialized tools**:
   - `find_code_files` for source code discovery
   - `semantic_analysis` for conceptual understanding
   - `search_in_files` for finding specific implementations
   - `analyze_workspace` for complex multi-language projects

### 💰 Token Efficiency:
- **99% compression** with quantum modes
- **10x reduction** with summary-ai format
- **Intelligent caching** for instant repeated queries
- **Automatic optimization** when AI_TOOLS environment variable is detected

## 🔍 What's New in Detail

### Removed Features:
- ❌ Interactive TUI mode (simplified for better focus)
- ❌ `inquire` dependency (cleaner build)

### Enhanced Features:
- ✅ Classic mode as default (beautiful trees by default)
- ✅ Comprehensive MCP tool suite (25+ specialized tools)
- ✅ Advanced semantic analysis (Omni-inspired wave grouping)
- ✅ Waste detection and optimization (Marie Kondo mode)
- ✅ Universal input processing (multiple data sources)
- ✅ Enhanced compression and token optimization

### New Capabilities:
- 🆕 Direct feedback submission to developers
- 🆕 Automatic update checking and notifications  
- 🆕 Advanced workspace analysis for complex projects
- 🆕 Multi-language code file discovery
- 🆕 Semantic file grouping and analysis
- 🆕 Comprehensive duplicate detection and cleanup suggestions

---

**Built with 💙 by the Smart Tree Team**

*Aye, Hue, Trish, and Omni approve this message!* ✨

---

*Remember: Always start with `quick_tree` - it's the king of directory exploration! 👑*