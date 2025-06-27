#!/usr/bin/env bash
# 🌟 Smart Tree Installation Script - Making Directories Sparkle Since 2025! 🌟
# By Aye, Hue, and Trisha from Accounting (who insisted on the colors)

set -euo pipefail

# ANSI Color codes - Because Trisha loves her neon!
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly MAGENTA='\033[0;35m'
readonly CYAN='\033[0;36m'
readonly BOLD='\033[1m'
readonly NC='\033[0m' # No Color

# Emojis for extra sparkle
readonly ROCKET="🚀"
readonly CHECK="✅"
readonly STAR="⭐"
readonly TREE="🌲"
readonly SPARKLES="✨"
readonly WARNING="⚠️"
readonly ROBOT="🤖"
readonly HEART="💖"

# Installation variables
readonly INSTALL_DIR="/usr/local/bin"
readonly CONFIG_DIR="$HOME/.st"
readonly MCP_CONFIG_FILE="$CONFIG_DIR/mcp-config.toml"
readonly PREFS_FILE="$CONFIG_DIR/preferences"
readonly REPO_URL="https://github.com/8b-is/smart-tree.git"
readonly BINARY_NAME="st"

# Print functions with Trisha's flair
print_header() {
    echo -e "\n${CYAN}${BOLD}$1${NC}"
    echo -e "${CYAN}$(printf '%.0s-' {1..60})${NC}"
}

print_success() {
    echo -e "${GREEN}${CHECK} $1${NC}"
}

print_info() {
    echo -e "${BLUE}${STAR} $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}${WARNING} $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_sparkle() {
    echo -e "${MAGENTA}${SPARKLES} $1 ${SPARKLES}${NC}"
}

# Welcome message
print_welcome() {
    clear
    echo -e "${CYAN}${BOLD}"
    echo "   ____                       __     ______               "
    echo "  / __/____ ___  _____ _____ / /_   /_  __/______ ___    "
    echo " _\ \ / __ \`__ \/ __ \`/ ___// __/    / /  / ___// _ \   "
    echo "/___//_/ /_/ /_/ /_/ /_/    \__/    /_/  /_/    \___/   "
    echo -e "${NC}"
    echo -e "${MAGENTA}${BOLD}Installation Script v1.0${NC}"
    echo -e "${GREEN}${TREE} Making your directories beautiful, one tree at a time! ${TREE}${NC}"
    echo
    print_sparkle "Brought to you by Aye, Hue, and Trisha from Accounting"
    echo
}

