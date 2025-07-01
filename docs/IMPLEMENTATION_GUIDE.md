# Implementation Guide: Creating Your Own Auto-Updating DXT

This guide walks you through adapting the Smart Tree Universal example for your own tool.

## Prerequisites

- A tool with pre-built binaries for multiple platforms
- A GitHub repository with releases
- Basic knowledge of Node.js
- Claude Desktop for testing

## Step-by-Step Implementation

### 1. Fork and Initialize Your Project

```bash
# Clone this example
git clone [this-example-url] my-tool-dxt
cd my-tool-dxt

# Remove the example's git history
rm -rf .git

# Initialize your own repository
git init
git add .
git commit -m "Initial commit: DXT package for my-tool"
```

### 2. Update Package Identity

Edit `manifest.json`:

```json
{
  "id": "com.mycompany.my-tool",
  "name": "my-tool",
  "display_name": "My Tool",
  "version": "1.0.0",
  "description": "Brief description of your tool",
  "author": {
    "name": "Your Name",
    "email": "your.email@example.com",
    "url": "https://yourwebsite.com"
  },
  "repository": {
    "type": "git",
    "url": "https://github.com/yourusername/my-tool"
  }
}
```

### 3. Configure Binary Distribution

Edit `server/install.js`:

```javascript
// Update these constants
const REPO = 'yourusername/my-tool';  // Your GitHub repo
const BINARY_NAME = 'my-tool';         // Your binary name

// Update platform mappings if your naming differs
const mapping = {
    'darwin-arm64': 'aarch64-apple-darwin',
    'darwin-x64': 'x86_64-apple-darwin',
    'linux-x64': 'x86_64-unknown-linux-gnu',
    'win32-x64': 'x86_64-pc-windows-msvc',
};
```

### 4. Define Your MCP Tools

In `manifest.json`, replace the tools array with your tool's capabilities:

```json
"tools": [
  {
    "name": "my_first_tool",
    "description": "Description of what this tool does",
    "input_schema": {
      "type": "object",
      "properties": {
        "param1": {
          "type": "string",
          "description": "Description of parameter"
        }
      },
      "required": ["param1"]
    }
  }
]
```

### 5. Set Up GitHub Releases

Create a GitHub Actions workflow (`.github/workflows/release.yml`):

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
    
    runs-on: ${{ matrix.os }}
    
    steps:
    - uses: actions/checkout@v4
    
    # Build your binary here
    - name: Build
      run: |
        # Your build commands
        # cargo build --release --target ${{ matrix.target }}
        # go build -o my-tool
        # etc.
    
    # Package the binary
    - name: Package
      run: |
        # Create tar.gz for Unix, zip for Windows
        # Include just the binary, nothing else
```

### 6. Test Your Package Locally

```bash
# Build the DXT package
./build-dxt.sh

# Install in Claude Desktop
# 1. Open Claude Desktop
# 2. Settings → Developer
# 3. Install from file → Select your .dxt file
```

### 7. Handle Updates Gracefully

The auto-update system checks for updates but doesn't interrupt the user. Updates are installed on the next restart. You can customize this behavior in `server/index.js`.

## Platform-Specific Considerations

### Windows
- Binary name should include `.exe`
- Use `.zip` for distribution
- Handle path separators correctly

### macOS
- Support both Intel (x86_64) and Apple Silicon (aarch64)
- Code sign your binaries to avoid security warnings
- Use `.tar.gz` for distribution

### Linux
- Make binaries executable after download
- Consider different distributions and glibc versions
- Use `.tar.gz` for distribution

## Security Best Practices

1. **Always use HTTPS** for downloading binaries
2. **Add checksum verification** (optional but recommended):
   ```javascript
   // In your release process, generate checksums
   // In install.js, verify before executing
   ```

3. **Limit file system access** using environment variables:
   ```javascript
   env: {
     "ALLOWED_PATHS": "${user_config.allowed_directories}"
   }
   ```

4. **Validate all inputs** before passing to your binary

## Debugging Tips

1. **Enable debug logging**:
   ```bash
   DEBUG=1 node server/index.js
   ```

2. **Test binary installation**:
   ```bash
   node server/install.js
   ```

3. **Check Claude Desktop logs**:
   - macOS: `~/Library/Logs/Claude/`
   - Windows: `%APPDATA%\Claude\logs\`
   - Linux: `~/.config/Claude/logs/`

## AI Optimization Best Practices

When building tools for AI consumption, consider Smart Tree's innovations:

### 1. Token-Efficient Formats
```javascript
// ❌ Bad: Pretty but wasteful
output = `File: ${name} (${size} bytes, modified ${date})`;

// ✅ Good: Compact and parseable
output = `${type} ${sizeHex} ${timestampHex} ${name}`;
```

### 2. Compression by Default
```javascript
// Enable for AI environments
if (process.env.AI_TOOLS === '1') {
  const zlib = require('zlib');
  output = zlib.gzipSync(output).toString('base64');
  console.log(`COMPRESSED:${output}:END_COMPRESSED`);
}
```

### 3. Structured Output with Markers
```javascript
// Help AI parse your output
console.log('OUTPUT_V1:'); // Version marker
console.log(data);
console.log('END_OUTPUT'); // Clear terminator
```

### 4. Digest/Summary Modes
```javascript
// Provide ultra-compact summaries
function digest(directory) {
  return `HASH:${hash} F:${fileCount} S:${sizeHex}`;
}
```

See [AI_OPTIMIZATION.md](AI_OPTIMIZATION.md) for detailed examples and benchmarks.

## Common Issues and Solutions

### Binary not found after installation
- Check file permissions (should be executable)
- Verify the binary name matches across all files
- Ensure extraction logic handles your archive format

### Updates not working
- Verify your GitHub releases API endpoint
- Check version string parsing
- Ensure network requests aren't blocked

### MCP server not starting
- Test your binary with `--mcp` flag manually
- Check for missing dependencies
- Verify stdin/stdout handling

## Publishing Your DXT

1. **Create a GitHub Release**:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. **Upload the DXT package** to your release

3. **Share with the community**:
   - Submit to DXT examples repository
   - Share in Claude Discord
   - Write a blog post about your tool

## Getting Help

- [DXT Documentation](https://github.com/anthropics/anthropic-sdk-typescript/tree/main/packages/dxt)
- [MCP Specification](https://modelcontextprotocol.com)
- [Claude Desktop Support](https://support.anthropic.com)

---

Remember: The goal is to make installation seamless for users. They shouldn't need to know about binaries, platforms, or updates - it should just work! ✨