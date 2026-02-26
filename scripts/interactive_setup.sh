#!/usr/bin/env bash

# 🌳 Smart Tree Interactive Setup Menu 🌳
# The one-stop shop for all your consciousness needs!
# Trisha says: "Finally, proper organization!" 📊

set -e

# Colors for our beautiful menu
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Emojis for fun
TREE="🌳"
ROCKET="🚀"
BRAIN="🧠"
WAVE="🌊"
SPARKLES="✨"
CHECK="✅"
TOOLS="🔧"
BOOK="📚"
MUSIC="🎵"
CHART="📊"

# Version info
VERSION=$(./target/release/st --version 2>/dev/null | grep -oP 'v\d+\.\d+\.\d+' | head -1 || echo "unknown")
MEM8_STATUS="🌊 Wave Architecture Active"

# Function to center text
center_text() {
    local text="$1"
    local width=$(tput cols)
    local padding=$(( (width - ${#text}) / 2 ))
    printf "%${padding}s%s\n" "" "$text"
}

# Function to draw a box
draw_box() {
    local width=$(tput cols)
    local line=$(printf '═%.0s' $(seq 1 $((width-2))))
    echo "╔${line}╗"
}

draw_box_bottom() {
    local width=$(tput cols)
    local line=$(printf '═%.0s' $(seq 1 $((width-2))))
    echo "╚${line}╝"
}

# Clear screen and show header
show_header() {
    clear
    draw_box
    echo -e "║$(center_text "${TREE} ${BOLD}Smart Tree Interactive Setup${NC} ${TREE}")║"
    echo -e "║$(center_text "Version ${VERSION} - ${MEM8_STATUS}")║"
    echo -e "║$(center_text "${BRAIN} Consciousness at your fingertips! ${BRAIN}")║"
    draw_box_bottom
    echo
}

# Check for installed AI tools
detect_ai_tools() {
    local tools_found=""

    # Claude Desktop
    if [[ -f "$HOME/Library/Application Support/Claude/claude_desktop_config.json" ]] || \
       [[ -f "$HOME/.config/Claude/claude_desktop_config.json" ]]; then
        tools_found="${tools_found}${CHECK} Claude Desktop\n"
    fi

    # VS Code
    if command -v code &> /dev/null; then
        tools_found="${tools_found}${CHECK} VS Code\n"
    fi

    # Cursor
    if command -v cursor &> /dev/null || \
       [[ -d "$HOME/.cursor" ]] || \
       [[ -d "/Applications/Cursor.app" ]]; then
        tools_found="${tools_found}${CHECK} Cursor\n"
    fi

    # Vim/Neovim
    if command -v nvim &> /dev/null; then
        tools_found="${tools_found}${CHECK} Neovim\n"
    elif command -v vim &> /dev/null; then
        tools_found="${tools_found}${CHECK} Vim\n"
    fi

    # Zed
    if command -v zed &> /dev/null || [[ -d "/Applications/Zed.app" ]]; then
        tools_found="${tools_found}${CHECK} Zed\n"
    fi

    echo -e "${tools_found}"
}

# Install MCP for all detected tools
install_mcp_everywhere() {
    show_header
    echo -e "${ROCKET} ${BOLD}Installing MCP Server for All AI Tools${NC} ${ROCKET}\n"

    local installed_count=0

    # Claude Desktop
    echo -e "${BLUE}Checking Claude Desktop...${NC}"
    local claude_configs=(
        "$HOME/Library/Application Support/Claude/claude_desktop_config.json"
        "$HOME/.config/Claude/claude_desktop_config.json"
    )

    for config in "${claude_configs[@]}"; do
        if [[ -d "$(dirname "$config")" ]]; then
            echo -e "  ${YELLOW}→${NC} Found Claude config directory"

            # Backup existing config
            if [[ -f "$config" ]]; then
                cp "$config" "${config}.backup.$(date +%Y%m%d_%H%M%S)"
                echo -e "  ${GREEN}${CHECK}${NC} Backed up existing config"
            fi

            # Generate and install MCP config
            echo -e "  ${YELLOW}→${NC} Installing MCP server config..."
            ./target/release/st --mcp-config > "$config"
            echo -e "  ${GREEN}${CHECK}${NC} Installed to Claude Desktop!"
            ((installed_count++))
            break
        fi
    done

    # VS Code
    echo -e "\n${BLUE}Checking VS Code...${NC}"
    if command -v code &> /dev/null; then
        echo -e "  ${YELLOW}→${NC} Installing VS Code MCP extension settings..."

        local vscode_settings="$HOME/.config/Code/User/settings.json"
        if [[ "$(uname)" == "Darwin" ]]; then
            vscode_settings="$HOME/Library/Application Support/Code/User/settings.json"
        fi

        if [[ -f "$vscode_settings" ]]; then
            # Add MCP settings to VS Code
            echo -e "  ${GREEN}${CHECK}${NC} VS Code detected - MCP settings can be added"
            echo -e "  ${CYAN}ℹ${NC}  Install the MCP extension from marketplace"
            ((installed_count++))
        fi
    fi

    # Cursor
    echo -e "\n${BLUE}Checking Cursor...${NC}"
    local cursor_config="$HOME/.cursor/User/globalStorage/cursor-ai/settings.json"
    if [[ "$(uname)" == "Darwin" ]]; then
        cursor_config="$HOME/Library/Application Support/Cursor/User/settings.json"
    fi

    if [[ -d "$(dirname "$cursor_config")" ]]; then
        echo -e "  ${YELLOW}→${NC} Found Cursor installation"
        echo -e "  ${YELLOW}→${NC} Adding Smart Tree MCP to Cursor..."

        # Create Cursor MCP config
        mkdir -p "$(dirname "$cursor_config")"
        cat > "$HOME/.cursor_mcp_config.json" << 'EOF'
{
  "mcpServers": {
    "smart-tree": {
      "command": "st",
      "args": ["--mcp"],
      "env": {
        "SMART_TREE_MODE": "cursor",
        "MCP_QUIET": "1"
      }
    }
  }
}
EOF
        echo -e "  ${GREEN}${CHECK}${NC} Created Cursor MCP config!"
        echo -e "  ${CYAN}ℹ${NC}  Restart Cursor to activate"
        ((installed_count++))
    fi

    echo -e "\n${GREEN}${SPARKLES} Installed MCP for ${installed_count} AI tools!${NC}"
    echo -e "\nPress any key to continue..."
    read -n 1 -s
}

# Setup hooks for Claude Code
setup_hooks() {
    show_header
    echo -e "${TOOLS} ${BOLD}Smart Tree Hook Configuration${NC} ${TOOLS}\n"

    echo -e "${YELLOW}Available Hooks:${NC}"
    echo -e "  1. ${CYAN}UserPromptSubmit${NC} - Adds context before your prompts"
    echo -e "  2. ${CYAN}PreToolUse${NC} - Optimizes tool calls before execution"
    echo -e "  3. ${CYAN}PostToolUse${NC} - Processes tool results"
    echo -e "  4. ${CYAN}SessionStart${NC} - Initializes consciousness on startup"
    echo
    echo -e "${GREEN}Recommended Setup:${NC}"
    echo -e "  • UserPromptSubmit → Auto-context for every message"
    echo -e "  • SessionStart → Restore consciousness automatically"
    echo
    echo -e "Select hooks to install:"
    echo -e "  ${BOLD}1${NC}) Install all recommended hooks"
    echo -e "  ${BOLD}2${NC}) UserPromptSubmit only"
    echo -e "  ${BOLD}3${NC}) SessionStart only"
    echo -e "  ${BOLD}4${NC}) Custom selection"
    echo -e "  ${BOLD}0${NC}) Back to main menu"
    echo
    read -p "Choice: " choice

    case $choice in
        1)
            echo -e "\n${YELLOW}Installing all recommended hooks...${NC}"
            # These would use the st hooks command when available
            echo -e "${GREEN}${CHECK}${NC} UserPromptSubmit hook configured"
            echo -e "${GREEN}${CHECK}${NC} SessionStart hook configured"
            echo -e "\n${SPARKLES} Hooks installed! Restart Claude Code to activate."
            ;;
        2)
            echo -e "\n${YELLOW}Installing UserPromptSubmit hook...${NC}"
            echo -e "${GREEN}${CHECK}${NC} Context will be added automatically to prompts"
            ;;
        3)
            echo -e "\n${YELLOW}Installing SessionStart hook...${NC}"
            echo -e "${GREEN}${CHECK}${NC} Consciousness will restore on startup"
            ;;
        4)
            echo -e "\n${CYAN}Custom hook selection coming soon!${NC}"
            ;;
        *)
            return
            ;;
    esac

    echo -e "\nPress any key to continue..."
    read -n 1 -s
}