# Check for required tools
check_requirements() {
    print_header "Checking Requirements ${ROBOT}"
    
    local missing_tools=()
    
    # Check for Rust/Cargo
    if ! command -v cargo &> /dev/null; then
        missing_tools+=("cargo (Rust)")
    fi
    
    # Check for Git
    if ! command -v git &> /dev/null; then
        missing_tools+=("git")
    fi
    
    # Check for curl
    if ! command -v curl &> /dev/null; then
        missing_tools+=("curl")
    fi
    
    if [ ${#missing_tools[@]} -eq 0 ]; then
        print_success "All requirements met!"
    else
        print_error "Missing required tools:"
        for tool in "${missing_tools[@]}"; do
            echo "  - $tool"
        done
        echo
        print_info "Please install the missing tools and try again."
        exit 1
    fi
}

# Prompt for yes/no with default
prompt_yn() {
    local prompt="$1"
    local default="${2:-y}"
    local response
    
    if [[ "$default" == "y" ]]; then
        read -p "$(echo -e "${BLUE}$prompt [Y/n]: ${NC}")" response
        response="${response:-y}"
    else
        read -p "$(echo -e "${BLUE}$prompt [y/N]: ${NC}")" response
        response="${response:-n}"
    fi
    
    [[ "$response" =~ ^[Yy]$ ]]
}

# Prompt for input with default
prompt_input() {
    local prompt="$1"
    local default="$2"
    local response
    
    read -p "$(echo -e "${BLUE}$prompt [$default]: ${NC}")" response
    echo "${response:-$default}"
}

# Install smart-tree
install_smart_tree() {
    print_header "Installing Smart Tree ${ROCKET}"
    
    # Create temp directory
    local temp_dir=$(mktemp -d)
    cd "$temp_dir"
    
    print_info "Cloning repository..."
    git clone --depth 1 "$REPO_URL" . &> /dev/null || {
        print_error "Failed to clone repository"
        exit 1
    }
    
    print_info "Building smart-tree (this may take a minute)..."
    if cargo build --release --features mcp &> /dev/null; then
        print_success "Build successful!"
    else
        print_error "Build failed. Trying without MCP feature..."
        cargo build --release &> /dev/null || {
            print_error "Build failed completely"
            exit 1
        }
    fi
    
    # Install binary
    print_info "Installing binary to $INSTALL_DIR..."
    if [[ -w "$INSTALL_DIR" ]]; then
        cp "target/release/$BINARY_NAME" "$INSTALL_DIR/"
    else
        print_warning "Need sudo access to install to $INSTALL_DIR"
        sudo cp "target/release/$BINARY_NAME" "$INSTALL_DIR/"
    fi
    
    print_success "Smart Tree installed successfully!"
    
    # Cleanup
    cd - > /dev/null
    rm -rf "$temp_dir"
}

# Setup shell aliases
setup_aliases() {
    print_header "Setting Up Aliases ${STAR}"
    
    local shell_config=""
    local shell_name=""
    
    # Detect shell
    if [[ -n "${BASH_VERSION:-}" ]]; then
        shell_name="bash"
        if [[ -f "$HOME/.bashrc" ]]; then
            shell_config="$HOME/.bashrc"
        elif [[ -f "$HOME/.bash_profile" ]]; then
            shell_config="$HOME/.bash_profile"
        fi
    elif [[ -n "${ZSH_VERSION:-}" ]]; then
        shell_name="zsh"
        shell_config="$HOME/.zshrc"
    fi
    
    if [[ -z "$shell_config" ]]; then
        print_warning "Could not detect shell configuration file"
        print_info "Please manually add these aliases to your shell config:"
        echo "  alias stree='st'"
        echo "  alias smart-tree='st'"
        return
    fi
    
    print_info "Adding aliases to $shell_config..."
    
    # Check if aliases already exist
    if grep -q "alias stree=" "$shell_config" 2>/dev/null; then
        print_warning "Alias 'stree' already exists in $shell_config"
    else
        echo "" >> "$shell_config"
        echo "# Smart Tree aliases (added by installer)" >> "$shell_config"
        echo "alias stree='st'" >> "$shell_config"
        echo "alias smart-tree='st'" >> "$shell_config"
        print_success "Aliases added successfully!"
    fi
}

# Setup Claude integration
setup_claude_integration() {
    print_header "Claude AI Integration ${ROBOT}"
    
    if prompt_yn "Would you like to enable optimized AI mode for Claude Desktop and Claude Code?"; then
        print_info "Setting up Claude integration..."
        
        # Create config directory
        mkdir -p "$CONFIG_DIR"
        
        # Setup environment for AI mode
        local env_file="$CONFIG_DIR/ai-env"
        cat > "$env_file" << 'EOF'
# Smart Tree AI Mode Configuration
# This enables optimized output for Claude and other AI assistants
export AI_TOOLS=1
export ST_DEFAULT_MODE=ai
EOF
        
        print_success "AI mode configuration created!"
        
        # Check for Claude Desktop config
        local claude_desktop_config="$HOME/Library/Application Support/Claude/claude_desktop_config.json"
        if [[ -f "$claude_desktop_config" ]] || prompt_yn "Would you like to add Smart Tree to Claude Desktop MCP servers?"; then
            print_info "Configuring Claude Desktop..."
            
            # Show the command to add to Claude Desktop
            echo -e "\n${YELLOW}Add this to your Claude Desktop MCP servers configuration:${NC}"
            echo -e "${CYAN}{
  \"mcpServers\": {
    \"smart-tree\": {
      \"command\": \"$INSTALL_DIR/st\",
      \"args\": [\"--mcp\"],
      \"env\": {
        \"AI_TOOLS\": \"1\"
      }
    }
  }
}${NC}"
            echo
            print_info "You can also run: ${BOLD}st --mcp-config${NC} to see this configuration"
        fi
        
        # Setup shell integration for AI mode
        print_info "To automatically enable AI mode in your terminal, add this to your shell config:"
        echo -e "${CYAN}source \"$CONFIG_DIR/ai-env\"${NC}"
    else
        print_info "Skipping AI mode setup. You can always enable it later with:"
        echo "  export AI_TOOLS=1"
    fi
}

