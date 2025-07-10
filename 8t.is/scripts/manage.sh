#!/usr/bin/env bash

# 🎸 8t Management Script - Get 80 Before You Get 80x The Context! 🎸
# Because managing Rust projects should be as smooth as a perfectly quantized signal

set -euo pipefail

# ANSI color codes for maximum pizzazz
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Non-interactive mode support
NON_INTERACTIVE="${NON_INTERACTIVE:-false}"

# ASCII Art Banner (because every good tool needs one)
print_banner() {
    if [[ "$NON_INTERACTIVE" != "true" ]]; then
        echo -e "${CYAN}"
        echo "   ___  _   "
        echo "  ( _ )| |_ "
        echo "  / _ \\| __|"
        echo " | (_) | |_ "
        echo "  \\___/ \\__|"
        echo -e "${NC}"
        echo -e "${MAGENTA}🎸 Get 80 Before You Get 80x The Context! 🎸${NC}"
        echo
    fi
}

# Helper function for status messages
status() {
    if [[ "$NON_INTERACTIVE" != "true" ]]; then
        echo -e "${BLUE}[8t]${NC} $1"
    else
        echo "[8t] $1"
    fi
}

success() {
    if [[ "$NON_INTERACTIVE" != "true" ]]; then
        echo -e "${GREEN}✅${NC} $1"
    else
        echo "[OK] $1"
    fi
}

error() {
    if [[ "$NON_INTERACTIVE" != "true" ]]; then
        echo -e "${RED}❌${NC} $1" >&2
    else
        echo "[ERROR] $1" >&2
    fi
}

warning() {
    if [[ "$NON_INTERACTIVE" != "true" ]]; then
        echo -e "${YELLOW}⚠️${NC} $1"
    else
        echo "[WARN] $1"
    fi
}

# Check if we're in the right directory
check_directory() {
    if [[ ! -f "Cargo.toml" ]]; then
        error "Not in an 8t project directory! Where's the Cargo.toml? 🤔"
        exit 1
    fi
}

# Build function
build() {
    local profile="${1:-release}"
    local features="${2:-}"
    
    status "Building 8t in ${profile} mode... 🔨"
    
    local cargo_cmd="cargo build"
    if [[ "$profile" == "release" ]]; then
        cargo_cmd="$cargo_cmd --release"
    fi
    
    if [[ -n "$features" ]]; then
        cargo_cmd="$cargo_cmd --features $features"
    fi
    
    if $cargo_cmd; then
        success "Build complete! Your bits are properly quantized! 🎵"
    else
        error "Build failed! The signal got distorted! 😭"
        exit 1
    fi
}

# Run function
run() {
    status "Running 8t... Let's compress some context! 🚀"
    shift # Remove 'run' from arguments
    cargo run --release -- "$@"
}

# Test function
test() {
    status "Running tests... Making sure our quantization is lossless! 🧪"
    
    if cargo test --all; then
        success "All tests passed! The signal is crystal clear! 🎯"
    else
        error "Tests failed! We've got some noise in the system! 📡"
        exit 1
    fi
}

# Format function
fmt() {
    status "Formatting code... Making it as clean as a sine wave! 🌊"
    
    if cargo fmt --all; then
        success "Code formatted! Looking sharp! ✨"
    else
        error "Formatting failed! The waveform is wonky! 🌀"
        exit 1
    fi
}

# Lint function
lint() {
    status "Running clippy... Hunting for imperfections! 🔍"
    
    if cargo clippy -- -D warnings; then
        success "No clippy warnings! Your code is harmonious! 🎼"
    else
        warning "Clippy found some dissonance in your code! 🎺"
        exit 1
    fi
}

# Clean function
clean() {
    status "Cleaning build artifacts... Clearing the signal path! 🧹"
    
    if cargo clean; then
        success "Clean complete! Ready for a fresh signal! 🌟"
    else
        error "Clean failed! The noise floor is still high! 📊"
        exit 1
    fi
}