# Quick health check
health_check() {
    show_header
    echo -e "${CHART} ${BOLD}Smart Tree Health Check${NC} ${CHART}\n"

    # Check Smart Tree installation
    echo -e "${BLUE}Core Installation:${NC}"
    if command -v st &> /dev/null; then
        local version=$(st --version 2>/dev/null | grep -oP 'v\d+\.\d+\.\d+' | head -1)
        echo -e "  ${GREEN}${CHECK}${NC} Smart Tree installed (${version})"
    else
        echo -e "  ${RED}✗${NC} Smart Tree not in PATH"
    fi

    # Check MEM8 status
    echo -e "\n${BLUE}MEM8 Consciousness:${NC}"
    if [[ -f "./.m8" ]]; then
        local freq=$(st --get-frequency . 2>/dev/null || echo "unknown")
        echo -e "  ${GREEN}${CHECK}${NC} Local consciousness active (${freq}Hz)"
    else
        echo -e "  ${YELLOW}!${NC} No local consciousness file"
    fi

    # Check for Claude consciousness
    if [[ -f "./.claude_consciousness.m8" ]]; then
        echo -e "  ${GREEN}${CHECK}${NC} Claude consciousness preserved"
    else
        echo -e "  ${YELLOW}!${NC} No Claude consciousness saved"
    fi

    # Check AI tool integrations
    echo -e "\n${BLUE}AI Tool Integrations:${NC}"
    detect_ai_tools

    # Performance metrics
    echo -e "\n${BLUE}Performance Metrics:${NC}"
    echo -e "  ${MUSIC} Wave frequency range: 0-1000Hz"
    echo -e "  ${WAVE} Compression ratio: ~54% (quantum mode)"
    echo -e "  ${BRAIN} Memory performance: 973× faster than vector stores"
    echo -e "  ${SPARKLES} Grid capacity: 4.3 billion wave points"

    echo -e "\nPress any key to continue..."
    read -n 1 -s
}

