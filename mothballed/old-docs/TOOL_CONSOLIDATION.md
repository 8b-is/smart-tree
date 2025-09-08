# Smart Tree MCP Tool Consolidation Guide

## Overview
We've consolidated Smart Tree's 50+ MCP tools down to ~15 consolidated tools to address Cursor's complaint about having too many tools. Each consolidated tool now uses a `mode`, `type`, or `operation` parameter to specify the specific action.

## Migration Guide

### 1. Find Tools → `find` tool
All find operations are now consolidated into a single `find` tool with a `type` parameter:

| Old Tool | New Usage |
|----------|-----------|
| `find_files` | `find` with `type: "files"` |
| `find_code_files` | `find` with `type: "code"` |
| `find_config_files` | `find` with `type: "config"` |
| `find_documentation` | `find` with `type: "documentation"` |
| `find_tests` | `find` with `type: "tests"` |
| `find_build_files` | `find` with `type: "build"` |
| `find_large_files` | `find` with `type: "large"` |
| `find_recent_changes` | `find` with `type: "recent"` |
| `find_in_timespan` | `find` with `type: "timespan"` |
| `find_duplicates` | `find` with `type: "duplicates"` |
| `find_empty_directories` | `find` with `type: "empty_dirs"` |

**Example:**
```javascript
// Old way
mcp.callTool('find_code_files', { path: '/src', languages: ['rust'] })

// New way
mcp.callTool('find', { type: 'code', path: '/src', languages: ['rust'] })
```

### 2. Analysis Tools → `analyze` tool
All analysis operations consolidated with a `mode` parameter:

| Old Tool | New Usage |
|----------|-----------|
| `analyze_directory` | `analyze` with `mode: "directory"` |
| `analyze_workspace` | `analyze` with `mode: "workspace"` |
| `get_statistics` | `analyze` with `mode: "statistics"` |
| `get_git_status` | `analyze` with `mode: "git_status"` |
| `get_digest` | `analyze` with `mode: "digest"` |
| `semantic_analysis` | `analyze` with `mode: "semantic"` |
| `directory_size_breakdown` | `analyze` with `mode: "size_breakdown"` |
| `analyze_ai_tool_usage` | `analyze` with `mode: "ai_tools"` |

**Example:**
```javascript
// Old way
mcp.callTool('get_statistics', { path: '/project', show_hidden: true })

// New way
mcp.callTool('analyze', { mode: 'statistics', path: '/project', show_hidden: true })
```

### 3. Overview Tools → `overview` tool
Quick tree and project overview consolidated:

| Old Tool | New Usage |
|----------|-----------|
| `quick_tree` | `overview` with `mode: "quick"` (default) |
| `project_overview` | `overview` with `mode: "project"` |

### 4. Smart Edit Tools → `edit` tool
All Smart Edit operations with an `operation` parameter:

| Old Tool | New Usage |
|----------|-----------|
| `smart_edit` | `edit` with `operation: "smart_edit"` |
| `get_function_tree` | `edit` with `operation: "get_functions"` |
| `insert_function` | `edit` with `operation: "insert_function"` |
| `remove_function` | `edit` with `operation: "remove_function"` |

### 5. History/Tracking Tools → `history` tool
File tracking operations consolidated:

| Old Tool | New Usage |
|----------|-----------|
| `track_file_operation` | `history` with `operation: "track"` |
| `get_file_history` | `history` with `operation: "get_file"` |
| `get_project_history_summary` | `history` with `operation: "get_project"` |

### 6. Context Tools → `context` tool
Project context and collaboration tools:

| Old Tool | New Usage |
|----------|-----------|
| `gather_project_context` | `context` with `operation: "gather_project"` |
| `get_collaboration_rapport` | `context` with `operation: "collaboration_rapport"` |
| `get_co_engagement_heatmap` | `context` with `operation: "engagement_heatmap"` |
| `get_cross_domain_patterns` | `context` with `operation: "cross_domain_patterns"` |
| `suggest_cross_session_insights` | `context` with `operation: "suggest_insights"` |

### 7. Memory Tools → `memory` tool
Collaborative memory operations:

| Old Tool | New Usage |
|----------|-----------|
| `anchor_collaborative_memory` | `memory` with `operation: "anchor"` |
| `find_collaborative_memories` | `memory` with `operation: "find"` |

### 8. Feedback Tools → `feedback` tool
Feedback system operations:

| Old Tool | New Usage |
|----------|-----------|
| `submit_feedback` | `feedback` with `operation: "submit"` |
| `request_tool` | `feedback` with `operation: "request_tool"` |
| `check_for_updates` | `feedback` with `operation: "check_updates"` |

### 9. Standalone Tools (unchanged)
These tools remain as standalone due to their unique nature:

- `search` (was `search_in_files`) - Core search functionality
- `compare` (was `compare_directories`) - Directory comparison
- `sse` (was `watch_directory_sse`) - Real-time monitoring
- `server_info` - Server information
- `verify_permissions` - Permission checking

## Benefits of Consolidation

1. **Reduced Tool Count**: From 50+ tools to ~15 tools
2. **Better Organization**: Related operations grouped logically
3. **Easier Discovery**: Fewer tools to remember
4. **Consistent Interface**: Similar operations use similar parameters
5. **Cursor Compatibility**: Works within Cursor's tool limits

## Implementation Notes

The consolidated tools maintain backward compatibility by:
- Preserving all original functionality
- Using the same internal implementation functions
- Supporting all original parameters
- Providing clear parameter enums for operation types

## Testing the Consolidated Tools

```bash
# Build with consolidated tools
cargo build --release

# Test a consolidated tool
echo '{"type": "code", "path": ".", "languages": ["rust"]}' | st --mcp-tool find

# List all consolidated tools
st --mcp-tools-consolidated
```

## Future Considerations

1. We could further consolidate if needed:
   - Merge `search` into `find` with `type: "content"`
   - Merge `compare` into `analyze` with `mode: "compare"`
   - Merge `server_info` and `verify_permissions` into a `system` tool

2. Consider creating tool aliases for commonly used combinations

3. Add a compatibility layer that translates old tool calls to new format

## For AI Assistants

When using Smart Tree MCP tools, prefer the consolidated versions:

```javascript
// Instead of many specific tools, use consolidated ones:
const tools = {
  find: { type: 'files|code|config|tests|...' },
  analyze: { mode: 'directory|workspace|statistics|...' },
  edit: { operation: 'smart_edit|get_functions|...' },
  search: { keyword: '...', path: '...' },
  overview: { mode: 'quick|project' },
  history: { operation: 'track|get_file|get_project' },
  context: { operation: 'gather_project|rapport|...' },
  memory: { operation: 'anchor|find' },
  feedback: { operation: 'submit|request_tool|check_updates' }
};
```

This consolidation makes Smart Tree more accessible while maintaining all its powerful features! 🌳✨