#!/bin/bash
# 🌳 Smart Tree - Gemini CLI Integration Setup
# This script sets up the Smart Tree MCP server and the BeforeAgent hook for Gemini CLI.

set -e

echo "🚀 Setting up Smart Tree for Gemini CLI..."

GEMINI_DIR="$HOME/.gemini"
SETTINGS_FILE="$GEMINI_DIR/settings.json"
HOOKS_DIR="$GEMINI_DIR/hooks"
HOOK_SCRIPT="$HOOKS_DIR/smart-tree-before-agent.sh"
ST_BIN=$(which st || echo "/opt/homebrew/bin/st")

if [ ! -f "$ST_BIN" ]; then
  echo "❌ Error: Smart Tree binary (st) not found in PATH."
  echo "Please install Smart Tree first: curl -sSL https://raw.githubusercontent.com/8b-is/smart-tree/main/scripts/install.sh | bash"
  exit 1
fi

if [ ! -d "$GEMINI_DIR" ]; then
  echo "❌ Error: Gemini CLI directory not found at $GEMINI_DIR."
  echo "Please run Gemini CLI at least once to initialize it."
  exit 1
fi

# 1. Create the hook script
mkdir -p "$HOOKS_DIR"

cat << 'EOF' > "$HOOK_SCRIPT"
#!/bin/bash
# Read JSON from stdin
INPUT=$(cat)

# Extract prompt
PROMPT=$(echo "$INPUT" | jq -r '.prompt // ""')

# Run smart-tree hook
# Capture stderr to avoid breaking JSON output
OUTPUT=$(echo "$PROMPT" | st --claude-user-prompt-submit 2>/dev/null)

# Generate JSON output for Gemini CLI BeforeAgent hook
jq -n --arg ctx "$OUTPUT" '{
  "decision": "allow",
  "hookSpecificOutput": {
    "hookEventName": "BeforeAgent",
    "additionalContext": $ctx
  }
}'
EOF

chmod +x "$HOOK_SCRIPT"
echo "✅ Created Gemini CLI hook script at $HOOK_SCRIPT"

# 2. Update settings.json to add MCP server and Hook
if [ ! -f "$SETTINGS_FILE" ]; then
  echo "{}" > "$SETTINGS_FILE"
fi

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo "❌ Error: jq is required to update settings.json. Please install it (e.g., brew install jq, apt install jq)."
    exit 1
fi

echo "🔄 Updating $SETTINGS_FILE..."

# Create a temporary file
TMP_FILE=$(mktemp)

# Add MCP Server and Hook
jq --arg st_bin "$ST_BIN" --arg hook_script "$HOOK_SCRIPT" '
  .mcpServers["smart-tree"] = {
    "command": $st_bin,
    "args": ["--mcp"]
  } |
  if .hooks == null then .hooks = {} else . else . end |
  if .hooks.BeforeAgent == null then .hooks.BeforeAgent = [] else . else . end |
  # Check if hook already exists to avoid duplicates
  if (.hooks.BeforeAgent | map(select(.hooks[0].command == $hook_script)) | length) == 0 then
    .hooks.BeforeAgent += [{"matcher": "*", "hooks": [{"type": "command", "command": $hook_script}]}]
  else
    .
  end
' "$SETTINGS_FILE" > "$TMP_FILE"

mv "$TMP_FILE" "$SETTINGS_FILE"
echo "✅ Updated Gemini CLI settings.json with MCP server and BeforeAgent hook."

echo ""
echo "🎉 Smart Tree is now integrated with Gemini CLI!"
echo "Please restart your Gemini CLI session for changes to take effect."
echo "You can now use Smart Tree tools via MCP and context will be automatically injected."
