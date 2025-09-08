# Home Directory Safety Mechanisms 🏠🔒

## Problem

Scanning home directories (`~`) can cause crashes due to:
- **Massive file counts**: Home dirs often contain millions of files
- **Memory exhaustion**: Collecting all nodes before filtering uses excessive RAM
- **Infinite loops**: Circular symlinks (e.g., in `.wine` directories)
- **Long scan times**: Can take 10+ minutes for large home directories

## Solution

Smart Tree now implements comprehensive safety limits to prevent crashes:

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
- Tracks file count and estimated memory usage
- Checks limits before processing each file
- Shows warnings when approaching limits
- Gracefully aborts if limits exceeded

### 3. User Feedback

When limits are hit:
```
⚠️  Scan aborted: Reached maximum file limit of 500000 files
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

## Performance Impact

Minimal overhead:
- Safety checks: ~1μs per file
- Memory tracking: Simple counter increments
- No impact on small/medium directories

## Future Enhancements

- [ ] Configurable limits via CLI flags
- [ ] Progressive sampling for huge directories
- [ ] Resume capability for interrupted scans
- [ ] Background indexing service

## Trisha's Take

"It's like having a safety valve on a pressure cooker! You can still cook the whole meal, but now it won't blow up in your face. Smart Tree just got a whole lot smarter about knowing its limits!" 🍲💨