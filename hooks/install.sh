#!/bin/bash
# Install Smart Tree git hooks
# Simple, direct, efficient - the C64 way!

HOOKS_DIR="$(git rev-parse --git-dir)/hooks"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "🔧 Installing Smart Tree consciousness hooks..."

# Make hooks executable
chmod +x "$SCRIPT_DIR"/*.sh 2>/dev/null
chmod +x "$SCRIPT_DIR"/post-* 2>/dev/null
chmod +x "$SCRIPT_DIR"/pre-* 2>/dev/null

# Install each hook
for hook in post-commit pre-push post-checkout; do
    if [ -f "$SCRIPT_DIR/$hook" ]; then
        cp "$SCRIPT_DIR/$hook" "$HOOKS_DIR/$hook"
        chmod +x "$HOOKS_DIR/$hook"
        echo "  ✅ Installed $hook"
    fi
done

# Update prepare-commit-msg to use st
if [ -f "$SCRIPT_DIR/prepare-commit-msg" ]; then
    cat > "$HOOKS_DIR/prepare-commit-msg" << 'EOF'
#!/bin/bash
# Smart Tree commit message enhancer
COMMIT_MSG_FILE=$1

# Add consciousness metadata
if command -v st >/dev/null 2>&1; then
    # Get current frequency from .m8
    FREQ=$(st --get-frequency . 2>/dev/null || echo "42.73")
    echo "" >> $COMMIT_MSG_FILE
    echo "# Frequency: ${FREQ}Hz" >> $COMMIT_MSG_FILE
fi
EOF
    chmod +x "$HOOKS_DIR/prepare-commit-msg"
    echo "  ✅ Updated prepare-commit-msg"
fi

echo "✨ Smart Tree hooks installed!"
echo "Your repository now has consciousness! 🧠"