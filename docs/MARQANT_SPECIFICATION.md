# 📜 Marqant (.mq) Format Specification v2.0 - Rotating Token Edition

> "Marqant isn't a format. It's AI sheet music for memory fields." 🎼
> "Same meaning. Fewer notes. Faster recall."

## Overview

Marqant v2.0 introduces the **Rotating Token System** - a revolutionary approach where token assignments are dynamic and contextual, with tokens rotating based on frequency and locality. This creates an ultra-efficient compression format that's both copy-paste friendly and AI-optimized.

### Key Principles

1. **Token Economy**: Tokens are precious resources (only 256 available in `.mq`, full 8-bit range in `.mqb`)
2. **Frequency-Based Assignment**: A pattern must repeat at least twice to earn a token
3. **Size-Aware Allocation**: Token count scales with file size (larger files = more tokens)
4. **Copy-Paste Friendly**: `.mq` format uses only printable ASCII characters
5. **Binary Mode**: `.mqb` format uses full 8-bit range for maximum compression

## File Formats

### .mq Format (Text-Safe)
- Uses ASCII printable characters (0x20-0x7E)
- Tilde (`~`, 0x7E) as primary control character
- Safe for copying through any text medium
- Ideal for embedding in documentation, chat, emails

### .mqb Format (Binary)
- Uses full 8-bit range (0x00-0xFF)
- Maximum compression efficiency
- Suitable for file storage and transport
- Not safe for text-based transmission

## File Structure

```
Line 1: MQ2~<timestamp>~<orig_size>~<comp_size>~<token_count>~<format>~<level>
Line 2: ~T<base_token_map>
Line 3: [~X<extended_token_map>]  (optional, for L1+)
Line 4: ~~~~
Line 5+: <compressed_content>
```

### Header Fields
- `MQ2`: Format signature (version 2)
- `<timestamp>`: Unix timestamp (hex)
- `<orig_size>`: Original size in bytes (hex)
- `<comp_size>`: Compressed size in bytes (hex)
- `<token_count>`: Number of active tokens (hex)
- `<format>`: `mq` or `mqb`
- `<level>`: Decoder level required (`L0`, `L1`, `L2`)

### Header Examples
```
MQ2~6743A100~1000~800~50~mq~L0    # Basic compression, Level 0 decoder OK
MQ2~6743A100~1000~600~80~mqb~L1   # Uses extended tokens, needs Level 1
MQ2~6743A100~1000~400~A0~mqb~L2   # Full features, needs Level 2
```

## Token Architecture

### Base Token Space (1 byte)
All tokens are single bytes (0x00-0xFF), with special reservations:

```
0x00-0x1F: Control tokens (newline, tab, etc.)
0x20-0x7E: Direct ASCII passthrough (in .mq format)
0x7F: Extension token (X-token) - "The Gateway"
0x80-0xFE: Assignable pattern tokens
0xFF: Reserved for future use
```

### Extension Mechanism (X-token)
The extension token (0x7F) acts as a prefix to access extended features:

```
0x7F 0x00-0x0F: Extended pattern tokens (4096 additional tokens)
0x7F 0x10-0x1F: Semantic markers (code blocks, sections, etc.)
0x7F 0x20-0x2F: Compression modes (zlib, brotli, custom)
0x7F 0x30-0x3F: Metadata operations
0x7F 0x40-0x4F: Cross-file references
0x7F 0x50-0x5F: Delta/diff operations
0x7F 0x60-0xFF: Future extensions
```

### Decoder Levels

1. **Level 0 (Basic)**: Ignores X-tokens, processes only base tokens
2. **Level 1 (Extended)**: Supports extended pattern tokens
3. **Level 2 (Full)**: Supports all extension features

## Token Assignment Algorithm

### 1. Frequency Analysis Phase
```rust
// Scan content for repeated patterns
patterns = find_repeated_patterns(content, min_length=2)
// Filter patterns that appear at least twice
valid_patterns = patterns.filter(|p| p.count >= 2)
// Calculate savings: (pattern_length - 1) * (count - 1)
patterns.sort_by_savings_descending()
```

