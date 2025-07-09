# Scripts Directory 📁

This directory contains helpful scripts for Smart Tree installation, development, and management.

## 🚀 Installation Scripts

### For End Users

**`install.sh`** - Quick installation from GitHub releases
```bash
# Download and install the latest release
curl -sSL https://raw.githubusercontent.com/8b-is/smart-tree/main/scripts/install.sh | bash
```
- ✅ Downloads pre-built binary
- ✅ No build dependencies required  
- ✅ Fast and simple
- ✅ Perfect for production use

### For Developers

**`build-and-install.sh`** - Build from source and install
```bash
# Build and install from current source
./scripts/build-and-install.sh
```
- ✅ Builds from current source code
- ✅ Clears shell cache automatically
- ✅ Perfect for development workflow
- ✅ Always gets latest local changes

## 🛠️ Development Scripts

**`shell-functions.sh`** - Development helper functions
```bash
# Add to your ~/.zshrc or ~/.bashrc
source /path/to/smart-tree/scripts/shell-functions.sh

# Then use:
st-rebuild   # Build and install with cache clearing
st-test      # Test local build without installing
st-refresh   # Clear cache and test version
st-versions  # Check both installed and local versions
st-killall   # Kill stuck st processes
```

**`kill-stuck-st.sh`** - Kill hung st processes
```bash
./scripts/kill-stuck-st.sh
```

**`manage.sh`** - Comprehensive project management
```bash
./scripts/manage.sh help    # See all available commands
./scripts/manage.sh build   # Build project
./scripts/manage.sh test    # Run tests
./scripts/manage.sh release v3.2.0 "Release notes"
```

**`send-to-claude.sh`** - Demo Claude API integration
```bash
export ANTHROPIC_API_KEY=your_key
./scripts/send-to-claude.sh /path/to/analyze
```

## 🎯 Quick Reference

| Use Case | Script |
|----------|--------|
| **Install latest release** | `curl ... install.sh \| bash` |
| **Install from local source** | `./build-and-install.sh` |
| **Development workflow** | `source shell-functions.sh` |
| **Fix hung processes** | `./kill-stuck-st.sh` |
| **Project management** | `./manage.sh` |

## 🌳 Happy Tree Building!

Each script is designed for a specific purpose. Choose the right tool for your needs! 