# Smart Tree documentation

Start with the [CLI cheat sheet](st-cheatsheet.md) to scan a directory, or [connect an MCP client](mcp-guide.md) to explore a project from an assistant.

| If you want to… | Read |
| --- | --- |
| Choose between a listing, summary, and fingerprint | [Mode selection](MODE_SELECTION_GUIDE.md) |
| Understand what a 64-byte MEM\|8 capsule preserves | [SMTR wire format and worked example](SMTR_CAPSULE.md) |
| Evaluate scan limits, graph analysis, and persistence guarantees | [Architecture review](ARCHITECTURE_REVIEW_V10_8.md) |
| Measure output size or reproduce a graph benchmark | [Performance measurements](PERFORMANCE_METRICS.md) |
| Browse operations guides, release evidence, and design notes | [Documentation index](INDEX.md) |

The [index](INDEX.md) distinguishes operating instructions from proposals and historical reports. Start from the guide that matches the build you run. `st --help` lists the CLI options and the MCP server advertises its enabled tools through `tools/list`.