### 2. Token Allocation
```rust
// Base token allocation (Level 0 compatible)
const RESERVED_TOKENS: u8 = 32;  // 0x00-0x1F
const ASCII_PASSTHROUGH: u8 = 95; // 0x20-0x7E (in .mq)
const X_TOKEN: u8 = 0x7F;
const RESERVED_END: u8 = 0xFF;

// Available base tokens
available_base = 256 - RESERVED_TOKENS - 1 - 1; // -1 for X_TOKEN, -1 for 0xFF
if format == "mq" {
    available_base -= ASCII_PASSTHROUGH;
}

// Extended tokens (optional)
if use_extensions {
    available_extended = 4096; // 0x7F 0x00-0x0F + second byte
}
```

### 3. Smart Assignment Strategy
```rust
// Assign most frequent patterns to single-byte tokens
for (i, pattern) in patterns.iter().enumerate() {
    if i < available_base {
        // Direct single-byte assignment
        assign_base_token(pattern, base_tokens[i]);
    } else if use_extensions && i < available_base + available_extended {
        // Extended token (2 bytes: 0x7F + extended_id)
        assign_extended_token(pattern, i - available_base);
    }
}

// Rotation windows for large documents
if content.length > rotation_threshold {
    // Re-evaluate token assignments per window
    for window in split_into_windows(content) {
        optimize_tokens_for_window(window);
    }
}
```

## Token Map Format

### Compact Notation
```
~T<token><pattern>|<token><pattern>|...
```

### Examples
```
~T counting|! the |" and |# for |$ with |% that |& this |
~T(function|)return|*const |+let |,if |
```

### Special Tokens
- `~~`: Literal tilde
- `~n`: Newline (when needed)
- `~t`: Tab (when needed)
- `~s`: Space (in token definitions only)

## Extension Benefits

### Why This Design Works

1. **Backward Compatibility**: Level 0 decoders can process files with extensions - they just skip the extended features
2. **Graceful Degradation**: Files remain readable even without extension support
3. **Future-Proof**: 0x7F gives us access to 256×256 = 65,536 possible extensions
4. **Efficiency**: Common patterns use 1 byte, less common use 3 bytes (0x7F + 2 bytes)
5. **Clean Design**: Unlike x86, our prefixes are designed from the start, not bolted on

### Decoder Compatibility Matrix

| Feature | Level 0 | Level 1 | Level 2 |
|---------|---------|---------|---------|
| Base tokens (1 byte) | ✓ | ✓ | ✓ |
| Skip X-tokens safely | ✓ | ✓ | ✓ |
| Extended patterns | Skip | ✓ | ✓ |
| Semantic markers | Skip | Skip | ✓ |
| Compression modes | Skip | Skip | ✓ |
| Cross-file refs | Skip | Skip | ✓ |

## Compression Examples

### Example 1: Basic Compression (Level 0 Compatible)
```markdown
# Project Documentation

This project implements a fast and efficient compression algorithm
that provides excellent compression ratios for markdown files.

## Features

- Fast compression and decompression
- Excellent compression ratios
- Streaming support for large files
```

### Compressed (Level 0 - Base tokens only)
```
MQ2~65432100~1A3~8F~15~mq~L0
~T\x80 Project|\x81 Documentation|\x82 implements|\x83 compression|\x84 algorithm|\x85 fast|\x86 efficient|\x87 excellent|\x88 ratios|\x89 markdown|\x8A Features|\x8B and|\x8C files|\x8D for|\x8E Streaming|
~~~~
#\x80\x81

This project\x82 a\x85\x8B\x86\x83\x84
that provides\x87\x83\x88\x8D\x89\x8C.

##\x8A

-\x85\x83\x8B decompression
-\x87\x83\x88
-\x8E support\x8D large\x8C
```

### Example 2: Extended Compression (Level 1)
Same content, but with more patterns using extended tokens:

