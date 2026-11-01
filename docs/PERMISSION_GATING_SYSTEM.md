# Permission-Based Tool Gating System 🔐

## Overview

Smart Tree MCP now implements an intelligent permission-based tool gating system that saves context and improves security by only exposing tools that can actually be used.

## How It Works

1. **First Step Required**: Always call `verify_permissions` on a path before using other tools
2. **Smart Tool Filtering**: Only shows tools that make sense based on the path's permissions
3. **Context Saving**: Reduces token usage by not showing irrelevant tools
4. **Clear Feedback**: Explains why certain tools are unavailable

## Example Workflow

```yaml
# Step 1: Verify permissions (always do this first!)
- tool: verify_permissions
  args:
    path: "/path/to/analyze"

# Response shows:
# ✅ Available tools (based on permissions)
# ❌ Unavailable tools (with reasons why)

# Step 2: Use only the available tools
- tool: analyze_directory  # Only if path is readable directory
  args:
    path: "/path/to/analyze"
    mode: "classic"
```

## Permission States

### Read-Only Directory
- ✅ Available: `analyze_directory`, `find_files`, `search_in_files`, `get_statistics`
- ❌ Unavailable: `smart_edit` (no write permission)

### Read-Write File
- ✅ Available: `get_function_tree`, `smart_edit`, `insert_function`, `remove_function`
- ❌ Unavailable: `analyze_directory` (not a directory)

### Non-Existent Path
- ✅ Available: `get_digest`, `server_info` (always available)
- ❌ Unavailable: All file/directory operations

## Benefits

1. **Efficiency**: "Why bring the whole toolbox if you can't use half the tools?" - Hue
2. **Security**: Prevents accidental operations on read-only or system files
3. **Clarity**: Clear explanations of why tools are unavailable
4. **Token Savings**: Reduces context by hiding irrelevant operations

## Implementation Details

The system uses a permission cache that:
- Caches permissions for 5 minutes
- Checks file/directory existence
- Verifies read/write permissions
- Determines file vs directory type

## Trisha's Take

"It's like checking if you have the keys before bringing the whole toolbox! No point hauling around a wrench set if you can't even open the hood. This is the kind of efficiency that makes my accounting heart sing!" 🔑

## Future Enhancements

- [ ] Group permissions (check entire directory trees)
- [ ] Permission prediction based on patterns
- [ ] Batch permission checking
- [ ] Integration with security vigilance mode