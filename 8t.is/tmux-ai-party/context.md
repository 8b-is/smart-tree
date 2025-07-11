# 🚀 Tmux AI Assistant Context

## Project Overview

An intelligent tmux session monitor that uses multiple AI providers (OpenAI, Google Gemini) to summarize session activity and generate helpful next steps. This is the central control system for your terminal AI companions.

## 🌟 Key Features

- Monitors tmux sessions for prompt patterns (e.g., " > ", "$ ", ">>> ")
- Captures and summarizes all activity since last interaction
- Multi-AI support: Uses either OpenAI or Google Gemini APIs for intelligence
- Generates contextual next steps based on the activity summary
- Configurable system prompts via markdown files with hot-reload capability
- Beautiful colorized terminal output (Trish approves!)
- Extensive logging for review and debugging

## 🏗️ Architecture

- **tmux_monitor.py**: Main application that monitors tmux sessions
- **mcp_server.py**: MCP server for deep research integration
- **test_monitor.py**: Comprehensive test suite for all AI providers
- **config/**: Directory containing system prompts and configuration
- **logs/**: Directory for detailed interaction logs
- **scripts/manage.sh**: Management script for starting, stopping, testing, etc.
- **libtmux**: Python library for tmux interaction
- **AI APIs**: OpenAI and Google Gemini for summarization and next-step generation

## 🧠 AI Integration Details

- **OpenAI**: Uses gpt-4o models by default for best quality
- **Google Gemini**: Uses gemini-1.5-flash for efficient processing
- Both providers implement the same interface for:
  - Activity summarization
  - Next step generation
  - Asynchronous operation for responsiveness

## ⚙️ Configuration

- YAML-based configuration system (config/config.yaml)
- Per-provider model configuration
- Adjustable parameters: check interval, max history, temperature, etc.
- System prompt customization via markdown files

## 💻 Critical Information

- Session monitoring happens in real-time with configurable check intervals
- Two-stage AI processing: summarization → next step generation
- All config files and system prompts can be hot-reloaded without restarting
- Supports multiple prompt patterns for flexibility
- All AI interactions are logged for later review
- Asynchronous operation for improved responsiveness

## 🔄 Recovery Notes

If starting fresh:

1. Set up Python virtual environment: `python -m venv .venv && source .venv/bin/activate`
2. Install dependencies: `pip install -r requirements.txt`
3. Configure API keys:
   - OpenAI: Set `OPENAI_API_KEY` in .env or pass via command line
   - Gemini: Set `GEMINI_API_KEY` in .env or pass via command line
4. Create system prompt markdown files in config/
5. Run with target tmux session name using the manage.sh script:
   - `./scripts/manage.sh start my-session openai` (for OpenAI)
   - `./scripts/manage.sh start my-session gemini` (for Gemini)

## 🧪 Testing

Run the test suite to verify all functionality:

```bash
pytest test_monitor.py -v --color=yes
```

Or use the manage.sh script:

```bash
./scripts/manage.sh test
```
