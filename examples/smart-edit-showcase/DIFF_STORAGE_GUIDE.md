# 🗂️ Smart Edit Diff Storage System

*"Every change tells a story. Let's remember them all."* — The Audit Chronicles

## 📊 Overview

Smart Tree's Smart Edit tools now automatically store diffs of all file modifications in a local `.st` folder. This provides:

- **Local audit trail** of all AI-assisted edits
- **Git-independent** change tracking
- **Timestamp-based** diff files
- **Easy rollback** capabilities
- **Minimal storage** overhead

## 🏗️ How It Works

### Automatic Setup
When you use Smart Edit operations, the system automatically:
1. Creates a `.st` folder in your project root
2. Adds `.st/` to your `.gitignore`
3. Stores diffs with Unix timestamps

### File Naming Convention
```
.st/
├── user_service.rs              # Original file backup
├── src-user_service.rs-1754631234  # Diff at timestamp 1754631234
├── src-user_service.rs-1754631289  # Diff at timestamp 1754631289
└── src-auth_handler.rs-1754631345  # Diff for another file
```

## 🛠️ Usage Examples

### 1. Smart Edit Creates Diffs Automatically
When you use any Smart Edit operation:
```javascript
// This automatically creates a diff in .st/
mcp.callTool('smart_edit', {
  file_path: 'src/user_service.rs',
  edits: [{
    operation: 'InsertFunction',
    name: 'delete_user',
    after: 'get_user',
    body: '...'
  }]
})
```

### 2. View File Edit History
```bash
# List all diffs for a file
ls -la .st/src-user_service.rs-*

# View a specific diff
cat .st/src-user_service.rs-1754631234

# See what changed
diff .st/user_service.rs src/user_service.rs
```

### 3. Restore Previous Version
```bash
# Restore from original backup
cp .st/user_service.rs src/user_service.rs

# Or apply a specific diff in reverse
patch -R < .st/src-user_service.rs-1754631234
```

## 📈 Storage Management

### Automatic Cleanup
Smart Edit can be configured to keep only the last N diffs per file:
```javascript
// Keep only last 10 diffs per file
mcp.callTool('cleanup_diffs', {
  project_root: '.',
  keep_count: 10
})
```

### Storage Statistics
```bash
# Check storage usage
du -sh .st/

# Count diffs per file
ls .st/ | grep -E "^src-.*-[0-9]+$" | cut -d'-' -f1-2 | sort | uniq -c
```

## 🎯 Benefits

### 1. **Complete Audit Trail**
Every AI-assisted edit is tracked with:
- Exact timestamp
- Full diff content
- Original file state

### 2. **Git Independence**
- Works without git initialization
- Tracks changes before commits
- Supplements version control

### 3. **Easy Recovery**
- Roll back any unwanted changes
- Compare multiple edit sessions
- Restore specific versions

### 4. **Minimal Overhead**
- Only stores diffs, not full files
- Automatic compression potential
- Configurable retention

## 🔍 Diff Format

Diffs are stored in unified format:
```diff
--- a/src/user_service.rs
+++ b/src/user_service.rs
@@ -35,6 +35,10 @@ impl UserService {
     pub fn get_user(&self, id: u64) -> Option<&User> {
         self.users.get(&id)
     }
+
+    pub fn delete_user(&mut self, id: u64) -> Option<User> {
+        self.users.remove(&id)
+    }
 }
```

## 💡 Pro Tips

### 1. **Regular Cleanup**
Set up a cron job or git hook to clean old diffs:
```bash
# Keep last 20 diffs per file
find .st -name "*-[0-9]*" -type f | sort | head -n -20 | xargs rm -f
```

### 2. **Pre-commit Review**
Review AI changes before committing:
```bash
# See all changes made today
find .st -name "*-$(date +%s | cut -c1-5)*" -exec echo {} \; -exec cat {} \;
```

### 3. **Integration with Git**
While `.st/` is gitignored, you can:
- Archive important diffs
- Create summary reports
- Generate changelog entries

## 🚀 Future Enhancements

Planned features:
- [ ] Compression of old diffs
- [ ] Web UI for diff browsing
- [ ] Integration with Smart Tree MCP tools
- [ ] Diff merging capabilities
- [ ] Change attribution (which AI made which edit)

## 🎨 Demo Scripts

Try the demo to see it in action:
```bash
cd examples/smart-edit-showcase
./demo_diff_storage.sh
```

---

*"In the space between intention and implementation, accountability lives."*

Crafted with care by Aye & Hue 🗂️✨

**Smart Tree v4.0.0** | **Automatic Diff Storage** | **Complete Audit Trail**