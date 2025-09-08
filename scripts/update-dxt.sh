#!/usr/bin/env bash
#
# Smart Tree DXT Auto-Updater
# 
# This script automatically downloads the latest smart-tree.dxt from GitHub releases
# and optionally installs it to Claude Desktop
#
# Usage:
#   ./scripts/update-dxt.sh                    # Download latest DXT
#   ./scripts/update-dxt.sh --install          # Download and install to Claude Desktop
#   ./scripts/update-dxt.sh --check            # Just check for updates
#

set -euo pipefail

# Configuration
GITHUB_REPO="8b-is/smart-tree"
DXT_NAME="smart-tree.dxt"
DOWNLOAD_DIR="${DOWNLOAD_DIR:-$HOME/Downloads}"
CLAUDE_DESKTOP_EXTENSIONS_DIR="$HOME/Library/Application Support/Claude/extensions"

# Colors for output
if [[ -t 1 ]] && [[ "${NO_COLOR:-}" != "1" ]]; then
    RED=$'\033[0;31m'
    GREEN=$'\033[0;32m'
    YELLOW=$'\033[1;33m'
    BLUE=$'\033[0;34m'
    PURPLE=$'\033[0;35m'
    CYAN=$'\033[0;36m'
    NC=$'\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    PURPLE=''
    CYAN=''
    NC=''
fi

