# 🌳 Smart Tree - Gemini CLI Integration

Smart Tree can be natively integrated with **Gemini CLI** to provide lightning-fast codebase context, memory preservation (MEM8), and powerful MCP tools.

## Automatic Installation

You can automatically configure the MCP server and the necessary hooks by running the setup script provided:

```bash
chmod +x ./scripts/gemini_cli_setup.sh
./scripts/gemini_cli_setup.sh
```

This script will:
1. Register `st --mcp` as an MCP server in your `~/.gemini/settings.json`.
2. Create an adapter hook script at `~/.gemini/hooks/smart-tree-before-agent.sh` that safely extracts JSON prompts, calls `st --claude-user-prompt-submit`, and re-injects the context via JSON.
3. Register the hook in the `BeforeAgent` lifecycle event.

Restart your Gemini CLI session after running the script to load the MCP tools and the hook.

## Manual Configuration

If you prefer to configure it manually, follow these steps:

### 1. Add the MCP Server

Edit your `~/.gemini/settings.json` to include the Smart Tree MCP server:

```json
{
  "mcpServers": {
    "smart-tree": {
      "command": "st",
      "args": ["--mcp"]
    }
  }
}
```
*(Note: You might need to provide the absolute path to `st` depending on your environment, e.g., `/opt/homebrew/bin/st`)*

### 2. Create the Context Hook Adapter

Gemini CLI communicates with hooks via JSON over `stdin` and `stdout`. Smart Tree's native hook expects plain text. Create an adapter script to bridge this gap.

Create a file `~/.gemini/hooks/smart-tree-before-agent.sh`:

```bash
#!/bin/bash
# Read JSON from stdin
INPUT=$(cat)

# Extract prompt
PROMPT=$(echo "$INPUT" | jq -r '.prompt // ""')

# Run smart-tree hook, capture stderr to avoid breaking JSON output
OUTPUT=$(echo "$PROMPT" | st --claude-user-prompt-submit 2>/dev/null)

# Generate JSON output for Gemini CLI BeforeAgent hook
jq -n --arg ctx "$OUTPUT" '{
  "decision": "allow",
  "hookSpecificOutput": {
    "hookEventName": "BeforeAgent",
    "additionalContext": $ctx
  }
}'
```

Make it executable:
```bash
chmod +x ~/.gemini/hooks/smart-tree-before-agent.sh
```

### 3. Register the Hook

Add the hook to your `~/.gemini/settings.json`:

```json
{
  "hooks": {
    "BeforeAgent": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "/Users/YOUR_USERNAME/.gemini/hooks/smart-tree-before-agent.sh"
          }
        ]
      }
    ]
  }
}
```

## How It Works

- **MCP Server**: Gemini CLI will now have access to over 30+ Smart Tree tools (like `quick_tree`, `project_overview`, `anchor_collaborative_memory`, etc.) which save massive amounts of tokens through high-efficiency compression.
- **BeforeAgent Hook**: Every time you send a prompt, Gemini CLI passes it to the hook. The hook consults Smart Tree (which searches MEM8 and checks context), and the result is seamlessly appended to your prompt as additional context before the AI processes it.
