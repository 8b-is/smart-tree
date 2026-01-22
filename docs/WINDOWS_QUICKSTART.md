# Windows Quick Start Guide for Smart Tree

**TL;DR** - Get started with Smart Tree on Windows in 2 minutes!

## Installation (Pick One)

### Option 1: PowerShell (Recommended)
```powershell
iwr -useb https://raw.githubusercontent.com/8b-is/smart-tree/main/scripts/install.ps1 | iex
```

### Option 2: Manual Download
1. Download the latest Windows release (`.zip`) from [releases](https://github.com/8b-is/smart-tree/releases/latest)
   - Look for: `st-{version}-x86_64-pc-windows-msvc.zip`
2. Extract `st.exe` to `C:\Program Files\st\` (or anywhere you like)
3. Add the folder to your PATH

### Option 3: Build from Source
```powershell
git clone https://github.com/8b-is/smart-tree
cd smart-tree
cargo build --release
# Binary will be at: target\release\st.exe
```

## First Commands

```powershell
# Verify installation
st --version

# Analyze current directory
st .

# Get help
st --help

# Try the interactive TUI
st --spicy
```

## Common Use Cases

### 1. Quick Directory Overview
```powershell
st .
```

### 2. AI-Optimized Output (80% smaller!)
```powershell
st --mode ai --compress .
```

### 3. Search for Files
```powershell
st --search "TODO" .
```

### 4. Find Specific File Types
```powershell
st --type rs .        # Rust files
st --type py .        # Python files
st --find "*.json" .  # JSON files
```

### 5. Export to JSON
```powershell
st --mode json . | ConvertFrom-Json
```

### 6. Deep Directory Analysis
```powershell
st --depth 10 C:\Users\YourName\Projects
```

## PowerShell Aliases (Add to Your Profile)

Edit your PowerShell profile: `notepad $PROFILE`

Add these lines:
```powershell
# Smart Tree aliases
Set-Alias tree st
function st-ai { st --mode ai --compress $args }
function st-search { param($pattern) st --search $pattern . }
function st-json { st --mode json . | ConvertFrom-Json }
```

Then restart PowerShell or run: `& $PROFILE`

## Windows Terminal Setup

For the best experience:

1. **Install Windows Terminal**
   ```powershell
   winget install Microsoft.WindowsTerminal
   ```

2. **Install a Nerd Font**
   ```powershell
   scoop bucket add nerd-fonts
   scoop install CascadiaCode-NF
   ```

3. **Enable UTF-8**
   - Settings → Defaults → Additional settings
   - Enable "Use Unicode UTF-8 for worldwide language support"

## Troubleshooting

### "st is not recognized"
- Restart your terminal
- Or run: `$env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")`

### Colors not showing
- Use Windows Terminal or PowerShell 7+
- Install: `winget install Microsoft.PowerShell`

### Permission errors
- Don't run as admin (not needed)
- Use `--skip-permission-errors` if scanning protected folders

### Slow performance
- Exclude from antivirus: `Add-MpPreference -ExclusionPath "C:\path\to\st.exe"`

## Integration with Claude Desktop (MCP)

Smart Tree can integrate with Claude Desktop via MCP:

```powershell
# Get configuration
st --mcp-config

# Add to Claude Desktop config at:
# %APPDATA%\Claude\claude_desktop_config.json
```

Example config:
```json
{
  "mcpServers": {
    "smart-tree": {
      "command": "C:\\Program Files\\st\\st.exe",
      "args": ["--mcp"],
      "env": {
        "AI_TOOLS": "1"
      }
    }
  }
}
```

## Next Steps

- Read the full [README](../README.md) for advanced features
- Check out [Windows Development Guide](WINDOWS_DEVELOPMENT.md) if contributing
- Try the [Spicy TUI mode](spicy-tui.md) for interactive exploration
- Explore [MCP tools](mcp-tools.md) for AI integration

## Common Windows Paths

```powershell
# User directories
st $env:USERPROFILE\Documents
st $env:USERPROFILE\Downloads
st $env:APPDATA

# System directories
st C:\Windows\System32 --depth 2
st "C:\Program Files"

# WSL access (if installed)
st \\wsl$\Ubuntu\home\username

# Network shares (UNC paths)
st \\server\share
```

## Performance Tips

```powershell
# Stream mode for large directories
st --stream C:\Windows

# Limit depth for faster scanning
st --depth 3 C:\

# Skip hidden files
st --no-hidden .

# Compress output for large trees
st --compress --mode ai .
```

## Environment Variables

Add to your PowerShell profile:
```powershell
$env:ST_DEFAULT_DEPTH = "5"
$env:ST_COLOR = "always"
$env:ST_NO_ICONS = "0"  # Set to 1 to disable emojis
```

## Getting Help

- **Issues**: https://github.com/8b-is/smart-tree/issues
- **Discussions**: https://github.com/8b-is/smart-tree/discussions
- **Documentation**: https://github.com/8b-is/smart-tree/tree/main/docs

---

**That's it! You're ready to use Smart Tree on Windows! 🌳🪟**
