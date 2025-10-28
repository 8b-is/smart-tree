# Copilot & MCP Guidelines — Smart Tree

Purpose: give concrete examples and templates so GitHub Copilot (and other LLM-based assistants) call Smart Tree's MCP tools correctly and predictably.

Why this helps
- Prevents malformed tool calls (e.g. "Missing path" errors).
- Encourages minimal, focused queries to avoid huge MCP responses.
- Provides quick-copy JSON templates Copilot can insert when generating tool calls.

Principles
- Always prefer read/list tools before destructive or large-content tools.
- Include an explicit path or path glob when searching. Example: `src/`, `docs/**`, or `.` for repo root.
- Limit token-heavy requests by using `include_content: false` for discovery, then fetch content for selected files.
- When in doubt, use small ranges (start_line/end_line) for file reads.

Common templates (copy-paste for Copilot)

1) Search for code snippets (recommended two-step flow)

Step A — narrow search (discovery):

```json
{
  "keyword": "def ",
  "file_type": "py",
  "include_content": false,
  "path": "src/"
}
```

Step B — fetch content for 1 file returned (example):

```json
{
  "file_path": "src/main.py",
  "start_line": 1,
  "end_line": 200
}
```

Explanation: the error `Missing path` occurs when `path` is omitted in discovery calls. Always include `path` (or `.` for repo root) to scope results.

2) Read a file (safe default):

```json
{
  "file_path": "README.md",
  "start_line": 1,
  "end_line": 400
}
```

3) Apply a patch (use concise patches; prefer one file per patch):

When instructing the assistant to edit the repository, use the apply-patch format the tooling expects. Example payload body (text follows the repository patch format used by maintainers):

```
*** Begin Patch
*** Update File: /path/to/file
@@
-old line
+new line
*** End Patch
```

Notes: keep the diff minimal and include the `*** Update File:` header for each changed file. If you need to add a file, use `*** Add File:` and include full content.

4) Run a terminal command (safe/explain):

```json
{
  "command": "cargo test --lib",
  "explanation": "Run unit tests to verify the change",
  "isBackground": false
}
```

Tool-specific quick tips
- search/find tools: always pass `path`. Use `include_content: false` for discovery.
- read_file: pass `file_path` and a reasonable line window. If the file is large, request a short window and iterate.
- apply_patch: produce minimal, well-formed diffs. Avoid changing unrelated lines.
- run_in_terminal: provide a one-line `explanation` and prefer non-background commands unless starting servers.

Best practices for Copilot authors
- When generating MCP calls, add a short comment explaining the goal (1–2 sentences). Example: `// find Python tests that mention 'mcp' in the tests/ folder`.
- Use a two-step pattern: discover -> fetch -> act. This reduces token usage and accidental huge outputs.
- If a tool returns "response exceeds maximum allowed tokens", retry with narrower path or request partial content.

Repository snippet you can surface to Copilot (optional)

If you'd like GitHub Copilot to surface this file as repository instructions, copy the content into `.github/COPILOT_MCP_GUIDELINES.md` and follow GitHub's repository instructions docs:

https://docs.github.com/en/copilot/how-tos/configure-custom-instructions/add-repository-instructions

— End of guidelines —

If you'd like, I can:
- Add a README pointer (short note in `README.md`) pointing to this file.
- Create a few automated tests that validate the sample search/read templates against the MCP test harness in `tests/`.

Please tell me which next step you'd like me to take.
