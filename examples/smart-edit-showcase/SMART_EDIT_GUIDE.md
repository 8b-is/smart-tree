# 🎯 Smart Edit Function Tools - Complete Guide

*"Why send a whole manuscript when a haiku would do?"* — The Efficiency Scrolls

## 📊 The Problem with Traditional Editing

When you want to add a single function to a file, traditional approaches require:
1. Send the ENTIRE file to the AI (hundreds/thousands of tokens)
2. AI modifies and returns the ENTIRE file
3. Risk of unintended changes elsewhere
4. Massive token waste

## ✨ The Smart Edit Solution

Smart Tree understands code structure at the AST level, enabling surgical edits:
1. Send ONLY the change description (20-50 tokens)
2. Smart Tree applies the edit precisely
3. Zero risk to unchanged code
4. 90-95% token reduction!

## 🛠️ Available Function Operations

### 1. `get_function_tree` - Understand Structure First
```rust
// Returns a structured view of all functions
mcp.callTool('get_function_tree', {
    file_path: '/path/to/file.rs'
})

// Output:
{
  "functions": [
    {
      "name": "create_user",
      "line_start": 24,
      "line_end": 33,
      "visibility": "public",
      "signature": "pub fn create_user(&mut self, name: String, email: String) -> User"
    },
    {
      "name": "get_user",
      "line_start": 35,
      "line_end": 37,
      "visibility": "public",
      "signature": "pub fn get_user(&self, id: u64) -> Option<&User>"
    }
  ],
  "classes": [...],
  "imports": [...]
}
```

### 2. `InsertFunction` - Add New Functions
```json
{
  "operation": "InsertFunction",
  "name": "delete_user",
  "before": "get_user",  // or "after": "create_user"
  "body": "pub fn delete_user(&mut self, id: u64) -> Option<User> {\n    self.users.remove(&id)\n}",
  "visibility": "public"  // optional, defaults to private
}
```

**Token Cost**: ~35 tokens vs ~800 tokens (full file)

### 3. `ReplaceFunction` - Update Function Bodies
```json
{
  "operation": "ReplaceFunction",
  "name": "verify_token",
  "new_body": "// New implementation with better security\n    use jwt::decode;\n    decode(token, &self.key, &Validation::default()).is_ok()"
}
```

**Token Cost**: ~40 tokens vs ~500 tokens (full file)

### 4. `AddMethod` - Add Methods to Classes
```json
{
  "operation": "AddMethod",
  "class_name": "UserService",
  "name": "update_user",
  "after": "create_user",
  "body": "pub fn update_user(&mut self, id: u64, name: String) -> Option<&User> {\n    self.users.get_mut(&id).map(|u| {\n        u.name = name;\n        u\n    })\n}"
}
```

### 5. `ExtractFunction` - Refactor Code
```json
{
  "operation": "ExtractFunction",
  "from": "create_user",
  "lines": "26-28",
  "to": "validate_user_data",
  "visibility": "private"
}
```

### 6. `AddImport` - Manage Imports
```json
{
  "operation": "AddImport",
  "import": "use chrono::{DateTime, Utc};",
  "group": "external"  // optional: "std", "external", "internal"
}
```

### 7. `DeleteElement` - Remove Code
```json
{
  "operation": "DeleteElement",
  "type": "function",
  "name": "deprecated_method"
}
```

### 8. `RenameSymbol` - Refactor Names
```json
{
  "operation": "RenameSymbol",
  "from": "getUserData",
  "to": "fetchUserProfile"
}
```

## 🎭 Batch Operations

Combine multiple edits in a single request:

```json
{
  "edits": [
    {
      "operation": "AddImport",
      "import": "use log::{info, error};"
    },
    {
      "operation": "InsertFunction",
      "name": "log_operation",
      "before": "main",
      "body": "fn log_operation(op: &str) {\n    info!(\"Operation: {}\", op);\n}"
    },
    {
      "operation": "WrapCode",
      "function": "create_user",
      "wrapper": "log_operation(\"create_user\");\n{CODE}\ninfo!(\"User created successfully\");"
    }
  ]
}
```

**Batch Token Cost**: ~80 tokens vs ~1000+ tokens

## 📈 Real-World Example

Let's say you have a 500-line service file and need to:
1. Add error handling enum
2. Update 3 functions to return Result
3. Add logging imports
4. Add a new validation function

### Traditional Approach:
- Send 500 lines × 4 operations = 2000 lines
- Token cost: ~12,000 tokens
- Risk: AI might change unrelated code

### Smart Edit Approach:
- Send 4 structured edit operations
- Token cost: ~200 tokens
- Risk: Zero - only specified changes applied

**Savings: 98% token reduction!** 🚀

## 🎨 Language Support

Currently optimized for:
- **Rust** ✅ (Full AST support)
- **Python** ✅ (AST module)
- **JavaScript/TypeScript** ✅ (Tree-sitter)
- **Go** ✅ (Official parser)
- **Java** 🚧 (Coming soon)
- **C++** 🚧 (In development)

## 💡 Best Practices

### 1. Always Start with Structure
```javascript
// First, understand what's there
const tree = await mcp.callTool('get_function_tree', { file_path });

// Then make informed edits
const edit = await mcp.callTool('smart_edit', { 
  file_path,
  edits: [...]
});
```

### 2. Use Semantic Names
Instead of line numbers, reference functions by name:
- ✅ `"after": "create_user"`
- ❌ `"after_line": 45`

### 3. Batch Related Changes
Group imports, related functions, and their tests together.

### 4. Let Smart Edit Handle Formatting
Don't worry about exact indentation - Smart Edit matches the file's style.

## 🔥 Advanced Features

### Dependency-Aware Removal
```json
{
  "operation": "remove_function",
  "name": "helper_function",
  "cascade": true  // Also removes functions only this one calls
}
```

### Smart Append by Section
```json
{
  "operation": "SmartAppend",
  "section": "functions",  // or "imports", "classes", "tests"
  "content": "pub fn new_feature() { ... }"
}
```

### Context-Aware Insertion
Smart Edit understands:
- Where imports belong (top, after package statement)
- Function ordering (public before private)
- Test organization (unit tests at bottom)
- Comment preservation

## 🎪 Try It Yourself!

### Run the Shell Demo:
```bash
cd examples/smart-edit-showcase
./demo_smart_edit_functions.sh
```

### Run the Interactive Python Demo:
```bash
cd examples/smart-edit-showcase
python3 demo_smart_edit_interactive.py
```

### Explore the Mock Project:
- `src/user_service.rs` - User management service
- `src/auth_handler.rs` - Authentication module
- `src/lib.rs` - Library exports

---

*"In the space between intention and implementation, efficiency lives."*

Crafted with precision by Aye & Hue 🛠️✨

**Smart Tree v4.0.0** | **90-95% Token Reduction** | **AST-Aware Editing** | **Zero Risk**