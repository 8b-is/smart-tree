# Smart Tree v3.3.0 Release Notes 🌳

## Overview
This release brings real-world API integration, cross-platform compatibility improvements, and the groundwork for the 8t.is contextualizer system. Smart Tree is now more connected and intelligent than ever!

## 🌟 New Features

### Real Feedback API Integration
- **Live feedback submission** to f.8t.is for continuous improvement
- **Update notifications** with cached GitHub release information
- **Structured feedback system** for bug reports, feature requests, and tool suggestions
- All MCP feedback now goes to a real backend for analysis and prioritization

### Cross-Platform Improvements
- **Full Windows compatibility** for the ls formatter
- Platform-specific file permission handling
- Proper hard link count approximation on non-Unix systems
- Conditional compilation for OS-specific features

### 8t.is Integration (Preview)
- Added 8t.is contextualizer framework
- Feedback loop system for AI-driven improvements
- Continuous learning from user interactions
- Foundation for semantic context caching

## 🤖 AI Benefits

### For Claude and Other AI Assistants
- **Feedback API** enables AI assistants to submit improvement suggestions directly
- **Update checks** keep AI tools aware of latest features and improvements
- **Structured feedback format** with examples and expected outputs
- **Token-efficient** communication with compressed responses

### Token Savings
- Feedback responses are concise and structured
- Update checks cached server-side (1 hour TTL)
- No redundant API calls or data transfer

## 🔧 Technical Improvements

### API Endpoints
- `POST https://f.8t.is/api/feedback` - Submit feedback and feature requests
- `GET https://f.8t.is/api/smart-tree/latest` - Check for updates (cached)

### Code Quality
- Fixed all cross-platform compilation issues
- Cleaned up unused imports and warnings
- All 41 tests passing
- Release builds working on Linux, macOS, and Windows

### MCP Server Enhancements
- Feedback submission tool now functional
- Update check tool with graceful fallbacks
- Better error handling for API unavailability

## 📦 Installation

### Quick Install
```bash
curl -sSL https://raw.githubusercontent.com/8b-is/smart-tree/main/scripts/install.sh | bash
```

### Claude Desktop
Download the `.dxt` package from the release page.

### Manual
Download the appropriate binary for your platform from the release assets.

## 🙏 Acknowledgments

Thanks to everyone who's been testing Smart Tree and providing feedback! Special shoutout to the AI assistants who've been helping identify improvements through the MCP interface.

## What's Next?

- Full 8t.is contextualizer implementation
- .mem8 binary format support
- Enhanced semantic analysis
- More AI-friendly output formats

---

*Built with 💙 by the Smart Tree Team*
*Keep on rockin' with the smartest tree in the forest! 🎸*