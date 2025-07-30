# Marqant Aggregate (.mq) Specification

> "Why read 20 files when one quantum singularity contains them all?" 🌌

> OBSOLETE

## Overview

Marqant Aggregate creates a single `.mq` file containing all markdown documentation from a project, with:
- File references preserved
- Cross-file deduplication
- 90%+ compression ratio
- Perfect navigation structure

## File Format

```
MARQANT_V2 2025-07-16T10:30:00Z 150000 8000 -aggregate -zlib
::manifest::
README.md:0:1234
docs/INSTALL.md:1234:2500
docs/API.md:3734:5000
::end-manifest::
T00=# 
T01=## 
T10=Smart Tree
T11=installation
...dynamic tokens...
---
::file:README.md::
T00T10 Documentation
...compressed content...
::file:docs/INSTALL.md::
T01T11 Guide
...compressed content...
::file:docs/API.md::
T01API Reference
...compressed content...
```

## Key Features

### 1. File Manifest
- Lists all included files with byte offsets
- Enables random access to any document
- Format: `path:start:length`

### 2. Unified Token Dictionary
- Tokens shared across ALL documents
- Massive savings on repeated terms
- Project-specific vocabulary emerges

### 3. File Markers
- `::file:path/to/file.md::` marks document boundaries
- Preserves relative paths from project root
- Enables selective extraction

### 4. Smart Deduplication
- Identical sections tokenized once
- Cross-references between documents
- "See also" patterns detected

## CLI Commands

### Create Aggregate
```bash
# Create project-wide .mq
mq aggregate .                    # All .md files in project
mq aggregate . -o smart-tree.mq   # Named output
mq aggregate --max-depth 3        # Limit directory depth
mq aggregate --pattern "*.md"     # Specific patterns
mq aggregate --exclude "vendor/*" # Exclude paths
```

### Extract from Aggregate
```bash
# List contents
mq list smart-tree.mq

# Extract specific file
mq extract smart-tree.mq --file README.md

# Extract all to directory
mq extract smart-tree.mq --all --output-dir extracted/

# Search within aggregate
mq search smart-tree.mq "installation"
```

### Analyze Aggregate
```bash
# Show statistics
mq stats smart-tree.mq

Output:
📦 Aggregate: smart-tree.mq
📄 Files: 15 markdown documents
📏 Original: 150KB total
🗜️  Compressed: 8KB (94.7% reduction)
🔤 Shared tokens: 127
🔗 Cross-references: 23
```

## Implementation Design

### Aggregator Algorithm
```rust
pub struct MarqantAggregator {
    files: Vec<MarkdownFile>,
    global_tokens: HashMap<String, String>,
    file_offsets: HashMap<String, FileOffset>,
}

impl MarqantAggregator {
    pub fn aggregate(root: &Path) -> Result<String> {
        // 1. Discover all markdown files
        let files = find_markdown_files(root)?;
        
        // 2. Build global frequency map
        let mut global_phrases = HashMap::new();
        for file in &files {
            analyze_phrases(&file.content, &mut global_phrases);
        }
        
        // 3. Assign tokens based on global frequency
        let tokens = assign_global_tokens(global_phrases);
        
        // 4. Compress each file with shared dictionary
        let mut compressed = String::new();
        let mut manifest = Vec::new();
        
        for file in files {
            let start = compressed.len();
            compressed.push_str(&format!("::file:{}::\n", file.path));
            compressed.push_str(&tokenize_with_dict(&file.content, &tokens));
            let length = compressed.len() - start;
            
            manifest.push(format!("{}:{}:{}", file.path, start, length));
        }
        
        // 5. Build final output
        Ok(build_aggregate(tokens, manifest, compressed))
    }
}
```

### Extraction Algorithm
```rust
pub fn extract_file(aggregate: &str, target_path: &str) -> Result<String> {
    let (manifest, tokens, content) = parse_aggregate(aggregate)?;
    
    // Find file in manifest
    let file_info = manifest.get(target_path)
        .ok_or("File not found in aggregate")?;
    
    // Extract compressed section
    let compressed = &content[file_info.start..file_info.end];
    
    // Decompress with shared dictionary
    decompress_with_tokens(compressed, &tokens)
}
```

## Benefits

### For AI Assistants
- **One file to rule them all**: Entire docs in single context
- **Cross-file understanding**: See relationships between docs
- **Efficient loading**: 94% smaller than original
- **Smart navigation**: Jump to any document instantly

### For Developers
- **Version control**: One file instead of many
- **Distribution**: Ship docs as single compressed file
- **Archival**: Complete documentation snapshots
- **Search**: Grep entire documentation at once

### For Teams
- **Consistency**: Shared vocabulary across all docs
- **Updates**: See what changed in aggregate diff
- **Reviews**: Review all documentation changes at once
- **Standards**: Enforce terminology through tokens

## Advanced Features

### 1. Incremental Updates
```bash
# Update only changed files
mq aggregate --update smart-tree.mq

# Show what would change
mq aggregate --dry-run smart-tree.mq
```

### 2. Semantic Grouping
```bash
# Group by document type
mq aggregate --group-by type

Structure:
::group:tutorials::
  - docs/tutorial/*.md
::group:api::
  - docs/api/*.md
::group:guides::
  - docs/guides/*.md
```

### 3. Cross-Reference Detection
```bash
# Analyze cross-references
mq analyze smart-tree.mq --cross-refs

Output:
🔗 Cross-references found:
- README.md → docs/INSTALL.md (3 references)
- docs/API.md → examples/*.md (12 references)
```

## Example Use Cases

### 1. Project Documentation
```bash
# Create for entire project
cd my-project
mq aggregate . -o my-project-docs.mq

# Size comparison
du -h docs/        # 2.5MB
du -h my-project-docs.mq  # 125KB (95% reduction!)
```

### 2. AI Context Loading
```javascript
// Load entire project documentation
const docs = await mq.extract('smart-tree.mq');
const context = await ai.createContext(docs);

// Or load specific section
const api = await mq.extractFile('smart-tree.mq', 'docs/API.md');
```

### 3. Documentation Distribution
```bash
# Include in releases
mq aggregate . -o docs-v3.2.0.mq
gh release upload v3.2.0 docs-v3.2.0.mq

# Users download single file
curl -L https://github.com/org/project/releases/download/v3.2.0/docs-v3.2.0.mq
```

## Future Enhancements

1. **Smart Chunking**: Split large aggregates by topic
2. **Language Support**: Aggregate code comments too
3. **Binary Assets**: Include images as base64
4. **Versioning**: Track changes between aggregates
5. **Federation**: Combine multiple project aggregates

## The Vision

Every project ships with a `[ProjectName].mq` that contains:
- All markdown documentation
- Cross-file token optimization
- Perfect compression
- Instant AI consumption

*"From forest of files to quantum singularity - all your docs in 8KB!"* 🎸✨