#!/bin/bash
# Attach to tmux session as a proper client with AI assistance
# 🎹 "La la la, di da da, La la, di da da da dum..."

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

# Default values
SESSION_NAME=${1:-""}
MODE=${2:-"attach"}  # attach, client, or web

# Help function
show_help() {
    echo -e "${BLUE}Tmux AI Client - Attach with Style! 🎹${NC}"
    echo ""
    echo "Usage: $0 [session_name] [mode]"
    echo ""
    echo "Modes:"
    echo "  attach   - Simple attachment (default)"
    echo "  client   - Advanced client with modes"
    echo "  web      - Start web interface"
    echo ""
    echo "Examples:"
    echo "  $0 mysession          # Simple attach"
    echo "  $0 mysession client   # Advanced client"
    echo "  $0 mysession web      # Web spectator mode"
    echo ""
    echo -e "${YELLOW}Tip: Use Ctrl+B D to detach properly!${NC}"
}

# Check for help
if [ "$1" = "-h" ] || [ "$1" = "--help" ] || [ -z "$SESSION_NAME" ]; then
    show_help
    exit 0
fi

# Activate virtual environment
if [ -d ".venv" ]; then
    source .venv/bin/activate
fi

# Check if session exists
if ! tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
    echo -e "${RED}Session '$SESSION_NAME' not found!${NC}"
    echo ""
    echo "Available sessions:"
    tmux list-sessions 2>/dev/null || echo "  No sessions found"
    exit 1
fi

case "$MODE" in
    attach)
        echo -e "${GREEN}Attaching to tmux session: $SESSION_NAME${NC}"
        echo -e "${YELLOW}This is a proper tmux client - the owner can kick you out!${NC}"
        echo ""
        python3 tmux_attach.py "$SESSION_NAME"
        ;;
        
    client)
        echo -e "${GREEN}Starting advanced AI client for: $SESSION_NAME${NC}"
        echo ""
        echo "Select mode:"
        echo "  1) Observe - Watch and get suggestions"
        echo "  2) Assist - Queue commands for approval"
        echo "  3) Collaborate - Auto-execute AI suggestions"
        echo "  4) Spectate - Web interface"
        echo ""
        read -p "Choice (1-4): " choice
        
        case $choice in
            1) MODE_FLAG="observe" ;;
            2) MODE_FLAG="assist" ;;
            3) MODE_FLAG="collaborate" ;;
            4) MODE_FLAG="spectate" ;;
            *) MODE_FLAG="observe" ;;
        esac
        
        if [ "$MODE_FLAG" = "spectate" ]; then
            read -p "Web port (default 8080): " PORT
            PORT=${PORT:-8080}
            python3 tmux_client.py "$SESSION_NAME" --mode "$MODE_FLAG" --web-port "$PORT"
        else
            python3 tmux_client.py "$SESSION_NAME" --mode "$MODE_FLAG"
        fi
        ;;
        
    web)
        echo -e "${GREEN}Starting web spectator mode for: $SESSION_NAME${NC}"
        read -p "Web port (default 8080): " PORT
        PORT=${PORT:-8080}
        
        echo -e "${BLUE}Starting coding carnival at http://localhost:$PORT 🎪${NC}"
        python3 tmux_client.py "$SESSION_NAME" --mode spectate --web-port "$PORT"
        ;;
        
    *)
        echo -e "${RED}Unknown mode: $MODE${NC}"
        show_help
        exit 1
        ;;
esac

echo -e "\n${BLUE}Thanks for joining the show! 🎹${NC}"