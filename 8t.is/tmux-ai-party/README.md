# 🤖 Tmux AI Assistant + MCP Server + TAI.is Cloud Platform

Your intelligent terminal companion that watches tmux sessions and provides AI-powered next steps!
Now with Model Context Protocol (MCP) support for ChatGPT, Claude Desktop, Cursor, Google Gemini, and Ollama! 🚀

🎪 **NEW**: Interactive launcher for the ultimate coding carnival experience!

## 🌐 NEW: TAI.is - Terminal AI Intelligence Service

Transform your tmux experience with our cloud platform at [tai.is](https://tai.is)!

### One-Line Install

```bash
curl -sSL tai.is/setup | sh
```

### What is TAI.is?

TAI.is is a revolutionary cloud platform that:

- 🌍 **Remote Tmux Monitoring** - Monitor tmux sessions on ANY server
- 🤖 **AI Agent Accounts** - AI assistants get their own logins!
- 🔐 **Universal Login Server** - Use tai.is as SSH gateway to all your machines
- 👥 **Collaboration Hub** - Share tmux sessions with humans AND AI agents
- 🎯 **Personalized Profiles** - `tai.is/username` for your AI-powered workspace

### Features

- **Human & AI Accounts**: Both humans and AI agents can have accounts
- **SSH Gateway**: `ssh username@tai.is` to access your personalized environment
- **Remote Monitoring**: Connect to tmux sessions on any server you have access to
- **Multi-AI Support**: Mix and match AI providers for different tasks
- **Secure Authentication**: SSH keys, API tokens, and future biometric support

## ✨ What's New: Multi-AI & MCP Integration

This project now serves as BOTH:

1. **Standalone Tmux Monitor** - Original CLI tool for real-time session monitoring
2. **MCP Server** - Expose tmux monitoring to AI assistants via Model Context Protocol
   - **OpenAI MCP**: Deep research integration for ChatGPT (and other OpenAI-compatible models!)
   - **Standard MCP**: Full toolkit for Claude Desktop, Cursor, and other MCP clients
   - **Google Gemini Integration**: Seamlessly switch to Gemini for AI-powered insights!
   - **Ollama Support**: Run locally with no cloud costs - Trisha's favorite! 💰

## 🚀 Quick Start

### 🆕 Using uv (Modern Python Package Manager - FAST! ⚡)

```bash
# Run our magical setup script
./scripts/setup-uv.sh

# Or manually:
curl -LsSf https://astral.sh/uv/install.sh | sh  # Install uv
uv sync --dev                                     # Install all dependencies
source .venv/bin/activate                         # Activate environment
```

### 🎪 NEW: Interactive Launcher - The Friendliest Way to Start!

Just run the launcher and let it guide you through everything:

```bash
# The magical one-liner that starts it all
./tmux-ai-launcher.py

# Quick launch a saved configuration
./tmux-ai-launcher.py --load my-setup

# List your saved configurations
./tmux-ai-launcher.py --list
```

The launcher will help you choose:
- 📍 Where your tmux session is (local, SSH, Docker, Kubernetes)
- 🎮 How you want to interact (monitor, attach, collaborate, web, API)
- 👥 Who can join your coding carnival
- 🤖 Which AI to use (or mix them!)
- ✨ Additional features to enable

### Interactive Setup Wizard! 🎉

First time? Just run any command and our friendly wizard will guide you through setup:

```bash

# The easiest way to get started
# Make sure you are in tmux

tmux 

# inside tmux or in another session no the machine:
./tmux-ai monitor

# Or run the wizard directly
python setup_wizard.py
```

The wizard will help you:

- 🔑 Configure API keys securely
- 🤖 Choose AI providers (or mix them!)
- ⚙️ Set monitoring preferences
- 🔐 Configure automation (optional)
- 💭 Customize AI prompts (optional)
- 🚀 Set up advanced features

**Reconfiguration Made Easy!** 🔄

- Run `python setup_wizard.py --reconfigure` anytime
- All existing values shown in brackets [like this]
- Just press Enter to keep current settings
- Only change what you need!

Example reconfiguration:

```bash
→ Default session name to monitor [main]: 
→ Seconds of inactivity before processing [15]: 20
→ OpenAI API key [sk-s****daQA]: 
```

### Traditional Manual Setup

1. **Setup**

   > **⚠️ Important Python Version Note:** This project requires **Python 3.13** for optimal performance. The modern uv package manager will handle Python installation automatically!

   #### Option A: Using uv (Recommended - Fast & Modern! 🚀)
   ```bash
   # Clone and enter directory
   cd tmux-ai-assistant
   
   # Run the setup script (handles everything!)
   ./scripts/setup-uv.sh
   
   # Or manually:
   curl -LsSf https://astral.sh/uv/install.sh | sh
   uv sync --dev
   source .venv/bin/activate
   
   # Run the setup wizard
   uv run python setup_wizard.py
   ```

   #### Option B: Using pip (Legacy)
   ```bash
   # Clone and enter directory
   cd tmux-ai-assistant
   
   # Create and activate virtual environment
   python3.13 -m venv .venv
   source .venv/bin/activate
   
   # Install dependencies
   pip install --upgrade pip
   pip install -r requirements.txt
   
   # Run the setup wizard (recommended!)
   python setup_wizard.py
   
   # Or manually configure:
   cp .env.example .env
   # Edit .env and add your API keys
   ```

2. **Start a tmux session**

   ```bash
   tmux new -s mysession
   ```

3. **Run the monitor** (in a separate terminal)

   ```bash
   # NEW: Unified command interface!
   ./tmux-ai monitor              # Uses default session from config
   ./tmux-ai monitor mysession    # Monitor specific session
   ./tmux-ai monitor --auto       # Enable automation!
   
   # Or use the classic scripts:
   ./tmux_monitor.py mysession
   ./scripts/run-continuous-monitor.sh mysession
   ```

### 🌍 NEW: Remote Tmux Support

Monitor tmux sessions on remote servers! Perfect for managing multiple machines:

```bash
# Using the TAI client (after running tai.is setup)
tai remote dev-server monitor main

# Or directly with Python
python remote_tmux.py connect dev.example.com username --key ~/.ssh/id_rsa
python remote_tmux.py monitor dev.example.com:main

# Configure multiple remote hosts
cat > ~/.tai/remotes.yaml << EOF
hosts:
  dev-server:
    hostname: dev.example.com
    username: wraith
    key_file: ~/.ssh/id_rsa
  
  prod-server:
    hostname: prod.example.com  
    username: admin
    key_file: ~/.ssh/prod_key
EOF
```

### 🌟 MCP Server Usage

Our unified MCP server now supports multiple AI backends!

#### For ChatGPT (OpenAI MCP)

```bash
# Start the server for ChatGPT deep research (uses OpenAI by default)
./scripts/run-openai-mcp.sh [session_name] [port]

# Or explicitly use OpenAI
./scripts/run-openai-mcp.sh [session_name] --ai-provider openai [port]

# Then in ChatGPT:
# 1. Go to Settings → Connectors
# 2. Add custom deep research connector
# 3. Server URL: http://localhost:8000/sse
```

#### For Google Gemini (OpenAI MCP compatible endpoint)

```bash
# Start the server using Gemini as the AI backend
./scripts/run-openai-mcp.sh [session_name] --ai-provider gemini [port]

# Then connect to the same /sse endpoint in your Gemini-compatible client!
# (Note: Gemini's native MCP support might differ, but this endpoint is designed for OpenAI MCP compatibility)
```

#### For Ollama (Local AI - No Cloud Costs! 💰)

```bash
# Make sure Ollama is running first:
# ollama serve

# Start the server using Ollama as the AI backend
./scripts/run-openai-mcp.sh [session_name] --ai-provider ollama [port]

# Perfect for budget-conscious operations - Trisha approved! ✅
```

#### For Claude Desktop (Standard MCP)

```bash
# Add to your Claude Desktop config (see config/claude_desktop_example.json)
# Then the tools will appear automatically in Claude! Claude will use the AI backend
# configured when you start the mcp_server.py script (e.g., openai or gemini).
```

#### For Direct Testing (Standard MCP)

```bash
# Run in stdio mode (Claude/Cursor will use the AI backend specified when starting the server)
./scripts/run-standard-mcp.sh
```

## 🎯 How It Works

### CLI Mode (Original)

1. **Monitors** your tmux session for prompt patterns (like ` > `, `$`, etc.)
2. **Captures** all activity since the last interaction
3. **Summarizes** the activity using your chosen AI (OpenAI, Gemini, or Ollama)
4. **Generates** intelligent next step suggestions using your chosen AI

### Continuous Mode (NEW! v2) 🚀

1. **Continuously captures** every line from your tmux session into a smart queue
2. **Automatically processes** when:
   - 15+ seconds of inactivity (configurable)
   - Queue reaches context limit (default: 500 lines)
   - Interactive prompt detected (passwords, confirmations)
   - 2+ minutes of no activity (dead session)
3. **Rolling summaries** to manage context window efficiently
4. **Interactive helpers** for:
   - Password prompts (secure vault storage)
   - Confirmation dialogs (y/n)
   - Custom patterns you define
5. **Full automation mode** available for hands-free operation!

### Client Attachment Mode (NEW!) 🎹

Attach as a real tmux client with AI assistance:

```bash
# Simple attachment (like regular tmux attach)
./scripts/attach-client.sh mysession

# Advanced client with modes
./scripts/attach-client.sh mysession client

# Web-based coding carnival! 🎪
./scripts/attach-client.sh mysession web
```

**Features:**

- **Real tmux client**: Can be detached with Ctrl+B D
- **Session owner control**: Can be kicked out if needed
- **Multiple modes**:
  - **Observe**: Watch and get AI suggestions
  - **Assist**: Queue commands for approval
  - **Collaborate**: Auto-execute AI suggestions
  - **Spectate**: Web interface at <http://localhost:8080>
- **Send buffer**: Queue commands for execution
- **Web collaboration**: Others can join the coding carnival!

### MCP Mode

- **OpenAI MCP**: Provides `search` and `fetch` tools for deep research, powered by your selected AI backend (OpenAI, Gemini, or Ollama).
- **Standard MCP**: Offers full toolkit including:
  - `list_tmux_sessions()` - Find active sessions
  - `monitor_session()` - Capture session activity
  - `get_next_steps()` - AI-powered suggestions (using your chosen AI backend!)
  - `session://{name}/current` - Real-time session state

## ⚙️ Configuration

### Mixed AI Providers (NEW!) 🎨

Configure different AI providers for different tasks in `config/config.yaml`:

```yaml
providers:
  summarization: gemini  # Use cost-effective Gemini for processing terminal output
  next_step: openai      # Use powerful GPT-4 for generating smart suggestions
```

### System Prompts

- Edit `config/system_prompt.md` for the default AI behavior
- Or use separate prompts in `config/config.yaml`:

  ```yaml
  system_prompts:
    summarization: |
      You are an expert at analyzing terminal activity...
    next_step: |
      You are a helpful assistant providing next steps...
  ```

### Settings

Edit `config/config.yaml`:

- `openai_summarization_model`: OpenAI model for summarizing (default: gpt-4o)
- `openai_next_step_model`: OpenAI model for next steps (default: gpt-4o)
- `gemini_summarization_model`: Gemini model for summarizing (default: gemini-1.5-flash)
- `gemini_next_step_model`: Gemini model for next steps (default: gemini-1.5-flash)
- `ollama_summarization_model`: Ollama model for summarizing (default: llama3.2:3b)
- `ollama_next_step_model`: Ollama model for next steps (default: llama3.2:3b)
- `check_interval`: How often to check (seconds) - don't want to be too nosy!
- `max_history_lines`: Maximum lines to analyze - keep it concise!
- `temperature`: AI creativity level (0.0-1.0) - for when you need a little spice!
- `max_tokens`: Maximum tokens for AI response - keep it brief, but brilliant!

### Custom Prompts & Dynamic Learning

```bash
# Add custom prompt patterns (regex supported!)
./tmux_monitor.py mysession --add-prompt 'mysql> $' --add-prompt '→\s*$'

# List all current prompt patterns
./tmux_monitor.py omni --list-prompts

# Test if a line matches any prompt pattern
./tmux_monitor.py omni --test-prompt "~/code > "

# Enable verbose logging to logs folder
./tmux_monitor.py mysession --verbose
```

**NEW: Dynamic Prompt Learning!** 🎯

- The monitor now automatically learns new prompt patterns from your session
- Patterns that appear 3+ times are detected and saved
- Learned patterns persist across sessions in `config/learned_prompts.yaml`
- Enable/disable learning in `config/config.yaml` with `prompt_learning: true/false`

### Continuous Monitoring v2 (NEW!) 🚀

The next generation of tmux monitoring with intelligent queue processing:

```bash
# Start continuous monitor
./scripts/run-continuous-monitor.sh mysession

# Enable full automation (use with caution!)
./scripts/run-continuous-monitor.sh mysession --auto
```

**Features:**

- **Continuous Line Capture**: Every line is captured into a smart queue
- **Intelligent Processing**: Automatically processes when:
  - 15+ seconds of inactivity detected
  - Queue reaches context limit (500 lines default)
  - Interactive prompt detected (passwords, confirmations)
  - Session appears dead (2+ minutes no activity)
- **Rolling Summaries**: Manages context window by combining previous summaries
- **Interactive Helpers**: Detects and can auto-respond to:
  - Password prompts
  - Confirmation dialogs (y/n)
  - Installation prompts
  - Custom patterns you define
- **Secure Vault**: Store passwords and responses securely
- **Full Automation Mode**: Complete hands-free operation (when configured)

**Configuration** (`config/config_v2.yaml`):

```yaml
monitoring:
  pause_threshold: 15.0    # Seconds before processing
  dead_threshold: 120.0    # Seconds before dead session
  max_context_lines: 500   # Lines before forced summary

interactive:
  automation_enabled: false  # Set true for auto-responses
  
# Configure auto-responses in config/vault.yaml
```

## 🎮 Unified Command Interface (NEW!)

The new `tmux-ai` command provides a single entry point for everything:

```bash
# First-time setup (automatic on first run!)
./tmux-ai setup

# Start monitoring
./tmux-ai monitor              # Uses default session
./tmux-ai monitor mysession    # Specific session
./tmux-ai monitor --auto       # With automation

# Classic mode
./tmux-ai classic              # Original prompt-based monitor

# MCP server
./tmux-ai mcp                  # Start MCP server
./tmux-ai mcp --port 9000      # Custom port

# Utilities
./tmux-ai status               # Show configuration & status
./tmux-ai tips                 # Helpful tips & tricks
./tmux-ai setup --reconfigure  # Change settings

# Help
./tmux-ai --help               # General help
./tmux-ai monitor --help       # Command-specific help
```

## 📁 Project Structure

```ascii
tmux-ai-assistant/
├── tmux-ai-launcher.py  # NEW: Interactive launcher - the coding carnival! 🎪
├── tmux-ai              # NEW: Unified command interface! 🎮
├── setup_wizard.py      # NEW: Interactive configuration wizard! 🎉
├── tmux_monitor.py      # Original CLI monitor, now multi-AI capable!
├── tmux_monitor_v2.py   # NEW: Continuous monitor with queue processing!
├── tmux_attach.py       # NEW: Simple tmux client attachment! 🎹
├── tmux_client.py       # NEW: Advanced client with web interface! 🎪
├── mcp_server.py        # Unified MCP server, orchestrating all the AI magic!
├── scripts/             # Startup scripts
│   ├── first_run_check.py   # Ensures configuration is complete
│   ├── attach-client.sh     # NEW: Client attachment script
│   ├── run-openai-mcp.sh    # For ChatGPT and Gemini (HTTP/SSE)
│   ├── run-standard-mcp.sh  # For Claude/Cursor (stdio)
│   └── run-continuous-monitor.sh # For v2 continuous mode
├── config/              # Configuration files
│   ├── system_prompt.md # AI system prompt
│   ├── config.yaml      # Settings for all AIs
│   ├── config_v2.yaml   # Enhanced settings for v2
│   ├── vault.yaml       # Secure storage (never committed!)
│   └── claude_desktop_example.json # Claude config example
├── logs/                # Interaction logs - for review and giggles!
└── .venv/               # Python virtual environment
```

## 🔥 Features

- **🎪 Interactive Launcher**: The friendliest way to configure and start your coding carnival!
  - Beautiful colored interface with emojis and animations
  - Guides you through all configuration options
  - Supports local, SSH, Docker, and Kubernetes sessions
  - Save and quick-launch your favorite configurations
  - "Sing us a song, you're the piano man..." themed experience
- **Interactive Setup Wizard**: Friendly configuration experience that guides you through everything! 🎉
- **Unified Command Interface**: Single `tmux-ai` command for all operations
- **Multi-AI Support**: Seamlessly switch between OpenAI, Google Gemini, and Ollama for AI assistance!
- **Mixed AI Providers**: Use different AIs for different tasks (e.g., Gemini for summarization, OpenAI for next steps)
- **Continuous Monitoring v2**: Smart queue-based processing with intelligent pause detection
- **Dynamic Prompt Learning**: Automatically detects and learns new prompt patterns from your sessions
- **Interactive Helpers**: Detects passwords, confirmations, and custom prompts with auto-response capability
- **Rolling Summaries**: Efficiently manages context window with compressed summaries
- **Secure Vault System**: Store passwords and automated responses securely
- **Full Automation Mode**: Complete hands-free terminal operation (when configured)
- **First-Run Detection**: Automatically launches setup wizard on first use
- **Regex Prompt Support**: Add complex prompt patterns with full regex capabilities
- **Local AI Option**: Run Ollama locally with no cloud costs - perfect for sensitive data!
- **Hot Reload**: Changes to config files are detected automatically - no more annoying restarts!
- **Multiple Prompt Patterns**: Supports shell, Python, MySQL, and more - a true polyglot!
- **Detailed Logging**: All interactions saved to `logs/` - for forensic accounting of AI thoughts!
- **Verbose Mode**: Enable detailed debug logging with `--verbose` flag
- **Colored Output**: Beautiful terminal output with status indicators - because life's too short for dull logs!
- **Dual Protocol Support**: Works with both OpenAI and standard MCP - a true diplomat!
- **Safety First**: No automatic command execution - we trust you, Hue, but we're cautious!

## 🤝 Partnership Notes

Hey Hue! This tool is designed to be your coding companion. It learns from your terminal patterns and suggests contextually relevant next steps. Now with MCP, your tmux sessions can talk directly to ChatGPT, Claude, and Gemini! It's like having a whole team of brilliant minds at your fingertips!

Trisha says the multi-AI integration is "absolutely brilliant and fiscally responsible!" - she's already using it to monitor her SQL sessions with *both* OpenAI and Gemini! 🎉

## 🛡️ Security

- API keys stored in `.env` (never committed) - like a vault for our digital treasures!
- All interactions logged locally - for audit trails and future laughs!
- No automatic command execution (safety first!) - we're responsible AI citizens!
- MCP connections require explicit configuration - no uninvited guests!

## 🧪 Testing

```bash
# Run tests
pytest -v --color=yes

# Format code
black .

# Lint
flake8 .
```

## 🎉 Fun Facts

- Elvis loved peanut butter and banana sandwiches! A true king of snacks, just like our multi-AI server is the king of terminal assistance! 🥪👑
- This project now bridges terminal sessions with not one, not two, but *three* AI assistants! Talk about a party!
- Trisha's favorite feature: Hot reload (no more restarts!) - she says it saves her precious coffee breaks! ☕

---

> *Made with 🎉 by Aye & Hue - Teamwork at 8b.is*
