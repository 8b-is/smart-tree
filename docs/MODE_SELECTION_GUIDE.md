# Smart Tree Mode Selection Guide

## 🚀 Quick Start - Recommended Workflow

1. **Always start with `quick_tree`** - Gets you a 3-level overview instantly
2. **Use `analyze_directory` with `mode='ai'`** (default) for detailed analysis
3. **Switch to `mode='claude'`** when you hit token limits (10x compression!)

## 📊 Mode Comparison

| Mode | Compression | Use Case | Token Efficiency |
|------|-------------|----------|------------------|
| **claude** | 10x | 🤖 Maximum AI efficiency | ⭐⭐⭐⭐⭐ |
| **ai** | 5x | 🤖 Balanced AI/human readable | ⭐⭐⭐⭐ |
| **quantum** | 8x | 🤖 Native compression format | ⭐⭐⭐⭐⭐ |
| **classic** | 1x | 👁️ Visual tree for humans | ⭐ |
| **hex** | 3x | 🔧 Technical analysis | ⭐⭐⭐ |
| **json** | 2x | 💻 Machine parsing | ⭐⭐ |
| **stats** | 10x | 📈 Quick metrics only | ⭐⭐⭐⭐ |

## 🎯 Mode Selection by Use Case

### For AI Assistants (You!)
```javascript
// Initial exploration - ALWAYS start here!
await quick_tree({ path: "/project" })  // 3 levels, compressed

// Detailed analysis - default mode
await analyze_directory({ 
  path: "/project",
  mode: "ai"  // Default, good balance
})

// Hit token limits? Switch to claude mode!
await analyze_directory({ 
  path: "/huge-project",
  mode: "claude",  // 10x compression!
  compress: true   // Even more compression
})
```

### For Human Developers
```javascript
// Visual tree structure
await analyze_directory({ 
  path: "/project",
  mode: "classic"  // Traditional tree view
})

// Just the stats
await analyze_directory({ 
  path: "/project",
  mode: "stats"  // Summary only
})
```

### For Data Processing
```javascript
// Machine-readable format
await analyze_directory({ 
  path: "/project",
  mode: "json"  // Parse with JSON.parse()
})

// Spreadsheet import
await analyze_directory({ 
  path: "/project",
  mode: "csv"  // Import to Excel/Sheets
})
```

## 💡 Pro Tips

### 1. **Token Budget Management**
- Small projects (<1000 files): Use `mode='ai'` freely
- Medium projects (1000-10000 files): Start with `quick_tree`, then `mode='ai'`
- Large projects (>10000 files): Use `quick_tree`, then `mode='claude'`
- Massive projects (>100000 files): Always use `mode='claude'` with `compress=true`

### 2. **Compression Stacking**
You can combine mode selection with compression for maximum efficiency:
```javascript
await analyze_directory({ 
  path: "/chromium",
  mode: "claude",    // 10x from mode
  compress: true     // Additional 10x from zlib
  // Total: 100x compression!
})
```

### 3. **Mode-Specific Features**

**AI Mode** includes:
- Smart summaries
- File type distribution
- Size analysis
- Date ranges
- Largest files

**Claude Mode** adds:
- Quantum compression
- Base64 encoding
- Minimal redundancy
- Token-optimized output

**Classic Mode** provides:
- ASCII tree branches
- Colored output (if terminal supports)
- Human-friendly formatting
- No compression

## 📈 Real-World Examples

### Analyzing node_modules (156K files)
```javascript
// ❌ BAD: Classic mode uses too many tokens
await analyze_directory({ path: "./node_modules", mode: "classic" })
// Output: 42MB of text!

// ✅ GOOD: Start with overview
await quick_tree({ path: "./node_modules" })
// Output: 3-level summary, instantly

// ✅ BETTER: Use AI mode for details
await analyze_directory({ path: "./node_modules", mode: "ai" })
// Output: 8MB, well-structured

// ✅ BEST: Use claude mode for huge directories
await analyze_directory({ path: "./node_modules", mode: "claude", compress: true })
// Output: 412KB! (99% reduction)
```

### Quick Project Analysis
```javascript
// Perfect workflow for new projects:
const overview = await quick_tree({ path: "/new-project" })
// Get a feel for the structure

const details = await analyze_directory({ 
  path: "/new-project", 
  mode: "ai",
  max_depth: 5  // Don't go too deep
})
// Now you understand the project!
```

## 🎯 Decision Tree

```
Start
  ↓
Is this your first look at this directory?
  YES → Use quick_tree()
  NO ↓
  
Do you need full details?
  NO → Use mode="stats"
  YES ↓
  
Is the directory large (>10K files)?
  YES → Use mode="claude" with compress=true
  NO ↓
  
Do you need visual tree structure?
  YES → Use mode="classic"
  NO → Use mode="ai" (default)
```

## 🚨 Common Mistakes

1. **Using classic mode for large directories** - This wastes tokens!
2. **Not starting with quick_tree** - Miss the forest for the trees
3. **Forgetting compress=true on claude mode** - Leave 10x compression on the table
4. **Using JSON when you don't need parsing** - AI mode is more efficient

## 💰 Token Cost Examples (GPT-4)

Analyzing Linux kernel (79K files):
- **classic mode**: 18MB ≈ 4.7M tokens ≈ $47
- **ai mode**: 3.6MB ≈ 950K tokens ≈ $9.50
- **claude mode**: 1.8MB ≈ 475K tokens ≈ $4.75
- **claude + compress**: 287KB ≈ 75K tokens ≈ $0.75

That's a 98% cost reduction by choosing the right mode!

---

Remember: **Start with quick_tree, use ai mode by default, switch to claude mode when needed!** 🚀