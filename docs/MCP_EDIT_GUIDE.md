# MCP Edit Tool Guide

## Overview

The Smart Tree MCP Edit tool provides revolutionary AST-aware code editing that achieves **90% token reduction** compared to traditional diff-based approaches. Instead of sending entire file contents or diffs, you describe what you want to change and the tool understands your code structure.

## Quick Start

### Create a New File

```json
{
  "operation": "create_file",
  "file_path": "src/utils.rs",
  "content": "// New utility file\npub fn helper() {\n    println!(\"Hello!\");\n}\n"
}
```

### View Code Structure

Before making changes, understand what's in your file:

```json
{
  "operation": "get_functions",
  "file_path": "src/main.rs"
}
```

Response shows all functions, classes, and their relationships.

### Add a Function

```json
{
  "operation": "insert_function",
  "file_path": "src/utils.rs",
  "name": "validate_input",
  "body": "fn validate_input(input: &str) -> bool {\n    !input.is_empty()\n}",
  "visibility": "public",
  "after": "helper"
}
```

### Remove a Function

```json
{
  "operation": "remove_function",
  "file_path": "src/utils.rs",
  "name": "old_function",
  "force": false
}
```

## Operation Types

### 0️⃣ create_file

**Purpose:** Create a new file with initial content

**Required Fields:**
- `file_path` - Path to create

**Optional Fields:**
- `content` - Initial content (defaults to empty if omitted)
**Features:**
- Automatically creates parent directories
- Fails if file already exists (prevents accidental overwrites)

**Example:**
```json
{
  "operation": "create_file",
  "file_path": "tests/test_utils.rs",
  "content": "#[cfg(test)]\nmod tests {\n    use super::*;\n}\n"
}
```

### 1️⃣ get_functions

**Purpose:** View code structure

**Required Fields:**
- `file_path` - File to analyze

**Returns:**
```json
{
  "language": "Rust",
  "functions": [
    {
      "name": "main",
      "signature": "fn main()",
      "start_line": 10,
      "end_line": 15,
      "calls": ["helper"]
    }
  ],
  "classes": [...],
  "imports": [...]
}
```

### 2️⃣ insert_function

**Purpose:** Add a new function to a file

**Required Fields:**
- `file_path` - Target file
- `name` - Function name
- `body` - Function body (without the signature)

**Optional Fields:**
- `after` - Insert after this function
- `before` - Insert before this function  
- `class_name` - Add as a method to this class
- `visibility` - `"public"`, `"private"`, or `"protected"` (default: `"private"`)

**Examples:**

Rust function:
```json
{
  "operation": "insert_function",
  "file_path": "src/lib.rs",
  "name": "process_data",
  "body": "fn process_data(input: Vec<i32>) -> Vec<i32> {\n    input.iter().map(|x| x * 2).collect()\n}",
  "visibility": "public"
}
```

Python method:
```json
{
  "operation": "insert_function",
  "file_path": "app.py",
  "name": "validate",
  "class_name": "DataProcessor",
  "body": "(self, data):\n    return len(data) > 0",
  "after": "__init__"
}
```

### 3️⃣ remove_function

**Purpose:** Remove a function with dependency checking

**Required Fields:**
- `file_path` - Target file
- `name` - Function to remove

**Optional Fields:**
- `class_name` - For removing methods
- `force` - Remove even if other code depends on it (default: `false`)
- `cascade` - Also remove orphaned functions (default: `false`)

**Example:**
```json
{
  "operation": "remove_function",
  "file_path": "src/deprecated.rs",
  "name": "old_api_call",
  "force": true
}
```

### 4️⃣ smart_edit

**Purpose:** Apply multiple AST-aware edits in one operation

**Required Fields:**
- `file_path` - Target file
- `edits` - Array of edit operations

**Edit Operations (currently supported):**

> Note: The SmartEdit engine currently supports only the operations documented in this section (for example, `InsertFunction` and `ReplaceFunction`).  
> The following SmartEdit sub-operations are **planned but not yet implemented** and will return `"Operation not yet implemented"` if used:  
> `InsertClass`, `AddMethod`, `WrapCode`, `DeleteElement`, `Rename`, `AddDocumentation`.
#### InsertFunction
Required: `name`, `body`
```json
{
  "operation": "InsertFunction",
  "name": "helper",
  "body": "fn helper() { }"
}
```

#### ReplaceFunction
Required: `name`, `new_body`
```json
{
  "operation": "ReplaceFunction",
  "name": "old_impl",
  "new_body": "fn old_impl() { /* new implementation */ }"
}
```

#### AddImport
Required: `import`
```json
{
  "operation": "AddImport",
  "import": "std::collections::HashMap"
}
```

#### InsertClass
Required: `name`, `body`
```json
{
  "operation": "InsertClass",
  "name": "Config",
  "body": "struct Config {\n    value: i32\n}"
}
```

#### AddMethod
Required: `class_name`, `method_name`, `body`
```json
{
  "operation": "AddMethod",
  "class_name": "MyClass",
  "method_name": "process",
  "body": "fn process(&self) { }"
}
```

#### SmartAppend
Required: `section`, `content`
```json
{
  "operation": "SmartAppend",
  "section": "functions",
  "content": "fn new_func() { }"
}
```

Sections: `"imports"`, `"functions"`, `"classes"`, `"main"`

#### DeleteElement
Required: `element_type`, `name`
```json
{
  "operation": "DeleteElement",
  "element_type": "function",
  "name": "unused"
}
```

