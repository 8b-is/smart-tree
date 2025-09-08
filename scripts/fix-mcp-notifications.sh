#!/usr/bin/env bash
# 🔇 Fix MCP "Fake Error" Notifications Script
# Because Claude Desktop is being a drama queen! 🎭

set -euo pipefail

# Colors for our fancy output
RED=$'\033[0;31m'
GREEN=$'\033[0;32m'
YELLOW=$'\033[1;33m'
BLUE=$'\033[0;34m'
NC=$'\033[0m' # No Color

print_header() {
    echo -e "\n${BLUE}🔇 $1 🔇${NC}\n"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}📋 $1${NC}"
}

print_header "Fixing Claude Desktop MCP Notification Drama"

# Detect Claude Desktop config location
if [[ "$OSTYPE" == "darwin"* ]]; then
    CLAUDE_CONFIG="$HOME/Library/Application Support/Claude/claude_desktop_config.json"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    CLAUDE_CONFIG="$HOME/.config/Claude/claude_desktop_config.json"
else
    CLAUDE_CONFIG="$APPDATA/Claude/claude_desktop_config.json"
fi

print_info "Claude Desktop config location: $CLAUDE_CONFIG"

# Get current directory for st binary path
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ST_BINARY="$PROJECT_DIR/target/release/st"

print_info "Smart Tree binary: $ST_BINARY"

# Check if binary exists
if [[ ! -f "$ST_BINARY" ]]; then
    echo -e "${RED}❌ Binary not found! Building release version...${NC}"
    cd "$PROJECT_DIR"
    cargo build --release
fi

# Create backup of existing config
if [[ -f "$CLAUDE_CONFIG" ]]; then
    cp "$CLAUDE_CONFIG" "${CLAUDE_CONFIG}.backup"
    print_success "Backed up existing config to ${CLAUDE_CONFIG}.backup"
fi

# Create the quiet MCP config
cat > temp_mcp_config.json << EOF
{
  "mcpServers": {
    "smart-tree": {
      "command": "$ST_BINARY",
      "args": ["--mcp"],
      "env": {
        "RUST_LOG": "error",
        "MCP_QUIET": "1",
        "NO_STARTUP_MESSAGES": "1"
      }
    }
  }
}
EOF

# Merge with existing config or create new one
if [[ -f "$CLAUDE_CONFIG" ]]; then
    # Extract existing mcpServers and merge
    if command -v jq >/dev/null 2>&1; then
        print_info "Merging with existing Claude config using jq..."
        jq -s '.[0] * .[1]' "$CLAUDE_CONFIG" temp_mcp_config.json > merged_config.json
        mv merged_config.json "$CLAUDE_CONFIG"
    else
        print_info "jq not found, replacing smart-tree config manually..."
        # Simple replacement - could be improved
        cp temp_mcp_config.json "$CLAUDE_CONFIG"
    fi
else
    print_info "Creating new Claude Desktop config..."
    mkdir -p "$(dirname "$CLAUDE_CONFIG")"
    cp temp_mcp_config.json "$CLAUDE_CONFIG"
fi

# Clean up
rm temp_mcp_config.json

print_success "MCP configuration updated with quiet settings!"
print_info "Restart Claude Desktop to see the changes"

echo -e "\n${GREEN}🎸 Elvis says: Thank ya, thank ya very much! Those notifications should be quiet now! 🕺${NC}"
echo -e "${YELLOW}💡 Pro Tip: If you still see startup messages, they're probably just one-time initialization and not actual errors!${NC}"