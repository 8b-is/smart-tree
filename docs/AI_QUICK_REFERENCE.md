# Exploring a repository with Smart Tree

Start with the question you need to answer. For “where does this Rust project handle updates?”, one small discovery call and one targeted search can be enough.

1. Call `overview` on the repository root with `mode: "quick"` and `depth: 2`. Find the source directories and check the reported scope.
2. Call `search` with the relevant term and `file_type: "rs"`, or use `find` to page through candidate paths.
3. Read the matching implementation or use `symbol` for definitions and references. Cite the underlying path when explaining behavior.
4. If a directory scan reports truncation, narrow the root and repeat. An omitted file is weak evidence about what exists outside the scanned scope.

Use `tools/list` on the connected server to discover supported fields. The [MCP quick reference](mcp-quick-reference.md) has copyable calls. For shell use, see the [CLI cheat sheet](st-cheatsheet.md). Persistent memory and editing are separate operations from read-only discovery.
