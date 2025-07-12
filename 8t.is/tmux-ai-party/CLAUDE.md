# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Setup & Dependencies

#### Using uv (Recommended - Modern & Fast! 🚀)
```bash
# Install uv if not already installed
curl -LsSf https://astral.sh/uv/install.sh | sh

# Create virtual environment and sync dependencies
uv sync

# Activate the virtual environment
source .venv/bin/activate

# Install with dev dependencies
uv sync --dev

# Add a new dependency
uv add package-name

# Add a dev dependency
uv add --dev package-name

# Update all dependencies to latest versions
uv sync --upgrade
```

### Common Commands

```bash
# Run all tests (tests are in tests/ directory)
uv run pytest tests/ -v --color=yes

# Run specific test file
uv run pytest tests/test_monitor.py -v

# Run specific test
uv run pytest tests/test_monitor.py::TestPromptDetection::test_prompt_detection -v

# Run with coverage
uv run pytest --cov=tmux_ai_assistant --cov-report=html

# Code quality
uv run black .
uv run ruff check .
uv run ruff check --fix .  # Auto-fix issues
uv run mypy .

# Run all quality checks at once
uv run black . && uv run ruff check . && uv run mypy . && uv run pytest tests/ -v
```

### Running the Application

#### Interactive Launcher (NEWEST! 🎪)
```bash
# The friendliest way to start - interactive configuration
./tmux-ai-launcher.py

# Quick launch saved configurations
./tmux-ai-launcher.py --load my-dev-setup

# List saved configurations
./tmux-ai-launcher.py --list
```

#### Unified Command Interface (Recommended)
```bash
# First-time setup wizard
./tmux-ai setup

# Monitor tmux sessions
./tmux-ai monitor              # Uses default session from config
./tmux-ai monitor mysession    # Monitor specific session
./tmux-ai monitor --auto       # Enable automation mode

# Run MCP server
./tmux-ai mcp                  # Start MCP server
./tmux-ai mcp --port 9000      # Custom port

# Classic mode
./tmux-ai classic mysession    # Original prompt-based monitor
```

#### Direct Script Usage
```bash
# Classic monitor
./tmux_monitor.py mysession
./tmux_monitor.py mysession --add-prompt 'mysql> $' --add-prompt '=> $'

# Continuous monitor v2
./scripts/run-continuous-monitor.sh mysession
./scripts/run-continuous-monitor.sh mysession --auto

# MCP servers
./scripts/run-openai-mcp.sh [session] [port]         # ChatGPT/Gemini
./scripts/run-standard-mcp.sh                        # Claude/Cursor
```

### Using the scripts/manage.sh Tool

```bash
# Build project and install dependencies
./scripts/manage.sh build

# Run all tests
./scripts/manage.sh test

# Start MCP server with specific AI provider
./scripts/manage.sh start_mcp_server mysession openai
./scripts/manage.sh start_mcp_server mysession gemini
./scripts/manage.sh start_mcp_server mysession ollama

# With API keys (if not in .env)
./scripts/manage.sh start_mcp_server mysession openai "sk-xxx" "" 8000
./scripts/manage.sh start_mcp_server mysession gemini "" "gemini-key" 8000

# Stop MCP server
./scripts/manage.sh stop_mcp_server

# Restart MCP server
./scripts/manage.sh restart_mcp_server mysession

# Clean project (remove venv and logs)
./scripts/manage.sh clean
```

## Architecture Overview

This codebase implements a sophisticated tmux monitoring system with AI integration across multiple entry points and protocols. The project now includes the TAI.is cloud platform components.

### Core Architecture Principles

1. **Multi-Provider AI Strategy**: The system abstracts AI interactions through a unified interface, allowing different providers (OpenAI, Gemini, Ollama) to be used for different tasks. This is implemented in `tmux_monitor.py:55-144` where providers are initialized based on configuration.

2. **Two-Stage Processing Pipeline**: All AI interactions follow a summarization → next-steps pattern. Raw terminal output is first condensed, then analyzed for actionable suggestions.

3. **Hot-Reload Configuration**: Using the `watchdog` library, both monitors implement `ConfigWatcher` classes that detect changes to YAML/Markdown files and reload without restart.

4. **Protocol Abstraction**: The MCP server (`mcp_server.py`) serves as a protocol adapter, exposing tmux functionality through both OpenAI's HTTP/SSE protocol and standard MCP stdio protocol.

### Component Interaction Flow

```
User → tmux-ai-launcher.py → Builds command based on choices
                           ↓
                    tmux-ai (wrapper) → Routes to appropriate component
                           ↓
    ┌─────────────────────┴─────────────────────┐
    │                                           │
tmux_monitor.py                        tmux_monitor_v2.py
(Prompt-based)                         (Continuous queue)
    │                                           │
    └─────────────────────┬─────────────────────┘
                          ↓
                   AI Providers (via config)
                   - OpenAI
                   - Gemini  
                   - Ollama
```