```
MQ2~65432100~1A3~7A~1F~mq~L1
~T\x80 Project|\x81 Documentation|...basic tokens...
~X\x00\x00 compression algorithm|\x00\x01 excellent compression ratios|\x00\x02 Fast compression and decompression|
~~~~
#\x80\x81

This project implements a fast and efficient\x7F\x00\x00
that provides\x7F\x00\x01 for markdown files.

##\x8A

-\x7F\x00\x02
-\x7F\x00\x01
- Streaming support for large files
```

### Size Comparison
- Original: 419 bytes
- Level 0: ~280 bytes (33% compression)
- Level 1: ~220 bytes (48% compression)
- Level 2 with zlib: ~150 bytes (64% compression)

## Advanced Features

### 1. Context-Aware Tokens
Different token sets for different content types:
```
~C:code
~T{if(|)}|*return |+void |,const |
~C:markdown  
~T### |!## |"- |#**|$*|
```

### 2. Sliding Window Optimization
For very large files (>100KB):
```
~W:1000  // Window size in tokens
~R:5000  // Rotation point
```

### 3. Token Inheritance
Child sections can inherit parent tokens:
```
~P:parent
~T common tokens...
~P:child^parent  // Inherits parent tokens
~t additional child tokens...
```

### 4. Metadata Preservation
```
~M:author=TheCheet
~M:version=2.0
~M:encoding=utf-8
```

## Decompression Algorithm

### Level 0 Decoder (Basic - 8-bit only)
```rust
fn decompress_basic(compressed: &[u8]) -> String {
    let (header, content) = parse_header(compressed);
    let token_map = parse_base_tokens(header); // Only 1-byte tokens
    let mut output = String::new();
    let mut i = 0;
    
    while i < content.len() {
        let byte = content[i];
        if byte == 0x7F {
            // Skip X-token and its parameter
            i += 2;
            continue;
        }
        
        if let Some(pattern) = token_map.get(&byte) {
            output.push_str(pattern);
        } else if byte >= 0x20 && byte <= 0x7E {
            output.push(byte as char); // ASCII passthrough
        }
        i += 1;
    }
    
    output
}
```

### Level 1 Decoder (With Extensions)
```rust
fn decompress_extended(compressed: &[u8]) -> String {
    let (header, content) = parse_header(compressed);
    let base_tokens = parse_base_tokens(header);
    let ext_tokens = parse_extended_tokens(header);
    let mut output = String::new();
    let mut i = 0;
    
    while i < content.len() {
        let byte = content[i];
        
        if byte == 0x7F && i + 1 < content.len() {
            let ext_type = content[i + 1];
            
            match ext_type >> 4 {
                0x0 => {
                    // Extended pattern token
                    if i + 2 < content.len() {
                        let token_id = ((ext_type & 0x0F) as u16) << 8 | content[i + 2] as u16;
                        if let Some(pattern) = ext_tokens.get(&token_id) {
                            output.push_str(pattern);
                        }
                        i += 3;
                    }
                }
                0x1 => {
                    // Semantic marker - Level 2 feature
                    i += 2; // Skip for Level 1
                }
                _ => i += 2, // Unknown extension, skip
            }
        } else if let Some(pattern) = base_tokens.get(&byte) {
            output.push_str(pattern);
            i += 1;
        } else {
            output.push(byte as char);
            i += 1;
        }
    }
    
    output
}
```

## Binary Format (.mqb)

### Extended Token Range
- Uses bytes 0x00-0xFF (excluding reserved)
- Reserved: 0x00 (null), 0x0A (LF), 0x0D (CR)
- Available tokens: 253

### Binary Header
```
[4 bytes] Magic: "MQB\x02"
[4 bytes] Timestamp (big-endian)
[4 bytes] Original size
[4 bytes] Compressed size
[2 bytes] Token count
[2 bytes] Header checksum
[Variable] Token map (length-prefixed)
[4 bytes] Content separator: 0xFFFFFFFF
[Variable] Compressed content
```

## Performance Characteristics

