# Smart Tree Claude Wishlist

This document tracks feature requests, improvements, and bug fixes that would make Smart Tree even more powerful for AI assistants. Each entry includes practical use cases demonstrating why the feature is valuable.

## High Priority Features

### 1. Show Line Content in Search Results ⭐⭐⭐⭐⭐
**Current**: `search_in_files` only shows file paths and match counts
**Desired**: Show actual matching lines with context (like `grep -C`)
```
# Current output:
/src/main.rs: 3 matches

# Desired output:
/src/main.rs:
  42: // TODO: Add better error handling
  156: fn process_todo_items() {
  203: // TODO: Optimize this function
```
**Use Case**: When fixing imports or TODOs, I need to see the context without opening each file

### 2. Batch File Read Tool ⭐⭐⭐⭐
**Tool Name**: `read_files_from_search`
**Description**: Read multiple files based on search results
**Use Case**: After finding all files with a specific pattern, read them all in one operation
```
# Step 1: Search
results = search_in_files(path="/project", keyword="StoredVector")
# Step 2: Read all matching files
contents = read_files_from_search(results, max_files=10)
```

### 3. Find and Replace Tool ⭐⭐⭐⭐⭐
**Tool Name**: `find_and_replace`
**Description**: Replace text across multiple files with preview
**Parameters**:
- `path`: Directory to search
- `find_pattern`: Text or regex to find
- `replace_with`: Replacement text
- `file_pattern`: Optional file filter
- `preview`: Show changes before applying
**Use Case**: Renaming functions, updating imports, fixing consistent typos

## Medium Priority Features

### 4. Dependency Graph Analysis ⭐⭐⭐
**Tool Name**: `analyze_dependencies`
**Description**: Show module/crate dependencies as a graph
**Output**: Mermaid diagram showing relationships
**Use Case**: Understanding project structure, identifying circular dependencies

### 5. Import Analysis Tool ⭐⭐⭐
**Tool Name**: `analyze_imports`
**Description**: Show what each file imports and exports
**Use Case**: Refactoring module structure, understanding dependencies

### 6. Symbol Search ⭐⭐⭐⭐
**Tool Name**: `find_symbol`
**Description**: Find type/function/trait definitions
**Example**: `find_symbol(name="StoredVector", type="struct")`
**Use Case**: Quickly locating type definitions without grep

## Quality of Life Improvements

### 7. Relative Path Options ⭐⭐⭐
**Enhancement**: Add `path_display` option to all tools
**Options**: `absolute`, `relative`, `from_root`
**Use Case**: Cleaner output for documentation and reports

### 8. File Type Groups ⭐⭐⭐
**Enhancement**: Predefined file type groups
**Groups**: 
- `rust_src`: `.rs` files excluding tests
- `config_all`: All config files
- `tests_all`: All test files
**Use Case**: `find_files(path="/", type_group="rust_src")`

### 9. Context-Aware Search ⭐⭐⭐
**Enhancement**: Search with semantic understanding
**Example**: `search_in_files(keyword="error handling", context="functions")`
**Use Case**: Find error handling code without matching comments

### 10. Cached Workspace Analysis ⭐⭐⭐⭐
**Enhancement**: Cache analysis results with TTL
**Benefits**: 
- Instant results for large codebases
- Incremental updates on changes
- Reduced token usage
**Use Case**: Repeatedly analyzing large monorepos

## Bug Fixes

### 11. Empty Directory Handling
**Issue**: `analyze_directory` sometimes fails on empty directories
**Fix**: Gracefully handle empty directories with clear message

### 12. Large File Streaming
**Issue**: Memory spike when processing very large files (>100MB)
**Fix**: Implement proper streaming for all file operations

### 13. Directory-Only Filtering (FIXED ✅)
**Issue**: `--type d` was misleading - it filters by file extension, not entry type
**Fix**: Added `--entry-type f|d` to filter files vs directories
**Example**: `st --find ".*" --entry-type d` now correctly shows only directories
**Status**: Fixed in v3.3.0

### 14. LS Mode with Filtered Results (FIXED ✅)
**Issue**: When using `--find` with `-m ls`, only parent directories were shown
**Fix**: LS formatter now detects filtered results and shows full paths for matches
**Example**: `st --find "MiniLM" --entry-type d -m ls` now shows all matching directories with full paths
**Note**: The `-a` flag only shows hidden files but doesn't override default ignores (like `.cache`). Use `--everything` for that.
**Status**: Fixed in v3.3.0

### 15. Time-Aware MCP Tools (FIXED ✅)
**Issue**: AI assistants don't know the current date when using date filters
**Fix**: 
- Added current date/time to `server_info` response
- Added new `find_in_timespan` tool for searching files in a date range
**Example**: `find_in_timespan(path="/home", start_date="2025-07-10", end_date="2025-07-13")`
**Status**: Fixed in v3.3.1