# Status function
show_status() {
    status "8t Project Status Report 📊"
    echo
    echo -e "${BOLD}Core Crates:${NC}"
    echo "  • eighty-core    - The quantum heart of 8t"
    echo "  • eighty-api     - RESTful semantic compression"
    echo "  • eighty-feedback - AI suggestion ingestion"
    echo "  • eighty-container - Self-aware context management"
    echo
    echo -e "${BOLD}Rust Version:${NC}"
    rustc --version
    echo
    echo -e "${BOLD}Cargo Version:${NC}"
    cargo --version
    echo
    echo -e "${BOLD}Project Structure:${NC}"
    if command -v tree &> /dev/null; then
        tree -L 2 -d 8t/
    else
        ls -la 8t/
    fi
}

# API Server function
run_api() {
    status "Starting 8t API server on port 8420... 🌐"
    cd 8t/api && cargo run --release
}

# Development mode with auto-reload
dev() {
    status "Starting development mode with auto-reload... 🔄"
    
    if ! command -v cargo-watch &> /dev/null; then
        warning "cargo-watch not installed. Installing it for you..."
        cargo install cargo-watch
    fi
    
    cargo watch -x 'run --release'
}

# Benchmark function
bench() {
    status "Running benchmarks... Let's see how fast we can quantize! ⚡"
    cargo bench
}

# Install function
install() {
    status "Installing 8t system-wide... Spreading the compression love! 💝"
    
    local install_dir="${1:-/usr/local/bin}"
    
    # Build in release mode first
    build release
    
    # Find and install all binaries
    for binary in target/release/eighty-*; do
        if [[ -f "$binary" && -x "$binary" ]]; then
            local bin_name=$(basename "$binary")
            status "Installing $bin_name to $install_dir"
            
            if [[ -w "$install_dir" ]]; then
                cp "$binary" "$install_dir/"
                success "Installed $bin_name!"
            else
                warning "Need sudo to install to $install_dir"
                sudo cp "$binary" "$install_dir/"
                success "Installed $bin_name with sudo!"
            fi
        fi
    done
}

# Help function
show_help() {
    print_banner
    echo -e "${BOLD}Usage:${NC} $0 [command] [options]"
    echo
    echo -e "${BOLD}Commands:${NC}"
    echo "  build [debug|release] [features]  Build the project (default: release)"
    echo "  run -- [args]                     Run 8t with arguments"
    echo "  test                              Run all tests"
    echo "  bench                             Run benchmarks"
    echo "  fmt, format                       Format code with rustfmt"
    echo "  lint                              Run clippy linter"
    echo "  clean                             Clean build artifacts"
    echo "  install [dir]                     Install binaries (default: /usr/local/bin)"
    echo "  status                            Show project status"
    echo "  api                               Run the API server"
    echo "  dev                               Development mode with auto-reload"
    echo "  help                              Show this help message"
    echo
    echo -e "${BOLD}Environment Variables:${NC}"
    echo "  NON_INTERACTIVE=true              Disable colors and emoji for CI/CD"
    echo
    echo -e "${BOLD}Examples:${NC}"
    echo "  $0 build                          Build in release mode"
    echo "  $0 build debug                    Build in debug mode"
    echo "  $0 run -- --mode quantum          Run with quantum mode"
    echo "  $0 test                           Run all tests"
    echo "  $0 api                            Start the API server"
    echo
    echo -e "${MAGENTA}Remember: Life's too short for uncompressed context! 🎸${NC}"
}

# Main script logic
main() {
    local cmd="${1:-help}"
    
    case "$cmd" in
        build)
            check_directory
            shift
            build "$@"
            ;;
        run)
            check_directory
            run "$@"
            ;;
        test)
            check_directory
            test
            ;;
        bench)
            check_directory
            bench
            ;;
        fmt|format)
            check_directory
            fmt
            ;;
        lint)
            check_directory
            lint
            ;;
        clean)
            check_directory
            clean
            ;;
        install)
            check_directory
            shift
            install "$@"
            ;;
        status)
            check_directory
            show_status
            ;;
        api)
            check_directory
            run_api
            ;;
        dev)
            check_directory
            dev
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            error "Unknown command: $cmd"
            echo
            show_help
            exit 1
            ;;
    esac
}

# Let's rock and roll! 🎸
if [[ "$NON_INTERACTIVE" != "true" ]]; then
    print_banner
fi

main "$@"