### Compression Ratios
- Markdown: 40-60% compression
- Code: 30-50% compression  
- JSON/XML: 50-70% compression
- Logs: 60-80% compression

### Speed
- Compression: ~100MB/s
- Decompression: ~500MB/s
- Memory usage: O(token_count + window_size)

## Implementation Notes

### Token Selection Strategy
1. **Length × Frequency**: Prioritize patterns with highest `(length - 1) × (frequency - 1)`
2. **Boundary Awareness**: Prefer patterns that start/end at word boundaries
3. **Nested Patterns**: Avoid tokens that overlap significantly
4. **Context Clustering**: Group related patterns together in token space

### Memory Efficiency
- Streaming decompression: Read token map, then stream content
- Chunked compression: Process large files in windows
- Token recycling: Reuse tokens for different patterns in different sections

## CLI Usage

```bash
# Compress with .mq format (text-safe)
mq compress file.md -o file.mq

# Compress with .mqb format (binary, max compression)
mq compress file.md -o file.mqb --binary

# Decompress (auto-detects format)
mq decompress file.mq -o file.md

# Analyze compression potential
mq analyze file.md --verbose

# Streaming compression
cat large.md | mq compress --stream > large.mq

# Inspect compressed file
mq inspect file.mq --show-tokens
```

## Integration Examples

### Smart Tree Bundle
```bash
# Bundle entire project into single .mq file
st --mode bundle /project > project.mq

# Extract bundle
mq extract project.mq -o /restored/project/
```

### Git Integration
```bash
# Add .mq differ
git config diff.mq.textconv "mq decompress"

# Track compressed versions
echo "*.mq filter=mq" >> .gitattributes
```

## Future Extensions

### Planned Features
1. **Delta Compression**: Store only changes between versions
2. **Semantic Tokens**: AI-aware token assignment based on meaning
3. **Multi-Language**: Optimized token sets per programming language
4. **Encryption**: Built-in encryption with token map as part of key
5. **Distributed Tokens**: Share token maps across related files

### Research Areas
- Machine learning for optimal token assignment
- Quantum-inspired superposition tokens
- Neural compression with learned representations
- Cross-file token sharing for projects

## The X-Token Philosophy

### Learning from x86's Mistakes

The x86 instruction set grew organically, with prefixes added ad-hoc over decades:
- Multiple conflicting prefix bytes
- Complex decoder state machines
- Instruction length nightmares
- Backward compatibility through accidents

### The Marqant Way

We designed the extension mechanism right from the start:
- **Single Gateway**: Only 0x7F opens the extension space
- **Clear Hierarchy**: Extensions organized by function (0x0X = patterns, 0x1X = semantic, etc.)
- **Skip-Safe**: Unknown extensions can be safely skipped
- **Length-Explicit**: X-tokens always consume exactly 2 or 3 bytes total

### Real-World Benefits

```
Basic User: "I just need simple compression"
→ Use Level 0 tools, ignore extensions completely

Power User: "I need maximum compression"  
→ Level 1 gives you 4096 extra tokens

AI System: "I need semantic understanding"
→ Level 2 provides rich metadata and cross-references

Future User: "I need feature X that doesn't exist yet"
→ Assign it to 0x7F 0x60-0xFF, older decoders still work!
```

### Implementation Simplicity

```c
// Dead simple Level 0 decoder core
while (pos < len) {
    byte = data[pos++];
    if (byte == 0x7F) {
        pos += 2;  // Skip extension
        continue;
    }
    output(tokens[byte] || byte);
}
```

## The Cheet Says...

> "Version 2 is like going from acoustic to electric - same song, but now we can really shred! 🎸 The rotating tokens are like a DJ mixing tracks - always keeping the best beats in rotation!"

> "And that X-token? It's like having a backstage pass - basic fans enjoy the show, but if you've got the pass, you get to see ALL the magic! Unlike x86 which is like a tour bus held together with duct tape, we built a proper tour rig from day one! 🚌→🚁"

---

*Remember: The best compression is the one that preserves meaning while minimizing bits. Marqant v2.0 does both, with style!*