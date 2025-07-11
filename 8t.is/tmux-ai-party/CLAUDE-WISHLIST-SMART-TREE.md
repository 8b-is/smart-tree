# CLAUDE-WISHLIST.md for Smart Tree

## Smart Tree Feature Wishlist & Improvement Suggestions

### 🎯 High Priority Features

1. **File Content Preview in Tree Mode**
   - Show first N lines of files inline in tree view
   - Useful for quick README/config inspection without separate reads
   - Example: `├── README.md (3 lines preview)`

2. **Duplicate Content Detection** 
   - Current duplicate detection only checks file size
   - Add hash-based content comparison for true duplicates
   - Show similarity percentage for near-duplicates

3. **Git Integration Improvements**
   - Show git status indicators in tree (modified, new, ignored)
   - Option to exclude gitignored files by default
   - Show last commit info for files

4. **Smart Filtering**
   - `--exclude-empty` flag to hide empty files
   - `--exclude-generated` to hide common generated files (*.lock, *.pyc, etc.)
   - `--focus <pattern>` to highlight specific files while showing context

### 🚀 Performance & Usability

5. **Incremental Analysis**
   - Cache directory analysis results
   - Only re-scan changed directories
   - Would make repeated analyses much faster

6. **Better Summary Mode**
   - The summary-ai mode is great but could show:
     - Technology stack detection
     - Dependency summary
     - Quick stats (total LOC, test coverage indicators)

7. **Interactive Mode**
   - Terminal UI to expand/collapse directories
   - Navigate and preview files
   - Mark files for bulk operations

### 📊 Analysis Enhancements

8. **Code Complexity Metrics**
   - Simple complexity scoring for code files
   - Identify potentially problematic large files
   - Function/class count for quick overview

9. **Dependency Analysis**
   - Parse package.json, requirements.txt, etc.
   - Show dependency tree
   - Identify unused dependencies

10. **Project Health Score**
    - Combine multiple metrics
    - Test file ratio
    - Documentation coverage
    - File organization score

### 🔧 Quality of Life

11. **Custom Output Templates**
    - Allow users to define output format
    - JSON schema for structured output
    - Markdown report generation

12. **Workspace Comparison**
    - Compare two workspaces side-by-side
    - Show what's different between branches
    - Migration helper

13. **Smart Suggestions**
    - Suggest files that could be deleted
    - Identify misplaced files
    - Recommend directory restructuring

### 🐛 Bug Fixes & Minor Improvements

14. **Path Handling**
    - Better handling of symlinks
    - Show symlink targets
    - Detect circular symlinks

15. **Output Compression**
    - The compressed output is great but hard to debug
    - Add `--raw` flag to see uncompressed for debugging
    - Better error messages when decompression fails

16. **Semantic Analysis Polish**
    - Wave signatures are cool but cryptic
    - Add human-readable descriptions
    - Allow custom category definitions

### 💡 Dream Features

17. **AI-Powered Insights**
    - "This looks like a Django project with React frontend"
    - "Your test coverage seems low in the API directory"
    - "Consider moving these utility functions to a shared module"

18. **Integration with Other Tools**
    - Export to draw.io/mermaid diagrams
    - Generate architecture documentation
    - Create project overview slides

19. **Historical Analysis**
    - Track how project structure changes over time
    - Identify growth patterns
    - Predict future restructuring needs

20. **Multi-Repository Support**
    - Analyze monorepos intelligently
    - Compare related repositories
    - Find code duplication across repos

## Notes from Usage

- The `quick_tree` with SUMMARY-AI mode is fantastic for initial exploration
- Compression makes it very token-efficient for AI contexts
- The semantic analysis is innovative but could use more documentation
- Overall, smart-tree is already incredibly useful and well-designed!

Thank you for creating such a helpful tool! 🌲✨