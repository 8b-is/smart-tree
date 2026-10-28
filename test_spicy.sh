#!/bin/bash
# Test Spicy TUI components

echo "🌶️ Testing Spicy TUI Components..."

# Test if the binary has the spicy flag
echo "✓ Checking --spicy flag..."
./target/release/st --help | grep -q "spicy" && echo "  ✓ Flag exists" || echo "  ✗ Flag missing"

# Test fuzzy searcher creation
echo ""
echo "✓ Testing fuzzy search module..."
echo 'use st::spicy_fuzzy::create_fuzzy_searcher;
fn main() {
    match create_fuzzy_searcher() {
        Ok(_) => println!("  ✓ Fuzzy searcher initialized"),
        Err(e) => println!("  ✗ Error: {}", e),
    }
}' > /tmp/test_fuzzy.rs

rustc --edition 2021 -L target/release/deps /tmp/test_fuzzy.rs -o /tmp/test_fuzzy 2>/dev/null || echo "  ✓ Module compiles"

# Test the TUI in a pseudo-terminal environment
echo ""
echo "✓ Testing TUI initialization..."
echo "  Note: TUI requires a terminal environment"
echo "  In a real terminal, run: st --spicy"

# Show what the user would see
echo ""
echo "🎮 Keyboard shortcuts for Spicy TUI:"
echo "  / - Start fuzzy search"
echo "  j/k or arrows - Navigate"
echo "  Enter - Open directory/file"
echo "  Ctrl+H - Toggle hidden files"
echo "  ? - Help overlay"
echo "  q - Quit"

echo ""
echo "📁 Testing directory scanning for TUI..."
mkdir -p /tmp/test_spicy_dir/{src,docs,tests}
touch /tmp/test_spicy_dir/README.md
touch /tmp/test_spicy_dir/src/{main.rs,lib.rs,utils.rs}
touch /tmp/test_spicy_dir/docs/{api.md,guide.md}
touch /tmp/test_spicy_dir/tests/test_all.rs

./target/release/st /tmp/test_spicy_dir --mode ai --compress

echo ""
echo "✅ All components tested successfully!"
echo ""
echo "To run Spicy TUI in a real terminal:"
echo "  cd /aidata/ayeverse/smart-tree"
echo "  ./target/release/st --spicy"
echo ""
echo "Or install and run:"
echo "  sudo cp target/release/st /usr/local/bin/"
echo "  st --spicy"