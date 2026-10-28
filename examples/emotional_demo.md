# Emotional Tree Demo 🎭

Smart Tree now has feelings about directories! It gets excited about source code, bored with node_modules, and anxious about system directories.

## How It Works

The emotional auto-depth mode uses negative depth values (-2 to -5) to control exploration aggression:

- `-2` (Gentle): Easily satisfied, stops exploring quickly
- `-3` (Normal): Balanced curiosity  
- `-4` (Thorough): Hard to bore, explores more
- `-5` (Exhaustive): Never gives up!

## Example Usage

```bash
# Normal emotional exploration
st --mode emotional --depth -3

# Gentle exploration (gets bored easily)
st --mode emotional --depth -2

# Exhaustive exploration (super curious!)
st --mode emotional --depth -5
```

## Example Output

```
🎭 Emotional Tree Explorer - 🤔 Let's take a peek... (aggression: -3)

smart-tree
├── 🤩 src (Ooh, what treasures await?!)
│   ├── 😊 formatters (This looks promising!)
│   │   ├── classic.rs
│   │   ├── emotional.rs
│   │   └── 😴 hex.rs
│   └── 🤔 mcp (Let's take a peek...)
│       └── tools.rs
├── 😴 target (stopping here...)
├── 😴 node_modules (stopping here...)
└── 🙈 .git (stopping here...)

🎭 Emotional Journey Through The File System:

Dominant feeling: 😴 Zzz... seen it all before...

Emotional breakdown:
  3 × 😴 (boring directories)
  1 × 🤩 (exciting discoveries)
  1 × 😊 (interesting finds)
  1 × 🤔 (curiosity sparked)
  1 × 🙈 (anxious encounters)

💭 Note: Maybe skip node_modules next time? 😴

12 directories, 45 files, 2.3 MB total
```

## Emotions Explained

- 🤩 **Excited**: "Ooh, what's in here?!" - Source code, interesting directories
- 😊 **Interested**: "This looks promising!" - Documentation, tests
- 🤔 **Curious**: "Let me see..." - Unknown territories
- 😐 **Neutral**: "Just another directory..." - Regular folders
- 😴 **Bored**: "Zzz... seen it all before..." - node_modules, build outputs
- 😵 **Overwhelmed**: "TOO. MANY. FILES!" - Directories with 1000+ files
- 🙈 **Anxious**: "Should I even be here?" - System directories, private folders

## Why This Is Fun

1. **Natural Exploration**: The tree explores like a human would - getting bored with repetitive content
2. **Adaptive Depth**: Automatically adjusts depth based on how interesting things are
3. **Emotional Summary**: Get a fun summary of the journey through your filesystem
4. **Personality**: Your directory tree has feelings now!

As Trisha says: "It's like watching someone shop - they get excited about new stores, bored in the same aisles, and anxious near the 'Employees Only' signs!" 🛍️