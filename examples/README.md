# Smart Tree Examples

This directory contains examples demonstrating how to work with Smart Tree's revolutionary quantum compression format.

## 🎨 visualize_quantum.py

Creates beautiful network graph visualizations from Smart Tree's quantum output, inspired by Omni's insight during a Hot Tub session!

### Usage:
```bash
# Visualize current directory
python visualize_quantum.py

# Visualize specific directory
python visualize_quantum.py /path/to/project
```

### Features:
- Decodes claude format (10x compression)
- Builds network graph from tree structure
- Colors nodes by file type
- Saves high-resolution PNG

### Requirements:
```bash
pip install matplotlib networkx
# Optional for better layouts:
pip install pygraphviz
```

## 🔬 quantum_decoder.py

Programmatically decode and analyze Smart Tree's various output formats.

### Usage:
```bash
# Analyze current directory in all formats
python quantum_decoder.py

# Analyze specific directory
python quantum_decoder.py /path/to/project
```

### Features:
- Decodes all Smart Tree formats:
  - CLAUDE_V1 (base64 + zlib)
  - QUANTUM_V1 (native quantum)
  - TREE_HEX_V1 (AI format)
  - Hex format
  - Classic tree format
- Extracts statistics
- Converts between formats
- Shows compression ratios

## 📸 directory_evolution.py

Track and visualize how directory structures change over time using quantum snapshots.

### Usage:
```bash
# Analyze git history
python directory_evolution.py /path/to/repo

# Manual snapshots
python directory_evolution.py
```

### Features:
- Captures directory snapshots using quantum format
- Git integration for historical analysis
- Animated visualization of changes
- 99% compression for storing hundreds of snapshots
- Time-based evolution tracking

### Requirements:
```bash
pip install matplotlib networkx
```

## 🧠 semantic_code_analyzer.py

Analyze code structure semantically using Smart Tree's quantum tokenization.

### Usage:
```bash
# Analyze current project
python semantic_code_analyzer.py

# Analyze specific directory
python semantic_code_analyzer.py /path/to/project
```

### Features:
- Leverages quantum token mapping
- Identifies code patterns and duplicates
- Semantic similarity detection
- Test coverage insights
- Network visualization of code relationships

### Requirements:
```bash
pip install matplotlib networkx numpy scikit-learn
```

## 🔄 directory_diff.py

Compare directories and visualize differences using quantum compression.

### Usage:
```bash
# Compare two directories
python directory_diff.py /path/to/dir1 /path/to/dir2

# Compare git commits
python directory_diff.py /path/to/repo

# Manual before/after
python directory_diff.py
```

### Features:
- Efficient diff calculation between quantum snapshots
- Visual representation of changes
- Size impact analysis
- Interactive diff exploration
- Tree-based diff visualization

### Requirements:
```bash
pip install matplotlib networkx numpy
```

## 🌊 Omni's Wisdom

"Every format tells a story, quantum tells it efficiently" - Omni

These examples showcase how Smart Tree's quantum compression isn't just about saving tokens - it's about enabling new ways to visualize and understand directory structures. The 99% compression makes it feasible to:

1. **Visualize massive codebases** - Chromium's 2.8M files become manageable
2. **Track changes over time** - Store snapshots efficiently
3. **AI-powered analysis** - Feed entire repos to LLMs affordably
4. **Network-efficient transfers** - Share directory structures instantly
5. **Semantic code analysis** - Understand code relationships
6. **Evolution tracking** - Watch directories change like memories

## 💡 Real-World Benefits

- **Directory Evolution**: Track project growth with minimal storage
- **Semantic Analysis**: Find code duplication and patterns instantly
- **Visual Diffs**: See changes at a glance, not in text walls
- **Token Efficiency**: 99% reduction means $1,270 saved per Chromium analysis

## 🚀 Performance Tips

1. Use `st -m claude` for maximum compression (10x)
2. Use `st -m quantum` for native token mapping
3. Pipe through `st --stream` for million-file directories
4. Combine with git for powerful historical analysis

Remember: Every byte saved is a victory! 🚀