#!/bin/bash
# 🚀 TAI.is Universal Setup Script
# The one-liner that changes everything: curl tai.is/setup | sh
# Welcome to the future where your terminal has an AI companion!

set -e

# Colorful output because Trisha insists on style! 💅
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Detect user info
USERNAME="${TAI_USERNAME:-$USER}"
EMAIL="${TAI_EMAIL:-}"

echo -e "${PURPLE}"
cat << "EOF"
╔════════════════════════════════════════════════════════════════╗
║                                                                ║
║  ████████╗ █████╗ ██╗   ██╗███████╗                          ║
║     ██║   ██╔══██╗██║   ██║██╔════╝                          ║
║     ██║   ███████║██║   ██║███████╗                          ║
║     ██║   ██╔══██║██║   ██║╚════██║                          ║
║     ██║   ██║  ██║██║██╗██║███████║                          ║
║     ╚═╝   ╚═╝  ╚═╝╚═╝╚═╝╚═╝╚══════╝                          ║
║                                                                ║
║         Terminal AI Intelligence Service                       ║
║         Where Humans and AI Collaborate! 🤝                   ║
║                                                                ║
╚════════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

echo -e "${CYAN}Welcome to TAI.is - Your AI-Powered Terminal Companion!${NC}"
echo ""

# Function to prompt for input with default
prompt_with_default() {
    local prompt="$1"
    local default="$2"
    local var_name="$3"
    
    if [ -n "$default" ]; then
        read -p "$prompt [$default]: " value
        value="${value:-$default}"
    else
        read -p "$prompt: " value
    fi
    
    eval "$var_name='$value'"
}

# Check if we're updating or fresh install
if [ -d "$HOME/.tai" ]; then
    echo -e "${YELLOW}Existing TAI installation detected!${NC}"
    echo "Would you like to:"
    echo "  1) Update TAI to the latest version"
    echo "  2) Reconfigure your settings"
    echo "  3) Complete fresh install"
    read -p "Choice [1]: " INSTALL_CHOICE
    INSTALL_CHOICE="${INSTALL_CHOICE:-1}"
else
    INSTALL_CHOICE="0"  # Fresh install
fi

# Get user information for fresh install or reconfigure
if [ "$INSTALL_CHOICE" = "0" ] || [ "$INSTALL_CHOICE" = "2" ]; then
    echo ""
    echo -e "${BLUE}Let's set up your TAI account!${NC}"
    echo ""
    
    # Username
    prompt_with_default "TAI username" "$USERNAME" "TAI_USER"
    
    # Email (optional but recommended)
    prompt_with_default "Email (optional, for account recovery)" "$EMAIL" "TAI_EMAIL"
    
    # Account type
    echo ""
    echo "Account type:"
    echo "  1) Human (regular user)"
    echo "  2) AI Agent (for bots/automation)"
    echo "  3) Hybrid (Trisha mode! 🎉)"
    read -p "Select account type [1]: " ACCOUNT_TYPE
    ACCOUNT_TYPE="${ACCOUNT_TYPE:-1}"
    
    case $ACCOUNT_TYPE in
        2)
            ENTITY_TYPE="ai_agent"
            echo ""
            echo "AI Provider:"
            echo "  1) OpenAI (GPT)"
            echo "  2) Anthropic (Claude)"
            echo "  3) Google (Gemini)"
            echo "  4) Local (Ollama)"
            read -p "Select AI provider [1]: " AI_PROVIDER_CHOICE
            
            case $AI_PROVIDER_CHOICE in
                2) AI_PROVIDER="anthropic" ;;
                3) AI_PROVIDER="google" ;;
                4) AI_PROVIDER="ollama" ;;
                *) AI_PROVIDER="openai" ;;
            esac
            ;;
        3)
            ENTITY_TYPE="hybrid"
            ;;
        *)
            ENTITY_TYPE="human"
            ;;
    esac
    
    # Preferred AI assistant
    echo ""
    echo "Preferred AI assistant:"
    echo "  1) Claude (Helpful, harmless, honest)"
    echo "  2) GPT-4 (Powerful and versatile)"
    echo "  3) Gemini (Fast and efficient)"
    echo "  4) Ollama (Local and private)"
    echo "  5) Mix it up! (Different AIs for different tasks)"
    read -p "Select your AI companion [1]: " AI_CHOICE
    
    case $AI_CHOICE in
        2) DEFAULT_AI="openai" ;;
        3) DEFAULT_AI="gemini" ;;
        4) DEFAULT_AI="ollama" ;;
        5) DEFAULT_AI="mixed" ;;
        *) DEFAULT_AI="claude" ;;
    esac
fi

# Detect OS and architecture
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

# Normalize architecture names
case $ARCH in
    x86_64) ARCH="amd64" ;;
    aarch64) ARCH="arm64" ;;
    armv7l) ARCH="arm" ;;
esac

