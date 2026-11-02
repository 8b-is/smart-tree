#!/usr/bin/env bash
#
# Smart Tree Completion Setup Script
#
# This script sets up shell completions for Smart Tree (st)
# It auto-detects your shell and installs appropriate completions
#
# Usage:
#   ./setup-completions.sh
#   curl -sSL https://raw.githubusercontent.com/8b-is/smart-tree/main/scripts/setup-completions.sh | bash

set -euo pipefail

# --- Configuration ---
GITHUB_REPO="8b-is/smart-tree"
BINARY_NAME="st"

# --- Helper Functions ---
print_info() {
    if [[ -t 1 ]]; then
        echo -e "\033[0;34m[INFO]\033[0m $1"
    else
        echo "[INFO] $1"
    fi
}

print_success() {
    if [[ -t 1 ]]; then
        echo -e "\033[0;32m[SUCCESS]\033[0m $1"
    else
        echo "[SUCCESS] $1"
    fi
}

print_error() {
    if [[ -t 1 ]]; then
        echo -e "\033[0;31m[ERROR]\033[0m $1" >&2
    else
        echo "[ERROR] $1" >&2
    fi
}

print_warning() {
    if [[ -t 1 ]]; then
        echo -e "\033[0;33m[WARNING]\033[0m $1"
    else
        echo "[WARNING] $1"
    fi
}

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if st is installed
if ! command_exists "$BINARY_NAME"; then
    print_error "Smart Tree ($BINARY_NAME) is not installed or not in PATH"
    print_info "Install it first with:"
    print_info "  curl -sSL https://raw.githubusercontent.com/$GITHUB_REPO/main/scripts/install.sh | bash"
    exit 1
fi

# Get st version
VERSION=$($BINARY_NAME --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)
print_info "Found Smart Tree version: $VERSION"

# --- Shell Detection and Completion Installation ---
detect_and_install_completions() {
    local shell_type
    
    # Detect user's shell
    if [[ -n "${SHELL:-}" ]]; then
        shell_type=$(basename "$SHELL")
    else
        shell_type=$(basename "$(getent passwd "$USER" | cut -d: -f7)")
    fi
    
    print_info "Detected shell: $shell_type"
    
    case "$shell_type" in
        bash)
            install_bash_completion
            ;;
        zsh)
            install_zsh_completion
            ;;
        fish)
            install_fish_completion
            ;;
        *)
            print_warning "Auto-completion setup not available for $shell_type"
            print_info "Available shells: bash, zsh, fish"
            print_info "You can generate completions manually with: st --completions <shell>"
            ;;
    esac
}

install_bash_completion() {
    local completion_dir
    local completion_file
    
    # Find bash completion directory (prefer user directory)
    if [[ -d "$HOME/.bash_completion.d" ]]; then
        completion_dir="$HOME/.bash_completion.d"
    elif [[ -d "$HOME/.local/share/bash-completion/completions" ]]; then
        completion_dir="$HOME/.local/share/bash-completion/completions"
    elif [[ -d "/etc/bash_completion.d" ]] && [[ -w "/etc/bash_completion.d" ]]; then
        completion_dir="/etc/bash_completion.d"
    else
        # Create user completion directory
        completion_dir="$HOME/.bash_completion.d"
        mkdir -p "$completion_dir"
    fi
    
    completion_file="$completion_dir/_st"
    
    print_info "Installing bash completions to $completion_file"
    if $BINARY_NAME --completions bash > "$completion_file" 2>/dev/null; then
        print_success "Bash completions installed!"
        
        # Check if user's bashrc sources completions
        if [[ -f "$HOME/.bashrc" ]] && ! grep -q "bash_completion.d\|bash-completion" "$HOME/.bashrc"; then
            echo ""
            print_info "To enable completions, add this to your .bashrc:"
            echo ""
            echo "# Smart Tree bash completions"
            echo "if [[ -d \"$completion_dir\" ]]; then"
            echo "    for f in \"$completion_dir\"/*; do"
            echo "        [[ -r \"\$f\" ]] && source \"\$f\""
            echo "    done"
            echo "fi"
            echo ""
            
            read -p "Add this to your .bashrc automatically? (y/n): " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                {
                    echo ""
                    echo "# Smart Tree bash completions"
                    echo "if [[ -d \"$completion_dir\" ]]; then"
                    echo "    for f in \"$completion_dir\"/*; do"
                    echo "        [[ -r \"\$f\" ]] && source \"\$f\""
                    echo "    done"
                    echo "fi"
                } >> "$HOME/.bashrc"
                print_success "Added to .bashrc!"
                print_info "Run 'source ~/.bashrc' or start a new terminal"
            fi
        else
            print_info "Completions will be available in new bash sessions"
        fi
    else
        print_error "Failed to generate bash completions"
    fi
}

