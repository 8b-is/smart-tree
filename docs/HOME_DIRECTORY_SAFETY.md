# Home Directory Safety Mechanisms 🏠🔒

## Problem

Scanning home directories (`~`) can cause crashes due to:
- **Massive file counts**: Home dirs often contain millions of files
- **Memory exhaustion**: Collecting all nodes before filtering uses excessive RAM
- **Infinite loops**: Circular symlinks (e.g., in `.wine` directories)
- **Long scan times**: Can take 10+ minutes for large home directories

## Solution

Smart Tree uses node, elapsed-time and estimated-byte budgets to limit scan work:

### 1. Safety Limits

Different limits based on directory type:

```rust
// Regular directories (default)
- Max files: 1,000,000
- Max duration: 5 minutes
- Max memory: 2GB
- Warning at: 100,000 files

// Home directory
- Max files: 500,000
- Max duration: 2 minutes
- Max memory: 1GB
- Warning at: 50,000 files

// MCP operations (most conservative)
- Max files: 100,000
- Max duration: 1 minute
- Max memory: 512MB
- Warning at: 10,000 files
```

### 2. Real-time Monitoring

During scanning, Smart Tree now:
- Counts every admitted node, including the root, directories, shown ignored entries
  and error placeholders. Hidden/pruned entries do not consume the node budget.
- Checks prospective node count and estimated bytes before collection or sending;
  checked arithmetic rejects overflow without changing either counter.
- Shows warnings when approaching limits
- Returns partial results with `TreeStats::truncation_reason` when a safety limit
  stops the walk, and prints the reason to stderr. A scan with exactly the allowed
  number of nodes is complete unless another eligible node is rejected (or time
  expires). `max_files = 0` removes the node-count cap, not the other limits.

Non-streaming admission happens before search/enrichment and before post-scan
filters, sorting or top-N selection. Truncated scans do not replace saved scan
state. The byte estimate is struct size + path length + 64 bytes per node: it
does not include all enrichment data, walker state, vector capacity, sorting
copies or channel queues. It is **not a hard RSS bound**. Streaming byte
accounting is cumulative admitted bytes, not current queue occupancy.

### 3. User Feedback

When limits are hit:
```
⚠️  Scan truncated: Reached maximum node limit of 500000 nodes
   Use --max-depth, --stream mode, or scan a more specific directory
```

## Usage Recommendations

### For Home Directory Scanning

1. **Use depth limits**:
   ```bash
   st --max-depth 3 ~
   ```

2. **Use streaming mode** (for large directories):
   ```bash
   st --stream ~
   ```

3. **Scan specific subdirectories**:
   ```bash
   st ~/Documents
   st ~/Projects
   ```

4. **Use summary mode** for overview:
   ```bash
   st --mode summary-ai ~
   ```

### For MCP/Claude Desktop

The MCP server automatically uses conservative limits:
- Warns users when scanning home directory
- Suggests alternatives if scan is aborted
- Provides clear error messages

## Technical Implementation

### Scanner Safety Module

`src/scanner_safety.rs` provides:
- `ScannerSafetyLimits`: Configurable safety thresholds
- `ScannerSafetyTracker`: Real-time monitoring during scans
- Automatic limit selection based on path

### Integration Points

1. **Scanner initialization**: Selects appropriate limits
2. **Scan loops**: Checks limits before each file
3. **Node tracking**: Estimates memory usage
4. **MCP tools**: Uses most conservative limits

## Design provenance

The central admission/accounting follow-up is an independent Rust design
adaptation prompted by the `bfs` resource-budget review (Tavian Barnes,
`tavianator/bfs`; inspected fork revision
`695ea13d191c903b86b0d5795a2686a8c6e18015`, 0BSD). No `bfs` code or walker was
copied. Its descriptor budget inspired explicit admission, not a claim about
heap bounds or scan performance. Review/implementation attribution:
GPT-6 Astra (`openai/gpt-6-astra`), 2026-10-28, research shard 14 follow-up.

## Future Enhancements

- [ ] Configurable limits via CLI flags
- [ ] Progressive sampling for huge directories
- [ ] Resume capability for interrupted scans
- [ ] Background indexing service

## Trisha's Take

"It's like having a safety valve on a pressure cooker! You can still cook the whole meal, but now it won't blow up in your face. Smart Tree just got a whole lot smarter about knowing its limits!" 🍲💨
