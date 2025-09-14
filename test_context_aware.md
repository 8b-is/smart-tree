# Context-Aware .m8 Loading Examples

## Scenario 1: General Browsing (No Context)
**You:** "What projects do we have?"
**Smart Tree loads:** Minimal - just essences
```
• Smart Tree - AI-optimized directory visualization
• 8b.is website - Company portal
• MEM8 - Wave-based memory system
```
**Tokens used:** ~50

## Scenario 2: Talking About Websites
**You:** "How's the 8b.is website coming along?"
**Smart Tree detects:** "website" keyword
**Auto-expands:** 8b.is/.m8 to medium detail
```
📂 8b.is (88.8Hz)
  8b.is website - Company portal for 8-bit inspired AI services
  Keywords: 8b.is, website, portal
  📁 Children:
    • frontend (92.3Hz)
    • api (87.5Hz)
    • docs (45.2Hz)
```
**Tokens used:** ~150

## Scenario 3: Specific Technical Discussion
**You:** "The 8b.is website API needs authentication"
**Smart Tree detects:** "8b.is", "website", "API" - HIGH RELEVANCE!
**Auto-expands:** Full detail + drills into api/
```
╭──────────────────────────────────────────────────
│ 📂 8b.is
│ 🌊 Frequency: 88.8Hz
│ 📝 8b.is website - Company portal for 8-bit inspired AI services
│ 🏷️ Keywords: 8b.is, website, portal
│ 📁 Children:
│   • frontend (92.3Hz)
│   • api (87.5Hz) ← AUTO-EXPANDING THIS
│   • docs (45.2Hz)
│ 🎯 Context Triggers:
│   website → frontend/
│   API → api/ ← TRIGGERED!
╰──────────────────────────────────────────────────

📂 8b.is/api/.m8 (87.5Hz)
  RESTful API with wave-based authentication
  Endpoints: /auth, /memories, /waves
  Port: 8420
  Auth: JWT with MEM8 signature
```
**Tokens used:** ~400 (but ONLY when needed!)

## Scenario 4: Smart Tree Development
**You:** "The tokenizer in Smart Tree needs work"
**Smart Tree detects:** "tokenizer" trigger
**Auto-loads:** src/tokenizer.rs context
```
📍 Drilling down: src/tokenizer.rs
  Tokenization system - 90% compression
  Patterns: node_modules→0x80, .rs→0x91
  Methods: tokenize(), decode(), compression_ratio()
```

## The Magic: Progressive Loading

### Without Context-Aware Loading:
- Load everything: 10,000+ tokens
- Most irrelevant to current discussion
- Context window fills quickly

### With Context-Aware .m8:
- Base load: 50 tokens (just essences)
- Relevant expansion: +100-400 tokens
- Total: 150-450 tokens (95% reduction!)

## Real Example:

**Starting context (50 tokens):**
```
Projects: smart-tree, 8b.is, mem8, marqant
```

**You mention "memory":**
```
+100 tokens: MEM8 expanded with wave frequencies
```

**You mention "binary format":**
```
+150 tokens: .m8 format specs loaded
```

**You mention "tokenization":**
```
+200 tokens: Smart Tree tokenizer details
```

**Total:** 500 tokens vs 10,000 tokens without context awareness!

## Implementation in Smart Tree:

```rust
// In scanner.rs when encountering .m8 files
if path.ends_with(".m8") {
    let keywords = extract_conversation_context();
    let content = context_reader.load_contextual(&path, &keywords)?;

    // Only expand if relevance > threshold
    if relevance > 0.7 {
        // Drill down automatically
        expand_children(&path, &keywords)?;
    }
}
```

## Frequency-Based Relevance:

- **High frequency (>150Hz):** Hot zones, recent work
- **Medium (50-150Hz):** Active projects
- **Low (<50Hz):** Documentation, archives

The .m8 frequency helps determine expansion priority!