#### Rename
Required: `old_name`, `new_name`
```json
{
  "operation": "Rename",
  "old_name": "oldName",
  "new_name": "newName",
  "scope": "global"
}
```

#### AddDocumentation
Required: `target_type`, `target_name`, `documentation`
```json
{
  "operation": "AddDocumentation",
  "target_type": "function",
  "target_name": "process",
  "documentation": "/// Processes the input data"
}
```

#### WrapCode
Required: `start_line`, `end_line`, `wrapper_type`
```json
{
  "operation": "WrapCode",
  "start_line": 10,
  "end_line": 15,
  "wrapper_type": "try",
  "condition": "/* catch handler */"
}
```

### Complete Example: Multiple Edits

```json
{
  "operation": "smart_edit",
  "file_path": "src/api.rs",
  "edits": [
    {
      "operation": "AddImport",
      "import": "serde_json"
    },
    {
      "operation": "InsertFunction",
      "name": "parse_response",
      "body": "fn parse_response(json: &str) -> Result<Response> {\n    serde_json::from_str(json)\n}",
      "visibility": "private",
      "after": "handle_request"
    },
    {
      "operation": "ReplaceFunction",
      "name": "handle_request",
      "new_body": "fn handle_request(req: Request) -> Response {\n    let json = req.body();\n    parse_response(json).unwrap_or_default()\n}"
    }
  ]
}
```

## Supported Languages

- Rust (`.rs`)
- Python (`.py`)
- JavaScript (`.js`, `.mjs`)
- TypeScript (`.ts`, `.tsx`)
- Go (`.go`)
- Java (`.java`)
- C# (`.cs`)
- C++ (`.cpp`, `.cc`, `.cxx`, `.hpp`, `.h`)
- Ruby (`.rb`)

## Common Patterns

### 1. Create and Initialize a File

```json
{
  "operation": "create_file",
  "file_path": "src/config.rs",
  "content": "use serde::{Deserialize, Serialize};\n\n#[derive(Debug, Serialize, Deserialize)]\npub struct Config {\n    pub debug: bool,\n}\n"
}
```

### 2. Add Multiple Functions to New File

```json
{
  "operation": "smart_edit",
  "file_path": "src/utils.rs",
  "edits": [
    {
      "operation": "AddImport",
      "import": "std::path::Path"
    },
    {
      "operation": "InsertFunction",
      "name": "validate_path",
      "body": "pub fn validate_path(path: &Path) -> bool {\n    path.exists()\n}"
    },
    {
      "operation": "InsertFunction",
      "name": "clean_path",
      "body": "pub fn clean_path(path: &str) -> String {\n    path.trim().to_string()\n}"
    }
  ]
}
```

### 3. Refactor with Position Control

```json
{
  "operation": "insert_function",
  "file_path": "src/main.rs",
  "name": "init_logging",
  "body": "fn init_logging() {\n    env_logger::init();\n}",
  "before": "main",
  "visibility": "private"
}
```

### 4. Safe Function Removal

```json
{
  "operation": "remove_function",
  "file_path": "src/deprecated.rs",
  "name": "old_helper",
  "force": false
}
```

If other functions depend on `old_helper`, this will fail with a helpful error message.

## Error Handling

### Common Errors

1. **"File already exists"** (create_file)
   - Solution: Use edit operations instead, or delete the file first

2. **"missing field `name`"** (InsertFunction, InsertClass)
   - Solution: Always provide the `name` field for these operations

3. **"file_path required"**
   - Solution: All operations need `file_path`

4. **"Unsupported language"**
   - Solution: Check that file extension is in the supported list

5. **"Function has dependencies"** (remove_function)
   - Solution: Use `"force": true` or remove dependent functions first

## Token Efficiency

Traditional approach for adding a function to a 500-line file:
```
Tokens needed: ~2,000 (entire file + diff)
```

Smart Edit approach:
```
Tokens needed: ~50 (just the function name and body)
```

**90% reduction in tokens!**

## Best Practices

1. **Always `create_file` first** before editing non-existent files
2. **Use `get_functions`** to understand code structure before changes
3. **Use positioning** (`after`, `before`) for precise placement
4. **Batch related changes** in a single `smart_edit` operation
5. **Set `force: false`** on `remove_function` for safety
6. **Use meaningful function names** for better positioning

## Integration with MCP Clients

### Claude Desktop

Add to your `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "smart-tree": {
      "command": "st",
      "args": ["--mcp"]
    }
  }
}
```

### Cline/Roo-Cline

Configure in MCP settings to use the `edit` tool with these operations.

## Troubleshooting

### Schema Validation Errors

If you get validation errors, check:
- Required fields are present for your operation
- Field names match exactly (case-sensitive)
- `operation` value matches one of the enum values

### AST Parsing Errors

If the tool can't parse your file:
- Check for syntax errors in the target file
- Ensure file extension matches content (`.rs` for Rust, etc.)
- Some malformed files may not parse completely but can still be edited

### Performance

For very large files (>10,000 lines):
- Consider breaking into smaller modules
- Use `get_functions` sparingly
- Batch multiple edits together

## Future Enhancements

Coming soon:
- Multi-file refactoring
- Automatic import management
- Code style preservation
- AI-suggested edits based on context

## Support

Issues? Questions? 
- GitHub: https://github.com/8b-is/smart-tree
- Docs: Check CONTRIBUTING.md for more details
