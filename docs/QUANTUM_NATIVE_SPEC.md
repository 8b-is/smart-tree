# Quantum Native Format Specification

## The Vision

Smart Tree's quantum scanner emits quantum format **natively** during tree traversal. No intermediate representation, no post-processing - just pure, efficient quantum output as we walk the filesystem.

## Architecture

```
Filesystem → Quantum Scanner → Quantum Stream → [Decoders] → Other Formats
                    ↓
              Direct Output
              (no buffering)
```

## Token Architecture

### Token ID Space (u16: 0x0000 - 0xFFFF)

```
0x0000-0x00FF: Reserved System Tokens
  0x0001-0x000F: Node types (dir, file, link, etc.)
  0x0010-0x001F: Common permissions
  0x0020-0x007F: Common extensions
  0x0080-0x00FF: Common directory names
  0x00A0-0x00AF: Size ranges
  0x00B0-0x00BF: Semantic patterns
  0x00C0-0x00FF: Reserved

0x0100-0xFFFF: Dynamic User Tokens
  - Created on-the-fly for frequently seen patterns
  - Transmitted in header for decoder sync
```

### Semantic Tokenization Examples

```
Before: package.json → After: [TOKEN: pkg.manifest]
Before: node_modules → After: [TOKEN: pkg.node_modules]
Before: 0o755       → After: [TOKEN: perm.default_dir]
Before: 1024 bytes  → After: [TOKEN: size.small] + [8-bit: 4] (4*256 = 1024)
```

## Stream Format

### Header Section
```
QUANTUM_NATIVE_V1:
TOKENS:
  <token_id>=<pattern>
  ...
DATA:
```

### Data Section (Binary)
Each entry: `[header_byte][data_fields][name][traversal_code]`

### Header Byte Encoding
```
Bit 7: Tokenized name follows
Bit 6: Has extended attributes
Bit 5: Is symbolic link
Bit 4: Is directory
Bit 3: Owner/group differ from parent
Bit 2: Time differs from parent
Bit 1: Permissions differ from parent
Bit 0: Has size field
```

### Traversal Codes
- `0x0B` (VT): Same level
- `0x0E` (SO): Go deeper (enter directory)
- `0x0F` (SI): Go back (exit directory)
- `0x0C` (FF): Summary follows

## Compression Advantages

1. **No Redundancy**: Each piece of information appears exactly once
2. **Delta Encoding**: Only differences from parent context
3. **Semantic Tokens**: Common patterns become single bytes
4. **Direct Streaming**: No memory overhead for large trees
5. **SIMD-Friendly**: Aligned data for vector processing

## Example Encoding

```
Directory: src/main.rs (755, 1234 bytes)
```

Traditional JSON (46 bytes):
```json
{"name":"src/main.rs","size":1234,"mode":755}
```

Quantum Native (8 bytes):
```
[Header: 0x11] [Size: 0x00 0xD2 0x04] [Token: 0x82] [Token: 0x91] [Traverse: 0x0B]
```

**Compression ratio: 83%**

## Decoder Architecture

Other formats are implemented as decoders from quantum:

```rust
trait QuantumDecoder {
    fn decode_entry(&mut self, quantum_entry: &[u8]) -> Result<()>;
}

struct JsonDecoder { ... }
struct ClassicDecoder { ... }
struct HexDecoder { ... }
```

## Future Optimizations

1. **Huffman Coding**: For non-tokenized strings
2. **Run-Length Encoding**: For similar entries
3. **Dictionary Building**: Dynamic token creation
4. **Parallel Processing**: SIMD operations on token streams
5. **Memory Mapping**: Direct filesystem → quantum mapping

## Implementation Status

- [x] Basic quantum scanner
- [x] Static token map
- [x] Size encoding
- [x] Permission deltas
- [ ] Dynamic tokenization
- [ ] Decoder framework
- [ ] SIMD optimization
- [ ] Streaming compression

## Philosophy

"The best format is no format - just pure, semantic information flowing directly from the filesystem to the consumer. Everything else is just a view into this quantum stream."

- The Smart Tree Team