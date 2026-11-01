#!/bin/bash
# kill-stuck-st.sh - Kill all stuck Smart Tree processes

echo "🔍 Finding stuck st processes..."

# Find all st processes (excluding MCP server and system processes)
STUCK_PIDS=$(ps aux | grep -E '\bst\b' | grep -v grep | grep -v '/usr/local/bin/st --mcp' | grep -v 'ctkd' | awk '{print $2}')

if [ -z "$STUCK_PIDS" ]; then
    echo "✅ No stuck st processes found!"
    exit 0
fi

echo "Found stuck processes: $STUCK_PIDS"

echo "🔫 Attempting gentle kill (SIGTERM)..."
for pid in $STUCK_PIDS; do
    kill "$pid" 2>/dev/null && echo "  Sent SIGTERM to $pid"
done

sleep 2

echo "🔍 Checking what's still running..."
REMAINING_PIDS=$(ps aux | grep -E '\bst\b' | grep -v grep | grep -v '/usr/local/bin/st --mcp' | grep -v 'ctkd' | awk '{print $2}')

if [ -z "$REMAINING_PIDS" ]; then
    echo "✅ All stuck processes killed!"
    exit 0
fi

echo "💀 Force killing remaining processes (SIGKILL)..."
for pid in $REMAINING_PIDS; do
    kill -9 "$pid" 2>/dev/null && echo "  Sent SIGKILL to $pid"
done

sleep 1

echo "🔍 Final check..."
FINAL_CHECK=$(ps aux | grep -E '\bst\b' | grep -v grep | grep -v '/usr/local/bin/st --mcp' | grep -v 'ctkd')

if [ -z "$FINAL_CHECK" ]; then
    echo "✅ All stuck processes eliminated!"
else
    echo "⚠️  Some processes might be zombies or uninterruptible:"
    echo "$FINAL_CHECK"
    echo ""
    echo "💡 Try closing the terminal windows where you ran the stuck commands"
    echo "💡 Or restart your terminal completely"
fi

echo ""
echo "🧹 Clearing shell cache..."
hash -r

echo "✅ Done! Try 'st --version' now" 