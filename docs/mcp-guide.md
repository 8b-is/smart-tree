# Connect an MCP client

Smart Tree's stdio server reads one JSON-RPC message per line and writes responses to stdout. A client launches the `st` executable with `--mcp`. You can use the server directly without starting the HTTP daemon.

## Configure the client

Locate the executable with `command -v st` on a Unix shell or `(Get-Command st).Source` in PowerShell. For a client that uses an `mcpServers` object, merge this entry into its existing configuration and replace the path with the absolute path to your executable.

```json
{
  "mcpServers": {
    "smart-tree": {
      "command": "/absolute/path/to/st",
      "args": ["--mcp"]
    }
  }
}
```

Restart the client after editing its configuration. `st --mcp-status` checks the supported client installations. `st --mcp-install` is the optional installer for users who want Smart Tree to edit those settings.

## Check the connection

The client sends `initialize`, then `notifications/initialized`, then `tools/list`. Discover tool names and parameter schemas on the connected server. The default configuration exposes consolidated tools, and feature settings can change the list.

To inspect the exchange directly, run the following in a Unix shell. Each line in the quoted input is one message. The two requests have IDs. The initialized notification does not take an ID or require a reply.

```sh
st --mcp <<'MCP'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"docs","version":"1"}}}
{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
MCP
```

The response to request 2 contains `result.tools`. To call a tool in an initialized session, send a `tools/call` request. Run the server from your project root for this relative-path example, or supply an absolute directory path.

```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"overview","arguments":{"mode":"quick","path":".","depth":2}}}
```

## Read a result

An overview returns a compact description of the project. Use `find` to locate files, `search` to locate text, and `analyze` with `mode: "directory"` and `format: "json"` for a structured listing. Check every content block for a truncation notice before drawing conclusions about omitted entries. A scan can also omit paths because of depth or ignore rules.

[Example calls](mcp-quick-reference.md) · [Troubleshooting](mcp-troubleshooting-cheat.md) · [Tool registration](../src/mcp/tools_consolidated_enhanced.rs) · [Server dispatch](../src/mcp/mod.rs)
