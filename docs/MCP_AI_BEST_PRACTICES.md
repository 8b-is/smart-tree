# 🤖 Smart Tree MCP Tools - AI Construction Helper Guide 🏗️

**Welcome to your construction site! Smart Tree is your helper, ready to hand you the right tools at the right time.**

## 🚨 Important: Compression Default Changed (v4.0.0+)

**Smart Tree now serves decompressed output by default!** 

Many AI systems struggle with base64/compressed content, so we've changed the default behavior:
- **Before**: AI modes (`ai`, `quantum`, etc.) compressed by default
- **Now**: ALL modes decompressed by default for maximum compatibility
- To enable compression: explicitly set `compress: true` (only if you can handle base64)

## 🚀 Quick Start - Your Construction Site Orientation! 🏗️

Hey there, AI friend! Think of Smart Tree as your experienced construction helper. Just like a good helper knows to hand you a hammer when you're working with nails, Smart Tree provides the right tools for your coding tasks. Follow these patterns to work efficiently!

## 🌟 The Golden Rule: Survey the Site First with `quick_tree` 🗺️

**ALWAYS** begin by surveying your construction site:
```
quick_tree(path=".")  # Survey the job site first!
```

Why? Just like a construction helper walks the site before starting work, this gives you a compressed 3-level overview that's perfect for understanding the project layout without overwhelming your context window!

## 📋 Recommended Workflow - How Your Helper Assists You 🏗️

### 1. Site Survey (Your helper walks the site first!)
```python
# Step 1: Survey the construction site
quick_tree(path=".")  # Your helper shows you the layout

# Step 2: Understand what we're building
project_overview(path=".")  # For single building projects
# OR
analyze_workspace(path=".")  # For complex construction sites with multiple buildings
```

### 2. Your Helper's Specialized Tools 🧰

#### 🔍 Finding Materials (Files) on the Site
```python
# Find all Python files
find_code_files(path=".", languages=["python"])

# Find configuration files
find_config_files(path=".")

# Find documentation
find_documentation(path=".")

# Find test files
find_tests(path=".")
```

#### 🧠 Structural Analysis (Deep Code Inspection)
```python
# Use quantum-semantic mode for best results!
analyze_directory(
    path="src",
    mode="quantum-semantic",  # HIGHLY RECOMMENDED!
    max_depth=10
)

# Or use semantic analysis for conceptual grouping
semantic_analysis(path=".")
```

#### 🔎 Looking for Specific Parts (Content Search)
```python
# Find where a function is defined
search_in_files(path=".", keyword="function_name")

# Find TODOs
search_in_files(path=".", keyword="TODO")
```

#### 📊 Measuring the Job (Project Metrics)
```python
# Get comprehensive statistics
get_statistics(path=".")

# Find large files
find_large_files(path=".", min_size="5M")

# Get directory size breakdown
directory_size_breakdown(path=".")
```

## 💡 Pro Tips from Your Experienced Helper 🏗️

### 1. **Compression is Your Friend**
- `summary-ai` mode = 10x compression!
- `quantum-semantic` = Best for code analysis
- Default compression is ON for AI modes

### 2. **Your Helper Knows Which Tool to Use**
- **Survey the site?** → `quick_tree` (walk around first)
- **Find materials?** → `find_*` tools (locate what you need)
- **Search for issues?** → `search_in_files` (find problems to fix)
- **Inspect structure?** → `quantum-semantic` mode (detailed analysis)
- **Measure progress?** → `get_statistics` (check the metrics)

### 3. **Your Helper Remembers** 🧠
Don't worry about asking for the same tool twice - your helper remembers what was already fetched (cache enabled)!

### 4. **Mode Selection Guide**
```python
# For initial exploration
mode="summary-ai"  # 10x compression, perfect overview

# For code understanding
mode="quantum-semantic"  # Semantic compression with tokens

# For human-readable output
mode="classic"  # Traditional tree view

# For data processing
mode="json"  # Structured data

# For maximum compression
mode="quantum"  # 90%+ compression (binary)
```

## 🎯 Common Construction Scenarios 🏗️

### Starting a New Construction Project
```python
1. quick_tree(path=".")
2. project_overview(path=".")
3. find_code_files(path=".", languages=["all"])
4. analyze_directory(path="src", mode="quantum-semantic")
```

### Locating Specific Building Components
```python
1. quick_tree(path=".")
2. search_in_files(path=".", keyword="className")
3. analyze_directory(path="found/directory", mode="ai")
```

### Site Safety Inspection (Project Health)
```python
1. get_statistics(path=".")
2. find_large_files(path=".")
3. find_duplicates(path=".")
4. find_empty_directories(path=".")
```

### Understanding the Building Blueprint
```python
1. quick_tree(path=".")
2. semantic_analysis(path=".")  # Groups by purpose!
3. find_build_files(path=".")
4. find_config_files(path=".")
```

## 🚨 Important Notes

1. **Security**: Some paths may be blocked (like /etc, /sys)
2. **Performance**: Large directories benefit from compression
3. **Caching**: Results are cached - don't hesitate to re-query
4. **Token Efficiency**: Use compressed modes for large outputs

## 🎸 Remember: Let Your Helper Survey First! 🏗️

If you remember only one thing: **Always let your construction helper survey the site with `quick_tree` first!**

Just like an experienced construction helper who walks the site before starting work, this gives you the perfect overview to plan your tasks efficiently.

---

*Happy building! Remember, Smart Tree is your construction site helper - always ready with the right tool, at the right time, to make your coding work fast, efficient, and enjoyable! 🏗️🌳*

*P.S. - Elvis says: "Start with quick_tree, baby!" 🎸* 