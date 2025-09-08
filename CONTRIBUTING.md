# Contributing to Smart Tree 🌳✨

Welcome, brave code warrior! You've stumbled upon our contributing guide, which means you're either:
1. A developer with brilliant ideas (Aye loves you already!)
2. Someone who found a bug (Hue is already investigating!)
3. Trisha from Accounting who got lost looking for the expense reports (wrong repo, Trish!)

Whatever brought you here, we're thrilled to have you! 🎉

## 🚀 Quick Start for Contributors

### The Three Commandments of Smart Tree
1. **Fast is better than slow** - If it takes longer than brewing coffee, it's too slow
2. **Pretty is better than ugly** - Code should sparkle like Trisha's spreadsheets
3. **Simple is better than complex** - If Hue can't understand it, it needs more comments

### Setting Up Your Development Environment

1. **Clone the repo** (you probably already did this, but Trisha insists we mention it):
   ```bash
   git clone https://github.com/8b-is/smart-tree.git
   cd smart-tree
   ```

2. **Build the project** (grab a coffee, this might take a minute):
   ```bash
   cargo build
   ```

3. **Run the tests** (make sure you didn't break anything that Aye worked hard on):
   ```bash
   cargo test
   ```

4. **Try it out**:
   ```bash
   cargo run -- --help
   ```

5. **Install locally** (recommended for development):
   ```bash
   ./scripts/build-and-install.sh
   ```

### 🔧 Development Tips

**Version Command Hanging?** If `st --version` hangs after rebuilding, your shell has cached the old binary location. Solutions:

**Quick fix:**
```bash
hash -r && st --version
```

**Automatic solution:** Use our build script:
```bash
./scripts/build-and-install.sh
```

**Add to your shell:** Source our helper functions for convenient development:
```bash
# Add to ~/.zshrc or ~/.bashrc
source /path/to/smart-tree/scripts/shell-functions.sh

# Then use these commands:
st-rebuild   # Build and install with cache clearing
st-test      # Test local build without installing  
st-refresh   # Clear cache and test version
st-versions  # Check both installed and local versions
```

## 🐛 Found a Bug?

Don't panic! Bugs happen to the best of us. Even Aye occasionally writes code that doesn't work on the first try (shocking, we know).

1. **Check if it's already reported** - Someone else might have beaten you to it
2. **Use our bug report template** - It's funnier than your average template
3. **Be descriptive** - "It doesn't work" is about as helpful as Trisha's diet tips
4. **Include your environment** - OS, Rust version, whether you're using it during a full moon, etc.

## 💡 Have a Feature Idea?

Brilliant! We love new ideas almost as much as Hue loves optimizing directory traversal algorithms.

1. **Use our feature request template** - It's got jokes AND structure
2. **Explain the why** - Why would this make Smart Tree even smarter?
3. **Consider alternatives** - Show us you've thought it through
4. **Be patient** - Good features take time, like aging cheese or Trisha's quarterly reports

## 🔧 Development Guidelines

### Code Style
- **Rust formatting**: Use `cargo fmt` - consistent code makes Aye happy
- **Comments**: Write them like you're explaining to future you at 3 AM
- **Error handling**: Use `Result<T>` and `anyhow` - panics are for parties, not production
- **Tests**: Write them! Even Hue's perfect code needs tests

### Commit Messages
Follow the format: `type: short description`

Types we recognize:
- `feat`: New feature (Aye gets excited)
- `fix`: Bug fix (Hue breathes a sigh of relief)
- `docs`: Documentation (Trisha actually reads these)
- `style`: Code style changes (make it pretty!)
- `refactor`: Code improvements (Hue's favorite)
- `test`: Adding tests (responsible development!)
- `chore`: Maintenance tasks (someone has to do it)

Example: `feat: add quantum compression for directory trees`

### Pull Request Process

1. **Fork the repo** (it's like borrowing, but for code)
2. **Create a branch** with a descriptive name: `feature/awesome-new-thing`
3. **Make your changes** (the fun part!)
4. **Write tests** (the responsible part)
5. **Update documentation** if needed (Trisha will thank you)
6. **Submit a PR** with a clear description
7. **Be patient** - we'll review it faster than Trisha processes expense reports

### What Makes a Good PR?

- **Clear description** - What does it do? Why does it matter?
- **Tests included** - Show us it works!
- **Documentation updated** - If you added features, document them
- **Single responsibility** - One thing at a time, please
- **Backwards compatible** - Don't break existing users' workflows

## 🎯 Areas Where We Need Help

- **Performance optimizations** (Hue's specialty, but all hands welcome!)
- **New output formats** (creativity encouraged!)
- **Documentation improvements** (Trisha's domain, but she welcomes help)
- **Platform-specific testing** (Windows, Linux, macOS, BSD, Commodore 64...)
- **Integration guides** (AI tools, editors, CI/CD systems)

## 📚 Architecture Overview

Smart Tree is structured like a well-organized filing cabinet (the kind Trisha dreams about):

- **Scanner**: The explorer (finds files, respects .gitignore, doesn't judge your file naming)
- **Formatters**: The artists (turn raw data into beautiful output)
- **Config**: The organizer (handles settings and preferences)
- **MCP**: The diplomat (talks to AI tools)

## 🧪 Testing Philosophy

We believe in testing like Trisha believes in properly organized spreadsheets:

- **Unit tests** for individual functions
- **Integration tests** for formatters and scanners
- **Performance tests** for large directories
- **Real-world tests** on actual codebases

Run tests with:
```bash
cargo test
cargo test --release  # For performance tests
```

## 📖 Documentation

Good documentation is like a good joke - if you have to explain it, it's not good enough. But here are our standards:

- **README**: Keep it current (first impressions matter!)
- **Code comments**: Explain the "why", not just the "what"
- **Examples**: Show don't tell
- **Changelog**: Document breaking changes (users will thank you)

## 🎉 Recognition

Contributors get:
- **Eternal gratitude** from Aye, Hue, and Trisha
- **Mention in the changelog** (internet fame!)
- **Warm fuzzy feelings** from helping the community
- **The satisfaction** of making directory trees more beautiful

## 📞 Getting Help

Stuck? Don't suffer in silence!

- **GitHub Issues**: For bugs and features
- **GitHub Discussions**: For questions and ideas
- **Email**: For sensitive matters (or if you want to send Trisha cookies)

## 🙏 Thank You!

Every contribution, from typo fixes to major features, makes Smart Tree better. You're the reason we can keep making directory visualization awesome.

Remember: We're building something that developers use every day. Your code could be the difference between someone having a good day or spending an hour trying to understand a directory structure.

Now go forth and make some beautiful code! 🚀

---

*"Contributing to open source is like tending a garden - with enough care and attention, beautiful things grow!" - Trisha from Accounting (who somehow became philosophical)*

**Happy coding!** 🌳✨ 