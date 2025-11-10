#!/bin/bash
# Test keyword search for projects

# This tests the find_projects MCP tool with keywords
echo "Testing Smart Tree keyword search..."

# Test 1: Search for 'cast' and 'tv' keywords in /aidata/ayeverse
echo ""
echo "Test 1: Searching for projects with keywords: cast, tv"
echo "Expected: Should find q8-caster project"
echo ""

# We'll use a simple rust test to call the function directly
cargo test --lib find_projects -- --nocapture 2>&1 | grep -A5 "q8-caster" || echo "Direct test not found, checking project structure..."

# Verify q8-caster exists and has the right content
if [ -d "/aidata/ayeverse/q8-caster" ]; then
    echo "✓ q8-caster directory exists"
    if [ -f "/aidata/ayeverse/q8-caster/README.md" ]; then
        echo "✓ README.md found"
        echo "  README first line:"
        head -1 /aidata/ayeverse/q8-caster/README.md | sed 's/^/    /'
    fi
else
    echo "✗ q8-caster not found"
fi

echo ""
echo "To manually test via MCP, run:"
echo "  ./target/release/st --mcp"
echo "Then use the find_projects tool with keywords: ['cast', 'tv']"
