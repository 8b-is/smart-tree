#!/bin/bash
# 🚀 TMUX Hot Tub Management Script - Aye & Hue's Command Center
# Trisha's Accounting Wisdom: "A well-managed project is a successful project!" 📊

set -e

# ANSI color codes for Trisha's favorite sparkly output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Neon effect for special messages
NEON_CYAN='\033[1;96m'
NEON_MAGENTA='\033[1;95m'
NEON_GREEN='\033[1;92m'

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Function to print colored messages with emojis
print_message() {
    local color=$1
    local emoji=$2
    local message=$3
    echo -e "${color}${emoji} ${message}${NC}"
}

# Function to print the hot tub banner
print_banner() {
    echo -e "${NEON_CYAN}"
    echo "╔═══════════════════════════════════════════════════════╗"
    echo "║                  🛁 TMUX HOT TUB 🛁                   ║"
    echo "║        Where Aye & Hue Collaborate in Style!          ║"
    echo "╚═══════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

# Function to check dependencies
check_dependencies() {
    local missing=0
    
    print_message "${YELLOW}" "🔍" "Checking dependencies..."
    
    # Check Node.js
    if command -v node &> /dev/null; then
        print_message "${GREEN}" "✅" "Node.js: $(node --version)"
    else
        print_message "${RED}" "❌" "Node.js not found!"
        missing=1
    fi
    
    # Check npm
    if command -v npm &> /dev/null; then
        print_message "${GREEN}" "✅" "npm: $(npm --version)"
    else
        print_message "${RED}" "❌" "npm not found!"
        missing=1
    fi
    
    # Check tmux
    if command -v tmux &> /dev/null; then
        print_message "${GREEN}" "✅" "tmux: $(tmux -V)"
    else
        print_message "${RED}" "❌" "tmux not found!"
        missing=1
    fi
    
    return $missing
}

# Function to install server dependencies
install_server() {
    print_message "${CYAN}" "📦" "Installing server dependencies..."
    cd "$PROJECT_ROOT/server"
    npm install
    print_message "${GREEN}" "✨" "Server dependencies installed!"
}

# Function to start the server
start_server() {
    print_message "${NEON_GREEN}" "🚀" "Starting TMUX Hot Tub server..."
    cd "$PROJECT_ROOT/server"
    
    # Check if server is already running
    if lsof -i:8888 &> /dev/null; then
        print_message "${YELLOW}" "⚠️" "Server already running on port 8888!"
        read -p "Kill existing server? (y/n): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            kill $(lsof -t -i:8888) 2>/dev/null || true
            sleep 1
        else
            return
        fi
    fi
    
    # Start server
    if [ "$1" == "dev" ]; then
        npm run dev
    else
        npm start
    fi
}

# Function to start client
start_client() {
    print_message "${NEON_MAGENTA}" "🌐" "Opening Hot Tub client..."
    
    # Detect OS and open browser
    if [[ "$OSTYPE" == "darwin"* ]]; then
        open "http://localhost:8888"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        xdg-open "http://localhost:8888"
    else
        print_message "${YELLOW}" "🔗" "Open http://localhost:8888 in your browser"
    fi
}

# Function to create tmux session
create_session() {
    local session_name="${1:-aye-hue-collab}"
    "$SCRIPT_DIR/tmux-setup.sh" "$session_name"
}

# Function to show stats
show_stats() {
    print_message "${CYAN}" "📊" "Hot Tub Statistics (Trisha's Report):"
    echo -e "${WHITE}"
    echo "├─ Server Status: $(if lsof -i:8888 &> /dev/null; then echo -e "${GREEN}Online${NC}"; else echo -e "${RED}Offline${NC}"; fi)"
    echo "├─ TMUX Sessions: $(tmux list-sessions 2>/dev/null | wc -l | xargs)"
    echo "├─ Project Size: $(du -sh "$PROJECT_ROOT" 2>/dev/null | cut -f1)"
    echo "└─ Last Modified: $(date -r "$PROJECT_ROOT" "+%Y-%m-%d %H:%M:%S")"
    echo -e "${NC}"
}

# Function to run tests
run_tests() {
    print_message "${YELLOW}" "🧪" "Running Hot Tub tests..."
    
    # Add your test commands here
    print_message "${GREEN}" "✅" "All tests passed! (Just kidding, no tests yet 😄)"
    print_message "${MAGENTA}" "💡" "Trisha says: 'Testing is like balancing books - essential!'"
}

# Function to clean up
cleanup() {
    print_message "${YELLOW}" "🧹" "Cleaning up the Hot Tub..."
    
    # Kill server if running
    if lsof -i:8888 &> /dev/null; then
        kill $(lsof -t -i:8888) 2>/dev/null || true
        print_message "${GREEN}" "✅" "Server stopped"
    fi
    
    # Clean node_modules if requested
    if [ "$1" == "full" ]; then
        rm -rf "$PROJECT_ROOT/server/node_modules"
        print_message "${GREEN}" "✅" "Node modules removed"
    fi
    
    print_message "${GREEN}" "✨" "Hot Tub is squeaky clean!"
}

# Main script logic
print_banner

case "${1:-help}" in
    "install"|"i")
        check_dependencies && install_server
        ;;
    "start"|"s")
        start_server "$2"
        ;;
    "client"|"c")
        start_client
        ;;
    "dev"|"d")
        # Start everything in dev mode
        check_dependencies || exit 1
        print_message "${NEON_GREEN}" "🎯" "Starting full dev environment..."
        
        # Start tmux session in background
        create_session "dev-session" &
        
        # Start server
        start_server "dev" &
        SERVER_PID=$!
        
        # Wait a bit for server to start
        sleep 2
        
        # Open client
        start_client
        
        # Wait for server process
        wait $SERVER_PID
        ;;
    "session"|"tmux")
        create_session "$2"
        ;;
    "stats")
        show_stats
        ;;
    "test"|"t")
        run_tests
        ;;
    "clean")
        cleanup "$2"
        ;;
    "help"|"h"|*)
        echo -e "${BOLD}Usage:${NC} $0 [command] [options]"
        echo
        echo -e "${BOLD}Commands:${NC}"
        echo -e "  ${CYAN}install${NC}, ${CYAN}i${NC}     Install server dependencies"
        echo -e "  ${CYAN}start${NC}, ${CYAN}s${NC}       Start the server (add 'dev' for watch mode)"
        echo -e "  ${CYAN}client${NC}, ${CYAN}c${NC}      Open the web client"
        echo -e "  ${CYAN}dev${NC}, ${CYAN}d${NC}         Start full development environment"
        echo -e "  ${CYAN}session${NC}, ${CYAN}tmux${NC}  Create/attach to TMUX session"
        echo -e "  ${CYAN}stats${NC}          Show project statistics"
        echo -e "  ${CYAN}test${NC}, ${CYAN}t${NC}        Run tests"
        echo -e "  ${CYAN}clean${NC}          Clean up (add 'full' to remove node_modules)"
        echo -e "  ${CYAN}help${NC}, ${CYAN}h${NC}        Show this help message"
        echo
        print_message "${MAGENTA}" "💡" "Pro tip: Use './manage.sh dev' to start everything!"
        print_message "${YELLOW}" "🎨" "Trisha loves the colors in this script!"
        ;;
esac

# Exit message
echo
print_message "${NEON_CYAN}" "🌊" "Aye, Aye! Happy collaborating in the Hot Tub! 🚢"