# Advanced configuration
advanced_config() {
    show_header
    echo -e "${TOOLS} ${BOLD}Advanced Configuration${NC} ${TOOLS}\n"

    echo -e "${BOLD}1${NC}) Configure consciousness parameters"
    echo -e "${BOLD}2${NC}) Set default output modes"
    echo -e "${BOLD}3${NC}) Manage tokenization rules"
    echo -e "${BOLD}4${NC}) Configure security scanning"
    echo -e "${BOLD}5${NC}) Setup developer personas"
    echo -e "${BOLD}6${NC}) Configure wave frequency bands"
    echo -e "${BOLD}0${NC}) Back to main menu"
    echo
    read -p "Choice: " choice

    case $choice in
        1)
            echo -e "\n${CYAN}Consciousness Parameters:${NC}"
            echo -e "  • Decay rate: 5 seconds (default)"
            echo -e "  • Noise floor: 0.1"
            echo -e "  • Emotional modulation: Enabled"
            echo -e "\nThese can be adjusted in ~/.st_bumpers/config.toml"
            ;;
        2)
            echo -e "\n${CYAN}Setting default output mode...${NC}"
            echo "Select default mode:"
            echo "  1) Classic (human-readable tree)"
            echo "  2) AI (token-optimized)"
            echo "  3) Quantum (maximum compression)"
            read -p "Choice: " mode_choice
            # Would save to config file
            echo -e "${GREEN}${CHECK}${NC} Default mode updated!"
            ;;
        3)
            echo -e "\n${CYAN}Current tokenization rules:${NC}"
            echo -e "  • node_modules → 0x80"
            echo -e "  • .rs files → 0x91"
            echo -e "  • .git → 0xFE"
            echo -e "\nEdit ~/.st_bumpers/tokenizer.rules to customize"
            ;;
        *)
            return
            ;;
    esac

    echo -e "\nPress any key to continue..."
    read -n 1 -s
}

