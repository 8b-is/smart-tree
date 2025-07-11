#!/bin/bash
# Run the continuous tmux monitor with smart processing
# Aye & Hue's intelligent terminal companion! 🚀

# Colors for pretty output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Help function
show_help() {
    echo -e "${BLUE}Tmux AI Assistant v2 - Continuous Monitor${NC}"
    echo ""
    echo "Usage: $0 [session_name] [--auto]"
    echo ""
    echo "Arguments:"
    echo "  session_name    Name of tmux session to monitor"
    echo "  --auto         Enable automation mode"
    echo ""
    echo "Examples:"
    echo "  $0 mysession              # Monitor with manual responses"
    echo "  $0 mysession --auto       # Full automation mode"
    echo ""
    echo "Configuration:"
    echo "  Edit config/config_v2.yaml for settings"
    echo "  Edit config/vault.yaml for automated responses"
}

# Check for help (moved before first_run_check and default_values)
if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
    show_help
    exit 0
fi

# Check for first run (using uv if available)
if command -v uv &> /dev/null && [ -f "pyproject.toml" ]; then
    uv run python scripts/first_run_check.py
else
    python3 scripts/first_run_check.py
fi
if [ $? -ne 0 ]; then
    exit 1
fi

# Default values (now after help check)
SESSION_NAME=${1:-""}
ENABLE_AUTO=${2:-""}


# Check session name
if [ -z "$SESSION_NAME" ]; then
    echo -e "${YELLOW}Error: Please provide a session name${NC}"
    show_help
    exit 1
fi

# Determine Python command based on available tools
if command -v uv &> /dev/null && [ -f "pyproject.toml" ]; then
    # Use uv run - no activation needed! 🚀
    echo -e "${GREEN}Using uv for modern Python management...${NC}"
    PYTHON_CMD="uv run python"
elif [ -d ".venv" ]; then
    # Traditional virtualenv
    echo -e "${GREEN}Activating virtual environment...${NC}"
    source .venv/bin/activate
    PYTHON_CMD="python3"
else
    echo -e "${YELLOW}Warning: No virtual environment found${NC}"
    PYTHON_CMD="python3"
fi

# Build command
CMD="$PYTHON_CMD tmux_monitor_v2.py $SESSION_NAME"

# Add automation flag if requested
if [ "$ENABLE_AUTO" = "--auto" ]; then
    CMD="$CMD --enable-automation"
    echo -e "${YELLOW}⚠️  Automation mode enabled!${NC}"
    echo -e "${YELLOW}Make sure vault.yaml is configured properly${NC}"
fi

# Add common options
CMD="$CMD --verbose"

# Run the monitor
echo -e "${GREEN}Starting continuous monitor for session: $SESSION_NAME${NC}"
echo -e "${BLUE}Press Ctrl+C to stop${NC}"
echo ""

exec $CMD