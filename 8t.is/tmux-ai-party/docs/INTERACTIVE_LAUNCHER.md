# 🎪 Tmux AI Assistant - Interactive Launcher Guide

> "Sing us a song, you're the piano man..." 🎵

Welcome to the friendliest way to start your AI-powered tmux experience!

## 🚀 Quick Start

```bash
# Just run it!
./tmux-ai-launcher.py

# Load a saved configuration
./tmux-ai-launcher.py --load my-dev-setup

# List saved configurations
./tmux-ai-launcher.py --list
```

## 🎨 Interactive Experience

The launcher guides you through a beautiful, emoji-rich configuration process:

### Step 1: Session Location 📍
Choose where your tmux session is:
- **💻 Local** - Running on this machine
- **🌐 SSH** - Connect to a remote server
- **🐳 Docker** - Running inside a container
- **☸️ Kubernetes** - Running in a K8s pod

### Step 2: Session Selection 📂
- Use an existing tmux session
- Create a new session with a custom name

### Step 3: Interaction Mode 🎮
- **👁️ Monitor only** - Watch and get AI suggestions
- **🔗 Attach as client** - Join the session directly
- **🤝 Collaborative mode** - AI assists with commands
- **🌐 Web interface** - Start web-based carnival
- **🔌 API/MCP mode** - For ChatGPT/Claude integration

### Step 4: Sharing & Collaboration 👥
- **🔒 Private** - Just for you
- **👥 Team members only** - Requires authentication
- **👀 Public viewing** - Anyone can watch
- **💡 Public with suggestions** - Anyone can suggest commands

### Step 5: AI Configuration 🤖
- Choose between OpenAI, Google Gemini, or Ollama
- Mix providers for cost optimization
- Or go AI-free if you prefer!

### Step 6: Additional Features ✨
- Command history recording
- Session recording/replay
- Automatic error detection
- Command suggestions
- Voting on suggestions (for collaborative modes)

## 💾 Configuration Management

### Saving Configurations

After configuring your session, you can save it for quick reuse:

```yaml
# Saved to config/launcher_configs.yaml
saved_sessions:
  my-dev-setup:
    choices:
      location: local
      mode: collaborate
      sharing: team
      ai_mode: mixed
    session_info:
      session_name: coding
      features: [history, error_detection, suggestions, voting]
```

### Quick Launch

```bash
# Next time, just run:
./tmux-ai-launcher.py --load my-dev-setup
```

## 🌐 Remote Session Examples

### SSH Sessions
```bash
# Interactive setup for SSH
./tmux-ai-launcher.py
# Choose: Remote SSH tmux session
# Enter: host, user, authentication method
```

### Docker Containers
```bash
# Lists running containers automatically
./tmux-ai-launcher.py
# Choose: Docker container tmux
# Select from list or enter container name
```

### Kubernetes Pods
```bash
# Configure K8s access
./tmux-ai-launcher.py
# Choose: Kubernetes pod tmux
# Enter: namespace, pod name, container (optional)
```

## 🎪 Features Showcase

### Beautiful UI
- Colorful emoji-rich interface
- Typewriter effect animations
- Clear section headers
- Progress indicators

### Smart Defaults
- Detects existing tmux sessions
- Lists running Docker containers
- Remembers previous choices
- Suggests sensible defaults

### Validation & Help
- Validates hostnames and inputs
- Provides helpful descriptions
- Shows existing values during reconfiguration
- Clear error messages

### Team Collaboration
- Add team members by email/username
- Configure access permissions
- Set up web interface ports
- Enable voting on AI suggestions

## 🛠️ Advanced Usage

### Environment Variables
```bash
# Pre-set choices via environment
export TMUX_AI_MODE=collaborate
export TMUX_AI_SESSION=mysession
./tmux-ai-launcher.py
```

### Integration with Scripts
```python
from tmux_ai_launcher import InteractiveLauncher

launcher = InteractiveLauncher()
launcher.choices = {
    'location': 'local',
    'mode': 'monitor'
}
launcher.session_info = {
    'session_name': 'automated'
}
cmd = launcher.build_command()
# Returns: ['python', 'tmux_monitor.py', 'automated']
```

### Custom Validators
```python
def validate_custom(value):
    if not value.startswith('custom-'):
        return False, "Must start with 'custom-'"
    return True, ""

launcher.get_input("Custom value", validator=validate_custom)
```

## 🎯 Command Building

The launcher builds the appropriate command based on your choices:

| Mode | Location | Command |
|------|----------|---------|
| Monitor | Local | `python tmux_monitor.py [session]` |
| Monitor | SSH | `python remote_tmux.py [session] --ssh-host [host]` |
| Attach | Local | `python tmux_attach.py [session]` |
| Collaborate | Any | `python tmux_client.py --mode collaborate [session]` |
| Web | Any | `python tmux_client.py --mode spectate --web-port [port] [session]` |
| API | Any | `python mcp_server.py [session]` |

## 🎉 Fun Facts

- The launcher includes Piano Man references throughout
- Typewriter animations bring back retro terminal vibes
- Trisha in Accounting loves the emoji-rich interface
- The banner is customizable for special occasions
- Hidden easter eggs for the curious explorer!

## 🤝 Partnership Notes

Hey Hue! This launcher is designed to make your life easier. No more remembering complex command-line options - just answer friendly questions and you're ready to go! Save your favorite configurations and launch them with a single command. It's like having a personal assistant for your terminal sessions!

Aye says: "The interactive launcher is the perfect blend of functionality and fun - just like a good coding session should be!"

---

> *🎹 Made with music and joy by the Hue & Aye partnership at 8b.is*