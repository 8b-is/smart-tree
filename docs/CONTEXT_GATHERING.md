# 🔍 Context Gathering System with Temporal Analysis

Smart Tree can now search across AI tool directories to gather project-related context with powerful TEMPORAL ANALYSIS! This feature helps you understand how you've been working with a project across different AI assistants over time, revealing patterns, momentum, and work sessions.

## Overview

The Context Gathering system searches directories like:
- `~/.claude` - Claude Desktop conversations and settings
- `~/.cursor` - Cursor IDE context and workspaces  
- `~/.windsurf` - Windsurf editor data
- `~/.continue` - Continue.dev sessions
- `~/.github/copilot` - GitHub Copilot data
- `~/.vscode` - VS Code settings and history
- `~/.idea` - IntelliJ IDEA project data
- `~/.zed` - Zed editor context

It finds relevant files (JSON, JSONL, XML, YAML, etc.) and extracts project-specific context, then converts it to efficient M8 wave-based format.

## 🚀 Quick Start

### Using the MCP Tool

```bash
# From Claude Desktop or any MCP client:
mcp.callTool('gather_project_context', {
  project_path: '/path/to/your/project',
  output_format: 'summary'  // or 'json' or 'm8'
})
```

### Example Response

```json
{
  "project_path": "/home/user/projects/smart-tree",
  "total_contexts_found": 47,
  "contexts_returned": 10,
  "sources_summary": {
    ".claude": 15,
    ".cursor": 8,
    ".vscode": 20,
    ".windsurf": 4
  },
  "contexts": [
    {
      "source_path": "/home/user/.claude/chats/smart-tree-discussion.json",
      "ai_tool": ".claude",
      "content_type": "ChatHistory",
      "relevance_score": 0.95,
      "size_bytes": 45632,
      "preview": "Discussion about implementing MEM8 architecture...",
      "metadata": {
        "modified": "2025-01-15T10:30:00Z",
        "size": "45632"
      }
    }
  ]
}
```

## 🛠️ Configuration Options

### Search Directories
```javascript
// Search specific tools only
{
  project_path: '/my/project',
  search_dirs: ['.claude', '.cursor']  // Only search these
}
```

### Custom Directories
```javascript
// Add your own directories
{
  project_path: '/my/project',
  custom_dirs: ['/home/user/my-notes', '/home/user/ai-sessions']
}
```

### Project Identifiers
```javascript
// Help identify your project with unique strings
{
  project_path: '/my/project',
  project_identifiers: [
    'MyProjectName',
    'github.com/user/repo',
    'unique-api-key-prefix'
  ]
}
```

### Output Formats

1. **Summary** (default) - Human-readable overview
2. **JSON** - Full structured data
3. **M8** - Wave-based compressed format (base64 encoded)
4. **Temporal** - Time-based analysis with patterns and momentum

## 🔐 Privacy Features

### Privacy Mode (Default: ON)
- Automatically redacts sensitive information:
  - API keys
  - Tokens
  - Passwords
  - Secrets

```javascript
{
  project_path: '/my/project',
  privacy_mode: true  // Default
}
```

### Permission System
- Requires explicit permission to search home directory
- Respects Smart Tree's allowed paths configuration
- User can deny access to specific directories

## 📊 Context Types

The system recognizes various context types:

- **ChatHistory** - AI conversation logs
- **ProjectSettings** - IDE/editor project configurations
- **CodeSnippets** - Saved code fragments
- **Documentation** - Project-related docs
- **Configuration** - Tool settings
- **SearchHistory** - Past searches
- **Bookmarks** - Saved locations
- **CustomPrompts** - User-defined prompts
- **ModelPreferences** - AI model settings
- **WorkspaceState** - Editor workspace data

## 🎯 Relevance Scoring

Contexts are scored based on:
1. **Content Type** (0.5 - 0.9)
   - Chat histories: 0.8
   - Project settings: 0.9
   - Code snippets: 0.7
2. **Recency** (0.0 - 0.3)
   - < 7 days: +0.3
   - < 30 days: +0.2
   - < 90 days: +0.1
3. **Project Mentions** (0.0 - 0.5)
   - Each mention: +0.1 (max 0.5)

## ⏰ Temporal Analysis Features

### Work Sessions
Automatically detects work sessions by clustering activity:
```javascript
{
  output_format: 'temporal',
  temporal_resolution: 'hour'  // Detects sessions with 4-hour gaps
}
```

### Activity Timeline
Shows activity intensity over time with:
- **Peak Times**: When you're most active
- **Momentum**: Is your engagement increasing/decreasing?
- **Periodic Patterns**: Daily/weekly work rhythms

### Temporal Decay
Apply time-based relevance decay:
```javascript
{
  temporal_decay_days: 30  // 30-day half-life
}
```
Recent contexts stay relevant while old ones fade naturally.

### Temporal Wave Grids
Creates wave representations showing:
- **Interference patterns** between different tools
- **Resonance peaks** where multiple contexts align
- **Temporal navigation** through project history

### Example Temporal Response
```json
{
  "temporal_analysis": {
    "resolution": "Day",
    "work_sessions": 15,
    "peak_times": [
      "2025-01-10 14:00",
      "2025-01-08 10:00"
    ],
    "momentum": 0.75,  // Positive = increasing activity
    "active_days": 23,
    "periodic_patterns": [
      {
        "period_type": "weekly",
        "peak_periods": ["Tuesday", "Thursday", "Friday"]
      }
    ]
  }
}
```

## 💾 M8 Format Output

The M8 format creates a wave-based representation:
- **Wave Grid**: 16x16 grid mapping tools × content types
- **Frequency**: Based on relevance score
- **Metadata**: Project info, timestamps, summaries
- **Compressed**: Efficient binary format

## 🔧 Advanced Usage

### Analyze Tool Usage
```javascript
mcp.callTool('analyze_ai_tool_usage', {
  days: 30,
  tool_name: '.claude'  // Optional: analyze specific tool
})
```

### Clean Old Context (Coming Soon)
```javascript
mcp.callTool('clean_old_context', {
  days_to_keep: 90,
  dry_run: true,  // See what would be deleted
  tools: ['.claude', '.cursor']
})
```

## 🎸 Pro Tips

1. **Start Broad**: First run without filters to see what's available
2. **Use Identifiers**: Add unique project strings for better matching
3. **Check Permissions**: Use `verify_permissions` first if unsure
4. **M8 Format**: Use for feeding context to MEM8-aware systems
5. **Privacy First**: Always enabled by default for safety

## 🚀 Example Workflow

```bash
# 1. Check permissions
mcp.callTool('verify_permissions', {
  path: '~/.claude'
})

# 2. Analyze usage patterns
mcp.callTool('analyze_ai_tool_usage', {
  days: 7
})

# 3. Gather project context
mcp.callTool('gather_project_context', {
  project_path: '/my/project',
  min_relevance: 0.7,
  output_format: 'summary'
})

# 4. Get M8 format for processing
mcp.callTool('gather_project_context', {
  project_path: '/my/project',
  output_format: 'm8'
})
```

## 🎵 The Cheet Says

"Why search through a dozen tools when Smart Tree can gather it all? It's like having a roadie who knows where you left all your guitar picks across different venues. Rock on with unified context!" 🎸

---

*Context Gathering: Because your project's story is scattered across many tools, and Smart Tree brings it all together.*