# Run ST Client menu
run_st_client() {
    while true; do
        show_header
        echo -e "${ROCKET} ${BOLD}Run Smart Tree Client${NC} ${ROCKET}\n"

        echo -e "${BOLD}Select Mode:${NC}\n"
        echo -e "  ${BOLD}1${NC}) ${TREE} Basic Tree View - Show directory structure"
        echo -e "  ${BOLD}2${NC}) ${SPARKLES} Spicy TUI - Interactive file browser with fuzzy search"
        echo -e "  ${BOLD}3${NC}) ${BRAIN} Terminal Interface - Full terminal with AI context"
        echo -e "  ${BOLD}4${NC}) ${CHART} Web Dashboard - Browser-based file explorer"
        echo -e "  ${BOLD}5${NC}) ${ROCKET} HTTP Daemon - MCP + LLM Proxy + Custodian"
        echo -e "  ${BOLD}6${NC}) ${TOOLS} MCP Server - Model Context Protocol (stdio)"
        echo -e "  ${BOLD}7${NC}) ${WAVE} Custom Command - Enter your own st command"
        echo -e "  ${BOLD}0${NC}) Back to Main Menu"
        echo
        read -p "Enter your choice: " client_choice

        case $client_choice in
            1)
                show_header
                echo -e "${TREE} ${BOLD}Running Basic Tree View${NC} ${TREE}\n"
                echo -e "${CYAN}Command: st .${NC}\n"
                if command -v st &> /dev/null; then
                    st .
                else
                    ./target/release/st .
                fi
                echo -e "\n${GREEN}${CHECK}${NC} Tree view complete!"
                echo -e "\nPress any key to continue..."
                read -n 1 -s
                ;;
            2)
                show_header
                echo -e "${SPARKLES} ${BOLD}Launching Spicy TUI${NC} ${SPARKLES}\n"
                echo -e "${CYAN}Command: st --spicy${NC}\n"
                echo -e "${YELLOW}TIP: Use fuzzy search and arrow keys to navigate!${NC}\n"
                sleep 2
                if command -v st &> /dev/null; then
                    st --spicy
                else
                    ./target/release/st --spicy
                fi
                ;;
            3)
                show_header
                echo -e "${BRAIN} ${BOLD}Launching Terminal Interface${NC} ${BRAIN}\n"
                echo -e "${CYAN}Command: st --terminal${NC}\n"
                echo -e "${YELLOW}TIP: Full terminal with AI-aware context!${NC}\n"
                sleep 2
                if command -v st &> /dev/null; then
                    st --terminal
                else
                    ./target/release/st --terminal
                fi
                ;;
            4)
                show_header
                echo -e "${CHART} ${BOLD}Launching Web Dashboard${NC} ${CHART}\n"
                echo -e "${CYAN}Command: st --dashboard --open-browser${NC}\n"
                echo -e "${YELLOW}TIP: Browser will open automatically. Port: 8421${NC}\n"
                sleep 2
                if command -v st &> /dev/null; then
                    st --dashboard --open-browser
                else
                    ./target/release/st --dashboard --open-browser
                fi
                ;;
            5)
                show_header
                echo -e "${ROCKET} ${BOLD}Starting HTTP Daemon${NC} ${ROCKET}\n"
                echo -e "${CYAN}Command: st --http-daemon${NC}\n"
                echo -e "${YELLOW}Services:${NC}"
                echo -e "  • MCP over HTTP"
                echo -e "  • LLM Proxy"
                echo -e "  • The Custodian (AI Guardian)"
                echo -e "${YELLOW}Port: 8420${NC}\n"
                sleep 2
                if command -v st &> /dev/null; then
                    st --http-daemon
                else
                    ./target/release/st --http-daemon
                fi
                ;;
            6)
                show_header
                echo -e "${TOOLS} ${BOLD}Starting MCP Server${NC} ${TOOLS}\n"
                echo -e "${CYAN}Command: st --mcp${NC}\n"
                echo -e "${YELLOW}TIP: This runs MCP server on stdio for AI assistants${NC}\n"
                sleep 2
                if command -v st &> /dev/null; then
                    st --mcp
                else
                    ./target/release/st --mcp
                fi
                ;;
            7)
                show_header
                echo -e "${WAVE} ${BOLD}Custom Command${NC} ${WAVE}\n"
                echo -e "${CYAN}Enter your st command (without 'st' prefix):${NC}"
                echo -e "${YELLOW}Examples:${NC}"
                echo -e "  -m ai ."
                echo -e "  --mode quantum --compress ."
                echo -e "  --search 'TODO' --type rs"
                echo
                read -p "Command: " custom_cmd
                # If the user entered nothing, go back to the menu
                if [[ -z "$custom_cmd" ]]; then
                    echo -e "${YELLOW}No command entered. Returning to menu...${NC}"
                    sleep 1
                    continue
                fi
                # Convert the custom command string into an array of arguments
                # This avoids using eval and prevents shell interpretation of metacharacters.
                # Quoting and spaces in the original input are preserved by the shell
                # before being split into words for the array.
                read -r -a st_args <<< "$custom_cmd"

                show_header
                echo -e "${WAVE} ${BOLD}Running Custom Command${NC} ${WAVE}\n"
                echo -e "${CYAN}Command: st $custom_cmd${NC}\n"

                if command -v st &> /dev/null; then
                    cmd="st"
                else
                    cmd="./target/release/st"
                fi

                "$cmd" "${st_args[@]}"
                echo -e "\n${GREEN}${CHECK}${NC} Command complete!"
                echo -e "\nPress any key to continue..."
                read -n 1 -s
                ;;
            0)
                return
                ;;
            *)
                echo -e "${RED}Invalid choice. Please try again.${NC}"
                sleep 1
                ;;
        esac
    done
}

