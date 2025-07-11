#!/bin/bash
# 🛁 TMUX Hot Tub Setup Script - Aye & Hue's Collaborative Environment
# Trisha says: "Organization is key to accounting AND coding!" 💼

set -e

# ANSI color codes for beautiful output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# Function to print colored messages
print_message() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

print_message "${CYAN}" "🌊 Welcome to TMUX Hot Tub Setup! 🛁"
print_message "${MAGENTA}" "Let's create the perfect collaborative environment!"

# Check if tmux is installed
if ! command -v tmux &> /dev/null; then
    print_message "${RED}" "❌ TMUX not found! Please install tmux first."
    print_message "${YELLOW}" "📦 On macOS: brew install tmux"
    print_message "${YELLOW}" "📦 On Ubuntu/Debian: sudo apt-get install tmux"
    exit 1
fi

# Default session name
SESSION_NAME="${1:-aye-hue-collab}"

# Check if session already exists
if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
    print_message "${YELLOW}" "🔍 Session '$SESSION_NAME' already exists!"
    read -p "Attach to existing session? (y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_message "${GREEN}" "🚀 Attaching to session..."
        tmux attach-session -t "$SESSION_NAME"
        exit 0
    fi
else
    # Create new session
    print_message "${GREEN}" "✨ Creating new TMUX session: $SESSION_NAME"
    
    # Create session with specific configuration
    tmux new-session -d -s "$SESSION_NAME" -n "hot-tub"
    
    # Configure the session for optimal collaboration
    tmux send-keys -t "$SESSION_NAME:hot-tub" "clear" C-m
    tmux send-keys -t "$SESSION_NAME:hot-tub" "echo '🛁 Welcome to the TMUX Hot Tub!'" C-m
    tmux send-keys -t "$SESSION_NAME:hot-tub" "echo '🌟 Aye & Hue are ready to collaborate!'" C-m
    tmux send-keys -t "$SESSION_NAME:hot-tub" "echo '💡 Trisha says: Remember to use ~~ markers for TTS!'" C-m
    tmux send-keys -t "$SESSION_NAME:hot-tub" "echo ''" C-m
    
    # Set up pane layout (optional - split into multiple panes)
    read -p "Create split layout? (y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # Create a 3-pane layout
        tmux split-window -h -t "$SESSION_NAME:hot-tub"
        tmux split-window -v -t "$SESSION_NAME:hot-tub.1"
        
        # Label the panes
        tmux select-pane -t "$SESSION_NAME:hot-tub.0" -T "Main Workspace"
        tmux select-pane -t "$SESSION_NAME:hot-tub.1" -T "Testing & Debug"
        tmux select-pane -t "$SESSION_NAME:hot-tub.2" -T "Logs & Monitoring"
        
        # Focus on main pane
        tmux select-pane -t "$SESSION_NAME:hot-tub.0"
    fi
    
    print_message "${GREEN}" "✅ Session created successfully!"
fi

# Create or update tmux configuration for better collaboration
TMUX_CONF="$HOME/.tmux.conf"
if [ ! -f "$TMUX_CONF" ] || ! grep -q "# Hot Tub Configuration" "$TMUX_CONF"; then
    print_message "${BLUE}" "📝 Adding Hot Tub configuration to ~/.tmux.conf"
    cat >> "$TMUX_CONF" << 'EOF'

# Hot Tub Configuration - Added by tmux-setup.sh
# Enable mouse support for easier navigation

set -g mouse on

# Better colors
set -g default-terminal "screen-256color"

# Status bar styling
set -g status-style bg=colour235,fg=colour136
set -g status-left '#[fg=colour214]🛁 #S #[fg=colour251]| '
set -g status-right '#[fg=colour251]| #[fg=colour214]%H:%M %d-%b-%y'

# Window status
set -g window-status-current-style bg=colour237,fg=colour214
set -g window-status-style bg=colour235,fg=colour250

# Pane borders
set -g pane-border-style fg=colour235
set -g pane-active-border-style fg=colour214

# Enable activity alerts
setw -g monitor-activity on
set -g visual-activity on

# Increase history limit
set -g history-limit 50000

# Quick pane switching with Alt+arrows
bind -n M-Left select-pane -L
bind -n M-Right select-pane -R
bind -n M-Up select-pane -U
bind -n M-Down select-pane -D

# Reload config with r
bind r source-file ~/.tmux.conf \; display-message "Config reloaded! 🔄"
EOF
fi

# Provide connection instructions
print_message "${CYAN}" "\n📱 Connection Instructions:"
print_message "${WHITE}" "1. Start the server: cd tmux-rtc/server && npm install && npm start"
print_message "${WHITE}" "2. Open the web client: http://localhost:8888"
print_message "${WHITE}" "3. Use session name: $SESSION_NAME"
print_message "${WHITE}" "4. Share the session ID with collaborators"

print_message "${MAGENTA}" "\n✨ Pro Tips from Trisha:"
print_message "${YELLOW}" "• Use ~~ Hue, check this out ~~ for TTS announcements"
print_message "${YELLOW}" "• The preview pane can show markdown, web pages, or stats"
print_message "${YELLOW}" "• Mobile users should rotate to landscape for best experience"

# Ask if user wants to attach now
read -p $'\n'"Attach to session now? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_message "${GREEN}" "🚀 Diving into the Hot Tub..."
    tmux attach-session -t "$SESSION_NAME"
else
    print_message "${BLUE}" "👍 Session '$SESSION_NAME' is ready when you are!"
    print_message "${WHITE}" "Run: tmux attach-session -t $SESSION_NAME"
fi