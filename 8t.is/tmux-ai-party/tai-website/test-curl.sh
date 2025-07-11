#!/bin/bash
# Test script to demonstrate tai.is curl detection

echo "🧪 Testing TAI.is curl detection..."
echo ""

# First, let's test the main page
echo "1. Testing main page with curl:"
echo "   curl tai.is"
echo "---"
curl -H "User-Agent: curl/7.64.1" http://localhost:3000/
echo ""
echo ""

# Test the install endpoint
echo "2. Testing smart installer:"
echo "   curl tai.is/install | sh"
echo "---"
curl -H "User-Agent: curl/7.64.1" http://localhost:3000/install > /tmp/tai-detect.sh
head -20 /tmp/tai-detect.sh
echo "..."
echo ""

# Test the setup endpoint
echo "3. Testing direct setup:"
echo "   curl tai.is/setup"
echo "---"
curl -H "User-Agent: curl/7.64.1" http://localhost:3000/setup | head -20
echo ""

# Test customized installer generation
echo "4. Testing customized installer generation:"
echo "   (This is what the detection script calls)"
echo "---"
PARAMS="os=linux&arch=x86_64&distro=ubuntu&shell=bash&python=3.10.0&tmux=yes&user_type=1"
curl -s "http://localhost:3000/setup/generate?$PARAMS" | head -30
echo ""

echo "✅ All tests complete!"
echo ""
echo "To run the full flow locally:"
echo "1. Install dependencies: pnpm install"
echo "2. Start the dev server: pnpm dev"
echo "3. In another terminal: curl -sSL localhost:3000/install | sh"