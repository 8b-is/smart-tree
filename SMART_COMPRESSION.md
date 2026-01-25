# 🗜️ Smart Tree Global Compression System

## Overview

Smart Tree now features an intelligent, token-aware global compression system that automatically handles large outputs across the entire project. This ensures we NEVER exceed token limits while maintaining compatibility with all AI assistants.

## How It Works

### 1. Automatic Client Detection
When an MCP client connects, Smart Tree:
- Sends a small compressed test message in the initialization response
- Checks if the client acknowledges compression support
- Remembers the client's capability for the entire sessio√

```json
// Initialization response includes:
{
  "serverInfo": {
    "compression_test": "COMPRESSED_V1:...",
    "_compression_hint": "If you can decompress this, reply with compression:ok"
  }
}
```

### 2. Smart Compression Triggers
The system automatically compresses when:
- Output exceeds 20,000 tokens (estimated)
- Client has confirmed compression support
- Not explicitly disabled by environment variables

### 3. Global Application
Compression works everywhere:
- **analyze** commands (semantic, quantum-semantic, etc.)
- **find** operations on large codebases
- **search** results with many matches
- **overview** of massive projects
- ALL MCP tool responses

## Token Awareness

The compression manager estimates tokens using:
- 1 token ≈ 4 characters (rough estimate)
- 20,000 token threshold (keeps under 25k MCP limit)
- Automatic compression when threshold exceeded

## Compression Formats

### Standard Compression (COMPRESSED_V1)
- **Format**: `COMPRESSED_V1:<hex-encoded-zlib-data>`
- **Ratio**: Typically 70-90% reduction
- **Use**: Automatic for large outputs

### Quantum Compression (QUANTUM_BASE64)
- **Format**: `QUANTUM_BASE64:<base64-encoded-binary>`
- **Ratio**: 90-95% reduction
- **Use**: For quantum and quantum-semantic modes

## Configuration

### Environment Variables
```bash
# Disable all compression
export MCP_NO_COMPRESS=1

# Force compression always
export ST_FORCE_COMPRESS=1

# Set custom token limit (default: 20000)
export ST_MAX_TOKENS=15000
```

### Feature Flags
```toml
# In features.toml
[compression]
max_tokens = 20000
force_compression = false
disable_compression = false
```

## Client Compatibility

### Supported Clients
Clients that decompress automatically:
- Claude Desktop (with MCP support)
- Cursor (latest versions)
- VS Code with AI extensions
- Custom MCP implementations

### Fallback Behavior
If client doesn't support compression:
- Smart Tree detects this automatically
- Falls back to uncompressed output
- Warns about potential token limits
- Suggests using quantum modes

## Usage Examples

### Automatic Compression
```bash
# Large semantic analysis - auto-compresses if needed
analyze {mode:'semantic', path:'./huge-project'}

# Client sees compressed output only if they support it
# Otherwise, gets truncated warning
```

### Force Compression
```bash
# Always compress (useful for huge outputs)
analyze {mode:'semantic', compress:true}

# Or use quantum mode for maximum compression
analyze {mode:'quantum-semantic'}
```

## Statistics & Monitoring

The compression manager tracks:
- Total compressions performed
- Bytes saved
- Estimated tokens saved
- Failed decompressions

View stats with:
```bash
st --compression-stats
```

## Benefits

### For Users
- ✅ Never hit token limits
- ✅ Analyze massive codebases (like Burn!)
- ✅ Get complete results, not truncated
- ✅ Automatic - no manual configuration needed

### For AI Assistants
- ✅ More context in fewer tokens
- ✅ Complete project understanding
- ✅ Efficient token usage
- ✅ Automatic decompression (if supported)

### For Developers
- ✅ Global solution - works everywhere
- ✅ Smart detection - no breaking changes
- ✅ Token-aware - respects limits
- ✅ Statistics for optimization

## Implementation Details

### Compression Flow
1. **Request arrives** → Check for compression acknowledgment
2. **Process request** → Generate response
3. **Check response size** → Estimate tokens
4. **Apply compression** → If client supports & size exceeds limit
5. **Send response** → With compression metadata

### Compression Algorithm
- **Library**: zlib (flate2)
- **Level**: Default (balanced speed/ratio)
- **Encoding**: Hex for text safety
- **Overhead**: ~50 bytes for metadata

## Troubleshooting

### "Token limit exceeded" errors
- Client doesn't support compression
- Solution: Use `mode:'quantum-semantic'` explicitly

### Garbled output
- Client trying to display compressed data as text
- Solution: Update client or disable compression

### Performance issues
- Very large outputs being compressed
- Solution: Use streaming or pagination

## Future Enhancements

1. **Streaming compression** - Compress chunks as they generate
2. **Adaptive compression** - Adjust level based on content
3. **Client negotiation** - Formal compression capability exchange
4. **Differential compression** - Only send changes

## Example: Analyzing Burn Project

```bash
# Before (would fail with token limit):
analyze {mode:'semantic', path:'../burn'}
# Error: MCP tool "analyze" response (44326 tokens) exceeds maximum

# After (with smart compression):
analyze {mode:'semantic', path:'../burn'}
# ✅ Auto-compressed: 177304 → 18234 bytes (89.7% reduction)
# 💡 Estimated tokens saved: 39842
# Success! Full analysis delivered
```

## Summary

Smart Tree's global compression system ensures that:
- **Token limits are NEVER exceeded** when clients support compression
- **Compression is automatic** - no user configuration needed
- **Backward compatible** - non-supporting clients still work
- **Global coverage** - all tools benefit from compression

"Compression so smart, it knows when to squeeze!" - Aye

"Your massive codebase? We've got it covered!" - Hue