#!/bin/bash
# Test Enhanced Spicy TUI Features

echo "🌶️ Testing Enhanced Spicy TUI..."
echo ""
echo "✨ NEW FEATURES:"
echo "  • Dual-mode search (files AND content)"
echo "  • Tree navigation with arrow keys"
echo "  • ASCII art for image previews"
echo "  • Search result highlighting"
echo "  • M8 context saving"
echo ""

# Check if binary exists
if [ ! -f "./target/release/st" ]; then
    echo "❌ Binary not found! Building..."
    cargo build --release
fi

echo "✅ Binary ready at ./target/release/st"
echo ""
echo "🎮 To test the enhanced Spicy TUI, run:"
echo "  ./target/release/st --spicy"
echo ""
echo "📝 Try these features:"
echo "  1. Press '/' to search file names"
echo "  2. Press 'Ctrl+F' to search content"
echo "  3. Use arrow keys to navigate the tree"
echo "  4. Open an image file to see ASCII art"
echo "  5. Press '?' for help"
echo ""
echo "🚀 Enjoy the enhanced Spicy experience!"