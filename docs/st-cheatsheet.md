# The Cheet's `st` cheat sheet 🌳

For a quick look at a new checkout, run `st --depth 2 .`. For an answer you intend to pipe into another tool, choose the output format explicitly. The examples below run from a project root.

| Task | Command |
| --- | --- |
| Browse locally with a fixed depth | `st --no-daemon --mode classic --depth 2 .` |
| Get a tree as JSON | `st --no-daemon --mode json --depth 2 .` |
| Find Rust source paths | `st --type rs .` |
| Find a basename by regex | `st --find 'README\.md$' .` |
| Search file contents for a literal string | `st --type rs --search 'TODO' .` |
| See a short project summary for an assistant | `st --mode summary-ai .` |
| Inspect the structural capsule | `st --no-daemon --mode louds .` |
| Get help with symbol lookup | `st symbol --help` |

## Scope and output

`--depth 0` selects an automatic depth. Set an explicit positive depth to compare runs. `--all` includes hidden entries. `--show-ignored` shows ignored entries in the output, while `--no-ignore` bypasses ignore-file rules. `--everything` enables hidden entries and disables ignore-file and built-in ignore rules. Check stderr for scan-budget warnings before treating a missing path as evidence.

Use `--no-daemon` to run the scan locally. Use `--spicy` to open the interactive file browser. `st --help` lists the options in your installed build, and `st --man` prints the generated manual.

[Choose a mode](MODE_SELECTION_GUIDE.md) · [Connect an MCP client](mcp-guide.md) · [Scan budgets](HOME_DIRECTORY_SAFETY.md)
