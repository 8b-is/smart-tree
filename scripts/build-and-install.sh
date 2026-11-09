#!/bin/bash
# build-and-install.sh - Smart Tree Build & Root Install Script
# Rebuilds Smart Tree, installs it as a canonical root program in /usr/local/bin,
# removes any duplicate user/system copies, and clears shell cache.

set -e  # Exit on any error

echo "🔨 Building Smart Tree..."
cargo build --release

echo "📦 Installing Smart Tree as root program in /usr/local/bin..."
for bin in st m8 n8x mq; do
    if [[ -f "./target/release/$bin" ]]; then
        sudo install -m 0755 "./target/release/$bin" "/usr/local/bin/$bin"
        echo "  ✓ /usr/local/bin/$bin"
    fi
done
# Retire legacy std binary if present
if [[ -e "/usr/local/bin/std" || -L "/usr/local/bin/std" ]]; then
    sudo rm -f "/usr/local/bin/std"
    echo "  🧹 Retired legacy /usr/local/bin/std"
fi

echo "🧹 Removing duplicate copies outside /usr/local/bin..."
for bin in st std m8 n8x mq smart-tree; do
    for dup_dir in "$HOME/.local/bin" "$HOME/.cargo/bin" "$HOME/bin" "/opt/homebrew/bin"; do
        if [[ -e "$dup_dir/$bin" || -L "$dup_dir/$bin" ]]; then
            rm -f "$dup_dir/$bin" 2>/dev/null || sudo rm -f "$dup_dir/$bin"
            echo "  🧹 Removed $dup_dir/$bin"
        fi
    done
    if [[ -e "/usr/bin/$bin" || -L "/usr/bin/$bin" ]]; then
        if [[ "$(readlink -f "/usr/bin/$bin" 2>/dev/null)" != "/usr/local/bin/$bin" ]]; then
            sudo rm -f "/usr/bin/$bin"
            echo "  🧹 Removed /usr/bin/$bin"
        fi
    fi
done

echo "🧹 Clearing shell cache..."
hash -r

echo "✅ Build and root install complete!"
echo "Testing version:"
/usr/local/bin/st --version

echo ""
echo "🎉 Smart Tree is ready to use at /usr/local/bin/st!"
echo "   Run 'st --help' to see all options"
echo "   Run 'st --mcp-config' to set up AI integration"