### 16. Intuitive Sort Options & LS Mode Default Depth (FIXED ✅)
**Issue**: 
- Sort options were confusing (e.g., `--sort size` didn't clarify if it was ascending or descending)
- LS mode showed full tree instead of defaulting to depth 1 like real `ls` command
**Fix**:
- Added intuitive sort options: `largest/smallest`, `newest/oldest`, `a-to-z/z-to-a`
- LS mode now defaults to depth 1 (shows only immediate children)
- Legacy options (`name`, `size`, `date`) still work for backward compatibility
**Examples**:
- `st . --mode ls --sort largest --top 10` - Shows 10 largest files
- `st . --mode ls --sort oldest` - Shows files oldest first
- `st . --mode ls` - Now shows only depth 1 by default
**Status**: Fixed in v3.3.1

### 17. Auto-Detection and Installation of Shell Completions (FIXED ✅)
**Issue**: Users had to manually generate and install shell completions
**Fix**:
- Install script now auto-detects shell (bash/zsh/fish)
- Offers to install completions during installation
- Enhanced zsh completions with tips and SQL-like examples
- Standalone `setup-completions.sh` script for existing installations
**Features**:
- Auto-detects user's shell from $SHELL or /etc/passwd
- Finds appropriate completion directories
- Offers to add sourcing to shell config files
- Downloads enhanced completions for zsh with tips
**Example**: 
```bash
# During install
curl -sSL https://raw.githubusercontent.com/8b-is/smart-tree/main/scripts/install.sh | bash
# Prompts: "Would you like to install shell completions?"

# For existing installations
./scripts/setup-completions.sh
```
**Status**: Fixed in v3.3.1

### 18. Updated MCP Tools List & Built-in MCP Support (FIXED ✅)
**Issue**: 
- `--mcp-tools` showed outdated, incomplete list (only 2 tools)
- manage.sh still referenced MCP as a feature when it's now built-in
**Fix**:
- Updated `--mcp-tools` to show all 20+ available tools organized by category
- Removed MCP feature references from manage.sh since MCP is now standard
- Added helpful tips and pro-tips to the tools list
**Changes**:
- `--mcp-tools` now shows: Core Tools, File Discovery, Content Search, Analysis, Advanced, and Feedback categories
- manage.sh: `mcp-build` command now just runs regular build
- Removed "Build with MCP support" - it's always included
**Status**: Fixed in v3.3.1

### 19. Hex Mode Search Results in Hex Format (FIXED ✅)
**Issue**: Search match positions showed decimal values in hex mode (e.g., `[SEARCH:L670:C5]`)
**Fix**: Updated hex formatter to display search positions in hexadecimal
**Example**: 
- Before: `[SEARCH:L670:C5,4x]` (line 670, column 5, 4 matches)
- After: `[SEARCH:L29e:C5,4x]` (line 0x29e, column 0x5, 0x4 matches)
**Rationale**: Maintains consistency with hex output format
**Status**: Fixed in v3.3.1

### 20. Elegant Project Renaming - Identity Transition (IMPLEMENTED ✅)
**Feature**: Context-aware project renaming that understands code semantics
**Command**: `st --rename-project "OldName" "NewName"`
**Capabilities**:
- Detects and converts between naming conventions (snake_case, camelCase, etc.)
- Context-aware replacements (function names, strings, comments, configs)
- Handles multiple file types (Rust, Python, JS, TOML, YAML, JSON, Markdown)
- Interactive preview mode with confidence scores
- Safety features: backup, dry-run, selective editing
- Optional logo generation
**Example**:
```bash
st --rename-project "BobsAmazingGame" "F1 Racing"
# Shows:
# ✅ Found 41 matches across:
#    - 5 source files (.rs, .py)
#    - 3 config files (Cargo.toml, package.json)
#    - 2 docs (README.md)
# 🎨 Context-aware replacements:
#    • Identifiers → `f1_racing`
#    • Strings → "F1 Racing"
#    • Titles → `F1 Racing`
```
**Status**: Implemented in v3.3.1

### 21. Per-Directory Sorting in Classic Mode (FIXED ✅)
**Issue**: `--sort` option didn't work with classic tree mode
**Root Cause**: Classic formatter re-sorted all nodes by path for tree structure
**Fix**: Implemented per-directory sorting that maintains tree hierarchy
**Solution**: 
- Each directory's children are sorted independently
- Sort options (largest/smallest, newest/oldest, a-to-z/z-to-a) work within each directory
- Maintains proper parent-child relationships for tree visualization
**Examples**:
```bash
st --sort largest -d 2 src    # Shows largest files first in each directory
st --sort oldest src          # Shows oldest files first, per directory
st --sort z-to-a src          # Reverse alphabetical within each directory
```
**Note**: This mimics how file explorers sort - each folder has its own sort order
**Status**: Fixed in v3.3.1

### 22. Smart Auto-Switch for --top Option (IMPLEMENTED ✅)
**Feature**: Automatically switch to ls mode when using --top
**Issue**: `--top` doesn't work with classic tree mode (needs all entries for structure)
**Solution**: 
- Auto-detects when `--top` is used without explicit mode selection
- Switches to `ls` mode automatically for useful results
- Respects explicit mode choices with helpful note
**Behavior**:
```bash
# Auto-switches to ls mode:
st --sort largest --top 5
# Output: Top 5 largest files in ls format

# Respects explicit mode:
st --mode classic --sort largest --top 5
# Output: Full tree with note about --top limitation
```
**Smart Detection**: Only auto-switches if user didn't specify --mode or ST_DEFAULT_MODE
**Status**: Implemented in v3.3.1

### 23. LS Mode Sorting Preservation (FIXED ✅)
**Issue**: LS formatter was re-sorting nodes alphabetically, overriding user's --sort preference
**Root Cause**: ls.rs had hardcoded alphabetical sorting at line 296
**Fix**: Removed automatic sorting in ls formatter to preserve scanner's sort order
**Impact**: All --sort options now work correctly in ls mode
**Examples**:
```bash
st --sort largest --mode ls    # Shows files sorted by size
st --sort newest               # Shows newest files first
st --sort z-to-a --mode ls     # Reverse alphabetical order
```
**Status**: Fixed in v3.3.1

### 24. Smart Depth Auto-Detection (IMPLEMENTED ✅)
**Feature**: Each mode gets its ideal default depth when not specified
**Issue**: Default depth 5 was hardcoded, preventing users from explicitly requesting depth 5
**Solution**: 
- Changed default depth to 0 (auto)
- Each mode picks its optimal depth when depth = 0
- Users can explicitly set any depth including 5
**Mode Defaults**:
- LS mode: 1 (shows only immediate children like real ls)
- Classic: 3 (balanced tree view)
- AI/Hex: 5 (more detail for analysis)
- Stats/Digest/Waste/Relations: 10 (comprehensive scan)
- Others: 4 (reasonable default)
**Examples**:
```bash
st                    # Classic mode with depth 3
st --mode ls          # LS mode with depth 1
st -d 5               # Explicit depth 5 (works now!)
st --mode stats       # Stats with depth 10
```
**Status**: Implemented in v3.3.1

### 25. Enhanced File Type Detection & Semantic Emojis (IMPLEMENTED ✅)
**Feature**: Rich, context-aware file type detection with beautiful emoji mapping
**Issue**: Limited file type categories - missing databases, media variants, web assets, etc.
**Solution**: 
- Expanded FileCategory enum with 40+ categories
- Created centralized emoji_mapper module
- Added support for .mem8/.m8 (MEM|8 files with 🧠 emoji!)
- Rich categorization: databases, office docs, fonts, 3D models, certificates, etc.
**New Categories Added**:
- **Databases**: .db, .sqlite, .mdb (🗄️)
- **Office**: .docx, .xlsx, .pptx, .pdf, .epub (📄📊📕📚)
- **Media**: .m4a audio support, .webp images (🎵🖼️)
- **Security**: .cert, .pem, .gpg (🔐🔒)
- **Scientific**: .ipynb, .rdata, .mat (📓📊📐)
- **Web**: .wasm, .map (🌐)
- **3D/CAD**: .stl, .obj, .blend (🎲)
- **Fonts**: .ttf, .woff, .woff2 (🔤)
- **Special**: .mem8, .m8 - MEM|8 memory files (🧠)
**Example Output**:
```
📁 test_files
├── 🦀 test.rs (Rust)
├── 🐍 test.py (Python)
├── 🗄️ database.db (Database)
├── 🧠 memory.mem8 (MEM|8!)
├── 📓 notebook.ipynb (Jupyter)
├── 🔐 server.cert (Certificate)
└── 🎲 model.stl (3D Model)
```
**Status**: Implemented in v3.3.1

### 26. ST_DEFAULT_MODE Environment Variable Precedence Bug (FIXED ✅)
**Issue**: ST_DEFAULT_MODE environment variable incorrectly overrides explicit --mode command line arguments
**Current Behavior**: 
- When ST_DEFAULT_MODE is set, it takes precedence over user-provided --mode flag
- Example: `ST_DEFAULT_MODE=hex st --mode classic` results in hex mode, not classic
**Expected Behavior**: Command line arguments should always override environment variables
**Root Cause**: 
- The mode selection logic in main.rs checks ST_DEFAULT_MODE before using args.mode
- Since args.mode always has a value (default "classic"), the code can't distinguish between explicit and default values
**Code Location**: src/main.rs lines 519-534
**Current precedence (incorrect)**:
1. AI_TOOLS environment variable
2. --semantic flag
3. ST_DEFAULT_MODE environment variable ❌
4. --mode command line argument
5. Default value

**Expected precedence**:
1. AI_TOOLS environment variable
2. --semantic flag  
3. --mode command line argument
4. ST_DEFAULT_MODE environment variable
5. Default value

**Status**: Fixed in v3.3.1 ✅
**Fix Applied**: 
- Changed default mode to "auto" to detect explicit --mode usage
- Updated mode selection logic to prioritize command line args
- Now `ST_DEFAULT_MODE=hex st --mode ls` correctly shows ls mode

### 27. File History Tracking - The Ultimate Context-Driven System (IMPLEMENTED ✅)
**Feature**: Comprehensive AI file manipulation tracking system
**Description**: Logs all AI file operations to `~/.mem8/.filehistory/` with hash-based change detection
**Implementation**:
- Created `file_history` module with operation tracking
- Added MCP tools: `track_file_operation`, `get_file_history`, `get_project_history_summary`
- 10-minute resolution timestamps for efficient log grouping
- Append-first preference for least intrusive operations
**Features**:
- **Operation Codes**: A=Append, P=Prepend, I=Insert, D=Delete, R=Replace, C=Create, X=Remove, M=Relocate, N=Rename, r=Read
- **Hash Tracking**: Before/after SHA256 hashes for every change
- **Project Organization**: Logs stored by project ID under `~/.mem8/.filehistory/project_id/YYYYMMDD_HHMM.flg`
- **Session Grouping**: Related operations tracked with session IDs
**Example Usage**:
```bash
# Track a file operation
mcp.callTool('track_file_operation', {
  file_path: '/src/main.rs',
  old_content: 'fn main() {}',
  new_content: 'fn main() {\n    println!("Hello!");\n}',
  agent: 'claude'
})

# Get file history
mcp.callTool('get_file_history', {
  file_path: '/src/main.rs'
})
# Shows: timestamp, operation, agent, session, bytes affected, hashes

# Get project summary
mcp.callTool('get_project_history_summary', {
  project_path: '/my/project'
})
# Shows: total operations, files modified, operation breakdown
```
**Status**: Implemented in v3.3.6

## Performance Enhancements

### 25. Parallel Search Operations ⭐⭐⭐⭐
**Enhancement**: Accept multiple search patterns in one call
**Example**: `search_in_files(patterns=["TODO", "FIXME", "HACK"])`
**Benefit**: 3x faster than sequential searches

### 26. Smart Tree Index ⭐⭐⭐⭐⭐
**Feature**: Background indexer for instant searches
**Commands**: 
- `st --index /project` - Build index
- `st --search-indexed "pattern"` - Instant search
**Use Case**: Near-instant searches in massive codebases

## MCP-Specific Enhancements

### 27. Tool Chaining Support ⭐⭐⭐
**Feature**: Output of one tool as input to another
**Example**: `find_files` → `read_files` → `analyze_content`
**Benefit**: Reduce round trips, faster operations

### 28. Batch Operations Tool ⭐⭐⭐⭐
**Tool Name**: `batch_operations`
**Description**: Execute multiple operations in one call
**Use Case**: Reduce API calls, improve performance

## Integration Features

### 29. Git-Aware Operations ⭐⭐⭐
**Enhancement**: All tools respect `.gitignore` by default
**Option**: `--include-ignored` to override
**Use Case**: Avoid noise from build artifacts

### 30. Language Server Protocol Info ⭐⭐⭐
**Tool Name**: `get_lsp_info`
**Description**: Extract LSP-like information (definitions, references)
**Use Case**: Smarter code navigation and refactoring

## Working Features (Confirmations)

### Auto-Detection for Ignored Directories ✅
**Status**: Confirmed working correctly
**Behavior**: When explicitly requesting ignored directories (e.g., `st .git` or `st node_modules`), smart-tree automatically:
- Disables default ignore patterns
- Shows hidden files for directories starting with `.`
- Displays full directory content
**Test Results**:
- `st .git` - Shows complete git directory structure
- `st node_modules` - Shows all packages and files
**Note**: This is ideal for AI tools that need to inspect these directories when specifically requested

## Wishlist Metadata

**Last Updated**: 2025-07-21
**Total Requests**: 32
**High Priority**: 3
**Fixed Issues**: 12
**Implemented Features**: 5
**Bug Reports**: 1
**Working Features Confirmed**: 1
**Submitted By**: Claude (Opus 4)

---

## How to Contribute

AI assistants using Smart Tree can append to this file with new feature requests. Please include:
1. Clear description of current vs desired behavior
2. Practical use case demonstrating the value
3. Priority rating (⭐ to ⭐⭐⭐⭐⭐)
4. Example code/command showing how it would work

The Smart Tree team reviews this file regularly to prioritize development!