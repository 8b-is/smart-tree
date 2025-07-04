#!/usr/bin/env bash
#
# Smart Tree Installer Script
#
# This script installs the 'st' binary on Linux and macOS.
#
# Usage:
#   curl -sSL https://raw.githubusercontent.com/user/repo/main/scripts/install.sh | bash
#
# You can customize the installation by setting environment variables:
#   - INSTALL_DIR: The directory to install the binary to (default: /usr/local/bin)
#   - VERSION: The version to install (default: latest)

set -euo pipefail

# --- Configuration ---
GITHUB_REPO="8b-is/smart-tree"
BINARY_NAME="st"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

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
    exit 1
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

# Function to check if a release has assets
check_release_assets() {
    local version="$1"
    local assets_count
    assets_count=$(curl -s "https://api.github.com/repos/$GITHUB_REPO/releases/tags/$version" | jq -r '.assets | length')
    
    if [[ "$assets_count" -gt 0 ]]; then
        return 0
    else
        return 1
    fi
}

# Function to get releases with assets
get_releases_with_assets() {
    curl -s "https://api.github.com/repos/$GITHUB_REPO/releases" | \
    jq -r '.[] | select(.assets | length > 0) | .tag_name' | \
    head -10
}

# Function to select version interactively
select_version() {
    print_warning "The latest version ($1) doesn't have any release binaries yet."
    print_info "Fetching other available versions with binaries..."
    
    local versions=()
    while IFS= read -r version; do
        versions+=("$version")
    done < <(get_releases_with_assets)
    
    if [[ ${#versions[@]} -eq 0 ]]; then
        print_error "No releases with binaries found!"
    fi
    
    print_info "Available versions with binaries:"
    for i in "${!versions[@]}"; do
        echo "  $((i+1)). ${versions[$i]}"
    done
    
    local selection
    while true; do
        read -p "Select a version (1-${#versions[@]}) or 'q' to quit: " selection
        
        if [[ "$selection" == "q" ]]; then
            print_info "Installation cancelled."
            exit 0
        fi
        
        if [[ "$selection" =~ ^[0-9]+$ ]] && (( selection >= 1 && selection <= ${#versions[@]} )); then
            VERSION="${versions[$((selection-1))]}"
            print_info "Selected version: $VERSION"
            break
        else
            print_error "Invalid selection. Please try again."
        fi
    done
}

# --- Main Installation Logic ---

# Check for required tools
if ! command_exists curl; then
    print_error "curl is required but not installed. Please install curl first."
fi

if ! command_exists jq; then
    print_warning "jq is not installed. Some features may not work properly."
    print_info "Consider installing jq for better release detection."
fi

# 1. Detect OS and Architecture
os_type="$(uname -s)"
architecture="$(uname -m)"

case "$os_type" in
    Linux)
        os="unknown-linux-gnu"
        ;;
    Darwin)
        os="apple-darwin"
        ;;
    *)
        print_error "Unsupported OS: $os_type"
        ;;
esac

case "$architecture" in
    x86_64)
        arch="x86_64"
        ;;
    arm64 | aarch64)
        arch="aarch64"
        ;;
    *)
        print_error "Unsupported architecture: $architecture"
        ;;
esac

# On Apple Silicon, prefer homebrew's default directory if it exists and the
# user has not specified a custom INSTALL_DIR.
if [[ "$os" == "apple-darwin" && "$arch" == "aarch64" && -d "/opt/homebrew/bin" ]]; then
    if [[ "$INSTALL_DIR" == "/usr/local/bin" ]]; then
        INSTALL_DIR="/opt/homebrew/bin"
    fi
fi

# 2. Determine Version to Install
if [[ -z "${VERSION:-}" ]]; then
    print_info "Fetching the latest version number..."
    # Use GitHub API to get the latest release tag
    latest_version_tag=$(curl --silent "https://api.github.com/repos/$GITHUB_REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    if [[ -z "$latest_version_tag" ]]; then
        print_error "Could not fetch the latest version. Please check the repository path and your network connection."
    fi
    
    # Check if latest version has assets
    if command_exists jq && ! check_release_assets "$latest_version_tag"; then
        select_version "$latest_version_tag"
    else
        VERSION="$latest_version_tag"
        print_info "Latest version is $VERSION"
    fi
else
    print_info "Installing specified version: $VERSION"
    
    # Check if specified version has assets
    if command_exists jq && ! check_release_assets "$VERSION"; then
        print_warning "Version $VERSION doesn't have any release binaries."
        read -p "Would you like to select another version? (y/n): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            select_version "$VERSION"
        else
            print_error "Cannot install version without binaries."
        fi
    fi
fi

# 3. Construct Download URL
# Note: As of v2.0.3, artifacts no longer include version in filename
archive_name="${BINARY_NAME}-${arch}-${os}.tar.gz"
download_url="https://github.com/$GITHUB_REPO/releases/download/$VERSION/$archive_name"

# 4. Download and Extract
tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT

print_info "Downloading from $download_url"
if ! curl --progress-bar --fail -L "$download_url" -o "$tmp_dir/$archive_name"; then
    print_error "Download failed. Please check the URL and your network."
fi

print_info "Extracting the binary..."
tar -xzf "$tmp_dir/$archive_name" -C "$tmp_dir"

# 5. Install the binary
print_info "Installing to $INSTALL_DIR..."
binary_path="$tmp_dir/$BINARY_NAME"
if [[ ! -f "$binary_path" ]]; then
    # Sometimes the binary is in a subdirectory, e.g. target/release/
    found_binary=$(find "$tmp_dir" -type f -name $BINARY_NAME | head -n 1)
    if [[ -z "$found_binary" ]]; then
        print_error "Could not find the '$BINARY_NAME' binary in the downloaded archive."
    fi
    binary_path=$found_binary
fi

if [[ -w "$INSTALL_DIR" ]]; then
    mv "$binary_path" "$INSTALL_DIR/$BINARY_NAME"
else
    print_info "Need sudo access to install to $INSTALL_DIR"
    sudo mv "$binary_path" "$INSTALL_DIR/$BINARY_NAME"
fi

# 6. Verify Installation
if ! command_exists "$BINARY_NAME"; then
    print_error "Installation failed. The '$BINARY_NAME' command is not available in your PATH."
fi

version_output=$($BINARY_NAME --version)
print_success "Successfully installed $BINARY_NAME to $INSTALL_DIR/$BINARY_NAME"
print_info "Version: $version_output"
print_info "You can now use the '$BINARY_NAME' command."

# Elvis has left the building! 🎸
print_success "Thank you, thank you very much! Smart Tree is ready to rock! 🌳🎸" 