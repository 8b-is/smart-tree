# Troubleshoot an MCP connection

| Symptom | Check | Next step |
| --- | --- | --- |
| The client cannot launch Smart Tree | Run the configured absolute executable path with `--version`. A GUI client can have a different PATH from your shell. | Correct the executable path in the existing client configuration and restart the client. |
| `st --mcp` appears to wait | The stdio server reads requests from stdin. | Run the initialize and list exchange in the [connection guide](mcp-guide.md). Verify that the reply to request 2 contains tools. |
| The client shows startup messages as errors | Diagnostic text goes to stderr. | Look for a successful protocol response. A startup banner alone does not establish that calls work. |
| A tool or argument is unknown | Compare it to the connected server's `tools/list` result. | For default consolidated tools, use `overview`, `find`, `search`, or `analyze` with their advertised fields. |
| A directory call fails or scans the wrong root | The client may start the process in a different directory. The server applies its configured path policy. | Supply an existing permitted directory as an absolute path. |
| A result is incomplete | Read all content blocks for a truncation notice and inspect the requested depth. | Narrow the project root and repeat the scan. |

Run `st --mcp-status` to inspect client installation status. For manual setup, edit the existing JSON object as shown in the [configuration example](mcp-guide.md). Appending another complete JSON object to the file makes it invalid.

[Quick reference](mcp-quick-reference.md) · [Server configuration and protocol](../src/mcp/mod.rs)
