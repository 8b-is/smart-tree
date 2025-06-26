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

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# --- Main Installation Logic ---

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

# 2. Determine Version to Install
if [[ -z "${VERSION:-}" ]]; then
    print_info "Fetching the latest version number..."
    # Use GitHub API to get the latest release tag
    latest_version_tag=$(curl --silent "https://api.github.com/repos/$GITHUB_REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    if [[ -z "$latest_version_tag" ]]; then
        print_error "Could not fetch the latest version. Please check the repository path and your network connection."
    fi
    VERSION="$latest_version_tag"
    print_info "Latest version is $VERSION"
else
    print_info "Installing specified version: $VERSION"
fi

# 3. Construct Download URL
archive_name="${BINARY_NAME}-${VERSION}-${arch}-${os}.tar.gz"
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