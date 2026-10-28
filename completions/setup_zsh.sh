#!/usr/bin/env zsh
# Smart Tree Zsh Completion Setup Script
# Sets up enhanced completions and auto-suggestions

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo "${BLUE}🌳 Smart Tree Zsh Completion Setup 🌳${NC}"
echo ""

# Find the script directory
SCRIPT_DIR="${0:A:h}"
ST_COMPLETION_FILE="$SCRIPT_DIR/_st_enhanced"

# Check if completion file exists
if [[ ! -f "$ST_COMPLETION_FILE" ]]; then
    echo "${RED}Error: Completion file not found at $ST_COMPLETION_FILE${NC}"
    exit 1
fi

# Determine completion directory
if [[ -n "$1" ]]; then
    COMPLETION_DIR="$1"
else
    # Try to find a suitable directory in fpath
    for dir in ${fpath[@]}; do
        if [[ -w "$dir" ]] && [[ "$dir" != "/usr/"* ]]; then
            COMPLETION_DIR="$dir"
            break
        fi
    done
    
    # Fallback to user directory
    if [[ -z "$COMPLETION_DIR" ]]; then
        COMPLETION_DIR="$HOME/.zsh/completions"
    fi
fi

# Create directory if needed
if [[ ! -d "$COMPLETION_DIR" ]]; then
    echo "${YELLOW}Creating completion directory: $COMPLETION_DIR${NC}"
    mkdir -p "$COMPLETION_DIR"
fi

# Install the completion file
echo "${GREEN}Installing enhanced completion to: $COMPLETION_DIR/_st${NC}"
cp "$ST_COMPLETION_FILE" "$COMPLETION_DIR/_st"
chmod 644 "$COMPLETION_DIR/_st"

# Check if directory is in fpath
if [[ ${fpath[(I)$COMPLETION_DIR]} -eq 0 ]]; then
    echo ""
    echo "${YELLOW}⚠️  The completion directory is not in your fpath!${NC}"
    echo "Add this line to your ~/.zshrc:"
    echo ""
    echo "  fpath=($COMPLETION_DIR \$fpath)"
    echo ""
fi

# Check for zsh-autosuggestions
if command -v brew &> /dev/null && brew list zsh-autosuggestions &> /dev/null; then
    echo "${GREEN}✓ zsh-autosuggestions is installed via Homebrew${NC}"
elif [[ -f /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh ]]; then
    echo "${GREEN}✓ zsh-autosuggestions is installed${NC}"
else
    echo ""
    echo "${YELLOW}📝 Optional: Install zsh-autosuggestions for better experience${NC}"
    echo "  macOS:  brew install zsh-autosuggestions"
    echo "  Ubuntu: sudo apt install zsh-autosuggestions"
    echo "  Arch:   sudo pacman -S zsh-autosuggestions"
fi

# Create config directory
CONFIG_DIR="$HOME/.config/st"
mkdir -p "$CONFIG_DIR"

# Create a starter config
if [[ ! -f "$CONFIG_DIR/config.zsh" ]]; then
    cat > "$CONFIG_DIR/config.zsh" << 'EOF'
# Smart Tree Zsh Configuration
# Source this file in your ~/.zshrc: source ~/.config/st/config.zsh

# Enable helpful aliases
export ST_COMPLETION_ALIASES=1

# Custom aliases for common operations
alias stai="st . --mode summary-ai -z"          # AI-optimized summary
alias stfind="st . --find"                      # Find files/dirs
alias stsearch="st . --search"                  # Search in files
alias stwaste="st . --mode waste"               # Find large files
alias stls="st . --mode ls"                     # ls-style output
alias stquick="st . -d 3 --mode summary-ai -z"  # Quick 3-level summary
alias strecent="st . --newer-than \$(date -d '7 days ago' +%Y-%m-%d 2>/dev/null || date -v-7d +%Y-%m-%d)"

# Function to show Smart Tree tips
alias sttips="st_tips"

# Auto-suggestion patterns (if zsh-autosuggestions is installed)
if [[ -n "$ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE" ]]; then
    # Seed history with common commands for auto-suggestions
    () {
        local st_commands=(
            "st . --mode summary-ai -z"
            "st . --find 'TODO' --mode ls"
            "st . --search 'function'"
            "st . --mode waste --min-size 10M"
            "st . --newer-than"
            "st . --mode quantum-semantic"
            "st . --file-type rs --mode stats"
        )
        
        for cmd in $st_commands; do
            print -S "$cmd"
        done
    }
fi
EOF
    echo "${GREEN}✓ Created config file: $CONFIG_DIR/config.zsh${NC}"
fi

# Show final instructions
echo ""
echo "${BLUE}=== Setup Complete! ===${NC}"
echo ""
echo "To activate the enhanced completions, add these lines to your ~/.zshrc:"
echo ""
echo "${YELLOW}# Smart Tree enhanced completions${NC}"
if [[ ${fpath[(I)$COMPLETION_DIR]} -eq 0 ]]; then
    echo "fpath=($COMPLETION_DIR \$fpath)"
fi
echo "autoload -Uz compinit && compinit"
echo "source ~/.config/st/config.zsh"
echo ""
echo "${GREEN}Then reload your shell:${NC}"
echo "  source ~/.zshrc"
echo ""
echo "${GREEN}Try these commands:${NC}"
echo "  st <TAB>           # See completion options"
echo "  st --mode <TAB>    # See all output modes with descriptions"
echo "  sttips             # Show tips and tricks"
echo "  stai               # Quick AI summary of current directory"
echo ""
echo "${BLUE}Happy tree climbing! 🌳${NC}"