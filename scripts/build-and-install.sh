#!/bin/bash
# build-and-install.sh - Smart Tree Build & Install Script
# Rebuilds Smart Tree and installs it, clearing shell cache to prevent hanging issues

set -e  # Exit on any error

echo "🔨 Building Smart Tree..."
cargo build --release

echo "📦 Installing Smart Tree..."
sudo cp ./target/release/st /usr/local/bin/st
sudo cp ./target/release/std /usr/local/bin/std

echo "🧹 Clearing shell cache..."
hash -r

echo "✅ Build and install complete!"
echo "Testing version:"
st --version

echo ""
echo "🎉 Smart Tree is ready to use!"
echo "   Run 'st --help' to see all options"
echo "   Run 'st --mcp-config' to set up AI integration" 