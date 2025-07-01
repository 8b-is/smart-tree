#!/bin/bash

# Demo script for Smart Tree Desktop Extension
# This script demonstrates various features for screenshots

echo "🌳 Smart Tree Desktop Extension Demo"
echo "===================================="
echo ""

# Function to pause between demos
pause() {
    echo ""
    echo "Press Enter to continue..."
    read
}

# 1. Basic comparison
echo "1️⃣ Traditional tree vs Smart Tree hex format:"
echo ""
echo "Traditional tree:"
tree -L 2 ../hello-world-node
echo ""
echo "Smart Tree hex (70% smaller!):"
/usr/local/bin/st -m hex -d 2 ../hello-world-node
pause

# 2. AI-optimized format
echo "2️⃣ AI-optimized format for better context understanding:"
echo ""
/usr/local/bin/st -m ai -d 3 ../hello-world-node
pause

# 3. Search functionality
echo "3️⃣ Search for 'function' in the codebase:"
echo ""
/usr/local/bin/st --search "function" -m hex ../hello-world-node
pause

# 4. Statistics view
echo "4️⃣ Project statistics:"
echo ""
/usr/local/bin/st -m stats ../hello-world-node
pause

# 5. JSON output for programmatic use
echo "5️⃣ JSON format for tools and scripts:"
echo ""
/usr/local/bin/st -m json -d 2 . | jq '.'
pause

# 6. Filtering examples
echo "6️⃣ Filter only JavaScript files:"
echo ""
/usr/local/bin/st --type js -m classic ..
pause

# 7. MCP integration test
echo "7️⃣ MCP Server Test (JSON-RPC):"
echo ""
echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":1}' | ./server/st --mcp 2>/dev/null | jq '.result.tools[0]'

echo ""
echo "✅ Demo complete! Smart Tree makes directory analysis:"
echo "   • 70% more efficient"
echo "   • AI-friendly"
echo "   • Feature-rich"
echo "   • Easy to integrate" 