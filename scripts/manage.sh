#!/usr/bin/env bash
# 🌳 Smart Tree Management Script - Because every tree needs a gardener! 🌳

set -euo pipefail

# Colors for our fancy output
if [[ -t 1 ]] && [[ "${NO_COLOR:-}" != "1" ]]; then
    RED=$'\033[0;31m'
    GREEN=$'\033[0;32m'
    YELLOW=$'\033[1;33m'
    BLUE=$'\033[0;34m'
    PURPLE=$'\033[0;35m'
    CYAN=$'\033[0;36m'
    NC=$'\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    PURPLE=''
    CYAN=''
    NC=''
fi

# Project info
PROJECT_NAME="Smart Tree (st)"
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY_NAME="st"

# Non-interactive mode flag
NON_INTERACTIVE=${NON_INTERACTIVE:-false}

# Emojis for fun (can be disabled)
if [[ "${NO_EMOJI:-}" == "1" ]]; then
    TREE="[TREE]"
    ROCKET="[GO]"
    GEAR="[BUILD]"
    TEST="[TEST]"
    CLEAN="[CLEAN]"
    INFO="[INFO]"
    CHECK="[OK]"
    CROSS="[FAIL]"
    SPARKLE="[*]"
else
    TREE="🌳"
    ROCKET="🚀"
    GEAR="⚙️"
    TEST="🧪"
    CLEAN="🧹"
    INFO="📊"
    CHECK="✅"
    CROSS="❌"
    SPARKLE="✨"
fi

# Helper functions
print_header() {
    echo -e "\n${CYAN}${TREE} $1 ${TREE}${NC}\n"
}

print_success() {
    echo -e "${GREEN}${CHECK} $1${NC}"
}

print_error() {
    echo -e "${RED}${CROSS} $1${NC}"
}

