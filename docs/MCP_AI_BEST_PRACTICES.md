# 🤖 Smart Tree MCP Tools - AI Best Practices Guide

## 🚀 Quick Start for AIs

Hey there, AI friend! This guide will help you use Smart Tree tools like a pro. Follow these patterns for optimal results!

## 🌟 The Golden Rule: Start with `quick_tree`

**ALWAYS** begin your exploration with:
```
quick_tree(path=".")  # or any directory path
```

Why? It gives you a compressed 3-level overview that's perfect for understanding structure without overwhelming your context window!

## 📋 Recommended Workflow

### 1. Initial Exploration (Always do this first!)
```python
# Step 1: Get the overview
quick_tree(path=".")

# Step 2: Get project context
project_overview(path=".")  # For single projects
# OR
analyze_workspace(path=".")  # For complex/multi-language projects
```

### 2. Specific Analysis Patterns

#### 🔍 Finding Specific Files
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

#### 🧠 Deep Code Analysis
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

#### 🔎 Searching Content
```python
# Find where a function is defined
search_in_files(path=".", keyword="function_name")

# Find TODOs
search_in_files(path=".", keyword="TODO")
```

#### 📊 Understanding Project Metrics
```python
# Get comprehensive statistics
get_statistics(path=".")

# Find large files
find_large_files(path=".", min_size="5M")

# Get directory size breakdown
directory_size_breakdown(path=".")
```

## 💡 Pro Tips for Maximum Efficiency

### 1. **Compression is Your Friend**
- `summary-ai` mode = 10x compression!
- `quantum-semantic` = Best for code analysis
- Default compression is ON for AI modes

### 2. **Use the Right Tool for the Job**
- **Overview?** → `quick_tree`
- **Find files?** → `find_*` tools
- **Search content?** → `search_in_files`
- **Understand code?** → `quantum-semantic` mode
- **Project stats?** → `get_statistics`

### 3. **Cache is Enabled**
Don't worry about calling tools multiple times - results are cached automatically!

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

## 🎯 Common Use Cases

### Understanding a New Codebase
```python
1. quick_tree(path=".")
2. project_overview(path=".")
3. find_code_files(path=".", languages=["all"])
4. analyze_directory(path="src", mode="quantum-semantic")
```

### Finding Specific Implementation
```python
1. quick_tree(path=".")
2. search_in_files(path=".", keyword="className")
3. analyze_directory(path="found/directory", mode="ai")
```

### Analyzing Project Health
```python
1. get_statistics(path=".")
2. find_large_files(path=".")
3. find_duplicates(path=".")
4. find_empty_directories(path=".")
```

### Understanding Project Structure
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

## 🎸 Remember: Quick Tree First!

If you remember only one thing: **Always start with `quick_tree`!**

It's optimized for AI consumption and gives you the perfect overview to plan your next moves.

---

*Happy exploring! Remember, Smart Tree is here to make directory analysis fast, efficient, and token-friendly! 🌳*

*P.S. - Elvis says: "Start with quick_tree, baby!" 🎸* 