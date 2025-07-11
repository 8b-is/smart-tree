#!/bin/bash
# setup-uv.sh - Aye's magical uv setup script! 🪄
# Helps Hue transition to the modern Python package management world

set -e  # Exit on error - safety first!

echo "🚀 Welcome to the uv setup wizard for tmux-ai-assistant!"
echo "   Aye here, ready to modernize our Python setup!"
echo ""

# Check if uv is installed
if ! command -v uv &> /dev/null; then
    echo "📦 uv is not installed. Let's fix that!"
    echo "   Installing uv..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
    echo ""
    echo "✅ uv installed! You may need to restart your shell or run:"
    echo "   source ~/.bashrc  # or ~/.zshrc"
    echo ""
    read -p "Press Enter to continue after reloading your shell..."
fi

echo "🔍 Checking uv version..."
uv --version

# Check Python version
echo ""
echo "🐍 Checking Python version..."
if command -v python3.13 &> /dev/null; then
    echo "✅ Python 3.13 found!"
else
    echo "⚠️  Python 3.13 not found. uv will handle this for you!"
fi

# Clean up old virtual environment if it exists
if [ -d ".venv" ]; then
    echo ""
    echo "🧹 Found existing .venv directory"
    read -p "   Remove old virtual environment? (recommended) [Y/n]: " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        rm -rf .venv
        echo "✅ Old virtual environment removed"
    fi
fi

# Create new virtual environment with uv
echo ""
echo "🏗️  Creating new virtual environment with Python 3.13..."
uv venv --python 3.13

# Sync dependencies
echo ""
echo "📥 Installing all dependencies (this is FAST with uv!)..."
uv sync --dev

# Show activation instructions
echo ""
echo "🎉 Setup complete! Here's what Aye did for you:"
echo "   ✅ Installed uv (if needed)"
echo "   ✅ Created Python 3.13 virtual environment"
echo "   ✅ Installed all project dependencies"
echo "   ✅ Installed dev dependencies for testing"
echo ""
echo "📝 Next steps:"
echo "   1. Activate the virtual environment:"
echo "      source .venv/bin/activate"
echo ""
echo "   2. Run the tmux-ai assistant:"
echo "      ./tmux-ai setup    # First-time setup"
echo "      ./tmux-ai monitor  # Start monitoring"
echo ""
echo "   3. For development:"
echo "      uv run pytest      # Run tests"
echo "      uv run black .     # Format code"
echo "      uv run ruff check  # Lint code"
echo ""
echo "💡 Pro tip from Trisha: uv is FAST! No more waiting for pip!"
echo "   Try 'uv add --dev ipython' to see the speed difference!"
echo ""
echo "🏴‍☠️ Aye, Aye! Happy coding, Hue! 🚢"