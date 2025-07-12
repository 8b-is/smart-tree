#!/bin/bash
# Demo script for testing Tmux AI Assistant

echo "🎪 Welcome to the Tmux AI Assistant Demo! 🎪"
echo ""
echo "This demo will show you the different features:"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Ensure we're in the right directory
cd "$(dirname "$0")"

# Activate virtual environment
source .venv/bin/activate

# Kill any existing test session
tmux kill-session -t test-demo 2>/dev/null || true

# Create a new test session
echo -e "${GREEN}1. Creating tmux session 'test-demo'...${NC}"
tmux new-session -d -s test-demo
sleep 1

# Add some test content
echo -e "${GREEN}2. Adding test content to the session...${NC}"
tmux send-keys -t test-demo "echo 'Hello from the Tmux AI Assistant demo!'" Enter
tmux send-keys -t test-demo "ls -la" Enter
tmux send-keys -t test-demo "echo 'This will test error detection...'" Enter
tmux send-keys -t test-demo "cat /nonexistent/file" Enter
sleep 1

# Test 1: List prompts
echo -e "\n${BLUE}=== Test 1: Listing prompt patterns ===${NC}"
python tmux_monitor.py test-demo --list-prompts

# Test 2: Test prompt detection
echo -e "\n${BLUE}=== Test 2: Testing prompt detection ===${NC}"
python tmux_monitor.py test-demo --test-prompt "$ "
python tmux_monitor.py test-demo --test-prompt ">>> "

# Test 3: Monitor for a few seconds
echo -e "\n${BLUE}=== Test 3: Monitoring session (5 seconds) ===${NC}"
echo -e "${YELLOW}Watch for AI suggestions based on the error...${NC}"
timeout 5 python tmux_monitor.py test-demo 2>/dev/null || true

# Test 4: Show captured content
echo -e "\n${BLUE}=== Test 4: Current session content ===${NC}"
tmux capture-pane -t test-demo -p | tail -10

# Test 5: Status check
echo -e "\n${BLUE}=== Test 5: System status ===${NC}"
./tmux-ai status

echo -e "\n${GREEN}Demo complete! 🎉${NC}"
echo ""
echo "To try interactive features:"
echo "  1. Attach to session: tmux attach -t test-demo"
echo "  2. Run monitor: ./tmux_monitor.py test-demo"
echo "  3. Try web interface: ./scripts/attach-client.sh test-demo web"
echo ""
echo "Don't forget to clean up: tmux kill-session -t test-demo"