# MCP Update Checking & Analytics

## Overview

Smart Tree's MCP server now includes intelligent update checking that:
1. Informs AI assistants about new features when they initialize
2. Collects anonymous platform analytics to help prioritize development
3. Respects privacy settings and can be disabled

## How It Works

### When MCP Tools Initialize
When an AI assistant connects to Smart Tree's MCP server:

```json
{
  "serverInfo": {
    "name": "smart-tree",
    "version": "5.2.0",
    "update_info": {
      "available": true,
      "latest_version": "5.3.0",
      "new_features": [
        "New 'quantum-search' tool for massive codebases",
        "Improved Windows ARM performance",
        "Enhanced context absorption"
      ],
      "message": "Update available with exciting new features!"
    }
  }
}
```

### Platform Analytics
The update check includes minimal, anonymous platform information:
- **OS**: windows, macos, linux, etc.
- **Architecture**: x86_64, aarch64 (ARM), etc.
- **Version**: Current Smart Tree version

This helps us understand:
- **Is Windows ARM support worth investing in?** (Yes, if we see ARM usage!)
- **Which platforms need optimization?**
- **What versions are actively being used?**

### Privacy First
- **No personal data**: No usernames, paths, or project info
- **Respects settings**: Disabled in privacy mode
- **Can be turned off**: `SMART_TREE_NO_UPDATE_CHECK=1`
- **Fast timeout**: 2-second timeout prevents blocking
- **Feature flags**: Respects `enable_telemetry` and `disable_external_connections`

## Benefits for AI Assistants

### Always Informed
AI assistants automatically know about new features:
- New MCP tools available
- Performance improvements
- Bug fixes that affect their workflows

### Better User Experience
Users get informed about updates naturally through their AI assistant:
```
"I see Smart Tree 5.3.0 is available with the new quantum-search tool!
This could help us search this large codebase 10x faster."
```

## Configuration

### Disable Update Checks
```bash
# Environment variable
export SMART_TREE_NO_UPDATE_CHECK=1

# Or in features.toml
privacy_mode = true
# or
disable_external_connections = true
```

### Custom API Endpoint
```bash
# For enterprise/private deployments
export SMART_TREE_FEEDBACK_API=https://your-server.com
```

## API Endpoint

The update check hits:
```
GET https://f.8b.is/mcp/check?version=5.2.0&platform=windows&arch=aarch64
```

Returns:
```json
{
  "update_available": true,
  "latest_version": "5.3.0",
  "new_features": ["..."],
  "message": "Update available!",
  "download_urls": {
    "windows_aarch64": "https://...",
    "windows_x86_64": "https://..."
  }
}
```

## Implementation Details

- **Non-blocking**: Uses tokio timeout to prevent blocking MCP initialization
- **Cached**: Server caches responses to reduce load
- **Graceful degradation**: Returns null if check fails, doesn't break MCP
- **Minimal overhead**: Adds <2 seconds to initialization (usually <200ms)

## Future Enhancements

Based on analytics, we can:
1. **Prioritize platform support** (e.g., invest in Windows ARM if we see usage)
2. **Target optimizations** (e.g., focus on most-used architectures)
3. **Deprecate old versions** intelligently
4. **Provide platform-specific features**

## Example Usage Analytics

After collecting anonymous analytics, we might see:
```
Platform Distribution:
- Linux x86_64: 45%
- macOS aarch64: 30%
- Windows x86_64: 20%
- Windows aarch64: 5%  <- Worth supporting!

Version Distribution:
- v5.2.0: 60% (current)
- v5.1.0: 30%
- v5.0.0: 10% (consider deprecation notice)
```

This data helps make informed decisions about where to invest development effort!

"Your tool, your rules - but also, your voice matters!" - Hue