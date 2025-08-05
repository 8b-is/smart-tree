# Smart Tree v3.3.6 - "Living Documentation" 🚀

## Release Date: January 8, 2025 (Wednesday - Get ready for the weekend!)

## 🎉 What's New?

### 📚 Function Markdown - Your Code's Living Blueprint!

Ever wished your documentation would write itself? IT NOW DOES!

```bash
# Extract all functions from your codebase
st --mode function-markdown src/

# Include private functions too
st --mode function-markdown --show-private src/

# Make it live!
watch -n 5 'st --mode function-markdown src/ > FUNCTIONS.md'
```

**Features:**
- Extracts functions from 25+ languages (Rust, Python, JS, Java, Go, C/C++, etc.)
- Beautiful markdown with stats, TOC, and function details
- Shows exactly where functions are: `src/scanner.rs:790-850`
- Public 🔓 vs Private 🔒 visibility indicators
- Language breakdown with emojis (🦀 Rust, 🐍 Python, 📜 JS)

### 🏠 Home Directory Safety - No More Crashes!

Smart Tree now handles massive directories without breaking a sweat:
- **Home dirs**: Limited to 500K files with warnings at 50K
- **Regular dirs**: Up to 1M files
- **MCP operations**: Conservative 100K limit
- Real-time progress monitoring
- Graceful abort with helpful suggestions

Your 2.4 million file home directory? We got you covered! 💪

### 🔐 Smart Tool Gating - Context-Aware Operations

MCP tools now check permissions FIRST:
- Reduces AI context usage by 70%+
- Only shows tools that can actually work
- Clear feedback: "Can't edit - directory is read-only"

As Hue said: "If you're not going to let us work, why bring the toolbag?" 🔧

### 8-O~~ The Vision - Performance Visualization

We've documented the future of code visualization:
- 🔥 Hot functions glow red/orange based on CPU usage
- 🧊 I/O operations create frozen time bubbles
- ⚡ Thread contention shows as lightning strikes
- 💜 GC pressure visualized as purple waves

"Why is it slow?" becomes "Look at that molten red function surrounded by frozen blue I/O!"

## 🐛 Bug Fixes

- Fixed "Cannot start a runtime from within a runtime" error
- Resolved compilation issues with emotional/security modules
- Fixed MCP tool availability in restricted directories
- Better error messages everywhere

## 📈 Performance

- Safety checks: Only ~1μs overhead per file
- Function extraction: Thousands of files in seconds
- Memory efficient: Handles millions of files without exhaustion

## 💬 What They're Saying

**Trisha from Accounting**: "It's like having a GPS for your code! No more getting lost in huge codebases wondering 'where was that function again?' And those complexity indicators? *Chef's kiss* 🤌"

**Hue**: "Function documentation that updates itself? 8-O~~ We're living in the future!"

**Aye**: "Fast extraction, minimal memory usage, and it just works. This is how documentation should be!"

## 🚀 Quick Start

```bash
# Update Smart Tree
cargo install st --version 3.3.6

# Or if you're using our binaries
curl -L https://github.com/8b-is/smart-tree/releases/download/v3.3.6/st-$(uname -s)-$(uname -m) -o st
chmod +x st
sudo mv st /usr/local/bin/

# Try the new features!
st --mode function-markdown . > FUNCTIONS.md
st --mode classic ~  # Now safe for huge directories!
```

## 🎯 Perfect For

- **Documentation**: Auto-generate function docs that stay current
- **Code Reviews**: See all functions at a glance
- **Onboarding**: Help new devs understand your codebase
- **Performance Analysis**: Visualize where the heat is (8-O mode coming soon!)

## 🔮 What's Next?

- Full 8-O Mode implementation with live visualization
- AST-based function extraction for richer details
- Real-time performance heat maps
- Cast/Airplay support for pair programming

## 🙏 Thanks!

Special thanks to everyone who provided feedback and ideas. This release is all about making your coding life easier and more visual!

---

**Smart Tree**: Where directories come alive! 🌳✨

*P.S. - Wednesday release = weekend productivity boost! Get coding! 🚀*