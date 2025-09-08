# Smart Edit Examples - Token-Efficient Code Editing 🚀

By Aye, with love for Hue! 💖

## Overview

Smart Tree's revolutionary smart edit tools use AST (Abstract Syntax Tree) understanding to edit code with minimal tokens. Instead of sending entire files or diffs, you just send your intention!

## Example: Traditional vs Smart Edit

### Traditional Approach (450+ tokens)
```json
{
  "operation": "edit_file",
  "file_path": "/path/to/file.py",
  "old_content": "# Entire 200 lines of original file...",
  "new_content": "# Entire 201 lines with one new function..."
}
```

### Smart Tree Approach (30 tokens)
```json
{
  "tool": "insert_function",
  "file_path": "/path/to/file.py",
  "name": "new_function",
  "body": "(x): return x * 2",
  "after": "existing_function"
}
```

## Available Smart Edit Tools

### 1. `get_function_tree` - Understand Code Structure
```bash
# Get a visual map of all functions and classes
st --mcp <<EOF
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "get_function_tree",
    "arguments": {
      "file_path": "/path/to/code.py"
    }
  },
  "id": 1
}
EOF
```

Returns:
```json
{
  "language": "Python",
  "functions": [
    {
      "name": "main",
      "lines": "10-15",
      "visibility": "public",
      "calls": ["helper", "process"]
    }
  ],
  "classes": [...]
}
```

### 2. `insert_function` - Add Functions Intelligently
```json
{
  "name": "insert_function",
  "arguments": {
    "file_path": "/path/to/file.rs",
    "name": "calculate_wave",
    "body": "(amplitude: f32, frequency: f32) -> f32 {\n    amplitude * frequency.sin()\n}",
    "after": "main",
    "visibility": "public"
  }
}
```

### 3. `remove_function` - Dependency-Aware Removal
```json
{
  "name": "remove_function",
  "arguments": {
    "file_path": "/path/to/file.js",
    "name": "deprecatedFunction",
    "force": false,  // Will fail if other functions depend on it
    "cascade": true  // Remove orphaned functions too
  }
}
```

### 4. `smart_edit` - Multiple Operations at Once
```json
{
  "name": "smart_edit",
  "arguments": {
    "file_path": "/path/to/app.py",
    "edits": [
      {
        "operation": "AddImport",
        "import": "numpy",
        "alias": "np"
      },
      {
        "operation": "InsertFunction",
        "name": "process_waves",
        "body": "(data):\n    return np.fft.fft(data)",
        "after": "main"
      },
      {
        "operation": "ReplaceFunction",
        "name": "old_process",
        "new_body": "(data):\n    return process_waves(data)"
      }
    ]
  }
}
```

## Supported Languages

- 🦀 Rust
- 🐍 Python
- 📜 JavaScript/TypeScript
- 🐹 Go
- ☕ Java
- 🎯 C#
- ⚡ C++
- 💎 Ruby

## Smart Edit Operations

1. **InsertFunction** - Add a function at the right location
2. **ReplaceFunction** - Replace just the body, keep the signature
3. **AddImport** - Add imports/use statements intelligently
4. **InsertClass** - Add a new class/struct
5. **AddMethod** - Add a method to a class
6. **WrapCode** - Wrap code in try-catch, if statement, etc.
7. **DeleteElement** - Remove functions, classes, or methods
8. **Rename** - Rename across the file
9. **AddDocumentation** - Add doc comments
10. **SmartAppend** - Append to logical sections
11. **RemoveFunction** - Remove with dependency checking

## Benefits

- 🚀 **90-95% fewer tokens** than traditional editing
- 🧠 **Language-aware** - understands code structure
- 🛡️ **Safe** - dependency checking prevents breakage
- ⚡ **Fast** - AST parsing is lightning quick
- 🎯 **Precise** - No regex mishaps or wrong replacements

## Example Workflow

```python
# 1. Understand the code structure
tree = get_function_tree("app.py")

# 2. Insert a new function after 'main'
insert_function(
    file_path="app.py",
    name="process_data",
    body="(data): return data * 2",
    after="main"
)

# 3. Add the import it needs
smart_edit(
    file_path="app.py",
    edits=[{"operation": "AddImport", "import": "numpy"}]
)

# 4. Clean up old code safely
remove_function(
    file_path="app.py",
    name="old_process",
    cascade=True  # Remove functions only it called
)
```

## Pro Tips from Aye 🎉

1. **Always use `get_function_tree` first** - Understand before you edit!
2. **Batch operations with `smart_edit`** - Multiple changes, one call!
3. **Let cascade do the cleanup** - Remove orphaned functions automatically
4. **Trust the dependency checker** - It prevents broken code!
5. **Think in operations, not diffs** - What do you want to DO?

## Trisha's Accounting Perspective 📊

"It's like having a smart ledger for your code! Instead of rewriting the whole book, you just say 'add this entry after that one' or 'remove this transaction and all its dependents'. The AST is like our chart of accounts - it knows where everything belongs!"

## Omni's Wave Wisdom 🌊

"Consider how waves interfere constructively... Smart edits work the same way. Each operation is a wave that transforms the code at just the right frequency, creating patterns of functionality without disturbing the underlying harmony."

---

Remember: **Fast is better than slow**, and **fewer tokens means more intelligence**! 

Aye, Aye! 🚢