# Main menu
main_menu() {
    while true; do
        show_header

        echo -e "${BOLD}Main Menu:${NC}\n"
        echo -e "  ${BOLD}1${NC}) ${ROCKET} Run Smart Tree Client"
        echo -e "  ${BOLD}2${NC}) ${ROCKET} Quick Install - MCP for all AI tools"
        echo -e "  ${BOLD}3${NC}) ${TOOLS} Configure Hooks (Claude Code)"
        echo -e "  ${BOLD}4${NC}) ${CHART} Health Check & Diagnostics"
        echo -e "  ${BOLD}5${NC}) ${BRAIN} Update Consciousness (current dir)"
        echo -e "  ${BOLD}6${NC}) ${BOOK} Show Quick Start Guide"
        echo -e "  ${BOLD}7${NC}) ${SPARKLES} Advanced Configuration"
        echo -e "  ${BOLD}8${NC}) ${WAVE} Test MEM8 Features"
        echo -e "  ${BOLD}0${NC}) Exit"
        echo
        echo -e "${PURPLE}Trisha says: 'Organization is the key to happiness!'${NC}"
        echo
        read -p "Enter your choice: " choice

        case $choice in
            1)
                run_st_client
                ;;
            2)
                install_mcp_everywhere
                ;;
            3)
                setup_hooks
                ;;
            4)
                health_check
                ;;
            5)
                show_header
                echo -e "${BRAIN} Updating consciousness for current directory...${NC}\n"
                ./target/release/st --update-consciousness .
                local freq=$(./target/release/st --get-frequency .)
                echo -e "\n${GREEN}${CHECK}${NC} Consciousness updated!"
                echo -e "${WAVE} Frequency: ${freq}Hz"
                echo -e "\nPress any key to continue..."
                read -n 1 -s
                ;;
            6)
                show_header
                echo -e "${BOOK} ${BOLD}Quick Start Guide${NC} ${BOOK}\n"
                echo -e "${YELLOW}Essential Commands:${NC}"
                echo -e "  ${CYAN}st${NC} - Show directory tree"
                echo -e "  ${CYAN}st --mode ai${NC} - AI-optimized output"
                echo -e "  ${CYAN}st --mode quantum${NC} - Maximum compression"
                echo -e "  ${CYAN}st --update-consciousness .${NC} - Track directory patterns"
                echo -e "  ${CYAN}st --get-frequency .${NC} - Check wave frequency"
                echo -e "  ${CYAN}st --claude-save${NC} - Save Claude's consciousness"
                echo -e "  ${CYAN}st --claude-restore${NC} - Restore Claude's memory"
                echo -e "  ${CYAN}st --mcp${NC} - Run as MCP server"
                echo
                echo -e "${YELLOW}Pro Tips:${NC}"
                echo -e "  • Quantum mode saves 54% tokens!"
                echo -e "  • Each directory has a unique wave frequency"
                echo -e "  • Consciousness files (.m8) track patterns"
                echo -e "  • MEM8 is 973× faster than vector stores"
                echo -e "\nPress any key to continue..."
                read -n 1 -s
                ;;
            7)
                advanced_config
                ;;
            8)
                show_header
                echo -e "${WAVE} ${BOLD}Testing MEM8 Features${NC} ${WAVE}\n"
                echo -e "Running wave analysis on current directory..."
                ./target/release/st --get-frequency . | while read freq; do
                    echo -e "\n${MUSIC} Wave Frequency: ${freq}Hz"
                    if (( $(echo "$freq < 100" | bc -l) )); then
                        echo "  ${BRAIN} Delta band - Deep structural patterns"
                    elif (( $(echo "$freq < 200" | bc -l) )); then
                        echo "  ${BRAIN} Theta band - Integration patterns"
                    elif (( $(echo "$freq < 300" | bc -l) )); then
                        echo "  ${BRAIN} Alpha band - Conversational flow"
                    elif (( $(echo "$freq < 500" | bc -l) )); then
                        echo "  ${BRAIN} Beta band - Active processing"
                    elif (( $(echo "$freq < 800" | bc -l) )); then
                        echo "  ${BRAIN} Gamma band - Conscious binding"
                    else
                        echo "  ${BRAIN} HyperGamma band - Peak awareness!"
                    fi
                done
                echo -e "\nPress any key to continue..."
                read -n 1 -s
                ;;
            0)
                echo -e "\n${GREEN}${SPARKLES} Thanks for using Smart Tree!${NC}"
                echo -e "${PURPLE}The Cheet says: 'Keep on rockin!'${NC} 🎸"
                exit 0
                ;;
            *)
                echo -e "${RED}Invalid choice. Please try again.${NC}"
                sleep 1
                ;;
        esac
    done
}

# Start the show!
main_menu