print_info() {
    echo -e "${BLUE}${INFO} $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Show animated spinner
spinner() {
    local pid=$1
    local delay=0.1
    local spinstr='⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏'
    while ps -p $pid > /dev/null 2>&1; do
        local temp=${spinstr#?}
        printf " [%c]  " "$spinstr"
        local spinstr=$temp${spinstr%"$temp"}
        sleep $delay
        printf "\b\b\b\b\b\b"
    done
    printf "    \b\b\b\b"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Build the project
build() {
    local build_type="${1:-release}"
    local features="${2:-}"
    print_header "Building $PROJECT_NAME in $build_type mode ${GEAR}"
    
    cd "$PROJECT_DIR"
    
    local feature_flags=""
    if [[ -n "$features" ]]; then
        feature_flags="--features $features"
        print_info "Building with features: $features"
    fi
    
    if [[ "$build_type" == "release" ]]; then
        print_info "Optimizing for maximum speed... ${ROCKET}"
        if [[ "$NON_INTERACTIVE" == "true" ]]; then
            cargo build --release $feature_flags
        else
            cargo build --release $feature_flags &
            spinner $!
        fi
        print_success "Release build complete! Binary size: $(du -h target/release/$BINARY_NAME | cut -f1)"
    else
        print_info "Building debug version with all the debugging goodies..."
        cargo build $feature_flags
        print_success "Debug build complete!"
    fi
}

# Run the project
run() {
    print_header "Running $PROJECT_NAME ${ROCKET}"
    cd "$PROJECT_DIR"
    
    # Default to current directory if no args provided
    if [[ $# -eq 0 ]]; then
        print_info "No arguments provided, analyzing current directory..."
        cargo run --release -- .
    else
        cargo run --release "$@"
    fi
}

# Run tests
test() {
    print_header "Testing $PROJECT_NAME ${TEST}"
    cd "$PROJECT_DIR"
    
    print_info "Running unit tests..."
    cargo test
    
    print_info "Running clippy (our friendly neighborhood linter)..."
    cargo clippy -- -D warnings || print_warning "Clippy found some issues!"
    
    print_info "Checking formatting..."
    cargo fmt -- --check || print_warning "Code needs formatting! Run './manage.sh format' to fix."
    
    print_success "All tests passed! Your tree is healthy! ${TREE}"
}

# Format code
format() {
    print_header "Formatting code ${SPARKLE}"
    cd "$PROJECT_DIR"
    
    cargo fmt
    print_success "Code formatted! Looking prettier than a bonsai tree! 🎋"
}

# Clean build artifacts
clean() {
    print_header "Cleaning up ${CLEAN}"
    cd "$PROJECT_DIR"
    
    cargo clean
    print_success "All clean! Fresh as a spring forest! 🌱"
}

# Show project status
status() {
    print_header "Project Status ${INFO}"
    cd "$PROJECT_DIR"
    
    echo -e "${PURPLE}Project:${NC} $PROJECT_NAME"
    echo -e "${PURPLE}Location:${NC} $PROJECT_DIR"
    echo -e "${PURPLE}Rust version:${NC} $(rustc --version)"
    echo -e "${PURPLE}Cargo version:${NC} $(cargo --version)"
    
    if [[ -f "target/release/$BINARY_NAME" ]]; then
        echo -e "${PURPLE}Release binary:${NC} $(du -h target/release/$BINARY_NAME | cut -f1)"
        echo -e "${PURPLE}Last modified:${NC} $(date -r target/release/$BINARY_NAME '+%Y-%m-%d %H:%M:%S')"
    else
        echo -e "${PURPLE}Release binary:${NC} Not built yet"
    fi
    
    echo -e "\n${PURPLE}Dependencies:${NC}"
    cargo tree --depth 1 | head -20
    
    echo -e "\n${PURPLE}Git status:${NC}"
    if command_exists git && git rev-parse --git-dir > /dev/null 2>&1; then
        git status --short || echo "  Clean working tree ${CHECK}"
    else
        echo "  Not a git repository"
    fi
}

# Run benchmarks
bench() {
    print_header "Running Benchmarks 📈"
    cd "$PROJECT_DIR"
    
    print_info "Building optimized version..."
    cargo build --release
    
    print_info "Benchmarking on current directory..."
    time ./target/release/$BINARY_NAME . -m hex > /dev/null
    
    print_info "Benchmarking with compression..."
    time ./target/release/$BINARY_NAME . -m ai -z > /dev/null
    
    if [[ -d "/usr" ]]; then
        print_info "Benchmarking on /usr (large directory)..."
        time ./target/release/$BINARY_NAME /usr -m hex --depth 3 > /dev/null || true
    fi
}

# Install binary
install() {
    print_header "Installing $PROJECT_NAME 🎯"
    cd "$PROJECT_DIR"
    
    local install_dir="${1:-/usr/local/bin}"
    
    print_info "Building release version..."
    cargo build --release
    
    if [[ -w "$install_dir" ]]; then
        cp "target/release/$BINARY_NAME" "$install_dir/"
        print_success "Installed to $install_dir/$BINARY_NAME"
    else
        print_warning "Need sudo access to install to $install_dir"
        sudo cp "target/release/$BINARY_NAME" "$install_dir/"
        print_success "Installed to $install_dir/$BINARY_NAME (with sudo)"
    fi
    
    print_info "You can now use '$BINARY_NAME' from anywhere! ${ROCKET}"
}

# Uninstall binary
uninstall() {
    print_header "Uninstalling $PROJECT_NAME 😢"
    
    local install_dir="${1:-/usr/local/bin}"
    local binary_path="$install_dir/$BINARY_NAME"
    
    if [[ -f "$binary_path" ]]; then
        if [[ -w "$install_dir" ]]; then
            rm "$binary_path"
        else
            sudo rm "$binary_path"
        fi
        print_success "Uninstalled from $binary_path"
    else
        print_error "$BINARY_NAME not found in $install_dir"
    fi
}

# Create a new GitHub release
release() {
    print_header "Creating a new GitHub release ${ROCKET}"

    local version="${1-}"
    if [[ -z "$version" ]]; then
        print_error "Version tag is required. Example: v1.0.0"
        exit 1
    fi

    if ! command_exists gh; then
        print_error "GitHub CLI (gh) is not installed. Please install it to create releases."
        print_info "See: https://cli.github.com/"
        exit 1
    fi

    if [[ -n "$(git status --porcelain)" ]]; then
        print_error "Your working directory is not clean. Please commit or stash your changes."
        git status --short
        exit 1
    fi

    print_info "Creating release for version: $version"

    # Artifacts directory
    local artifact_dir="release_artifacts"
    rm -rf "$artifact_dir"
    mkdir -p "$artifact_dir"

    # Get target triple for artifact naming
    local target_triple
    target_triple=$(rustc -vV | grep "host:" | cut -d ' ' -f2)

    # --- Build standard version ---
    print_info "Building standard release for ${target_triple}..."
    build "release" ""
    
    # Determine artifact name based on platform
    local artifact_name
    local archive_ext
    if [[ "$target_triple" == *"windows"* ]]; then
        artifact_name="${BINARY_NAME}-${target_triple}.exe"
        archive_ext="zip"
    else
        artifact_name="${BINARY_NAME}-${target_triple}"
        archive_ext="tar.gz"
    fi
    
    # Copy binary to artifact directory
    cp "target/release/${BINARY_NAME}" "${artifact_dir}/${BINARY_NAME}"
    
    # Create archive
    if [[ "$archive_ext" == "zip" ]]; then
        # For Windows, include .exe in the archive
        mv "${artifact_dir}/${BINARY_NAME}" "${artifact_dir}/${BINARY_NAME}.exe"
        (cd "$artifact_dir" && zip -q "${artifact_name}.zip" "${BINARY_NAME}.exe")
        rm "${artifact_dir}/${BINARY_NAME}.exe"
        local standard_artifact_path="${artifact_dir}/${artifact_name}.zip"
    else
        # For Unix, create tar.gz
        (cd "$artifact_dir" && tar -czf "${artifact_name}.tar.gz" "${BINARY_NAME}")
        rm "${artifact_dir}/${BINARY_NAME}"
        local standard_artifact_path="${artifact_dir}/${artifact_name}.tar.gz"
    fi
    
    print_success "Standard artifact created: ${standard_artifact_path}"

    # Note: We only create one set of artifacts since MCP is a default feature
    
    # --- Build DXT package ---
    print_info "Building DXT package..."
    if [[ -f "dxt/build-dxt.sh" ]]; then
        (cd dxt && ./build-dxt.sh)
        if [[ -f "dxt/smart-tree.dxt" ]]; then
            cp "dxt/smart-tree.dxt" "${artifact_dir}/"
            local dxt_artifact_path="${artifact_dir}/smart-tree.dxt"
            print_success "DXT package created: ${dxt_artifact_path}"
        else
            print_warning "DXT build completed but smart-tree.dxt not found"
            local dxt_artifact_path=""
        fi
    else
        print_warning "DXT build script not found, skipping DXT package"
        local dxt_artifact_path=""
    fi

    # --- Create GitHub Release ---
    local release_title="Smart Tree ${version}"
    shift # remove version from args
    local release_notes="${*:-Release ${version}}"

    print_info "Creating GitHub release..."
    print_info "Title: $release_title"
    print_info "Notes: $release_notes"

    if [[ "$NON_INTERACTIVE" == "true" ]]; then
        # Create git tag
        print_info "Tagging and pushing version $version..."
        git tag "$version"
        git push origin "$version"

        print_info "Creating GitHub release and uploading artifacts..."
        # Note: This will only upload the artifact for the current platform
        # For multi-platform releases, you'll need to build on each platform or use CI/CD
        if [[ -n "$dxt_artifact_path" ]]; then
            gh release create "$version" "$standard_artifact_path" "$dxt_artifact_path" --title "$release_title" --notes "$release_notes"
        else
            gh release create "$version" "$standard_artifact_path" --title "$release_title" --notes "$release_notes"
        fi
    else
        echo -e "\n${YELLOW}Ready to create release. Review details above.${NC}"
        read -p "Do you want to proceed? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_warning "Release cancelled."
            rm -rf "$artifact_dir"
            exit 1
        fi
        
        # Create git tag and push it
        print_info "Tagging version $version..."
        git tag "$version"
        git push origin "$version"
        print_success "Pushed tag $version to origin."

        print_info "Creating GitHub release and uploading artifacts..."
        # Note: This will only upload the artifact for the current platform
        # For multi-platform releases, you'll need to build on each platform or use CI/CD
        if [[ -n "$dxt_artifact_path" ]]; then
            gh release create "$version" "$standard_artifact_path" "$dxt_artifact_path" --title "$release_title" --notes "$release_notes"
        else
            gh release create "$version" "$standard_artifact_path" --title "$release_title" --notes "$release_notes"
        fi
    fi

    print_success "Release ${version} created successfully! ${TREE} ${SPARKLE}"
    rm -rf "$artifact_dir"
}

# Setup shell completions
completions() {
    print_header "Setting up shell completions 🐚"
    cd "$PROJECT_DIR"

    print_info "Building release binary to generate completions..."
    cargo build --release

    local shell_type
    shell_type="$(basename "$SHELL")"
    local completion_dir

    print_info "Detected shell: $shell_type"

    case "$shell_type" in
        bash)
            completion_dir="/etc/bash_completion.d"
            if [[ ! -d "$completion_dir" ]]; then
                completion_dir="$HOME/.local/share/bash-completion/completions"
                mkdir -p "$completion_dir"
            fi
            
            print_info "Generating bash completions..."
            ./target/release/st completions bash > "$completion_dir/st"
            
            print_success "Bash completions installed in $completion_dir/st"
            print_info "Please restart your shell or run 'source ~/.bashrc' for changes to take effect."
            ;;
        zsh)
            # Zsh can use a directory in FPATH for completions
            completion_dir=$(zsh -c 'echo $fpath[1]')
            if [[ -z "$completion_dir" ]] || [[ ! -w "$completion_dir" ]]; then
                completion_dir="$HOME/.zsh/completions"
                mkdir -p "$completion_dir"
                print_warning "Could not find a writable fpath directory."
                print_info "Created completions directory at $completion_dir"
                print_info "Add the following to your ~/.zshrc:"
                echo -e "${YELLOW}  fpath=(\$HOME/.zsh/completions \$fpath)${NC}"
            fi

            print_info "Generating zsh completions..."
            ./target/release/st completions zsh > "$completion_dir/_st"

            print_success "Zsh completions installed in $completion_dir/_st"
            print_info "Please restart your shell or run 'source ~/.zshrc' for changes to take effect."
            ;;
        fish)
            completion_dir="${XDG_CONFIG_HOME:-$HOME/.config}/fish/completions"
            mkdir -p "$completion_dir"

            print_info "Generating fish completions..."
            ./target/release/st completions fish > "$completion_dir/st.fish"
            
            print_success "Fish completions installed in $completion_dir/st.fish"
            print_info "Please restart your shell for changes to take effect."
            ;;
        *)
            print_warning "Unsupported shell: $shell_type"
            print_info "You can generate completions manually for your shell:"
            echo "  ./target/release/st completions [bash|zsh|fish|elvish|powershell] > /path/to/completions"
            ;;
    esac
}

# Install/Uninstall man page
manage_man_page() {
    print_header "Managing Man Page 📖"
    cd "$PROJECT_DIR"

    if ! command_exists pandoc; then
        print_error "pandoc is not installed. Please install it to continue."
        print_info "On Debian/Ubuntu: sudo apt-get install pandoc"
        print_info "On macOS (Homebrew): brew install pandoc"
        print_info "On Arch Linux: sudo pacman -S pandoc"
        exit 1
    fi

    local man_dir="/usr/local/share/man/man1"
    local man_page_src="docs/st-cheetsheet.md"
    local man_page_dest="$man_dir/st.1"

    if [[ "$1" == "install" ]]; then
        print_info "Generating and installing man page from $man_page_src..."
        
        if [[ ! -d "$man_dir" ]]; then
            print_info "Creating man directory: $man_dir"
            sudo mkdir -p "$man_dir"
        fi

        # Use a temporary file for pandoc output before moving with sudo
        local temp_file
        temp_file=$(mktemp)
        pandoc "$man_page_src" -s -t man -o "$temp_file"
        
        print_info "Installing to $man_page_dest"
        sudo mv "$temp_file" "$man_page_dest"
        
        print_info "Updating man database..."
        sudo mandb
        
        print_success "Man page for 'st' installed. Try 'man st'!"

    elif [[ "$1" == "uninstall" ]]; then
        if [[ -f "$man_page_dest" ]]; then
            print_info "Uninstalling man page: $man_page_dest"
            sudo rm "$man_page_dest"
            
            print_info "Updating man database..."
            sudo mandb
            
            print_success "Man page for 'st' uninstalled."
        else
            print_warning "Man page not found at $man_page_dest. Nothing to do."
        fi
    else
        print_error "Unknown argument. Use 'install' or 'uninstall'."
    fi
}

# MCP server functions
mcp_build() {
    print_header "Building $PROJECT_NAME with MCP support 🤖"
    build release mcp
}

mcp_run() {
    print_header "Running MCP server 🤖"
    cd "$PROJECT_DIR"
    
    if [[ ! -f "target/release/$BINARY_NAME" ]] || ! ./target/release/$BINARY_NAME --help 2>&1 | grep -q "mcp"; then
        print_warning "MCP features not found in binary. Building with MCP support..."
        mcp_build
    fi
    
    print_info "Starting MCP server on stdio..."
    print_info "Press Ctrl+C to stop"
    ./target/release/$BINARY_NAME --mcp
}

mcp_config() {
    print_header "MCP Configuration 🤖"
    cd "$PROJECT_DIR"
    
    if [[ ! -f "target/release/$BINARY_NAME" ]] || ! ./target/release/$BINARY_NAME --help 2>&1 | grep -q "mcp"; then
        print_warning "Building with MCP support first..."
        mcp_build
    fi
    
    ./target/release/$BINARY_NAME --mcp-config
}

mcp_tools() {
    print_header "MCP Tools Documentation 🤖"
    cd "$PROJECT_DIR"
    
    if [[ ! -f "target/release/$BINARY_NAME" ]] || ! ./target/release/$BINARY_NAME --help 2>&1 | grep -q "mcp"; then
        print_warning "Building with MCP support first..."
        mcp_build
    fi
    
    ./target/release/$BINARY_NAME --mcp-tools
}

# Demo streaming feature
demo_stream() {
    print_header "Demonstrating Streaming Mode ${ROCKET}"
    cd "$PROJECT_DIR"
    
    if [[ ! -f "target/release/$BINARY_NAME" ]]; then
        print_warning "Building release version first..."
        build release
    fi
    
    print_info "Streaming the current project directory in hex format..."
    print_info "Notice how output appears immediately as files are discovered!"
    echo ""
    ./target/release/$BINARY_NAME --stream -m hex . | head -20
    echo "... (truncated for demo)"
    
    print_info "\nStreaming with AI format:"
    ./target/release/$BINARY_NAME --stream -m ai . | head -25
    echo "... (truncated for demo)"
    
    print_success "Streaming is perfect for large directories! ${SPARKLE}"
}

# Demo search feature
demo_search() {
    print_header "Demonstrating Search Feature 🔍"
    cd "$PROJECT_DIR"
    
    if [[ ! -f "target/release/$BINARY_NAME" ]]; then
        print_warning "Building release version first..."
        build release
    fi
    
    print_info "Searching for 'Scanner' in the source code..."
    ./target/release/$BINARY_NAME --search "Scanner" -m hex src | grep SEARCH || print_warning "No matches found"
    
    print_info "\nSearching for 'TODO' comments..."
    ./target/release/$BINARY_NAME --search "TODO" -m hex . | grep SEARCH || print_info "Good news! No TODOs found!"
    
    print_info "\nSearch works great with streaming too:"
    ./target/release/$BINARY_NAME --stream --search "fn" --type rs -m hex src | head -10
    
    print_success "Search helps you find content quickly! ${SPARKLE}"
}

# Demo relations feature
demo_relations() {
    print_header "Demonstrating Code Relations Feature 🔗"
    cd "$PROJECT_DIR"
    
    if [[ ! -f "target/release/$BINARY_NAME" ]]; then
        print_warning "Building release version first..."
        build release
    fi
    
    print_info "Analyzing relationships in src/ directory..."
    ./target/release/$BINARY_NAME -m relations src | head -30
    echo "... (truncated for demo)"
    
    print_info "\nWith import filter:"
    ./target/release/$BINARY_NAME -m relations --relations-filter imports src | head -20
    echo "... (truncated for demo)"
    
    print_info "\nFocusing on main.rs:"
    ./target/release/$BINARY_NAME -m relations --focus src/main.rs src | head -15
    
    print_success "Relations feature provides semantic X-ray vision for codebases! ${SPARKLE}"
}

# Show usage examples
examples() {
    print_header "Usage Examples ${SPARKLE}"
    
    cat << EOF
${CYAN}Basic usage:${NC}
  $BINARY_NAME                          # Analyze current directory
  $BINARY_NAME /path/to/dir             # Analyze specific directory
  
${CYAN}Output modes:${NC}
  $BINARY_NAME -m hex                   # Hexadecimal format (AI-friendly)
  $BINARY_NAME -m json                  # JSON output
  $BINARY_NAME -m ai                    # AI-optimized format
  $BINARY_NAME -m digest                # Super compact digest (hash + stats)
  $BINARY_NAME -m stats                 # Statistics only
  
${CYAN}Filtering:${NC}
  $BINARY_NAME --find "*.rs"            # Find Rust files
  $BINARY_NAME --type rs                # Only .rs files
  $BINARY_NAME --min-size 1M            # Files larger than 1MB
  $BINARY_NAME --newer-than 2024-01-01  # Recent files
  
${CYAN}Options:${NC}
  $BINARY_NAME --no-emoji               # Plain text output
  $BINARY_NAME --depth 3                # Limit depth
  $BINARY_NAME -z                       # Compress output
  
${CYAN}🆕 Streaming Mode:${NC}
  $BINARY_NAME --stream                 # Stream output as files are found
  $BINARY_NAME --stream -m hex /large   # Great for huge directories
  $BINARY_NAME --stream -m ai           # Real-time AI format output
  
${CYAN}🆕 File Content Search:${NC}
  $BINARY_NAME --search "TODO"          # Find TODO in all text files
  $BINARY_NAME --type rs --search "fn"  # Search for "fn" in Rust files
  $BINARY_NAME -m hex --search "error"  # Hex output with search positions
  
${CYAN}🔗 Code Relations (NEW!):${NC}
  $BINARY_NAME -m relations             # Analyze code relationships
  $BINARY_NAME -m relations --focus main.rs  # Focus on specific file
  $BINARY_NAME -m relations --relations-filter imports  # Show only imports
  
${CYAN}AI usage:${NC}
  AI_TOOLS=1 $BINARY_NAME               # Auto AI mode + compression
  $BINARY_NAME -m digest                # Quick digest for AI pre-check
  $BINARY_NAME -m ai -z | base64 -d    # Decode compressed output
  
${CYAN}MCP (Model Context Protocol):${NC}
  $0 mcp-build                          # Build with MCP support
  $0 mcp-run                            # Run as MCP server
  $0 mcp-config                         # Show Claude Desktop config
  $0 mcp-tools                          # Show available MCP tools
EOF
}

# Show help
show_help() {
    cat << EOF
${CYAN}${TREE} Smart Tree Management Script ${TREE}${NC}

${YELLOW}Usage:${NC} $0 [command] [options]

${YELLOW}Commands:${NC}
  ${GREEN}build${NC} [debug|release] [features]  Build the project
  ${GREEN}run${NC} [args...]         Run st with arguments
  ${GREEN}test${NC}                  Run tests, linting, and format check
  ${GREEN}format${NC}                Format code with rustfmt
  ${GREEN}clean${NC}                 Clean build artifacts
  ${GREEN}status${NC}                Show project status
  ${GREEN}bench${NC}                 Run performance benchmarks
  ${GREEN}install${NC} [dir]         Install binary (default: /usr/local/bin)
  ${GREEN}uninstall${NC} [dir]       Uninstall binary
  ${GREEN}release${NC} <vX.Y.Z> [notes] Create a GitHub release with standard and MCP assets
  ${GREEN}completions${NC}           Setup shell completions
  ${GREEN}man-install${NC}           Generate and install the man page
  ${GREEN}man-uninstall${NC}         Uninstall the man page
  ${GREEN}examples${NC}              Show usage examples
  ${GREEN}demo-stream${NC}           Demo streaming feature
  ${GREEN}demo-search${NC}           Demo search feature
  ${GREEN}demo-relations${NC}        Demo code relations feature 🔗
  ${GREEN}help${NC}                  Show this help message

${YELLOW}MCP Commands:${NC}
  ${GREEN}mcp-build${NC}             Build with MCP support
  ${GREEN}mcp-run${NC}               Run as MCP server
  ${GREEN}mcp-config${NC}            Show Claude Desktop configuration
  ${GREEN}mcp-tools${NC}             Show available MCP tools

${YELLOW}Feedback System Commands:${NC}
  ${GREEN}feedback-build${NC}        Build feedback system containers
  ${GREEN}feedback-run${NC}          Run feedback worker locally
  ${GREEN}feedback-deploy${NC} [type] Deploy feedback system (local|hetzner|registry)
  ${GREEN}feedback-test${NC}         Test feedback system
  ${GREEN}feedback-status${NC}       Check feedback system health

${YELLOW}Environment Variables:${NC}
  ${PURPLE}NO_EMOJI=1${NC}           Disable emojis in output
  ${PURPLE}NON_INTERACTIVE=true${NC}  Disable interactive features

${YELLOW}Examples:${NC}
  $0 build              # Build release version
  $0 build release mcp  # Build with MCP support
  $0 run -- -m hex .    # Run with hex output on current dir
  $0 test               # Run all tests
  $0 release v1.0.0 "My first release!" # Create a new release
  $0 mcp-run            # Start MCP server

${CYAN}Made with ${SPARKLE} and 🌳 by the Smart Tree team!${NC}
EOF
}

# Feedback system functions
feedback_build() {
    print_header "Building Feedback System Containers 🔨"
    cd "$PROJECT_DIR/feedback-worker"
    
    print_info "Building feedback API container..."
    docker build -t ghcr.io/8b-is/smart-tree-feedback-api:latest ../feedback-api/
    
    print_info "Building feedback worker container..."
    docker build -t ghcr.io/8b-is/smart-tree-feedback-worker:latest .
    
    print_success "Feedback containers built successfully!"
}

feedback_run() {
    print_header "Running Feedback Worker Locally 🎸"
    cd "$PROJECT_DIR/feedback-worker"
    
    if [[ -z "${GITHUB_TOKEN:-}" ]]; then
        print_warning "GITHUB_TOKEN not set - worker will run without GitHub integration"
    fi
    
    print_info "Starting feedback system with docker-compose..."
    docker-compose up -d
    
    print_success "Feedback system running!"
    print_info "  API: http://localhost:8422"
    print_info "  Metrics: http://localhost:9090/metrics"
    print_info "  Grafana: http://localhost:3000 (admin/admin)"
}

feedback_deploy() {
    local deploy_type="${1:-local}"
    print_header "Deploying Feedback System - Type: $deploy_type 🚀"
    cd "$PROJECT_DIR/feedback-worker"
    
    ./deploy.sh "$deploy_type"
}

feedback_test() {
    print_header "Testing Feedback System 🧪"
    cd "$PROJECT_DIR/feedback-worker"
    
    if ! command_exists python3; then
        print_error "Python 3 is required for testing"
        return 1
    fi
    
    print_info "Running feedback system tests..."
    python3 test_worker.py
}

feedback_status() {
    print_header "Feedback System Status 📊"
    
    # Check API
    if curl -sf http://localhost:8422/health > /dev/null 2>&1; then
        print_success "API is healthy"
        curl -s http://localhost:8422/health | jq . || true
    else
        print_error "API is not responding"
    fi
    
    # Check worker metrics
    if curl -sf http://localhost:9090/metrics > /dev/null 2>&1; then
        print_success "Worker is healthy"
        echo "  $(curl -s http://localhost:9090/metrics | grep -E '^feedback_processed_total' | head -1)"
    else
        print_error "Worker is not responding"
    fi
    
    # Check Redis
    if docker exec $(docker ps -qf "name=redis" 2>/dev/null) redis-cli ping > /dev/null 2>&1; then
        print_success "Redis is healthy"
    else
        print_error "Redis is not responding"
    fi
    
    # Check containers
    print_info "Running containers:"
    docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | grep -E "(feedback|redis)" || echo "  No feedback containers running"
}

# Main command dispatcher
main() {
    if [[ $# -eq 0 ]]; then
        show_help
        exit 0
    fi
    
    case "$1" in
        build)
            shift
            build "$@"
            ;;
        run)
            shift
            run "$@"
            ;;
        test)
            test
            ;;
        format|fmt)
            format
            ;;
        clean)
            clean
            ;;
        status|info)
            status
            ;;
        bench|benchmark)
            bench
            ;;
        install)
            shift
            install "$@"
            ;;
        uninstall|remove)
            shift
            uninstall "$@"
            ;;
        release)
            shift
            release "$@"
            ;;
        completions|complete)
            completions
            ;;
        man-install)
            manage_man_page install
            ;;
        man-uninstall)
            manage_man_page uninstall
            ;;
        examples|ex)
            examples
            ;;
        mcp-build)
            mcp_build
            ;;
        mcp-run)
            mcp_run
            ;;
        mcp-config)
            mcp_config
            ;;
        mcp-tools)
            mcp_tools
            ;;
        demo-stream)
            demo_stream
            ;;
        demo-search)
            demo_search
            ;;
        demo-relations)
            demo_relations
            ;;
        feedback-build)
            feedback_build
            ;;
        feedback-run)
            feedback_run
            ;;
        feedback-deploy)
            feedback_deploy "${@:2}"
            ;;
        feedback-test)
            feedback_test
            ;;
        feedback-status)
            feedback_status
            ;;
        help|h|-h|--help)
            show_help
            ;;
        *)
            print_error "Unknown command: $1"
            echo ""
            show_help
            exit 1
            ;;
    esac
}

# Let's go! 🚀
main "$@"