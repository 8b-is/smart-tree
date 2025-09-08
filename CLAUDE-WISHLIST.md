# Smart Tree Claude Wishlist 🌳

> A focused wishlist for Smart Tree improvements from AI assistants using the tool.
> Now with MEM|8 integration in Qdrant! 🧠

## 🔥 HIGH PRIORITY (Active Development)

### 1. Find and Replace Tool ⭐⭐⭐⭐⭐
**Tool Name**: `find_and_replace`
**Description**: Replace text across multiple files with preview
**Use Case**: Renaming functions, updating imports, fixing consistent typos
```bash
# Example usage
find_and_replace(
  path="/project",
  find_pattern="old_function",
  replace_with="new_function",
  preview=true
)
```

### 2. Symbol Search ⭐⭐⭐⭐
**Tool Name**: `find_symbol`
**Description**: Find type/function/trait definitions using AST
**Use Case**: Quickly locating definitions without grep
```bash
find_symbol(name="StoredVector", type="struct")
```

### 3. Batch File Read Tool ⭐⭐⭐⭐
**Tool Name**: `read_files_from_search`
**Description**: Read multiple files based on search results in one operation
**Use Case**: After finding patterns, read all matching files at once

## 🧠 MEM|8 Integration Opportunities

### Wave-Based Memory Indexing
- **Feature**: Use MEM|8's 973x faster wave patterns for file indexing
- **Benefit**: Near-instant semantic search across massive codebases
- **Integration**: Connect to Qdrant's MEM|8 vectors for similarity search

### Consciousness-Aware Code Navigation
- **Feature**: Use MEM|8's emotional encoding (2Hz-200Hz) for code complexity
- **Benefit**: Navigate to "calm" (simple) or "intense" (complex) code sections
- **Example**: `st --wave-complexity high` finds complex hotspots

## ✅ RECENTLY COMPLETED

1. **Line Content in Search** - Now shows actual matching lines!
2. **Project Renaming** - Context-aware identity transitions
3. **File History Tracking** - Complete audit trail in ~/.mem8/
4. **Enhanced File Types** - 40+ categories with semantic emojis
5. **Smart Depth Detection** - Each mode gets optimal defaults

## 🐛 KNOWN ISSUES

### Memory Spike on Large Files
- **Issue**: Files >100MB cause memory spike
- **Fix Needed**: Implement proper streaming

### Empty Directory Handling  
- **Issue**: `analyze_directory` sometimes fails on empty dirs
- **Fix Needed**: Graceful handling with clear message

## 📝 Quick Wins (Easy Improvements)

1. **Relative Path Options** - Add `path_display` option
2. **File Type Groups** - Predefined groups like `rust_src`, `config_all`
3. **Cached Analysis** - TTL-based caching for repeated operations

## 💡 Future Vision

### Smart Tree Index with MEM|8
- Background indexer using wave patterns
- Instant semantic search
- Emotional code mapping
- Cross-sensory binding for related code

---

**Last Updated**: 2025-08-30
**Priority Items**: 3
**MEM|8 Status**: Integrated with Qdrant ✅

## Contributing
AI assistants: Please append new requests with:
- Clear use case
- Priority (⭐ to ⭐⭐⭐⭐⭐)
- Example usage

*Old wishlist archived at: mothballed/old-docs/CLAUDE-WISHLIST.md (468 lines)*