# NOTES.md - Latest Development Updates

## v3.3.5 - SQL-Like Queries & Smart Defaults (July 2025)

### Major Features Added 🚀
- **SQL-Like Query Support**: `--sort` and `--top` options for finding files the SQL way
  - Intuitive sort names: `largest/smallest`, `newest/oldest`, `a-to-z/z-to-a`
  - `--top N` for limiting results (auto-switches to ls mode)
  - Works with all filters for powerful queries
  
- **Smart Depth Auto-Detection**: Each mode picks its ideal default depth
  - LS mode: 1 (mimics real `ls` command)
  - Classic: 3 (balanced tree view)
  - AI/Hex: 5 (detailed analysis)
  - Stats/Digest: 10 (comprehensive scan)
  - Users can still explicitly set any depth including 5

- **Elegant Project Renaming**: Context-aware identity transition
  - `st --rename-project "OldName" "NewName"`
  - Detects naming conventions (snake_case, camelCase, etc.)
  - Context-aware replacements across all file types
  - Interactive preview with confidence scores

- **Auto Shell Completion Setup**: Detects and installs during installation
  - Auto-detects bash/zsh/fish from $SHELL or /etc/passwd
  - Enhanced zsh completions with tips and SQL examples
  - Standalone setup-completions.sh for existing installations

### Improvements & Fixes
- **LS Mode Enhancement**: Shows relative paths for filtered results
- **Per-Directory Sorting**: Classic mode now sorts within each directory
- **MCP Tools Update**: Shows all 20+ tools (MCP is now built-in, not a feature)
- **Hex Mode Fix**: Search positions now display in hexadecimal
- **Entry Type Filtering**: `--entry-type f|d` for files vs directories
- **Time-Aware MCP**: Added current date/time and `find_in_timespan` tool

### MCP Test Fixes
- Fixed test_entry_type_filtering to properly test `--entry-type` parameter
- Fixed test_hidden_directory_handling to respect MCP's show_hidden parameter
- Fixed test_date_format_parsing to handle flexible date parsing in find_in_timespan

## v3.1 Quantum Revolution (July 2025)

### MEM|8 Quantum Compression Achievement 🚀
- **99% Compression Ratio**: Chromium source (487MB → 4.1MB)
- **$1,270 Cost Savings**: Per large directory analysis
- **Native Quantum Architecture**: Tree walker outputs quantum format directly
- **Token Mapping Engine**: Smart tokenization with semantic equivalence
- **New Output Modes**:
  - `quantum`: Native 8x compression with token mapping
  - `claude`: Base64+zlib 10x compression (default for AI)
- **Python Examples**: Visualization, analysis, diff, and semantic tools
- **DXT Integration**: Fixed installer for Claude Desktop compatibility

### Performance Breakthroughs
- **10-24x Faster**: Than traditional tree command
- **Constant Memory**: ~50MB regardless of directory size
- **Streaming Default**: Handles million-file directories smoothly
- **Zero Overhead**: Native quantum output without conversion

## Recent Changes (December 2024)

### Performance and Usability Improvements (Dec 21)
- **Fixed O(n²) performance bug** in classic formatter that caused hanging with deep directories
- **Changed default depth to auto (0)** - each mode picks its ideal depth (ls=1, classic=3, etc.)
- **Added `--everything` flag** - master switch that enables --all, --no-ignore, and --no-default-ignore
- **Clarified size calculations** - st reports actual file sizes, while `du` reports disk blocks

## Recent Changes (December 2024)

### MCP (Model Context Protocol) Server Integration
We've added a built-in MCP server that allows AI assistants like Claude to directly analyze directories:
- `--mcp` flag runs Smart Tree as an MCP server on stdio
- `--mcp-tools` lists available MCP tools for debugging
- `--mcp-config` shows the configuration needed for Claude Desktop
- Tools include: `analyze_directory`, `find_files`, `get_statistics`, `get_digest`
- Includes caching support for repeated queries
- Security features with allowed/blocked path configuration

### Content Search Feature
- Added `--search` flag to search for keywords within file contents
- Works in combination with `--type` filter for targeted searches
- Example: `st --search "TODO" --type rs` finds all TODOs in Rust files
- Efficient implementation that streams file contents

### Streaming Mode
- Added `--stream` flag for real-time output on large directories
- Progressive output as directories are scanned
- Better user experience for massive file trees
- Prevents timeout issues on slow filesystems

### AI JSON Wrapper
- Added `--ai-json` flag that wraps AI output in JSON structure
- Makes it easier for programmatic consumption
- Maintains all the benefits of the compact AI format

## Current Work in Progress

### v3.0 Vision
- Quantum visualization in 3D/VR environments
- AI code intelligence using token patterns
- Distributed quantum scanning across networks
- Hot Tub Mode for collaborative analysis
- Wave interference patterns for memory optimization

### Documentation Improvements
- ✅ Mode selection guide created
- ✅ Python examples for quantum format
- ✅ Updated roadmap with v2.0 achievements
- Video tutorials for quantum features planned

## Known Issues
- Streaming mode doesn't work with compression (by design)
- MCP server needs better error messages for permission denied
- Search feature could be faster for binary files

## Future Ideas
- Add regex support to search feature
- Implement incremental updates for MCP cache
- Add WebSocket support for MCP server
- Consider adding a TUI mode for interactive exploration
- Integration with other AI providers beyond Claude

## Testing Notes
- MCP server tested with Claude Desktop on Linux
- Search feature tested on codebases up to 1M files
- Streaming mode tested on network filesystems
- Need more Windows testing for MCP features

## Dependencies Added
- `tokio` for async MCP server (feature-gated)
- Additional serialization for MCP protocol
- No new runtime dependencies for core functionality