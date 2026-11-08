#!/usr/bin/env bash
# Sovereign Codeberg CI Chore-Polish & Release Agent for smart-tree
set -euo pipefail

MODE="${1:---chore-polish}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "==> [smart-tree Codeberg Agent] Mode: $MODE"

export SMART_TREE_NONINTERACTIVE=1
export CI=1

run_chore_polish() {
    echo "--> 1. Checking Rust formatting (cargo fmt --all -- --check)..."
    cargo fmt --all -- --check

    echo "--> 2. Running Clippy lints..."
    cargo clippy --lib --bins -- -D warnings

    echo "--> 3. Running library & integration tests (non-interactive)..."
    cargo test --lib
    cargo test --test test_claude_integration
    echo "    ✓ smart-tree chore-polish checks passed."
}

run_release() {
    run_chore_polish
    if [ -x ./scripts/release.sh ]; then
        ./scripts/release.sh
    else
        cargo build --release --bins
    fi
}

case "$MODE" in
    --chore-polish)
        run_chore_polish
        ;;
    --release)
        run_release
        ;;
    *)
        echo "Usage: $0 [--chore-polish | --release]"
        exit 1
        ;;
esac
