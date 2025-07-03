# Smart Tree Interactive Mode Demo 🌳

The interactive mode provides a rich Terminal User Interface (TUI) for exploring directories with various visualization modes and filters.

## Launch Interactive Mode

```bash
st --interactive
# or
st -i
# or with a specific directory
st -i /path/to/directory
```

## Features

### 1. **Main Menu Options**
- 📁 **Show directory tree** - Display the current view
- 👁️ **Change view mode** - Switch between different visualizations
- 🔍 **Filter files** - Filter by file extensions
- 🔎 **Search in file contents** - Search for keywords within files
- 🔗 **Analyze code relationships** - View imports, dependencies, etc.
- 💾 **Export for AI/documentation** - Export in various formats
- 📂 **Change directory** - Navigate to a different path
- 👋 **Quit** - Exit interactive mode

### 2. **View Modes**
- **Classic tree view** - Traditional tree structure with emojis
- **Summary overview** - Quick project summary (default)
- **Semantic grouping** - Files grouped by purpose
- **Code relationships** - Import/dependency analysis
- **Mermaid diagram** - Visual flowchart
- **Markdown report** - Complete documentation

### 3. **Export Formats**
- **AI-optimized format** - Compressed for LLMs
- **Quantum compressed** - 90%+ compression
- **Quantum semantic** - With AST analysis
- **JSON format** - Structured data
- **CSV spreadsheet** - For data analysis

## Example Session

```
════════════════════════════════════════════════════════════════════════════════
Smart Tree Interactive Mode 🌳
════════════════════════════════════════════════════════════════════════════════

Current State:
  📍 Path: /home/user/my-project
  👁️  View: Summary
  🔍 Filters: rs, toml
  🔎 Search: TODO
  📊 Depth: 5
  👻 Hidden: hidden

  🚀 Detected: Rust project

What would you like to do?
> 📁 Show directory tree
  👁️  Change view mode
  🔍 Filter files
  🔎 Search in file contents
  🔗 Analyze code relationships
  💾 Export for AI/documentation
  📂 Change directory
  👋 Quit
```

## Interactive Features

### Content Detection
The interactive mode automatically detects the type of directory:
- 🚀 **Code projects** (Rust, Python, Node.js, etc.)
- 🎬 **Media libraries** (videos, audio files)
- 📚 **Document archives** (PDFs, docs, etc.)
- 📷 **Photo collections**
- 🔬 **Data science workspaces**
- 📦 **Mixed content**

### Smart Filtering
Use the filter option to show only specific file types:
- Select from common extensions (rs, py, js, ts, etc.)
- Multiple selections allowed
- Filters persist across views

### Content Search
Search for keywords within file contents:
- Highlights matches in the tree view
- Works with filters for targeted search
- Perfect for finding TODOs, FIXMEs, or specific functions

### Export Options
Export your analysis in various formats:
- Automatic compression for AI formats
- Custom filenames
- Size-optimized outputs

## Keyboard Navigation

- **↑/↓** - Navigate menu options
- **Space** - Select/deselect in multi-select
- **Enter** - Confirm selection
- **Esc** - Cancel current operation
- **q** - Quit from main menu

## Tips

1. **Start with Summary view** to understand the project structure
2. **Use filters** to focus on specific file types
3. **Export to AI format** for sharing with LLMs
4. **Semantic view** helps understand project organization
5. **Relations view** is perfect for understanding code dependencies

## Future Enhancements

- Real-time file watching
- Diff view between directories
- Git integration
- Custom themes
- Export to cloud services

The interactive mode transforms Smart Tree from a simple directory visualizer into a powerful project exploration tool! 🎸✨