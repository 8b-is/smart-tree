# Server-Sent Events (SSE) Support in Smart Tree

Smart Tree now supports Server-Sent Events (SSE) for real-time directory monitoring and streaming analysis results. This enables continuous monitoring of file system changes and live updates to connected clients.

## Features

- 🔄 Real-time file system monitoring
- 📊 Streaming directory analysis
- 💓 Automatic heartbeat to keep connections alive
- 📈 Periodic statistics updates
- 🎯 Multiple output formats (hex, ai, quantum, json)
- 🔍 Pattern-based filtering

## SSE Event Types

### 1. `scan_complete`
Fired when initial directory scan is finished.
```json
{
  "type": "scan_complete",
  "path": "/path/to/directory",
  "stats": {
    "total_files": 1234,
    "total_dirs": 56,
    "total_size": 78901234,
    "scan_time_ms": 1500
  }
}
```

### 2. `created`
Fired when a new file or directory is created.
```json
{
  "type": "created",
  "path": "/path/to/new/file.txt",
  "node": {
    "path": "/path/to/new/file.txt",
    "is_dir": false,
    "size": 1024,
    "permissions": 644
  }
}
```

### 3. `modified`
Fired when a file or directory is modified.
```json
{
  "type": "modified",
  "path": "/path/to/modified/file.txt",
  "node": { /* FileNode details */ }
}
```

### 4. `deleted`
Fired when a file or directory is deleted.
```json
{
  "type": "deleted",
  "path": "/path/to/deleted/file.txt"
}
```

### 5. `analysis`
Periodic analysis updates in the specified format.
```json
{
  "type": "analysis",
  "path": "/path/to/directory",
  "format": "ai",
  "data": "/* Formatted output */"
}
```

### 6. `stats`
Periodic statistics updates.
```json
{
  "type": "stats",
  "path": "/path/to/directory",
  "stats": {
    "total_files": 1250,
    "total_dirs": 58,
    "total_size": 79123456,
    "scan_time_ms": 500
  }
}
```

### 7. `heartbeat`
Keep-alive signal sent periodically.
```json
{
  "type": "heartbeat"
}
```

## Using SSE with MCP

### 1. Configure SSE Watch

First, use the MCP tool to configure directory watching:

```javascript
// Using MCP client
const result = await mcp.callTool('watch_directory_sse', {
  path: '/path/to/watch',
  format: 'ai',
  heartbeat_interval: 30,
  stats_interval: 60,
  include_content: false,
  max_depth: 5,
  include_patterns: ['*.rs', '*.toml'],
  exclude_patterns: ['target/*', '*.log']
});
```

### 2. Connect to SSE Stream

```javascript
// Browser example
const source = new EventSource('/mcp/sse/watch');

source.addEventListener('scan_complete', (e) => {
  const data = JSON.parse(e.data);
  console.log('Initial scan complete:', data.stats);
});

source.addEventListener('created', (e) => {
  const data = JSON.parse(e.data);
  console.log('New file created:', data.path);
});

source.addEventListener('modified', (e) => {
  const data = JSON.parse(e.data);
  console.log('File modified:', data.path);
});

source.addEventListener('error', (e) => {
  console.error('SSE error:', e);
});
```

### 3. Node.js Example

```javascript
const EventSource = require('eventsource');

const source = new EventSource('http://localhost:8080/mcp/sse/watch');

source.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Event:', data.type, data);
};

source.onerror = (error) => {
  console.error('SSE error:', error);
};
```

## SSE Formatter

Smart Tree includes a dedicated SSE formatter that can be used for streaming output:

```bash
# Stream directory changes as SSE events
st --stream --mode sse /path/to/directory

# Output format:
# id: 1
# event: scan
# data: {"type":"scan_complete","path":"/path","stats":{...}}
#
# id: 2
# event: node
# data: {"type":"node","node":{"name":"file.txt",...}}
```

## Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `path` | string | required | Directory path to watch |
| `format` | enum | "ai" | Output format: hex, ai, quantum, json, summary |
| `heartbeat_interval` | integer | 30 | Heartbeat interval in seconds |
| `stats_interval` | integer | 60 | Statistics update interval in seconds |
| `include_content` | boolean | false | Include file contents in events |
| `max_depth` | integer | null | Maximum directory depth to watch |
| `include_patterns` | array | [] | File patterns to include |
| `exclude_patterns` | array | [] | File patterns to exclude |

## Performance Considerations

1. **File Watcher Limits**: System file watcher limits may restrict the number of files that can be monitored
2. **Network Bandwidth**: Frequent updates can consume significant bandwidth
3. **Memory Usage**: Large directories may require more memory for tracking changes
4. **Compression**: Consider using compression for large analysis outputs

## Security

- Path validation ensures only allowed directories can be watched
- Blocked paths (e.g., /etc, /sys, /proc) cannot be monitored
- Authentication should be implemented at the application level

## Troubleshooting

### Connection Drops
- Check heartbeat interval settings
- Verify network proxy configurations
- Monitor for rate limiting

### Missing Events
- Check file watcher limits: `sysctl fs.inotify.max_user_watches`
- Verify include/exclude patterns
- Check max_depth setting

### High CPU Usage
- Reduce stats_interval for less frequent updates
- Use exclude_patterns to skip large directories
- Consider max_depth to limit recursion