# Setup preferences
setup_preferences() {
    print_header "Smart Tree Preferences ${SPARKLES}"
    
    print_info "Let's configure your Smart Tree preferences!"
    echo
    
    # Default values
    local default_mode="classic"
    local use_color="auto"
    local use_emoji="yes"
    local max_depth=""
    local show_hidden="no"
    
    # Prompt for preferences
    default_mode=$(prompt_input "Default display mode (classic/hex/json/ai/stats/csv/tsv/digest)" "$default_mode")
    use_color=$(prompt_input "Use colors (always/never/auto)" "$use_color")
    use_emoji=$(prompt_input "Use emojis" "$use_emoji")
    max_depth=$(prompt_input "Maximum depth (leave empty for unlimited)" "$max_depth")
    show_hidden=$(prompt_input "Show hidden files by default" "$show_hidden")
    
    # Create preferences file
    mkdir -p "$CONFIG_DIR"
    cat > "$PREFS_FILE" << EOF
# Smart Tree Preferences
# Generated by installer on $(date)
# With love from Aye, Hue, and Trisha ${HEART}

# Default display mode
ST_DEFAULT_MODE=$default_mode

# Color settings
$([ "$use_color" = "always" ] && echo "ST_COLOR=always" || echo "# ST_COLOR=auto")
$([ "$use_color" = "never" ] && echo "NO_COLOR=1" || echo "# NO_COLOR=0")

# Emoji settings
$([ "$use_emoji" = "no" ] && echo "NO_EMOJI=1" || echo "# NO_EMOJI=0")

# Tree settings
$([ -n "$max_depth" ] && echo "ST_MAX_DEPTH=$max_depth" || echo "# ST_MAX_DEPTH=unlimited")
$([ "$show_hidden" = "yes" ] && echo "ST_SHOW_HIDDEN=1" || echo "# ST_SHOW_HIDDEN=0")

# Hot Tub Mode (for collaborative debugging) 🛁
# HOT_TUB_MODE=1  # Uncomment to enable
EOF
    
    print_success "Preferences saved to $PREFS_FILE"
    
    # Create MCP config if it doesn't exist
    if [[ ! -f "$MCP_CONFIG_FILE" ]]; then
        cat > "$MCP_CONFIG_FILE" << 'EOF'
# MCP Configuration for Smart Tree
# Because even AI needs boundaries!

[cache]
enabled = true
ttl_seconds = 300
max_size_mb = 100

[paths]
# Allowed paths (empty means all paths allowed)
allowed = []

# Blocked paths (for security)
blocked = [
    "/etc/passwd",
    "/etc/shadow",
    "**/.env",
    "**/.git/config",
    "**/secrets",
    "**/credentials"
]
EOF
        print_success "MCP configuration created at $MCP_CONFIG_FILE"
    fi
}

# Final setup and instructions
finalize_installation() {
    print_header "Installation Complete! ${ROCKET}${CHECK}"
    
    print_sparkle "Smart Tree is now installed and ready to use!"
    echo
    
    print_info "Quick start guide:"
    echo "  ${BOLD}st${NC}              - Show current directory tree"
    echo "  ${BOLD}stree${NC}           - Alias for st"
    echo "  ${BOLD}smart-tree${NC}      - Another alias for st"
    echo "  ${BOLD}st --help${NC}       - Show all options"
    echo "  ${BOLD}st --mode ai${NC}    - Use AI-optimized output"
    echo "  ${BOLD}st --mcp-tools${NC}  - Show MCP tools"
    echo
    
    if [[ -f "$PREFS_FILE" ]]; then
        print_info "Your preferences are saved in: $PREFS_FILE"
        print_info "To load preferences automatically, add to your shell config:"
        echo -e "  ${CYAN}source \"$PREFS_FILE\"${NC}"
    fi
    
    echo
    print_warning "Please restart your terminal or run: ${BOLD}source ~/.bashrc${NC} (or ~/.zshrc)"
    echo
    
    # Trisha's special message
    print_sparkle "Trisha from Accounting says: \"Make those directory trees sparkle! ${SPARKLES}\""
    echo -e "${MAGENTA}Remember: Fast is better than slow, and pretty is better than ugly!${NC}"
    echo
    echo -e "${GREEN}${BOLD}Happy tree viewing! Aye, aye! 🚢${NC}"
}

# Main installation flow
main() {
    print_welcome
    
    # Check if already installed
    if command -v "$BINARY_NAME" &> /dev/null; then
        print_warning "Smart Tree is already installed at: $(which $BINARY_NAME)"
        if ! prompt_yn "Would you like to reinstall/update?"; then
            print_info "Installation cancelled. Have a sparkly day! ${SPARKLES}"
            exit 0
        fi
    fi
    
    check_requirements
    
    # Run installation steps
    install_smart_tree
    
    # Update todo list
    setup_aliases
    setup_claude_integration
    setup_preferences
    
    finalize_installation
}

# Run the installer
main "$@"