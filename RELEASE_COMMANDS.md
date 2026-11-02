# 🚀 Smart Tree v3.3.5 Release Commands

## Build & Test
```bash
# Run full test suite
cargo test

# Build release version
cargo build --release

# Run integration tests
./scripts/manage.sh test
```

## Create GitHub Release
```bash
# Tag the release
git tag -a v3.3.5 -m "Smart Tree v3.3.5: Quantum Awakening"

# Push to GitHub
git push origin main --tags

# Create release with manage.sh
./scripts/manage.sh release v3.3.5 "Quantum Awakening - Semantic compression meets code understanding!"
```

## Demo Commands for Release Notes
```bash
# Show the evolution
st --mode classic        # Traditional tree
st --mode quantum        # 90% compression
st --mode quantum-semantic  # 95% compression with meaning!

# Relations visualization
st --mode relations src/

# Content-aware (coming soon)
st  # Auto-detects human vs AI mode
```

## Publishing to crates.io
```bash
# Ensure all metadata is correct
cargo publish --dry-run

# Publish to crates.io
cargo publish
```

## Announcement Template
```
🎉 Smart Tree v3.3.5: Quantum Awakening is here!

🧬 Semantic compression that understands your code
🔗 Relations mode for visualizing dependencies  
🤖 AI-optimized summaries
💥 95% compression with 100% meaning

Get it now:
cargo install smart-tree

Details: https://github.com/8b-is/smart-tree/releases/tag/3.3.5
```