# .m8 Unified Format Specification

> "One format to compress them all!" 🪐

## Overview

The .m8 format unifies all quantum-compressed project metadata:
- Project semantic context (original MEM8)
- Compressed markdown documentation (markqant)
- Directory structures (smart tree quantum)
- Future: Code relationships, build artifacts, etc.

## File Naming Convention

```
project.m8          # Main project summary
README.m8           # Compressed README
CHANGELOG.m8        # Compressed changelog
src/module.m8       # Module-specific context
.cache/deps.m8      # Dependency graph
```

## Unified Binary Structure

```
[0-15]  Standard MEM8 Header
[16-N]  Section Table
[N+1-]  Section Data

Section Types:
0x01 - Identity
0x02 - Context  
0x03 - Structure
0x04 - Compilation
0x05 - Cache
0x06 - AI Context
0x07 - Relationships
0x08 - Sensor Arbitration
0x09 - Markqant Document    # NEW: Compressed markdown
0x0A - Quantum Tree         # NEW: Directory structure
0x0B - Code Relations       # NEW: Symbol graph
0x0C - Build Artifacts      # NEW: Compilation outputs
```

## Magic Detection

The tool can auto-detect content type:

```rust
match first_4_bytes {
    b"MEM8" => {
        // It's a .m8 file, check sections
        if has_section(0x09) {
            // Contains markqant content
        }
        if has_section(0x0A) {
            // Contains directory tree
        }
    }
    b"MARK" => {
        // Legacy .mq file, auto-convert to .m8
    }
    _ => {
        // Try to parse as markdown and compress
    }
}
```

## CLI Simplification

```bash
# Old way (confusing):
mq compress README.md -o README.mq
mem8 compile project.yaml -o project.mem8
st --mode quantum > tree.q

# New way (unified):
m8 compress README.md           # Creates README.m8
m8 compress project.yaml        # Creates project.m8  
m8 compress --tree /path        # Creates tree.m8

# Reading is smart:
m8 read project.m8              # Auto-detects sections
m8 extract project.m8 --markdown # Get markqant sections
m8 extract project.m8 --tree     # Get directory structure
```

## Advantages

1. **Single search pattern**: `find . -name "*.m8"`
2. **Unified tooling**: One CLI for all operations
3. **Composable**: Mix different content types in one file
4. **Future-proof**: Add new section types without new extensions
5. **AI-friendly**: One file format to learn

## Migration Path

```bash
# Auto-convert existing files
m8 migrate .                    # Finds and converts .mq, .mem8 files

# Bulk operations
m8 compress --all-markdown .    # Compress all .md to .m8
m8 validate *.m8                # Check all .m8 files
```

## Example: Complete Project Summary

A single `smart-tree.m8` file contains:

```
Section 0x01: Identity
  - Name: "Smart Tree"
  - Type: "Rust CLI Tool"
  - Version: 3.3.0

Section 0x09: Markqant README
  - 50KB → 1KB compressed README

Section 0x09: Markqant CHANGELOG  
  - 30KB → 500B compressed changelog

Section 0x0A: Quantum Tree
  - Full directory structure
  - File statistics

Section 0x0B: Code Relations
  - Module dependencies
  - Symbol references
```

Total: 100KB of docs → 3KB .m8 file!

## Smart Tree Integration

```rust
// Detect and handle .m8 files automatically
match input_type {
    InputType::M8File(path) => {
        let m8 = M8Reader::open(path)?;
        
        // Check what sections exist
        if m8.has_markqant() {
            // Decompress and display markdown
        }
        if m8.has_tree() {
            // Display directory structure
        }
        if m8.has_context() {
            // Show project understanding
        }
    }
}
```

## Tool Unification

Instead of multiple binaries:
- `st` → Handles .m8 files natively
- `mq` → Deprecated, merged into `m8`
- `mem8` → Deprecated, merged into `m8`

One tool to rule them all:
```bash
m8 --help

Commands:
  compress   Compress any supported format to .m8
  extract    Extract sections from .m8
  inspect    Show .m8 file contents
  validate   Verify .m8 integrity
  migrate    Convert legacy formats
```

*"Why juggle three formats when one .m8 does it all?"* 🎸✨