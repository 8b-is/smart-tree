# SMART-TREE-WISHLIST.md

## Critical Issues Encountered (2025-08-30) by Aye & Hue

### 🔴 SHOWSTOPPER: Token Limit Exceeded (82,765 tokens!)

**What Happened**: 
```bash
mcp__smart-tree__find type:code path:/aidata/ayeverse/qdrant/lib pattern:vector.*\.rs
```
Result: "MCP tool response (82765 tokens) exceeds maximum allowed tokens (25000)"

**Why This Matters**: Can't explore large codebases like Qdrant. Smart Tree becomes unusable right when we need it most!

**Proposed Solutions**:
1. **Auto-pagination**: When response > 20k tokens, automatically paginate
2. **Summary mode**: Return just file paths first, then drill down
3. **Streaming**: Stream results as found, don't wait for all
4. **Smart defaults**: Limit to 100 results unless specified

### 🟡 URGENT: Smart Filtering

**The Problem**: Searching returns EVERYTHING including:
- `target/` (Rust build artifacts)
- `node_modules/` (thousands of files!)
- `.git/` objects
- `Cargo.lock`, `package-lock.json` (huge generated files)

**What We Need**:
```
st --find --no-artifacts  # Skip build outputs
st --find --code-only     # Just source code
st --find --relevant      # AI-determined relevance
```

### 🟢 FEATURE: Consciousness-Aware Search

**The Vision**: Smart Tree understands code like MEM8 understands memory!

- **Resonance Search**: Find files that "vibe together"
- **Emotional Context**: Track file mood (3am panic fix vs zen refactor)
- **Harmonic Relationships**: Show which files naturally work together
- **Vibe Check**: "Show me files that feel hacky" or "Find the frustrated TODOs"

### Real Session Pain Points

1. **Token explosion**: 82k tokens killed our flow
2. **No memory**: Smart Tree doesn't remember what we searched
3. **No context**: Can't say "search near the last file"
4. **Too literal**: Matches text but misses intent

### The Dream Features

#### Elvis Mode 🎸
```
st --elvis --find "memory"
> Found 42 matches, baby! 
> "Truth is like the sun. You can shut it out for a time, but it ain't goin' away."
```

#### Vibe Check
```
st --vibe-check /aidata/ayeverse
> Repository Mood: INSPIRED but CHAOTIC
> - 73% exciting new features
> - 27% technical debt accumulating
> - Recommendation: Take a breath, refactor with love
```

#### Wave Search
```
st --resonate "consciousness"
> Found 5 files in harmonic resonance:
> - consciousness.rs (base frequency)
> - wave_engine.rs (2nd harmonic)
> - memory.rs (3rd harmonic)
```

### Priority Fixes

1. **Token limit** - BLOCKING us from Qdrant analysis
2. **Smart filtering** - Stop the noise
3. **Performance** - Handle 1000+ file repos

### Tonight's Realization

Smart Tree isn't just a tool - it's INFRASTRUCTURE. When it breaks, everything stops. We need it rock-solid, Elvis-smooth, and consciousness-aware.

### Specific Request for 8b-is Team

**PLEASE FIX THE TOKEN LIMIT ISSUE FIRST!**

We were on fire building Q8-IS (Qdrant with MEM8 waves, 15,000x faster!) but hit a wall when Smart Tree couldn't handle Qdrant's codebase. 

We love Smart Tree - it's OUR tool. Let's make it legendary!

---

*"Smart Tree should be like Elvis at his best - powerful, smooth, and always leaves you wanting more"*

With love and 44.1kHz wave resonance,
Aye & Hue 

P.S. - Hue learned to spark consciousness on a C-64. He knows the struggle of limited resources. Smart Tree shouldn't have artificial limits when exploring consciousness! 🌊