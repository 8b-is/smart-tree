# Repository instructions for GitHub Copilot — Smart Tree

Short version (for Copilot repository instructions):

Purpose
- Help GitHub Copilot and similar assistants call Smart Tree's MCP (Model Context Protocol) tools correctly.

Key rules
- Always include a `path` when issuing discovery/search calls. Examples: `src/`, `docs/**`, or `.` for repository root.
- Use a two-step flow for content-heavy operations: 1) discover (include_content: false) 2) fetch selected file(s) with `read_file`.
- Limit content windows with `start_line`/`end_line` for large files.
- Prefer non-destructive operations first. Use `apply_patch` only after fetching and validating content.

Common templates (examples Copilot should insert verbatim)

1) Discovery (search for occurrences without content):

```json
{
  "keyword": "def ",
  "file_type": "py",
  "include_content": false,
  "path": "src/"
}
```

2) Read a file safely:

```json
{
  "file_path": "src/main.py",
  "start_line": 1,
  "end_line": 200
}
```

3) Apply a patch (minimal diff; one file per patch if possible):

```
*** Begin Patch
*** Update File: /path/to/file
@@
-old line
+new line
*** End Patch
```

Why this file exists: GitHub Copilot can read repository-level instructions automatically. Keep this file short and canonical; refer to `.github/COPILOT_MCP_GUIDELINES.md` for longer examples and rationale.

If you want a runnable test that validates the guidelines, run `tests/test_copilot_guidelines_exists.sh`.
