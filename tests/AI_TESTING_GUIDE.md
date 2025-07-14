# AI Assistant Testing Guide for Smart Tree v3.3.1

This guide helps AI assistants verify that Smart Tree MCP tools work correctly.

## Quick Test Commands

### 1. Check Current Time Awareness
```
Call: server_info()
Verify: Response includes current_time with local and UTC times
```

### 2. Test Entry Type Filtering
```
Call: find_files(path="/home", pattern=".*", entry_type="d")
Verify: Only directories are returned, no files
```

### 3. Test Date Range Search
```
Call: find_in_timespan(path="/home", start_date="2025-07-10", end_date="2025-07-13")
Verify: Only files modified between these dates are shown
```

### 4. Test Hidden Directory Behavior
```
Call: analyze_directory(path="/home", show_hidden=false)
Verify: No .hidden directories or their contents appear

Call: analyze_directory(path="/home", show_hidden=true) 
Verify: .hidden directories are shown
```

## Common Issues to Watch For

1. **Date Format Errors**
   - Always use YYYY-MM-DD format
   - Check server_info for current date reference

2. **Hidden Directory Confusion**
   - Without show_hidden, NO hidden content should appear
   - No depth jumps (0→2) without showing level 1

3. **Entry Type Confusion**
   - "d" = directories only
   - "f" = files only
   - Not specifying = both files and directories

## Recommended Test Flow

1. Start with `server_info()` to see current date/time
2. Use `quick_tree()` for initial directory overview
3. Test `find_in_timespan()` with dates around today
4. Verify `find_files()` with entry_type filtering
5. Check hidden directory handling consistency

## Expected Behaviors

### Hidden Directories
- Default: Hidden directories and ALL their contents are excluded
- With `-a` or `show_hidden=true`: Hidden directories shown but marked
- No partial traversal into hidden directories

### Date Filtering
- `newer_than`: Files modified AFTER this date (exclusive)
- `older_than`: Files modified BEFORE this date (exclusive)
- `find_in_timespan`: Combines both for a date range

### Entry Type Filtering
- Filters are applied BEFORE tree construction
- Parent directories are always shown for context
- Use with `--find` patterns for powerful filtering

## Performance Tips

1. Always use `quick_tree` first (3-level, compressed)
2. Use `summary-ai` mode for large directories
3. Enable compression for MCP responses
4. Cache is automatic - repeated calls are instant