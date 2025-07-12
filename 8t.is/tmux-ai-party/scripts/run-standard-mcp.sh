#!/bin/bash
# 🌟 Run Tmux AI Assistant as Standard MCP Server 🌟
# For Claude Desktop, Cursor, and other MCP clients!
# Trisha says this one's her favorite! 🎉

set -e

# Colors for our beautiful output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to display our fancy header
show_header() {
    echo -e "${CYAN}"
    echo "╔═══════════════════════════════════════════════════════╗"
    echo "║     🌈 Tmux AI Assistant - Standard MCP Server 🌈     ║"
    echo "║      For Claude Desktop & Cursor Integration          ║"
    echo "║         Made with 💖 by Aye, Hue & Trisha            ║"
    echo "╚═══════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

# Show the header
show_header

# Check environment and setup Python command
if command -v uv &> /dev/null && [ -f "pyproject.toml" ]; then
    echo -e "${BLUE}🚀 Using uv for modern Python management...${NC}"
    PYTHON_CMD="uv run python"
    # Check if dependencies are synced
    if [ ! -d ".venv" ]; then
        echo -e "${YELLOW}📦 First time setup with uv...${NC}"
        uv sync
    fi
elif [ -d ".venv" ]; then
    echo -e "${BLUE}🔧 Activating virtual environment...${NC}"
    source .venv/bin/activate
    PYTHON_CMD="python3"
else
    echo -e "${RED}❌ No Python environment found!${NC}"
    echo -e "${YELLOW}Please run: ./scripts/setup-uv.sh${NC}"
    exit 1
fi

# Check if dependencies are installed
if command -v uv &> /dev/null && [ -f "pyproject.toml" ]; then
    # uv handles dependencies automatically
    echo -e "${GREEN}✅ Dependencies managed by uv${NC}"
else
    if ! python3 -c "import mcp" 2>/dev/null; then
        echo -e "${YELLOW}📦 Installing dependencies...${NC}"
        pip install --upgrade pip
        pip install -r requirements.txt
    fi
fi

# Check .env file
if [ ! -f ".env" ]; then
    echo -e "${RED}❌ .env file not found!${NC}"
    echo -e "${YELLOW}Please copy .env.example to .env and add your OpenAI API key${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}🎯 Starting Standard MCP Server${NC}"
echo ""
echo -e "${PURPLE}📌 Connection Instructions:${NC}"
echo ""
echo -e "${CYAN}For Claude Desktop:${NC}"
echo -e "   1. Add to claude_desktop_config.json:"
echo -e "${YELLOW}"
cat << 'EOF'
   {
     "mcpServers": {
       "tmux-assistant": {
         "command": "uv",
         "args": ["run", "python", "${PWD}/mcp_server.py", "--stdio"],
         "env": {}
       }
     }
   }
EOF
echo -e "${NC}"
echo ""
echo -e "${CYAN}For direct testing:${NC}"
echo -e "   ${GREEN}mcp connect stdio -- python mcp_server.py --stdio${NC}"
echo ""
echo -e "${BLUE}Press Ctrl+C to stop the server${NC}"
echo ""

# Run the server in stdio mode
$PYTHON_CMD mcp_server.py --stdio