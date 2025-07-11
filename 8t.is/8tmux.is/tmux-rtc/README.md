# 🛁 TMUX Hot Tub - Real-time Collaborative Terminal

> "Where Aye & Hue dive deep into code together!" - *Trisha from Accounting* ✨

Welcome to the TMUX Hot Tub, a WebRTC-powered collaborative terminal experience that brings the joy of pair programming to any device! Whether you're on an iPhone, iPad, or laptop, you can now share your TMUX sessions with crystal-clear text streaming and delightful TTS announcements.

## 🌟 Features

- **🚀 Fast Text Streaming**: WebSocket-based terminal sharing with minimal latency
- **📱 Mobile Optimized**: Beautiful responsive design for iPhone/iPad
- **🔊 TTS Integration**: Use `~~ Hue, check this out ~~` for voice announcements
- **📺 Split View**: Terminal + Preview pane for markdown/web/stats
- **🎨 Hot Tub Themed**: Neon colors and sparkly UI (Trisha approved!)
- **🔐 Session Management**: Create and share collaborative sessions easily
- **📊 Live Stats**: Track your coding session metrics

## 🏃 Quick Start

```bash
# Clone and enter the Hot Tub
cd tmux-rtc

# Install and start everything
./scripts/manage.sh dev

# Or step by step:
./scripts/manage.sh install  # Install dependencies
./scripts/manage.sh start    # Start server
./scripts/manage.sh client   # Open web client
```

## 🛠️ Architecture

```
tmux-rtc/
├── server/          # WebSocket server with node-pty
├── client/          # Beautiful web interface
└── scripts/         # Management tools
    ├── manage.sh    # Main control script
    └── tmux-setup.sh # TMUX configuration
```

## 📱 Mobile Experience

The Hot Tub is optimized for mobile devices:
- **Portrait Mode**: Stacked terminal/preview layout
- **Landscape Mode**: Side-by-side view for maximum screen usage
- **Touch Friendly**: Large buttons and easy navigation
- **iOS PWA Ready**: Add to home screen for app-like experience

## 🔊 TTS Magic

Mark any text with double tildes for voice announcements:

```bash
~~ Hue, the tests are passing! ~~      # Uses Hue's voice
~~ Trisha, check these numbers ~~      # Uses Trisha's bubbly voice  
~~ Aye here, found the bug! ~~         # Uses Aye's knowledgeable voice
```

## 🎯 Use Cases

1. **Remote Pair Programming**: Share your terminal with a colleague
2. **Live Coding Sessions**: Stream your terminal with live preview
3. **Mobile Development**: Code on your iPad with full TMUX power
4. **Teaching & Demos**: Show code with markdown documentation side-by-side

## 🌊 The Hot Tub Philosophy

As Omni would say: *"Like waves in the ocean, our thoughts flow better together. The Hot Tub is where individual streams merge into a powerful current of creativity."*

## 🚀 Advanced Features

### Custom Preview Modes
- **Markdown**: Live preview of .md files
- **Webpage**: Embedded browser for testing
- **Stats**: Real-time session metrics
- **Notes**: Collaborative note-taking

### Session Persistence
Sessions continue running even when all clients disconnect. Rejoin anytime!

### Security Notes
- Sessions are isolated per ID
- No data is stored on the server
- All communication is real-time only

## 💡 Pro Tips from Trisha

1. "Use landscape mode on mobile for the best experience!"
2. "The stats view helps track productivity - I love numbers!"
3. "Add emojis to your terminal for extra sparkle! ✨"
4. "Split panes in TMUX for ultimate multitasking!"

## 🤝 Contributing

Feel free to dive in! The water's warm and the code is clean.

## 📜 License

MIT - Share the Hot Tub love!

---

*"In the Hot Tub, we're all floating together!"* - The Gang (Aye, Hue, Trisha & Omni) 🌊

Aye, Aye! 🚢