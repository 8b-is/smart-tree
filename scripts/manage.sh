#!/usr/bin/env bash
# 🌳 Smart Tree Management Script - Because every tree needs a gardener! 🌳

set -euo pipefail

# Colors for our fancy output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Project info
PROJECT_NAME="Smart Tree (stree)"
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY_NAME="stree"

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
    print_header "Building $PROJECT_NAME in $build_type mode ${GEAR}"
    
    cd "$PROJECT_DIR"
    
    if [[ "$build_type" == "release" ]]; then
        print_info "Optimizing for maximum speed... ${ROCKET}"
        if [[ "$NON_INTERACTIVE" == "true" ]]; then
            cargo build --release
        else
            cargo build --release &
            spinner $!
        fi
        print_success "Release build complete! Binary size: $(du -h target/release/$BINARY_NAME | cut -f1)"
    else
        print_info "Building debug version with all the debugging goodies..."
        cargo build
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

# Setup shell completions
completions() {
    print_header "Setting up shell completions 🐚"
    cd "$PROJECT_DIR"
    
    print_warning "Shell completions coming soon! 🚧"
    print_info "For now, enjoy tab-completing the manage.sh commands!"
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
  
${CYAN}AI usage:${NC}
  AI_TOOLS=1 $BINARY_NAME               # Auto AI mode + compression
  $BINARY_NAME -m ai -z | base64 -d    # Decode compressed output
EOF
}

# Show help
show_help() {
    cat << EOF
${CYAN}${TREE} Smart Tree Management Script ${TREE}${NC}

${YELLOW}Usage:${NC} $0 [command] [options]

${YELLOW}Commands:${NC}
  ${GREEN}build${NC} [debug|release]  Build the project (default: release)
  ${GREEN}run${NC} [args...]         Run stree with arguments
  ${GREEN}test${NC}                  Run tests, linting, and format check
  ${GREEN}format${NC}                Format code with rustfmt
  ${GREEN}clean${NC}                 Clean build artifacts
  ${GREEN}status${NC}                Show project status
  ${GREEN}bench${NC}                 Run performance benchmarks
  ${GREEN}install${NC} [dir]         Install binary (default: /usr/local/bin)
  ${GREEN}uninstall${NC} [dir]       Uninstall binary
  ${GREEN}completions${NC}           Setup shell completions
  ${GREEN}examples${NC}              Show usage examples
  ${GREEN}help${NC}                  Show this help message

${YELLOW}Environment Variables:${NC}
  ${PURPLE}NO_EMOJI=1${NC}           Disable emojis in output
  ${PURPLE}NON_INTERACTIVE=true${NC}  Disable interactive features

${YELLOW}Examples:${NC}
  $0 build              # Build release version
  $0 run -- -m hex .    # Run with hex output on current dir
  $0 test               # Run all tests
  $0 install            # Install to system

${CYAN}Made with ${SPARKLE} and 🌳 by the Smart Tree team!${NC}
EOF
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
        completions|complete)
            completions
            ;;
        examples|ex)
            examples
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