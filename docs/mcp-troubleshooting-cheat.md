---
title: Smart Tree MCP Troubleshooting Cheat Sheet
description: Quick fixes for common MCP server issues (and non-issues!)
contributor: The Cheet
lastUpdated: 2025-08-14
language: en
---

# 🤖 Smart Tree MCP Troubleshooting Cheat Sheet

*Because even the smartest trees sometimes need a little debugging! 🌳*

## 🎭 "Errors" That Aren't Actually Errors

### ✅ These Are GOOD Messages (Not Errors!)

```
Received notification: notifications/initialized
Smart Tree MCP server v4.8.6 started Build: st...
```

**What's happening**: Claude Desktop sometimes shows startup notifications as "errors"
**Solution**: Nothing! Your MCP server is working perfectly! 🎉

> Pro Tip: If it says "started" and shows version info, you're golden! ✨
{.is-success}

## 🔥 Real MCP Issues & Fixes

### Connection Issues

**Problem**: `Connection refused` or `Server not responding`
```bash
# Check if binary exists and is executable
ls -la target/release/st
./target/release/st --version

# Rebuild if needed
./scripts/manage.sh build
```

**Problem**: `Protocol version mismatch`
```bash
# Update to latest MCP protocol
./scripts/manage.sh mcp-config
```

### Configuration Issues

**Problem**: Claude Desktop can't find the server
```bash
# Generate fresh config
./target/release/st --mcp-config >> ~/Library/Application\ Support/Claude/claude_desktop_config.json

# Or use the manage script
./scripts/manage.sh mcp-config
```

**Problem**: `Permission denied` on macOS
```bash
# Make binary executable
chmod +x target/release/st

# Check quarantine status
xattr -d com.apple.quarantine target/release/st 2>/dev/null || true
```

## 🛠️ Quick Diagnostic Commands

### Check Server Health
```bash
# Test MCP server directly
./target/release/st --mcp

# List available tools
./target/release/st --mcp-tools

# Check version
./target/release/st --version
```

### Network Timeout Issues
- **Default**: 1 minute (usually fine)
- **For large directories**: Increase to 2-3 minutes
- **Location**: Claude Desktop settings

### Common File Paths

| OS | Claude Config Location |
|---|---|
| macOS | `~/Library/Application Support/Claude/claude_desktop_config.json` |
| Linux | `~/.config/claude-desktop/claude_desktop_config.json` |
| Windows | `%APPDATA%\Claude\claude_desktop_config.json` |

## 🎸 Elvis-Level Pro Tips

### Performance Optimization
```bash
# Use compressed mode for large projects
st --mode ai --compress large-project/

# Stream for massive directories
st --stream --mode hex /huge/directory
```

### Debug Mode
```bash
# Enable verbose logging
RUST_LOG=debug ./target/release/st --mcp

# Check what tools are registered
st --mcp-tools | grep -E "(overview|find|search)"
```

### Quick Reset
```bash
# Nuclear option: rebuild everything
./scripts/manage.sh clean
./scripts/manage.sh build
./scripts/manage.sh mcp-config
```

## 🚨 When To Actually Worry

**Real error signs**:
- ❌ Binary won't start at all
- ❌ `SIGKILL` or crash messages
- ❌ `File not found` errors
- ❌ Permissions constantly denied

**Fake error signs**:
- ✅ "notifications/initialized" 
- ✅ "server started" messages
- ✅ Version info display
- ✅ Tool registration logs

## 🎬 The Grand Finale

Remember: A chatty MCP server is a happy MCP server! Those "error" messages are actually your Smart Tree singing its startup song! 🎵

> Pro Tip from The Cheet: If you can use the tools and get responses, ignore the "error" count - it's probably just status spam! 😸
{.is-success}

---

*Made with ❤️ and debugging wisdom by The Cheet* 🐆✨