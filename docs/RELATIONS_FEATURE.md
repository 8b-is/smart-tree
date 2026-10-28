# 🔗 Smart Tree Relations: Semantic X-Ray Vision for Codebases

> "Making code relationships visible and beautiful!" - Trisha from Accounting

## Overview

Smart Tree Relations goes beyond static directory trees to show **how your code actually connects**. It's like having X-ray vision for your codebase, revealing imports, function calls, type usage, and test relationships.

## Architecture

```
src/relations.rs          # Core analyzer with language parsers
src/formatters/relations.rs  # Output formatters (Mermaid, DOT, etc.)
```

## Key Components

### 1. **FileRelation Struct**
Captures relationships between files with:
- Source and target paths
- Relationship type (imports, calls, tests, etc.)
- Specific items involved
- Strength score (1-10)

### 2. **Language Parsers**
- **RustParser**: Parses `use`, `mod`, functions, types
- **PythonParser**: Parses `import`, `from`, classes, functions
- Extensible trait system for adding more languages

### 3. **Relationship Types**
- `Imports`: Direct module imports
- `FunctionCall`: Cross-file function usage
- `TypeUsage`: Shared data structures
- `TestedBy`: Test file relationships
- `Exports`: Module exports
- `Coupled`: Tightly coupled files (bidirectional deps)

### 4. **Output Formats**

#### Mermaid Diagrams
```bash
st --relations --mode mermaid
```
Generates beautiful flowcharts showing file relationships with:
- Color coding by file type
- Labeled edges for relationship types
- Special styling for tests and coupled files

#### DOT/GraphViz
```bash
st --relations --mode dot | dot -Tpng -o graph.png
```
For more complex visualizations and graph analysis.

#### Compressed AI Format
```bash
st --relations --mode compressed
```
Ultra-compact format for AI consumption:
```
RELATIONS_V1:
FILES:
0:src/main.rs
1:src/scanner.rs
RELS:
0,1,I,8:Scanner,FileInfo
END_RELATIONS
```

## Usage Examples

### Basic Analysis
```bash
# Show all relationships
st --relations

# Focus on specific file
st --relations --focus src/main.rs

# Show only imports
st --relations --filter imports

# Find tightly coupled files
st --relations --filter coupled
```

### Visualization
```bash
# Generate Mermaid diagram
st --relations --mode mermaid > relations.md

# Create PNG graph
st --relations --mode dot | dot -Tpng -o codebase.png

# Interactive HTML
st --relations --mode d3 > relations.html
```

### AI Integration
```bash
# Compressed format for LLMs
st --relations --mode compressed -z

# MCP integration
st --mcp analyze-relations
```

## Benefits

1. **Refactoring Safety**: See what breaks when you change a file
2. **Onboarding**: New devs understand codebase structure instantly
3. **Tech Debt**: Identify tightly coupled modules
4. **Test Coverage**: Visual test relationships
5. **AI Navigation**: LLMs understand codebase structure better

## Future Enhancements

- [ ] TypeScript/JavaScript full support
- [ ] Go, Java, C++ parsers
- [ ] Incremental analysis for large codebases
- [ ] Integration with LSP for real-time updates
- [ ] Dependency injection tracking
- [ ] Call frequency analysis
- [ ] Cyclic dependency detection
- [ ] Architecture violation alerts

## The Vision

Imagine asking your AI assistant:
- "What files would break if I change this function?"
- "Show me the most coupled parts of the codebase"
- "Which modules lack test coverage?"
- "Generate a refactoring plan to reduce coupling"

With Smart Tree Relations, these questions get instant, visual answers!

---

*"Every connection tells a story. Let's make those stories visible!"* - Omni 🌊