print_header() {
    echo -e "\n${CYAN}🌳 Smart Tree DXT Auto-Updater 🌳${NC}\n"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
    exit 1
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check dependencies
check_dependencies() {
    if ! command_exists curl; then
        print_error "curl is required but not installed"
    fi
    
    if ! command_exists jq; then
        print_warning "jq not found, using basic parsing (install jq for better experience)"
    fi
}

# Get latest release info from GitHub API
get_latest_release() {
    local api_url="https://api.github.com/repos/$GITHUB_REPO/releases/latest"
    
    print_info "Fetching latest release information..."
    
    if command_exists jq; then
        # Use jq for proper JSON parsing
        local release_info
        release_info=$(curl -s "$api_url")
        
        local tag_name
        tag_name=$(echo "$release_info" | jq -r '.tag_name')
        
        local dxt_download_url
        dxt_download_url=$(echo "$release_info" | jq -r ".assets[] | select(.name == \"$DXT_NAME\") | .browser_download_url")
        
        if [[ "$dxt_download_url" == "null" || -z "$dxt_download_url" ]]; then
            print_error "DXT file not found in latest release $tag_name"
        fi
        
        echo "$tag_name|$dxt_download_url"
    else
        # Fallback parsing without jq
        local release_info
        release_info=$(curl -s "$api_url")
        
        local tag_name
        tag_name=$(echo "$release_info" | grep -o '"tag_name":"[^"]*"' | cut -d'"' -f4)
        
        local dxt_download_url
        dxt_download_url=$(echo "$release_info" | grep -o "\"browser_download_url\":\"[^\"]*$DXT_NAME\"" | cut -d'"' -f4)
        
        if [[ -z "$dxt_download_url" ]]; then
            print_error "DXT file not found in latest release $tag_name"
        fi
        
        echo "$tag_name|$dxt_download_url"
    fi
}

# Check current installed version (if any)
get_installed_version() {
    local installed_dxt="$CLAUDE_DESKTOP_EXTENSIONS_DIR/$DXT_NAME"
    
    if [[ -f "$installed_dxt" ]]; then
        # Try to extract version from DXT manifest
        local temp_dir
        temp_dir=$(mktemp -d)
        trap 'rm -rf "$temp_dir"' EXIT
        
        if unzip -q "$installed_dxt" -d "$temp_dir" 2>/dev/null; then
            if [[ -f "$temp_dir/manifest.json" ]]; then
                if command_exists jq; then
                    jq -r '.version // "unknown"' "$temp_dir/manifest.json" 2>/dev/null || echo "unknown"
                else
                    grep -o '"version":"[^"]*"' "$temp_dir/manifest.json" 2>/dev/null | cut -d'"' -f4 || echo "unknown"
                fi
            else
                echo "unknown"
            fi
        else
            echo "unknown"
        fi
    else
        echo "not_installed"
    fi
}

# Download DXT file
download_dxt() {
    local download_url="$1"
    local version="$2"
    local output_path="$DOWNLOAD_DIR/$DXT_NAME"
    
    print_info "Downloading Smart Tree DXT v$version..."
    print_info "From: $download_url"
    print_info "To: $output_path"
    
    mkdir -p "$DOWNLOAD_DIR"
    
    if curl -L -o "$output_path" "$download_url"; then
        print_success "Downloaded $DXT_NAME to $output_path"
        echo "$output_path"
    else
        print_error "Failed to download DXT file"
    fi
}

# Install DXT to Claude Desktop
install_dxt() {
    local dxt_path="$1"
    local version="$2"
    
    print_info "Installing Smart Tree DXT v$version to Claude Desktop..."
    
    # Create extensions directory if it doesn't exist
    mkdir -p "$CLAUDE_DESKTOP_EXTENSIONS_DIR"
    
    # Backup existing installation
    local installed_dxt="$CLAUDE_DESKTOP_EXTENSIONS_DIR/$DXT_NAME"
    if [[ -f "$installed_dxt" ]]; then
        local backup_path="${installed_dxt}.backup.$(date +%Y%m%d_%H%M%S)"
        print_info "Backing up existing installation to $(basename "$backup_path")"
        cp "$installed_dxt" "$backup_path"
    fi
    
    # Copy new DXT
    cp "$dxt_path" "$installed_dxt"
    
    print_success "Smart Tree DXT v$version installed successfully!"
    print_info "Restart Claude Desktop to load the updated extension"
    
    # Show installation instructions
    echo ""
    echo -e "${YELLOW}📋 Manual Installation (if auto-install doesn't work):${NC}"
    echo "   1. Open Claude Desktop"
    echo "   2. Go to Settings > Developer"
    echo "   3. Click 'Install from file'"
    echo "   4. Select: $installed_dxt"
}

# Check for updates only
check_updates() {
    print_header
    
    local release_info
    release_info=$(get_latest_release)
    local latest_version
    latest_version=$(echo "$release_info" | cut -d'|' -f1)
    
    local installed_version
    installed_version=$(get_installed_version)
    
    print_info "Latest version: $latest_version"
    print_info "Installed version: $installed_version"
    
    if [[ "$installed_version" == "not_installed" ]]; then
        print_warning "Smart Tree DXT is not installed"
        echo -e "\n${BLUE}To install: $0 --install${NC}"
    elif [[ "$installed_version" == "$latest_version" ]] || [[ "$installed_version" == "${latest_version#v}" ]]; then
        print_success "Smart Tree DXT is up to date!"
    else
        print_warning "Update available: $installed_version → $latest_version"
        echo -e "\n${BLUE}To update: $0 --install${NC}"
    fi
}

# Main update function
main_update() {
    local should_install="$1"
    
    print_header
    check_dependencies
    
    # Get latest release info
    local release_info
    release_info=$(get_latest_release)
    local latest_version
    latest_version=$(echo "$release_info" | cut -d'|' -f1)
    local download_url
    download_url=$(echo "$release_info" | cut -d'|' -f2)
    
    print_success "Found latest version: $latest_version"
    
    # Check current version
    local installed_version
    installed_version=$(get_installed_version)
    
    if [[ "$installed_version" != "not_installed" ]]; then
        print_info "Currently installed: $installed_version"
        
        if [[ "$installed_version" == "$latest_version" ]] || [[ "$installed_version" == "${latest_version#v}" ]]; then
            print_success "Already up to date!"
            if [[ "$should_install" != "true" ]]; then
                exit 0
            fi
            print_info "Reinstalling anyway..."
        fi
    fi
    
    # Download DXT
    local downloaded_dxt
    downloaded_dxt=$(download_dxt "$download_url" "$latest_version")
    
    # Install if requested
    if [[ "$should_install" == "true" ]]; then
        install_dxt "$downloaded_dxt" "$latest_version"
    else
        print_success "Download complete!"
        print_info "To install: $0 --install"
        print_info "DXT location: $downloaded_dxt"
    fi
}

# Show help
show_help() {
    cat << EOF
${CYAN}🌳 Smart Tree DXT Auto-Updater${NC}

${YELLOW}Usage:${NC}
  $0                    Download latest DXT to ~/Downloads
  $0 --install          Download and install to Claude Desktop
  $0 --check            Check for updates without downloading
  $0 --help             Show this help message

${YELLOW}Environment Variables:${NC}
  ${PURPLE}DOWNLOAD_DIR${NC}         Directory to download DXT (default: ~/Downloads)
  ${PURPLE}NO_COLOR=1${NC}           Disable colored output

${YELLOW}Examples:${NC}
  $0 --check            # Check if updates are available
  $0 --install          # Download and install latest version
  DOWNLOAD_DIR=/tmp $0  # Download to /tmp instead

${YELLOW}Requirements:${NC}
  - curl (required)
  - jq (optional, for better JSON parsing)
  - unzip (for version checking)

${CYAN}Made with 🌳 by the Smart Tree team!${NC}
EOF
}

# Parse command line arguments
case "${1:-}" in
    --install)
        main_update "true"
        ;;
    --check)
        check_updates
        ;;
    --help|-h)
        show_help
        ;;
    "")
        main_update "false"
        ;;
    *)
        print_error "Unknown option: $1\nUse --help for usage information"
        ;;
esac 