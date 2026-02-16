# Smart Tree Development Shell Functions
# Add these to your ~/.zshrc or ~/.bashrc for convenient development

# Function to rebuild and install Smart Tree
st-rebuild() {
    if [[ ! -f Cargo.toml ]] || [[ ! -d src ]]; then
        echo "❌ Please run this from the smart-tree project directory"
        return 1
    fi
    
    echo "🔨 Building Smart Tree..."
    cargo build --release || return 1
    
    echo "📦 Installing Smart Tree..."
    sudo cp ./target/release/st /usr/local/bin/st || return 1
    
    echo "🧹 Clearing shell cache..."
    hash -r
    
    echo "✅ Smart Tree rebuilt and installed!"
    st --version
}

# Function to quickly test Smart Tree without installing
st-test() {
    if [[ ! -f Cargo.toml ]] || [[ ! -d src ]]; then
        echo "❌ Please run this from the smart-tree project directory"
        return 1
    fi
    
    echo "🔨 Building Smart Tree..."
    cargo build --release || return 1
    
    echo "🧪 Testing local build:"
    ./target/release/st "$@"
}

# Function to clear Smart Tree cache and test
st-refresh() {
    echo "🧹 Clearing shell cache..."
    hash -r
    echo "✅ Cache cleared!"
    st --version
}

# Function to quickly check both installed and local versions
st-versions() {
    echo "📦 Installed version:"
    if command -v st >/dev/null 2>&1; then
        which st
        st --version 2>/dev/null || echo "  ❌ Installed version not working"
    else
        echo "  ❌ No installed version found"
    fi
    
    echo ""
    echo "🔨 Local build version:"
    if [[ -f ./target/release/st ]]; then
        echo "  ./target/release/st"
        ./target/release/st --version 2>/dev/null || echo "  ❌ Local build not working"
    else
        echo "  ❌ No local build found (run 'cargo build --release')"
    fi
}

# Function to kill stuck st processes
st-killall() {
    echo "🔍 Finding stuck st processes..."
    
    # Find stuck processes (excluding MCP server)
    local stuck_pids=$(ps aux | grep -E '\bst\b' | grep -v grep | grep -v '/usr/local/bin/st --mcp' | grep -v 'ctkd' | awk '{print $2}')
    
    if [ -z "$stuck_pids" ]; then
        echo "✅ No stuck st processes found!"
        return 0
    fi
    
    echo "Found stuck processes: $stuck_pids"
    echo "🔫 Attempting to kill..."
    
    for pid in $stuck_pids; do
        kill -9 "$pid" 2>/dev/null && echo "  Killed $pid"
    done
    
    sleep 1
    
    # Check if any remain
    local remaining=$(ps aux | grep -E '\bst\b' | grep -v grep | grep -v '/usr/local/bin/st --mcp' | grep -v 'ctkd')
    
    if [ -n "$remaining" ]; then
        echo "⚠️  Some processes are still stuck (uninterruptible state):"
        echo "$remaining"
        echo ""
        echo "💡 Solutions:"
        echo "   1. Close the terminal windows where you ran the stuck commands"
        echo "   2. Use Activity Monitor to force quit them"
        echo "   3. Run: sudo pkill -9 st (kills ALL st processes)"
        echo "   4. Restart your terminal"
    else
        echo "✅ All stuck processes killed!"
    fi
    
    hash -r
}

echo "🌳 Smart Tree development functions loaded!"
echo "Available commands:"
echo "  st-rebuild  - Build and install Smart Tree"
echo "  st-test     - Test local build without installing"
echo "  st-refresh  - Clear shell cache and test version"
echo "  st-versions - Check both installed and local versions"
echo "  st-killall  - Kill all stuck st processes" 