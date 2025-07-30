#!/bin/bash
# File History Demo - Shows how to use the new file history tracking system
# 🎸 The Cheet says: "Track every change, remember every moment!"

echo "=== Smart Tree File History Demo ==="
echo "The Ultimate Context-Driven System"
echo ""

# Test file for demonstration
TEST_FILE="/tmp/demo_file.txt"

echo "1. Creating a test file..."
echo "Hello, World!" > "$TEST_FILE"

echo ""
echo "2. Tracking file creation with MCP..."
cat << 'EOF'
mcp.callTool('track_file_operation', {
  file_path: '/tmp/demo_file.txt',
  operation: 'create',
  new_content: 'Hello, World!',
  agent: 'demo-agent',
  session_id: 'demo-session-1'
})
EOF

echo ""
echo "3. Appending content (favored operation)..."
echo "This is an append operation." >> "$TEST_FILE"

echo ""
echo "4. Tracking the append..."
cat << 'EOF'
mcp.callTool('track_file_operation', {
  file_path: '/tmp/demo_file.txt',
  old_content: 'Hello, World!',
  new_content: 'Hello, World!\nThis is an append operation.',
  agent: 'demo-agent',
  session_id: 'demo-session-1'
})
EOF

echo ""
echo "5. Getting file history..."
cat << 'EOF'
mcp.callTool('get_file_history', {
  file_path: '/tmp/demo_file.txt'
})
EOF

echo ""
echo "6. Getting project summary..."
cat << 'EOF'
mcp.callTool('get_project_history_summary', {
  project_path: '/tmp'
})
EOF

echo ""
echo "=== File History Storage Location ==="
echo "All history is stored in: ~/.mem8/.filehistory/"
echo "Format: project_id/YYYYMMDD_HHMM.flg"
echo ""
echo "Each log entry contains:"
echo "- Timestamp (unix time)"
echo "- File path"
echo "- Operation code (A=Append, C=Create, R=Replace, etc.)"
echo "- Before/after hashes"
echo "- Agent identifier"
echo "- Session ID"
echo ""
echo "🎸 The Cheet says: 'Every file has a story - now we can tell it!'"