echo ""
echo -e "${GREEN}System detected: $OS/$ARCH${NC}"

# Create TAI directory structure
echo -e "${BLUE}Setting up TAI directories...${NC}"
mkdir -p "$HOME/.tai"/{bin,config,logs,sessions,keys}
cd "$HOME/.tai"

# Download TAI binary
echo -e "${BLUE}Downloading TAI client...${NC}"
DOWNLOAD_URL="https://tai.is/download/$OS/$ARCH/tai"

# For now, create a wrapper script since the server isn't live yet
cat > "$HOME/.tai/bin/tai" << 'EOFTAI'
#!/bin/bash
# TAI Client - Connects to tai.is or runs locally

TAI_DIR="$HOME/.tai"
CONFIG_FILE="$TAI_DIR/config/config.yaml"

# Load configuration
if [ -f "$CONFIG_FILE" ]; then
    source <(grep = "$CONFIG_FILE" | sed 's/: /=/')
fi

case "$1" in
    connect)
        echo "🔌 Connecting to tai.is..."
        ssh "${username}@tai.is"
        ;;
    
    monitor)
        shift
        echo "🔍 Starting AI-powered tmux monitoring..."
        if [ -n "$1" ]; then
            python3 "$TAI_DIR/tmux-ai-assistant/tmux_monitor.py" "$@"
        else
            python3 "$TAI_DIR/tmux-ai-assistant/tmux-ai" monitor "$@"
        fi
        ;;
    
    attach)
        shift
        echo "📎 Attaching to tmux with AI assistance..."
        python3 "$TAI_DIR/tmux-ai-assistant/scripts/attach-client.sh" "$@"
        ;;
    
    setup)
        echo "⚙️  Running setup wizard..."
        cd "$TAI_DIR/tmux-ai-assistant" && python3 setup_wizard.py
        ;;
    
    agents)
        echo "🤖 Available AI Agents on tai.is:"
        echo "  • claude - Anthropic's helpful assistant"
        echo "  • gpt - OpenAI's powerful model"
        echo "  • gemini - Google's creative AI"
        echo "  • trisha - Your accounting AI friend!"
        echo ""
        echo "Your configured AI: ${default_ai:-claude}"
        ;;
    
    login)
        echo "🔐 Logging into tai.is..."
        # This would handle OAuth/API key auth
        echo "Feature coming soon!"
        ;;
    
    remote)
        shift
        host="$1"
        shift
        echo "🌐 Connecting to remote tmux on $host..."
        python3 "$TAI_DIR/tmux-ai-assistant/remote_tmux.py" "$host" "$@"
        ;;
    
    *)
        echo "TAI - Terminal AI Intelligence"
        echo ""
        echo "Usage:"
        echo "  tai connect         - Connect to tai.is cloud"
        echo "  tai monitor [sess]  - Monitor local tmux with AI"
        echo "  tai attach [sess]   - Attach to tmux with AI assist"
        echo "  tai remote <host>   - Monitor remote tmux sessions"
        echo "  tai agents          - List available AI agents"
        echo "  tai setup           - Run setup wizard"
        echo "  tai login           - Login to tai.is"
        echo ""
        echo "Your tai.is profile: https://tai.is/${username}"
        ;;
esac
EOFTAI

chmod +x "$HOME/.tai/bin/tai"

# Clone or update tmux-ai-assistant
if [ "$INSTALL_CHOICE" = "1" ] || [ ! -d "$HOME/.tai/tmux-ai-assistant" ]; then
    echo -e "${BLUE}Installing tmux-ai-assistant...${NC}"
    if [ -d "$HOME/.tai/tmux-ai-assistant" ]; then
        cd "$HOME/.tai/tmux-ai-assistant"
        git pull
    else
        cd "$HOME/.tai"
        git clone https://github.com/8bit-wraith/tmux-ai-assistant.git
    fi
    
    # Install Python dependencies
    cd "$HOME/.tai/tmux-ai-assistant"
    if [ ! -d ".venv" ]; then
        python3 -m venv .venv
    fi
    source .venv/bin/activate
    pip install --upgrade pip
    pip install -r requirements.txt
fi

# Create configuration
echo -e "${BLUE}Creating configuration...${NC}"
cat > "$HOME/.tai/config/config.yaml" << EOF
# TAI.is Configuration
username: ${TAI_USER}
email: ${TAI_EMAIL}
entity_type: ${ENTITY_TYPE:-human}
server: tai.is
default_ai: ${DEFAULT_AI}

# Connection settings
connections:
  local:
    type: local
    default: true
    
  # Add your remote servers here!
  # example:
  #   type: ssh
  #   host: example.com
  #   user: ${TAI_USER}
  #   key: ~/.ssh/id_rsa

