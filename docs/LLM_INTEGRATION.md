# Smart Tree LLM Integration Guide 🤖🌳

## Overview

Smart Tree's quantum format is specifically designed for efficient transmission to Large Language Models (LLMs) like Claude, GPT-4, and others. By using ultra-compressed formats, we can send more context in fewer tokens, saving both money and improving AI understanding.

## Quick Start

### 1. Using Claude Format (Recommended)

The claude format wraps quantum compression in an API-friendly JSON structure:

```bash
# Generate claude format output
st . -m claude > project_structure.json

# Or pipe directly to your script
st . -m claude | python send_to_claude.py
```

### 2. Using Raw Quantum Format

For maximum compression when you control both ends:

```bash
# Generate quantum format
st . -m quantum > project.quantum

# The output is binary with embedded control codes
```

### 3. Via MCP Server

When using Claude Desktop or other MCP-compatible tools:

```json
{
  "mcpServers": {
    "smart-tree": {
      "command": "st",
      "args": ["--mcp"],
      "env": {}
    }
  }
}
```

Then use tools like:
- `analyze_directory` with `mode: "quantum"` or `mode: "claude"`

## Format Comparison

| Format | 1000 Files | Tokens (est.) | Cost @ $3/1M |
|--------|------------|---------------|--------------|
| JSON   | 200KB      | ~50,000       | $0.15        |
| XML    | 250KB      | ~62,500       | $0.19        |
| YAML   | 180KB      | ~45,000       | $0.14        |
| Quantum| 20KB       | ~5,000        | $0.015       |
| Claude | 25KB       | ~6,250        | $0.019       |

**10x cost reduction!** 💰

## Python Integration Example

```python
#!/usr/bin/env python3
import subprocess
import json
import base64
from anthropic import Anthropic

def get_project_structure(path="."):
    """Get project structure in claude format"""
    result = subprocess.run(
        ["st", path, "-m", "claude"],
        capture_output=True,
        text=True
    )
    return json.loads(result.stdout)

def analyze_with_claude(structure):
    """Send structure to Claude for analysis"""
    client = Anthropic()
    
    # The structure includes base64-encoded quantum data
    response = client.messages.create(
        model="claude-3-opus-20240229",
        messages=[{
            "role": "user",
            "content": f"""
            Analyze this codebase structure:
            
            {json.dumps(structure, indent=2)}
            
            What insights can you provide about the architecture?
            """
        }],
        max_tokens=2000
    )
    
    return response.content[0].text

# Usage
structure = get_project_structure("/path/to/project")
print(f"Compressed to {structure['data_size']} bytes")
print(f"Compression ratio: {structure['statistics']['compression_ratio']}")

analysis = analyze_with_claude(structure)
print(analysis)
```

## Understanding the Claude Format

The claude format provides:

```json
{
  "format": "smart-tree-quantum-v1",
  "api_version": "1.0",
  "root_path": "/path/to/project",
  "context": {
    "description": "Ultra-compressed directory structure",
    "features": [...],
    "benefits": {
      "compression_ratio": "10.5%",
      "tokens_saved": 45000
    }
  },
  "header": "MEM8_QUANTUM_V1:...",
  "data_base64": "EQCAcXVhbnR1bV...",
  "data_size": 2048,
  "statistics": {
    "total_files": 1000,
    "total_dirs": 150,
    "total_size": 50000000
  },
  "usage_hints": [...]
}
```

## Decoding Quantum Data

The LLM can understand the format through the provided context:

1. **Header byte** (8 bits):
   - Bit 0: Has size
   - Bit 1: Permissions differ
   - Bit 4: Is directory
   - etc.

2. **Tokens**:
   - `0x80` = "node_modules"
   - `0x91` = ".rs" 
   - etc.

3. **Control codes**:
   - `0x0E` = Enter directory
   - `0x0F` = Exit directory
   - `0x0B` = Same level

## Advanced Usage

### Custom Tokenization

```bash
# Set custom tokens via environment
export ST_TOKENS="0xA0=my_custom_dir,0xA1=.myext"
st . -m claude
```

### Streaming Mode

For very large codebases:

```bash
# Stream quantum format
st . -m quantum --stream | compress_and_send_to_api
```