install_zsh_completion() {
    local completion_dir
    local completion_file
    local use_enhanced=false
    
    # Check if enhanced completion is available
    if command_exists curl; then
        print_info "Checking for enhanced zsh completions..."
        if curl -fsS "https://raw.githubusercontent.com/$GITHUB_REPO/main/completions/_st_enhanced" >/dev/null 2>&1; then
            echo ""
            print_success "Enhanced completions available! Features:"
            echo "  ✨ Context-aware tips and suggestions"
            echo "  🎯 SQL-like query examples"
            echo "  💡 Common command patterns"
            echo "  🚀 Performance optimization hints"
            echo ""
            read -p "Install enhanced completions? (recommended) (y/n): " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                use_enhanced=true
            fi
        fi
    fi
    
    # Find zsh completion directory (prefer user directory)
    if [[ -d "$HOME/.zsh/completions" ]]; then
        completion_dir="$HOME/.zsh/completions"
    elif [[ -d "/usr/local/share/zsh/site-functions" ]] && [[ -w "/usr/local/share/zsh/site-functions" ]]; then
        completion_dir="/usr/local/share/zsh/site-functions"
    elif [[ -d "/opt/homebrew/share/zsh/site-functions" ]] && [[ -w "/opt/homebrew/share/zsh/site-functions" ]]; then
        completion_dir="/opt/homebrew/share/zsh/site-functions"
    else
        # Create user completion directory
        completion_dir="$HOME/.zsh/completions"
        mkdir -p "$completion_dir"
    fi
    
    completion_file="$completion_dir/_st"
    
    if [[ "$use_enhanced" == "true" ]]; then
        print_info "Downloading enhanced completions..."
        if curl -fsS "https://raw.githubusercontent.com/$GITHUB_REPO/main/completions/_st_enhanced" > "$completion_file"; then
            print_success "Enhanced zsh completions installed!"
            echo ""
            print_info "🎉 Special features enabled:"
            echo "  • Type 'st_tips' to see all tips and tricks"
            echo "  • Context-aware suggestions as you type"
            echo "  • SQL-like query examples in completions"
            echo ""
        else
            print_warning "Failed to download enhanced completions, using basic"
            use_enhanced=false
        fi
    fi
    
    if [[ "$use_enhanced" == "false" ]]; then
        print_info "Installing basic zsh completions to $completion_file"
        if $BINARY_NAME --completions zsh > "$completion_file" 2>/dev/null; then
            print_success "Zsh completions installed!"
        else
            print_error "Failed to generate zsh completions"
            return
        fi
    fi
    
    # Check if fpath includes our directory
    if [[ -f "$HOME/.zshrc" ]]; then
        if ! grep -q "$completion_dir" "$HOME/.zshrc" && ! grep -q "fpath.*completions" "$HOME/.zshrc"; then
            echo ""
            print_info "To enable completions, add this to your .zshrc:"
            echo ""
            echo "# Smart Tree completions"
            echo "fpath=(\"$completion_dir\" \$fpath)"
            echo "autoload -Uz compinit && compinit"
            echo ""
            
            read -p "Add this to your .zshrc automatically? (y/n): " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                {
                    echo ""
                    echo "# Smart Tree completions"
                    echo "fpath=(\"$completion_dir\" \$fpath)"
                    echo "autoload -Uz compinit && compinit"
                } >> "$HOME/.zshrc"
                print_success "Added to .zshrc!"
                print_info "Run 'source ~/.zshrc' or start a new terminal"
            fi
        else
            print_info "Completions will be available after running 'compinit'"
        fi
    fi
}

install_fish_completion() {
    local completion_dir="$HOME/.config/fish/completions"
    local completion_file="$completion_dir/st.fish"
    
    mkdir -p "$completion_dir"
    
    print_info "Installing fish completions to $completion_file"
    if $BINARY_NAME --completions fish > "$completion_file" 2>/dev/null; then
        print_success "Fish completions installed!"
        print_info "Completions are now available in fish"
    else
        print_error "Failed to generate fish completions"
    fi
}

# --- Main ---
echo "🌳 Smart Tree Completion Setup 🌳"
echo ""

detect_and_install_completions

echo ""
print_info "Test completions by typing: $BINARY_NAME <TAB><TAB>"
print_success "Setup complete! Happy tree browsing! 🎸"