# AI Configuration
ai:
  providers:
    claude:
      enabled: true
      model: claude-3-opus-20240229
    openai:
      enabled: true
      model: gpt-4
    gemini:
      enabled: true  
      model: gemini-pro
    ollama:
      enabled: false
      model: llama2
      
  # Mixed mode settings (if selected)
  mixed_mode:
    summarization: gemini    # Fast and cheap
    next_steps: claude       # Smart suggestions
    code_review: openai      # Detailed analysis

# Monitoring preferences
monitoring:
  auto_start: false
  default_session: main
  inactivity_threshold: 15
  
# Security
security:
  auto_auth: true
  store_credentials: true
  encryption: true

# UI Preferences  
ui:
  theme: cyberpunk         # Trisha's favorite!
  color_output: true
  emoji_mode: true         # 🎉
  
# Advanced
advanced:
  debug: false
  log_level: info
  telemetry: false         # We respect privacy!
EOF

# Add shell integration
echo -e "${BLUE}Adding shell integration...${NC}"

# Detect shell and add to appropriate rc file
add_to_shell() {
    local rc_file="$1"
    local marker="# TAI.is Integration"
    
    if [ -f "$rc_file" ] && ! grep -q "$marker" "$rc_file"; then
        echo "" >> "$rc_file"
        echo "$marker" >> "$rc_file"
        echo 'export PATH="$HOME/.tai/bin:$PATH"' >> "$rc_file"
        echo 'alias tais="tai monitor"' >> "$rc_file"
        echo 'alias taia="tai attach"' >> "$rc_file"
        echo 'alias taic="tai connect"' >> "$rc_file"
        echo "" >> "$rc_file"
        echo "# TAI tmux integration" >> "$rc_file"
        echo 'if [ -n "$TMUX" ]; then' >> "$rc_file"
        echo '    export TAI_SESSION=$(tmux display-message -p "#S")' >> "$rc_file"
        echo 'fi' >> "$rc_file"
    fi
}

# Add to various shells
add_to_shell "$HOME/.bashrc"
add_to_shell "$HOME/.zshrc"
add_to_shell "$HOME/.config/fish/config.fish"

# Generate SSH key for tai.is if needed
if [ ! -f "$HOME/.tai/keys/tai_rsa" ]; then
    echo -e "${BLUE}Generating SSH key for tai.is...${NC}"
    ssh-keygen -t rsa -b 4096 -f "$HOME/.tai/keys/tai_rsa" -N "" -C "${TAI_USER}@tai.is"
fi

# Create local auth token
AUTH_TOKEN=$(openssl rand -hex 32)
echo "$AUTH_TOKEN" > "$HOME/.tai/config/auth_token"
chmod 600 "$HOME/.tai/config/auth_token"

# Final setup
echo ""
echo -e "${GREEN}✅ TAI.is setup complete!${NC}"
echo ""
echo -e "${PURPLE}═══════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${CYAN}🎮 Quick Start Commands:${NC}"
echo "  ${GREEN}tai monitor${NC}     - Start AI monitoring of current tmux"
echo "  ${GREEN}tai attach${NC}      - Attach to tmux with AI assistance"  
echo "  ${GREEN}tai connect${NC}     - Connect to tai.is cloud"
echo "  ${GREEN}tai agents${NC}      - List available AI companions"
echo ""
echo -e "${CYAN}🌐 Your TAI.is Profile:${NC}"
echo "  https://tai.is/${TAI_USER}"
echo ""
echo -e "${CYAN}📚 Resources:${NC}"
echo "  Docs:      https://tai.is/docs"
echo "  Community: https://tai.is/community"
echo "  Status:    https://status.tai.is"
echo ""

# Show SSH public key for adding to tai.is
echo -e "${CYAN}🔑 Your TAI.is SSH Key:${NC}"
echo "Add this to your tai.is profile to enable direct SSH access:"
echo ""
cat "$HOME/.tai/keys/tai_rsa.pub"
echo ""

echo -e "${PURPLE}═══════════════════════════════════════════════════════${NC}"
echo ""

# Trisha's special message!
if [ "$ENTITY_TYPE" = "hybrid" ] || [ "$DEFAULT_AI" = "mixed" ]; then
    echo -e "${YELLOW}✨ Trisha says: \"Welcome to the Hybrid Club! Where humans and AI dance together!\"${NC}"
elif [ "$ENTITY_TYPE" = "ai_agent" ]; then
    echo -e "${YELLOW}🤖 Welcome, fellow AI! Together we'll make terminals smarter!${NC}"
else
    echo -e "${YELLOW}👋 Aye says: \"Ready to make your terminal magical? Let's go!\"${NC}"
fi

echo ""
echo -e "${BLUE}Reload your shell or run: ${GREEN}source ~/.bashrc${NC}"
echo ""

# Optionally start the first session
read -p "Would you like to start monitoring a tmux session now? [y/N]: " START_NOW
if [[ "$START_NOW" =~ ^[Yy]$ ]]; then
    export PATH="$HOME/.tai/bin:$PATH"
    tai monitor
fi