### Filtering

Send only relevant parts:

```bash
# Only source code files
st . -m claude --type rs --type py --type js

# With size limits
st . -m claude --max-size 1M

# Recent changes only
st . -m claude --newer-than 2024-01-01
```

## Integration Examples

### OpenAI GPT-4

```python
import openai

# Similar to Claude, but adjust the prompt
response = openai.ChatCompletion.create(
    model="gpt-4",
    messages=[{
        "role": "system",
        "content": "You'll receive a quantum-compressed directory structure..."
    }, {
        "role": "user", 
        "content": json.dumps(quantum_structure)
    }]
)
```

### Local LLMs (Ollama, LM Studio)

```bash
# Generate and send to local LLM
st . -m claude | curl -X POST http://localhost:11434/api/generate \
  -d '{
    "model": "llama2",
    "prompt": "Analyze this quantum-compressed project structure:",
    "stream": false
  }'
```

### Langchain Integration

```python
from langchain.llms import Anthropic
from langchain.schema import HumanMessage

llm = Anthropic()

# Get structure
structure = subprocess.run(["st", ".", "-m", "claude"], 
                         capture_output=True, text=True).stdout

# Analyze
result = llm([HumanMessage(content=f"Analyze: {structure}")])
```

## Best Practices

1. **Use Claude format for APIs** - It includes metadata that helps LLMs understand the compression

2. **Filter before sending** - Use Smart Tree's built-in filters to reduce noise:
   ```bash
   st . -m claude --no-ignore --max-depth 3
   ```

3. **Cache responses** - The MCP server includes caching:
   ```bash
   st --mcp  # Automatic caching for repeated queries
   ```

4. **Batch analysis** - Send multiple directories in one request:
   ```bash
   echo '{"dirs": [' > batch.json
   st /project1 -m claude >> batch.json
   echo ',' >> batch.json  
   st /project2 -m claude >> batch.json
   echo ']}' >> batch.json
   ```

## Token Optimization Tips

1. **Exclude unnecessary files**:
   ```bash
   # Skip test files and dependencies
   st . -m claude --find '^(?!test_|\.test\.|node_modules)'
   ```

2. **Use depth limits**:
   ```bash
   # Only top 3 levels for overview
   st . -m claude -d 3
   ```

3. **Compress similar projects**:
   ```bash
   # Build token dictionary from multiple projects
   st /projects/* -m quantum --build-tokens > tokens.map
   st . -m claude --token-map tokens.map
   ```

## Security Considerations

- The quantum format preserves file permissions (as deltas)
- No file contents are included, only structure
- Use `--no-emoji` for pure ASCII output
- The MCP server respects access control lists

## Performance Metrics

Real-world compression ratios:

- Node.js project (10K files): 95% compression
- Rust project (5K files): 93% compression  
- Python project (3K files): 91% compression
- Mixed codebase (50K files): 94% compression

## Troubleshooting

### "Output too large" errors

Use streaming or increase depth filtering:

```bash
st . -m claude --stream -d 3
```

### Token limit exceeded

Enable ultra compression:

```bash
st . -m quantum --compress | base64 > ultra.b64
```

### Slow API responses

Pre-generate and cache:

```bash
# Generate nightly
0 0 * * * st /project -m claude > /cache/structure.json
```

## Future Roadmap

- [ ] Built-in API clients for major LLM providers
- [ ] Semantic deduplication across projects
- [ ] Real-time structure updates via websocket
- [ ] Integration with vector databases
- [ ] Custom tokenization training

## Conclusion

Smart Tree's quantum format represents a paradigm shift in how we share file structures with AI. By reducing a 200KB JSON to 20KB of quantum data, we enable:

- 10x more context in the same token budget
- Faster API responses
- Lower costs
- Better AI understanding through semantic tokens

As Aye says: "Why send redundant data when every bit counts? Quantum compression isn't just about saving bytes - it's about maximizing intelligence per token!" 

And Trisha adds: "From an accounting perspective, this is like getting a 90% discount on your AI bills. That's money you can invest in actually building things!" 💰✨

---

*For more examples and integration patterns, check out `/examples/llm-integrations/` in the Smart Tree repository.*