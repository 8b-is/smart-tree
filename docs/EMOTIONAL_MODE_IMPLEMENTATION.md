# Emotional Auto-Depth Mode Implementation Plan 🎭

## What We've Built So Far

1. **Core Emotional System** (`src/emotional_depth.rs`)
   - Emotion enum with 7 states (Excited, Interested, Curious, Neutral, Bored, Overwhelmed, Anxious)
   - EmotionalDepthAnalyzer that tracks patterns and repetition
   - Depth modifiers based on emotions (-3 to +2 depth adjustment)
   - Emotional journey summary

2. **Formatter Skeleton** (`src/formatters/emotional.rs`)
   - EmotionalFormatter that integrates with the analyzer
   - Shows emotions as emojis next to directories
   - Provides journey summary at the end

3. **Documentation** (`examples/emotional_demo.md`)
   - Usage examples
   - Emotion explanations
   - Fun output examples

## What Still Needs Implementation

### 1. CLI Integration
```rust
// In main.rs, need to:
// - Change depth from usize to i32 to allow negative values
// - Add logic to detect emotional mode when depth is negative
// - Pass aggression level to formatter
```

### 2. Scanner Integration
```rust
// In scanner.rs, need to:
// - Add optional emotional analyzer to Scanner
// - Implement dynamic depth calculation during traversal
// - Pass directory contents to analyzer for emotion calculation
```

### 3. Formatter Connection
```rust
// In main.rs formatter selection:
OutputMode::Emotional => {
    Box::new(EmotionalFormatter::new(
        args.depth, // aggression level
        args.path_mode,
        use_color
    ))
}
```

### 4. Mode Selection Logic
When depth is negative (-2 to -5), automatically switch to emotional mode:
```rust
let mode = if args.depth < 0 {
    OutputMode::Emotional
} else {
    args.mode
};
```

## Fun Implementation Details

### Emotion Triggers
- **Excited**: src/, lib/, core/ directories
- **Interested**: docs/, tests/, examples/
- **Bored**: node_modules/, .git/, target/, build/, dist/, cache/, vendor/
- **Anxious**: Windows/, System32/, private/, secret/
- **Overwhelmed**: Any directory with 1000+ files

### Dynamic Depth Calculation
```
effective_depth = current_depth + base_aggression + emotion_modifier

Where:
- base_aggression: -2 to -5 (from CLI)
- emotion_modifier: -3 to +2 (from emotion)
```

### Repetition Tracking
The system tracks patterns like "f10_d5" (10-19 files, 5-9 dirs) and gets bored after seeing the same pattern 5+ times.

## Example Journey

```
User: st --depth -3

Tree: "Let me explore with normal curiosity..."
  /project
    /src 🤩 "Ooh, source code!"         [goes deeper]
    /docs 😊 "Documentation, nice!"      [explores]
    /node_modules 😴 "Ugh, boring..."    [stops]
    /build 😴 "More build stuff..."      [stops]
    /.secret 🙈 "Should I be here?"      [backs away]
```

## Why This Is Awesome

1. **Human-like exploration**: Gets bored with repetitive content
2. **Adaptive**: Automatically adjusts based on content
3. **Entertaining**: Makes directory exploration fun!
4. **Educational**: Shows which directories are "interesting"

## Next Steps

1. Modify CLI to accept negative depth values
2. Create scanner hooks for emotional analysis
3. Wire up the formatter
4. Test with various directory structures
5. Add more personality with random comments

As Hue said: "Every directory should have an emotional feeling... like when I went into c:\windows... Ok... This is getting boring now... ;)"

This perfectly captures the human experience of filesystem exploration! 🎭