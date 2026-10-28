# Quantum Format JSON Safety

## The Problem

The quantum format uses binary data with ASCII control codes (0x0E, 0x0F, 0x0B) that aren't valid UTF-8. This causes issues when transmitting through JSON-based protocols like:
- MCP (Model Context Protocol)
- REST APIs
- WebSocket messages
- Any JSON-based transport

## The Solution

We've implemented automatic base64 encoding for quantum format when used through MCP:

```rust
// In mcp/tools.rs
if args.mode == "quantum" {
    // Quantum format contains binary data, so base64-encode it for JSON safety
    format!("QUANTUM_BASE64:{}", base64::encode(&output))
}
```

## How It Works

### 1. Normal CLI Usage (Binary Output)
```bash
st . -m quantum > output.quantum
# Raw binary output with control codes
```

### 2. MCP Usage (Base64-Encoded)
When requesting quantum format through MCP:
```json
{
  "tool": "analyze_directory",
  "arguments": {
    "path": ".",
    "mode": "quantum"
  }
}
```

Returns:
```
QUANTUM_BASE64:TUVNOF9RVUFOVFVNX1YxOgpLRVk6SFNTUw...
```

### 3. Decoding Base64 Quantum

Use the provided decoder:
```bash
# From MCP output
echo "QUANTUM_BASE64:..." | python3 tools/decode-quantum-base64.py | python3 tools/quantum-decode.py

# Or save and decode
echo "QUANTUM_BASE64:..." > quantum.b64
python3 tools/decode-quantum-base64.py quantum.b64
python3 tools/quantum-decode.py quantum_decoded.bin
```

## Alternative: Claude Format

For API usage, the claude format is recommended as it provides:
- JSON-safe structure
- Base64-encoded quantum data
- Metadata and statistics
- Usage hints for LLMs

```bash
st . -m claude  # Recommended for APIs
```

## Implementation Details

### Why Base64?

1. **JSON Compatibility**: JSON strings must be valid UTF-8
2. **Preserves Binary**: No data loss from encoding
3. **Standard Format**: Widely supported across languages
4. **Reasonable Overhead**: ~33% size increase

### Performance Impact

- Original quantum: 2KB
- Base64 encoded: 2.7KB
- Still 85% smaller than JSON!

## Usage Examples

### Python
```python
import base64
import subprocess
import json

# Get quantum format via MCP
result = subprocess.run(['st', '.', '-m', 'quantum'], capture_output=True)
if result.stdout.startswith(b'QUANTUM_BASE64:'):
    quantum_data = base64.b64decode(result.stdout[15:])
    # Now process the binary quantum data
```

### JavaScript
```javascript
// Decode base64 quantum format
const response = await fetch('/mcp/analyze_directory', {
  method: 'POST',
  body: JSON.stringify({ path: '.', mode: 'quantum' })
});
const data = await response.text();
if (data.startsWith('QUANTUM_BASE64:')) {
  const quantum = atob(data.slice(15));
  // Process binary data
}
```

## Troubleshooting

### "Invalid UTF-8" Error
If you see this error, ensure:
1. The MCP server is running the latest version
2. The quantum formatter is using base64 encoding
3. Restart the MCP server after updates

### Decoding Issues
1. Check the prefix is exactly `QUANTUM_BASE64:`
2. Ensure no whitespace in base64 data
3. Use appropriate base64 decoder for your language

## Future Improvements

1. **Protocol Buffers**: Binary-safe alternative to JSON
2. **MessagePack**: More efficient than JSON, binary-safe
3. **Native Binary MCP**: Extend MCP to support binary responses
4. **Streaming Base64**: For very large outputs

## Summary

The quantum format's binary nature requires special handling for JSON protocols. We solve this with automatic base64 encoding in MCP while preserving raw binary output for CLI usage. This maintains the format's extreme efficiency while ensuring compatibility with modern APIs.

As Aye says: "Sometimes you need to wrap your quantum particles in a JSON-safe container. It's like putting a wild tiger in a cage - still powerful, just transport-friendly!" 🐅📦