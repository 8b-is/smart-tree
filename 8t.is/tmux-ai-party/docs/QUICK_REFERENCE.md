# 🚀 Tmux AI Assistant - Quick Reference Guide

## 🎪 Getting Started (The Fun Way!)

```bash
# Interactive launcher - the coding carnival!
./tmux-ai-launcher.py
```

## 📋 Quick Commands

### Setup & Configuration
```bash
# Interactive setup wizard
python setup_wizard.py

# Reconfigure with existing values shown
python setup_wizard.py --reconfigure

# Check system status
./tmux-ai status
```

### Monitoring Sessions
```bash
# Interactive launcher (recommended!)
./tmux-ai-launcher.py

# Monitor with unified CLI
./tmux-ai monitor              # Default session
./tmux-ai monitor mysession    # Specific session
./tmux-ai monitor --auto       # Automation enabled

# Classic monitoring
./tmux_monitor.py mysession
./tmux_monitor.py mysession --verbose
./tmux_monitor.py mysession --add-prompt 'mysql> $'
```

### Prompt Management
```bash
# List all prompt patterns
./tmux_monitor.py session --list-prompts

# Test if a line matches prompts
./tmux_monitor.py session --test-prompt "~/code > "

# Add custom prompt patterns (regex supported!)
./tmux_monitor.py session --add-prompt '➜\s+.*\$'
./tmux_monitor.py session --add-prompt 'custom:\d+> $'
```

### Client Attachment
```bash
# Simple attachment (like tmux attach)
./scripts/attach-client.sh mysession

# Advanced client with modes
./scripts/attach-client.sh mysession client

# Web interface
./scripts/attach-client.sh mysession web
# Then open http://localhost:8080
```

### MCP Server
```bash
# For ChatGPT/OpenAI
./scripts/run-openai-mcp.sh mysession 8000

# With different AI providers
./scripts/run-openai-mcp.sh mysession --ai-provider gemini
./scripts/run-openai-mcp.sh mysession --ai-provider ollama

# For Claude Desktop
./scripts/run-standard-mcp.sh
```

### Continuous Monitoring
```bash
# Start continuous monitor
./scripts/run-continuous-monitor.sh mysession

# With automation (use carefully!)
./scripts/run-continuous-monitor.sh mysession --auto
```

## ⚙️ Configuration Files

### Main Configuration
```yaml
# config/config.yaml
providers:
  summarization: gemini    # Cost-effective
  next_step: openai       # Powerful

openai_api_key: ${OPENAI_API_KEY}
gemini_api_key: ${GEMINI_API_KEY}

prompt_learning: true     # Enable dynamic learning
check_interval: 5
max_history_lines: 100
```

### Learned Prompts
```yaml
# config/learned_prompts.yaml
# Automatically populated!
learned_patterns:
  - pattern: "~/projects/\\w+\\s*[>$]\\s*"
    regex: true
    count: 15
```

### Launcher Configurations
```yaml
# config/launcher_configs.yaml
saved_sessions:
  my-dev-setup:
    choices:
      location: local
      mode: collaborate
    session_info:
      session_name: coding
```

## 🎮 Interactive Launcher Options

### Session Locations
- **Local**: Running on this machine
- **SSH**: Remote server connection
- **Docker**: Container tmux
- **Kubernetes**: K8s pod tmux

### Interaction Modes
- **Monitor**: Watch and get AI suggestions
- **Attach**: Join as tmux client
- **Collaborate**: AI assists with commands
- **Web**: Browser-based interface
- **API/MCP**: For AI tool integration

### Sharing Options
- **Private**: Just for you
- **Team**: Authenticated users only
- **Public View**: Anyone can watch
- **Public Suggest**: Anyone can contribute

## 🔑 Environment Variables

```bash
# .env file
OPENAI_API_KEY=sk-...
GEMINI_API_KEY=...
OLLAMA_HOST=http://localhost:11434

# Optional
DEFAULT_SESSION=mysession
AUTOMATION_ENABLED=false
PROMPT_LEARNING=true
```

## 🎯 Common Workflows

### First Time Setup
1. Run `./tmux-ai-launcher.py`
2. Choose local session
3. Create new session
4. Select monitor mode
5. Configure AI (or use existing)
6. Save configuration
7. Launch!

### Daily Development
```bash
# Quick launch saved config
./tmux-ai-launcher.py --load daily-dev

# Or just monitor default session
./tmux-ai monitor
```

### Team Collaboration
1. Run launcher
2. Choose collaborative mode
3. Configure team members
4. Enable web interface
5. Share URL with team

### Remote Monitoring
1. Run launcher
2. Choose SSH location
3. Enter server details
4. Select monitoring mode
5. Save for quick access

## 🚦 Status Indicators

### Monitor Output
- 🤖 AI processing
- ✨ New suggestion available
- 🔄 Configuration reloaded
- ⚠️ Error or warning
- ✅ Success

### Prompt Detection
- ✓ Line matches prompt
- ✗ No prompt match
- 🎯 New pattern learned

## 💡 Pro Tips

1. **Save Time**: Use the launcher to save configurations for quick reuse
2. **Mix AIs**: Use Gemini for processing, OpenAI for suggestions
3. **Learn Patterns**: Enable prompt learning for automatic detection
4. **Use Verbose**: Add `--verbose` for detailed logs in `logs/` folder
5. **Hot Reload**: Edit configs while running - changes apply instantly
6. **Test First**: Use `--test-prompt` to verify pattern matching
7. **Budget Mode**: Use Ollama for free local AI processing

## 🎹 Easter Eggs

- The launcher includes Piano Man references
- Trisha loves the emoji-rich interfaces
- Hidden animations in the launcher
- Special messages on certain dates
- Try different session names for surprises!

---

> *🎪 Quick reference for the coding carnival! Made with joy by Hue & Aye at 8b.is*