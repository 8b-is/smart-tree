#!/usr/bin/env bash
#
# Smart Tree Installer Script
#
# This script installs the 'st' binary on Linux and macOS.
#
# Usage:
#   curl -sSL https://raw.githubusercontent.com/8b-is/smart-tree/main/scripts/install.sh | bash
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

# --- Main Installation Logic ---

# Check for required tools
if ! command_exists curl; then
    print_error "curl is required but not installed. Please install curl first."
fi

if ! command_exists jq; then
    print_error "jq is required to select a published release asset. Please install jq first."
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

# 2. Fetch exactly the requested release; never fall back to an older release.
release_endpoint="https://api.github.com/repos/$GITHUB_REPO/releases"
requested_version="${VERSION:-}"
if [[ -z "$requested_version" ]]; then
    release_endpoint+="/latest"
else
    [[ "$requested_version" =~ ^[a-zA-Z0-9][a-zA-Z0-9._-]*$ ]] || \
        print_error "Invalid VERSION: $requested_version"
    release_endpoint+="/tags/$requested_version"
fi

if ! release_json=$(curl --fail --silent --show-error --location "$release_endpoint"); then
    print_error "Could not fetch GitHub release metadata for ${requested_version:-latest} (HTTP or network failure)."
fi
# Reject API error objects, malformed JSON, and incomplete metadata before download.
if ! VERSION=$(jq -ers '
    if length == 1 then .[0] else error("Expected one release") end |
    select(type == "object") |
    select(.tag_name | type == "string" and test("^[a-zA-Z0-9][a-zA-Z0-9._-]*$")) |
    select(.assets | type == "array") |
    select(all(.assets[]; type == "object" and
        (.name | type == "string") and (.browser_download_url | type == "string"))) |
    .tag_name' <<< "$release_json"); then
    print_error "Invalid GitHub release metadata for ${requested_version:-latest}."
fi
if [[ -n "$requested_version" && "$VERSION" != "$requested_version" ]]; then
    print_error "Requested $requested_version but GitHub returned $VERSION; refusing to install another version."
fi
print_info "Selected release: $VERSION"

# 3. Match exact target filenames in the inventory, including historical names.
target="${arch}-${os}"
download_url=""
for archive_name in \
    "${BINARY_NAME}-${VERSION}-${target}.tar.gz" \
    "${BINARY_NAME}-${VERSION#v}-${target}.tar.gz" \
    "${BINARY_NAME}-${target}.tar.gz"; do
    expected_url="https://github.com/$GITHUB_REPO/releases/download/$VERSION/$archive_name"
    if download_url=$(jq -er --arg name "$archive_name" --arg url "$expected_url" '
        [.assets[] | select(.name == $name)] |
        select(length == 1) | .[0].browser_download_url | select(. == $url)
        ' <<< "$release_json"); then
        break
    fi
done
if [[ -z "$download_url" ]]; then
    print_error "Release $VERSION has no supported published asset for $target. No installation performed. Build from source or explicitly choose another VERSION."
fi

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
version_output=$("$INSTALL_DIR/$BINARY_NAME" --version)
print_success "Successfully installed $BINARY_NAME to $INSTALL_DIR/$BINARY_NAME"
print_info "Version: $version_output"
print_info "You can now use the '$BINARY_NAME' command."

# --- Shell Completion Installation ---
install_shell_completions() {
    local shell_type
    local completion_installed=false
    
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
            print_info "Auto-completion setup not available for $shell_type"
            print_info "You can generate completions manually with: st --completions <shell>"
            return
            ;;
    esac
}

install_bash_completion() {
    local completion_dir
    local completion_file
    
    # Find bash completion directory
    if [[ -d "/etc/bash_completion.d" ]] && [[ -w "/etc/bash_completion.d" ]]; then
        completion_dir="/etc/bash_completion.d"
    elif [[ -d "$HOME/.local/share/bash-completion/completions" ]]; then
        completion_dir="$HOME/.local/share/bash-completion/completions"
    elif [[ -d "$HOME/.bash_completion.d" ]]; then
        completion_dir="$HOME/.bash_completion.d"
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
        if [[ -f "$HOME/.bashrc" ]] && ! grep -q "bash_completion.d" "$HOME/.bashrc"; then
            print_info "Would you like to add completion sourcing to your .bashrc?"
            read -p "(y/n): " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                echo "" >> "$HOME/.bashrc"
                echo "# Smart Tree bash completions" >> "$HOME/.bashrc"
                echo "if [[ -d \"$completion_dir\" ]]; then" >> "$HOME/.bashrc"
                echo "    for f in \"$completion_dir\"/*; do" >> "$HOME/.bashrc"
                echo "        [[ -r \"\$f\" ]] && source \"\$f\"" >> "$HOME/.bashrc"
                echo "    done" >> "$HOME/.bashrc"
                echo "fi" >> "$HOME/.bashrc"
                print_success "Added completion sourcing to .bashrc"
                print_info "Run 'source ~/.bashrc' to enable completions in current session"
            fi
        fi
    else
        print_warning "Failed to generate bash completions"
    fi
}

install_zsh_completion() {
    local completion_dir
    local completion_file
    local use_enhanced=false
    
    # Check if enhanced completion exists
    if command_exists curl && curl -s "https://raw.githubusercontent.com/$GITHUB_REPO/$VERSION/completions/_st_enhanced" >/dev/null 2>&1; then
        print_info "Enhanced zsh completions with tips are available!"
        read -p "Install enhanced completions with tips and examples? (y/n): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            use_enhanced=true
        fi
    fi
    
    # Find zsh completion directory
    if [[ -d "/usr/local/share/zsh/site-functions" ]] && [[ -w "/usr/local/share/zsh/site-functions" ]]; then
        completion_dir="/usr/local/share/zsh/site-functions"
    elif [[ -d "$HOME/.zsh/completions" ]]; then
        completion_dir="$HOME/.zsh/completions"
    else
        # Create user completion directory
        completion_dir="$HOME/.zsh/completions"
        mkdir -p "$completion_dir"
    fi
    
    completion_file="$completion_dir/_st"
    
    if [[ "$use_enhanced" == "true" ]]; then
        print_info "Installing enhanced zsh completions to $completion_file"
        if curl -s "https://raw.githubusercontent.com/$GITHUB_REPO/$VERSION/completions/_st_enhanced" > "$completion_file"; then
            print_success "Enhanced zsh completions installed!"
            print_info "Tips: Use 'st_tips' command to see all tips and tricks"
        else
            print_warning "Failed to download enhanced completions, falling back to basic"
            use_enhanced=false
        fi
    fi
    
    if [[ "$use_enhanced" == "false" ]]; then
        print_info "Installing zsh completions to $completion_file"
        if $BINARY_NAME --completions zsh > "$completion_file" 2>/dev/null; then
            print_success "Zsh completions installed!"
        else
            print_warning "Failed to generate zsh completions"
            return
        fi
    fi
    
    # Check if user's zshrc includes the completion directory in fpath
    if [[ -f "$HOME/.zshrc" ]] && ! grep -q "$completion_dir" "$HOME/.zshrc"; then
        print_info "Would you like to add the completion directory to your .zshrc?"
        read -p "(y/n): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            echo "" >> "$HOME/.zshrc"
            echo "# Smart Tree zsh completions" >> "$HOME/.zshrc"
            echo "fpath=(\"$completion_dir\" \$fpath)" >> "$HOME/.zshrc"
            echo "autoload -Uz compinit && compinit" >> "$HOME/.zshrc"
            print_success "Added completion setup to .zshrc"
            print_info "Run 'source ~/.zshrc' to enable completions in current session"
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
        print_info "Completions will be available in new fish sessions"
    else
        print_warning "Failed to generate fish completions"
    fi
}

# Offer to install completions
# Only prompt if running in a TTY; otherwise skip interactive prompt.
if [[ -t 0 && -t 1 ]]; then
    print_info ""
    read -r -n 1 -p "Would you like to install shell completions for $BINARY_NAME? (y/n): " REPLY
    printf "\n"
else
    # Non-interactive session: do not prompt
    REPLY="n"
fi

if [[ $REPLY =~ ^[Yy]$ ]]; then
    install_shell_completions
else
    print_info "You can install completions later with: $BINARY_NAME --completions <shell>"
fi

# Elvis has left the building! 🎸
print_success "Thank you, thank you very much! Smart Tree is ready to rock! 🌳🎸"