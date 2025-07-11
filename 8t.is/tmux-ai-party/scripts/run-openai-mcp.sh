#!/bin/bash
# 🌟 Run Tmux AI Assistant as OpenAI MCP Server 🌟
# For ChatGPT Deep Research integration!
# Aye & Hue's magical connection script!

set -e

# Colors for our beautiful output (Trish loves colors!)
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Default settings
DEFAULT_PORT=8000
DEFAULT_SESSION=""

# Function to display our fancy header
show_header() {
    echo -e "${PURPLE}"
    echo "╔═══════════════════════════════════════════════════════╗"
    echo "║     🚀 Tmux AI Assistant - OpenAI MCP Server 🚀      ║"
    echo "║        For ChatGPT Deep Research Integration          ║"
    echo "║         Made with 💖 by Aye, Hue & Trisha            ║"
    echo "╚═══════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

# Parse command line arguments
SESSION_NAME=${1:-$DEFAULT_SESSION}
PORT=${2:-$DEFAULT_PORT}

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
elif ! python3 -c "import mcp, fastapi, uvicorn" 2>/dev/null; then
    echo -e "${YELLOW}📦 Installing dependencies...${NC}"
    pip install --upgrade pip
    pip install -r requirements.txt
fi

# Check .env file
if [ ! -f ".env" ]; then
    echo -e "${RED}❌ .env file not found!${NC}"
    echo -e "${YELLOW}Please copy .env.example to .env and add your OpenAI API key${NC}"
    exit 1
fi

# List available tmux sessions if no session specified
if [ -z "$SESSION_NAME" ]; then
    echo -e "${YELLOW}📋 Available tmux sessions:${NC}"
    tmux list-sessions 2>/dev/null || echo "  No tmux sessions found"
    echo ""
    echo -e "${BLUE}ℹ️  No session specified. Server will run without active monitoring.${NC}"
    echo -e "${BLUE}   You can specify a session: $0 <session_name> [port]${NC}"
fi

# Start the server
echo ""
echo -e "${GREEN}🎯 Starting OpenAI MCP Server${NC}"
echo -e "   Session: ${YELLOW}${SESSION_NAME:-'None (manual monitoring)'}${NC}"
echo -e "   Port: ${YELLOW}${PORT}${NC}"
echo ""
echo -e "${PURPLE}📌 Connection Instructions for ChatGPT:${NC}"
echo -e "   1. Go to ChatGPT Settings → Connectors"
echo -e "   2. Add a custom deep research connector"
echo -e "   3. Server URL: ${GREEN}http://localhost:${PORT}/sse${NC}"
echo -e "   4. Add usage instructions about tmux monitoring"
echo ""
echo -e "${BLUE}Press Ctrl+C to stop the server${NC}"
echo ""

# Run the server
$PYTHON_CMD mcp_server.py $SESSION_NAME $PORT