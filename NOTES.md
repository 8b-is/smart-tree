# NOTES.md - Latest Development Updates

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
- Example: `stree --search "TODO" --type rs` finds all TODOs in Rust files
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

### Performance Optimizations
- Investigating SIMD optimizations for pattern matching
- Looking at memory pool implementation for large directories
- Profiling streaming mode for further improvements

### Documentation
- Need to update manage.sh script with new commands
- Consider adding example MCP usage documentation
- Video tutorial for MCP setup might be helpful

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