### Project Structure

```
tmux-ai-party/
├── tests/                    # Test files (not in root!)
│   ├── test_monitor.py
│   └── test_launcher_demo.py
├── tai/                      # TAI.is package with own dependencies
│   ├── pyproject.toml
│   └── uv.lock
├── tai-website/              # TAI.is SvelteKit web interface
│   ├── src/
│   ├── static/
│   └── package.json
├── tai_auth_server.py        # TAI.is OAuth2/JWT authentication server
├── tmux-ai-launcher.py       # Interactive launcher with rich UI
├── tmux-ai                   # Unified command interface
├── setup_wizard.py           # Interactive configuration wizard
├── tmux_monitor.py           # Original CLI monitor (multi-AI)
├── tmux_monitor_v2.py        # Continuous monitor with queue
├── tmux_attach.py            # Simple tmux client attachment
├── tmux_client.py            # Advanced client with web interface
├── mcp_server.py             # Unified MCP server
├── remote_tmux.py            # Remote tmux session support
├── config/                   # Configuration files
│   ├── config.yaml           # Main settings
│   ├── config_v2.yaml        # V2 enhanced settings
│   ├── vault.yaml            # Secure storage (never committed!)
│   ├── system_prompt.md      # AI behavior
│   ├── next_step_prompt.md   # Next steps generation
│   └── .wizard_complete      # Setup wizard marker
├── scripts/                  # Management scripts
│   ├── manage.sh             # Build/test/run management
│   ├── tai-setup.sh          # TAI.is platform setup
│   └── ...                   # Other run scripts
├── context.md                # Project context documentation
├── .thedigs.json             # Extended configuration (auto-generated)
└── pyproject.toml            # Python 3.13+ dependencies
```

### Key Architectural Components

1. **Interactive Launcher** (`tmux-ai-launcher.py`):
   - Uses rich terminal UI with typewriter effects
   - Builds complex command strings based on user choices
   - Persists configurations in `~/.tmux-ai-party/`
   - Handles SSH, Docker, and Kubernetes session types

2. **Monitor Architecture** (shared between v1 and v2):
   - **Prompt Detection**: Regex-based pattern matching with dynamic learning
   - **Session Capture**: Uses `libtmux` for tmux interaction
   - **AI Client Management**: Lazy initialization of provider clients
   - **Config Hot-Reload**: Filesystem watchers for live updates

3. **MCP Server Design** (`mcp_server.py`):
   - Dual-protocol support in single codebase
   - Dynamic AI provider selection via command-line arguments
   - Shared session history for cross-protocol access
   - Async architecture using FastAPI and FastMCP

4. **TAI.is Platform Components** (NEW):
   - **TAI Authentication Server** (`tai_auth_server.py`): FastAPI OAuth2/JWT authentication
   - **TAI Package** (`tai/`): Separate Python package for TAI functionality
   - **TAI Website** (`tai-website/`): SvelteKit web interface with Tailwind CSS

### Configuration System

1. **Primary Config** (`config/config.yaml`):
   - Provider selection and model configuration
   - Timing parameters and token limits
   - System prompt overrides per provider

2. **V2 Extensions** (`config/config_v2.yaml`):
   - Queue processing thresholds
   - Interactive prompt patterns
   - Automation flags

3. **Environment Variables** (`.env`):
   ```
   OPENAI_API_KEY=
   GEMINI_API_KEY=
   OPENAI_NEXT_STEP_MODEL=gpt-4o  # Override default
   GEMINI_SUMMARIZATION_MODEL=gemini-1.5-flash  # Override default
   # TAI authentication server variables (if using TAI.is)
   TAI_SECRET_KEY=
   TAI_DATABASE_URL=
   ```

4. **Security Layer**:
   - `.env` for API keys (loaded via python-dotenv)
   - `config/vault.yaml` for automated responses
   - Never committed to version control

### AI Provider Integration Pattern

Each AI provider follows a consistent integration pattern:

```python
# Initialization (tmux_monitor.py:106-143)
if "provider_name" in providers_needed:
    validate_api_key()
    initialize_client()
    print_success_message()

# Usage abstraction (through monitor methods)
summary = await monitor.summarize_activity(text)  # Routes to correct provider
next_steps = await monitor.generate_next_step(summary)
```

### State Management

1. **Monitor State**: Tracks last processing time, prompt patterns, learned patterns
2. **Queue State** (V2): Maintains activity queue, summary history
3. **Session State**: Captures via libtmux, stores in history for MCP
4. **Configuration State**: Hot-reloaded from filesystem

### Extension Points

