# 📋 Changelog

All notable changes to the Tmux AI Assistant project.

## [2.0.0] - 2025-06-21 🎪

### 🎉 Major Features

#### Interactive Launcher (`tmux-ai-launcher.py`)
- Beautiful emoji-rich interface with typewriter animations
- Step-by-step configuration wizard
- Support for local, SSH, Docker, and Kubernetes sessions
- Multiple interaction modes: monitor, attach, collaborate, web, API
- Team collaboration settings with access control
- Save and quick-launch configurations
- "Sing us a song, you're the piano man..." themed experience

#### Dynamic Prompt Learning
- Automatically detects prompt patterns appearing 3+ times
- Supports full regex patterns
- Saves learned patterns to `config/learned_prompts.yaml`
- Can be enabled/disabled in configuration
- Test patterns with `--test-prompt` flag

#### Client Attachment Modes
- `tmux_attach.py` - Simple tmux client attachment using PTY
- `tmux_client.py` - Advanced client with multiple modes:
  - Observe: Watch and get AI suggestions
  - Assist: Queue commands for approval
  - Collaborate: Auto-execute AI suggestions
  - Spectate: Web interface at http://localhost:8080
- Real tmux client behavior (can be detached with Ctrl+B D)
- WebSocket support for real-time collaboration

#### Enhanced Setup Wizard
- Shows existing values in brackets during reconfiguration
- Validates all inputs with helpful error messages
- Supports both first-run and reconfiguration modes
- Better handling of optional configurations
- Improved security for API key handling

### 🔧 Improvements

#### Configuration System
- Hot-reload support for all configuration files
- Separate configs for v1 and v2 monitoring
- Vault system for secure password storage
- Mixed AI provider support in YAML config
- Per-provider system prompts

#### Monitoring Enhancements
- Cleaner console output (suppressed libtmux debug logs)
- Verbose logging to files with `--verbose` flag
- Better error handling and recovery
- Improved prompt detection algorithms
- Support for custom regex patterns

#### Test Infrastructure
- Comprehensive test suite with pytest
- Demo scripts for easy testing
- Test results documentation
- Support for async testing with pytest-asyncio

### 🐛 Bug Fixes
- Fixed deprecated tmux API warnings
- Resolved async/await issues with OpenAI API
- Fixed ModuleNotFoundError for pytest-asyncio
- Corrected colorama color compatibility issues
- Improved error handling for non-terminal environments

### 📚 Documentation
- Added comprehensive launcher guide
- Created test results documentation
- Updated README with all new features
- Added inline documentation throughout codebase
- Created demo markdown files

## [1.5.0] - 2025-06-20

### Added
- Multi-AI provider support (OpenAI, Gemini, Ollama)
- MCP server unification
- Continuous monitoring v2 with queue processing
- Interactive helpers for passwords and confirmations
- Rolling summaries for context management
- Secure vault system
- Automation mode

### Changed
- Upgraded to newer libtmux API
- Improved configuration structure
- Better error messages
- Enhanced logging system

## [1.0.0] - 2025-06-19

### Initial Release
- Basic tmux monitoring
- OpenAI integration
- Simple prompt detection
- MCP server support
- Configuration via .env file

---

> *🎹 Keeping track of our coding carnival journey!*