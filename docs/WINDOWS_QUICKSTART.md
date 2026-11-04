# Smart Tree on Windows

The [v11.3.1 release](https://github.com/8b-is/smart-tree/releases/tag/v11.3.1) offers a Linux x86-64 executable archive and a source archive. For that release, WSL on an x86-64 machine is the documented Linux-binary route. Check the asset names on the release you select before downloading one for a different platform.

## With a native Windows executable

If you have built `st.exe`, run these commands from the project root in PowerShell. Adjust the executable path to match your build.

```powershell
$st = (Resolve-Path .\target\release\st.exe).Path
& $st --help
& $st --no-daemon --depth 2 .
& $st --no-daemon --type rs --search TODO .
& $st --no-daemon --mode json . | ConvertFrom-Json
```

The [Windows development notes](WINDOWS_DEVELOPMENT.md) describe the native Rust/MSVC build environment. Current [CI](../.github/workflows/ci.yml) runs on Ubuntu and macOS. Native Windows builds need their own validation.

## MCP configuration

For a Windows-native MCP client, set `command` to the absolute Windows path of your `st.exe`. Double each backslash in the JSON string. Merge the server entry into the client's existing configuration and restart the client.

```json
{"mcpServers":{"smart-tree":{"command":"C:\\Tools\\SmartTree\\st.exe","args":["--mcp"]}}}
```

[CLI cheat sheet](st-cheatsheet.md) · [MCP setup and connection test](mcp-guide.md)