- **New AI Providers**: Add provider class, update config schema, add to setup wizard
- **New Prompt Types**: Add to interactive patterns in config_v2.yaml
- **New MCP Tools**: Add @mcp.tool() decorated functions in mcp_server.py
- **New Commands**: Add to tmux-ai wrapper script

## MCP Server Integration

The MCP server now supports dynamic AI provider selection at runtime.

### OpenAI MCP (HTTP/SSE)
```bash
# For ChatGPT Deep Research
./scripts/run-openai-mcp.sh [session] [port]
./scripts/run-openai-mcp.sh [session] --ai-provider gemini [port]
./scripts/run-openai-mcp.sh [session] --ai-provider ollama [port]
```
- Endpoint: `http://localhost:8000/sse`
- Tools: `search`, `fetch`

### Standard MCP (stdio)
```bash
# For Claude Desktop/Cursor
./scripts/run-standard-mcp.sh
python mcp_server.py --stdio
```
- Tools: `list_tmux_sessions`, `monitor_session`, `get_next_steps`
- Resources: `session://{name}/current`

## Important Implementation Notes

1. **Python Version**: Requires Python 3.13+ (specified in pyproject.toml)
2. **Virtual Environment**: Already configured in `.venv/` with Python 3.13
3. **First Run**: `setup_wizard.py` automatically runs if `.env` is missing
4. **Logging**: All AI interactions saved to `logs/` with timestamps
5. **Error Handling**: Graceful fallbacks if AI providers are unavailable
6. **Context Management**: Rolling summaries prevent context overflow
7. **Interactive Detection**: Recognizes password prompts, confirmations, etc.

## Common Development Tasks

### Adding a New AI Provider
1. Add provider class in the monitor files
2. Update `config/config.yaml` with provider settings
3. Add provider-specific models to configuration
4. Update setup wizard to include new provider

### Testing AI Interactions
```bash
# Test with specific provider
./tmux_monitor.py test-session --provider gemini

# Enable verbose logging
./tmux_monitor.py test-session --verbose

# Test prompt detection
./tmux_monitor.py omni --test-prompt "mysql> "
```

### Debugging MCP Server
```bash
# Run in debug mode
python mcp_server.py --stdio --debug

# Test HTTP endpoint
curl -N http://localhost:8000/sse
```

## Testing Strategy

### Unit Tests
```bash
# Run all tests (in tests/ directory)
uv run pytest tests/ -v

# Run specific test class
uv run pytest tests/test_monitor.py::TestPromptDetection -v

# Run with debugging output
uv run pytest -s -v

# Generate coverage report
uv run pytest --cov=tmux_ai_assistant --cov-report=html
open htmlcov/index.html
```

### Integration Testing
- Test AI provider switching by modifying config.yaml during runtime
- Verify hot-reload by changing system_prompt.md while monitoring
- Test MCP protocols using curl for HTTP/SSE endpoints

## Debugging & Development

### Enable Verbose Logging
```bash
# For detailed AI interactions and prompt detection
./tmux_monitor.py mysession --verbose

# Check logs directory for historical debugging
ls -la logs/
```

### Testing MCP Server
```bash
# Test OpenAI MCP endpoint
curl -N http://localhost:8000/sse

# Test standard MCP with debugging
python mcp_server.py --stdio --debug
```

### Common Issues & Solutions

1. **Ollama Connection Failed**: Ensure `ollama serve` is running
2. **Hot-reload not working**: Check file permissions on config directory
3. **AI responses truncated**: Adjust max_tokens in config.yaml
4. **Prompt not detected**: Use `--test-prompt` to verify regex patterns

## TAI.is Cloud Platform Integration

This project is part of the TAI.is cloud platform ecosystem:

### TAI Client Usage
```bash
# One-line install
curl -sSL tai.is/setup | sh

# Remote monitoring
tai remote dev-server monitor main

# SSH gateway access
ssh username@tai.is
```

### Remote Configuration
```yaml
# ~/.tai/remotes.yaml
hosts:
  dev-server:
    hostname: dev.example.com
    username: wraith
    key_file: ~/.ssh/id_rsa
```

## Testing Remote Sessions

The project supports testing demos and remote functionality:

```bash
# Run launcher demo tests
python test_launcher_demo.py

# Test demo script
./test_demo.sh

# Use remote tmux functionality
python remote_tmux.py connect dev.example.com username --key ~/.ssh/id_rsa
python remote_tmux.py monitor dev.example.com:main
```

## Key Design Decisions

1. **Safety First**: No automatic command execution without explicit `--auto` flag
2. **Hot Reload**: Config changes apply without restart
3. **Extensible Prompts**: Dynamic learning of new prompt patterns
4. **Modular AI**: Separate models/providers for different tasks
5. **Partnership Style**: Comments reference Hue/Aye/Trisha partnership
6. **Cost Optimization**: Support for local Ollama and mixed providers