# MCP quick reference

These are `params` objects for `tools/call` on the default consolidated interface. In an MCP client, pass the object to the tool call. For raw JSON-RPC, put it inside the request envelope shown in the [connection guide](mcp-guide.md). Use a permitted project directory. An absolute path removes ambiguity about the client's working directory.

| Task | Tool | Fields to choose |
| --- | --- | --- |
| Survey a directory | `overview` | `mode: "quick"`, `depth`, `path` |
| Page through source paths | `find` | `type: "code"`, `limit`, `offset` |
| Locate text in files | `search` | `keyword`, `file_type`, `include_content` |
| Format a scan | `analyze` | `mode: "directory"` selects the operation. `format` selects the renderer. |
| Look up source symbols | `symbol` | Discover operations and required fields with `tools/list`. |

### Overview

```json
{"name":"overview","arguments":{"mode":"quick","path":".","depth":2}}
```

### Find code

```json
{"name":"find","arguments":{"type":"code","path":".","limit":20,"offset":0}}
```

To advance, add the returned `found` count to the offset. Results can include parent directories. The server's number-format setting may present counts in hexadecimal. Keep the tree stable between pages.

### Search source text

```json
{"name":"search","arguments":{"keyword":"TODO","file_type":"rs","include_content":true,"path":"."}}
```

### Get a structured tree

```json
{"name":"analyze","arguments":{"mode":"directory","format":"json","max_depth":2,"path":"."}}
```

A successful directory call returns the formatter output as text in `result.content`. Parse the first text block as JSON when requesting JSON and inspect any additional notice blocks. [Directory handler](../src/mcp/tools/directory.rs)

Earlier guides and the unconsolidated interface use names such as `quick_tree`, `find_files`, and `analyze_directory`. Use the names and schemas in your own server's `tools/list`.
