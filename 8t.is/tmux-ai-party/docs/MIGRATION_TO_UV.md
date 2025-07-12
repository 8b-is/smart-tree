# 🚀 Migration Guide: pip → uv

Welcome to the future of Python package management! This guide helps you migrate from pip to uv. Aye here to make this journey smooth as silk! 🏴‍☠️

## Why uv? 

- **⚡ Speed**: 10-100x faster than pip (Trisha tested it - she's impressed!)
- **🔒 Lockfile**: Reproducible builds with uv.lock
- **🐍 Python Management**: Automatically handles Python versions
- **📦 Modern**: Built in Rust, designed for the future
- **🎯 Unified**: Replaces pip, pip-tools, pipx, poetry, pyenv, virtualenv

## Quick Migration (5 minutes!)

### 1. Run the Setup Script
```bash
./scripts/setup-uv.sh
```

This magical script will:
- Install uv
- Create a new virtual environment with Python 3.13
- Install all dependencies
- Show you the next steps

### 2. Manual Migration Steps

If you prefer doing it manually:

```bash
# Install uv
curl -LsSf https://astral.sh/uv/install.sh | sh

# Remove old virtual environment (optional but recommended)
rm -rf .venv

# Create new environment and sync
uv sync --dev

# Activate (same as before!)
source .venv/bin/activate
```

## Command Comparison

Here's how your muscle memory translates:

| Old (pip) | New (uv) | What it does |
|-----------|----------|--------------|
| `pip install -r requirements.txt` | `uv sync` | Install dependencies |
| `pip install package` | `uv add package` | Add new dependency |
| `pip install -e .` | `uv sync` | Install in editable mode |
| `pip list` | `uv pip list` | List installed packages |
| `pip freeze` | `uv pip freeze` | Show exact versions |
| `python script.py` | `uv run python script.py` | Run without activation |

## New Superpowers! 🦸

### 1. Run Without Activation
```bash
# No need to activate .venv first!
uv run pytest
uv run python tmux_monitor.py
```

### 2. Add Dependencies Properly
```bash
# Adds to pyproject.toml AND installs
uv add requests
uv add --dev pytest-mock
```

### 3. Update Everything
```bash
# Update all dependencies to latest compatible versions
uv sync --upgrade
```

### 4. Lock Dependencies
```bash
# Creates uv.lock for reproducible installs
uv lock
```

## Project Structure Changes

### New Files
- `pyproject.toml` - Modern Python project configuration
- `uv.lock` - Exact dependency versions (commit this!)
- `.python-version` - Specifies Python 3.13

### Keep These
- `requirements.txt` - For backwards compatibility
- `.env` - Your configuration (unchanged)
- Everything else stays the same!

## Common Questions

**Q: Do I need to change my code?**  
A: Nope! Your Python code works exactly the same. 🎉

**Q: Can I still use pip?**  
A: Yes! The venv works with both. But why would you? 😉

**Q: What about my CI/CD?**  
A: Add one line: `pip install uv` then use `uv sync`

**Q: Is it stable?**  
A: Used by Instagram, Mozilla, and many more. Very stable!

## Troubleshooting

### Issue: "command not found: uv"
```bash
# Reload your shell
source ~/.bashrc  # or ~/.zshrc
# Or add to PATH manually
export PATH="$HOME/.cargo/bin:$PATH"
```

### Issue: "Python 3.13 not found"
```bash
# uv handles this automatically!
uv sync  # Will download Python 3.13 if needed
```

### Issue: "Module not found" errors
```bash
# Ensure you're in the activated environment
source .venv/bin/activate
# Or use uv run
uv run python your_script.py
```

## Tips from the Crew

**Aye says**: "The speed difference is incredible. What took minutes now takes seconds!"

**Hue loves**: "No more pip version conflicts or dependency hell!"

**Trisha's favorite**: "The colorful output! And it's SO much faster for her budget builds!"

## Need Help?

- Run `uv --help` for built-in docs
- Check [uv documentation](https://github.com/astral-sh/uv)
- Ask in our tmux-ai-assistant discussions!

---

Welcome to the future of Python package management! 🚀 The water's warm in the uv pool - jump in! 🏊‍♂️