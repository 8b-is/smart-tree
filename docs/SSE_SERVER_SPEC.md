# Smart Tree SSE Server Specification

> "Real-time directory streaming at quantum speeds!" 🌊⚡

## Overview

Smart Tree SSE server mode provides real-time directory monitoring with quantum-compressed updates streamed to clients via Server-Sent Events.

## Usage

```bash
# Start SSE server on default port 8973
st --serve /path/to/watch

# Custom port and options
st --serve /path/to/watch --port 3000 --mode quantum

# Multiple paths
st --serve /project1 /project2 --mode summary-ai

# With CORS for web apps
st --serve . --cors "*" --compress
```

## HTTP Endpoints

### SSE Stream: `GET /events`

```javascript
const eventSource = new EventSource('http://localhost:8973/events');

eventSource.onmessage = (event) => {
    const data = JSON.parse(event.data);
    console.log('Tree update:', data);
};

eventSource.addEventListener('quantum', (event) => {
    // Quantum-compressed update
    const compressed = event.data;
    // Decompress and process...
});
```

### REST Endpoints

```
GET /tree?path=/src&mode=quantum
GET /stats?path=/src  
GET /search?q=TODO&path=/src
GET /compress?path=/src/file.md&format=marqant
POST /watch { "path": "/new/path" }
DELETE /watch { "path": "/old/path" }
```

## Event Types

### 1. Initial Tree Event
```json
{
    "event": "tree",
    "data": {
        "path": "/project",
        "mode": "quantum",
        "compressed": "QTN1:AQID...", 
        "stats": {
            "files": 1234,
            "dirs": 56,
            "size": 78901234
        }
    }
}
```

### 2. File Change Events
```json
{
    "event": "change",
    "data": {
        "type": "modified|created|deleted|renamed",
        "path": "/project/src/main.rs",
        "size": 1234,
        "timestamp": "2025-07-16T10:30:00Z",
        "delta": "QTN1:DELTA..." // Quantum delta
    }
}
```

### 3. Directory Statistics
```json
{
    "event": "stats",
    "data": {
        "path": "/project",
        "interval": "5m",
        "changes": 42,
        "growth": "+1.2MB",
        "hot_files": ["src/main.rs", "Cargo.toml"]
    }
}
```

### 4. Compressed Batch Updates
```json
{
    "event": "quantum",
    "data": "QTN1:BATCH:..." // Multiple changes compressed
}
```

## Implementation Architecture

```rust
pub struct SseServer {
    watcher: RecommendedWatcher,
    clients: Arc<Mutex<Vec<SseClient>>>,
    compression_mode: OutputMode,
    paths: Vec<PathBuf>,
}

impl SseServer {
    pub async fn start(config: SseConfig) -> Result<()> {
        // Setup file watcher
        let (tx, rx) = channel();
        let watcher = RecommendedWatcher::new(tx, Config::default())?;
        
        // Watch paths
        for path in &config.paths {
            watcher.watch(path, RecursiveMode::Recursive)?;
        }
        
        // HTTP server
        let app = Router::new()
            .route("/events", get(sse_handler))
            .route("/tree", get(tree_handler))
            .route("/stats", get(stats_handler))
            .with_state(AppState { watcher, clients });
            
        Server::bind(&config.addr)
            .serve(app.into_make_service())
            .await?;
    }
}
```

## Compression Strategies

### 1. Full Tree Compression
- Initial connection: Send full quantum tree
- Updates: Send quantum deltas only

### 2. Incremental Updates
- Track client state
- Send only changes since last event
- Automatic rebasing on reconnect

### 3. Smart Batching
- Aggregate rapid changes
- Compress multiple events together
- Send every N ms or M changes

## Client Libraries

### JavaScript/TypeScript
```typescript
import { SmartTreeSSE } from '@smart-tree/sse-client';

const tree = new SmartTreeSSE('http://localhost:8973');

tree.on('change', (event) => {
    console.log(`File ${event.path} was ${event.type}`);
});

tree.on('quantum', async (compressed) => {
    const tree = await tree.decompress(compressed);
    renderTree(tree);
});
```

### Rust Client
```rust
use smart_tree_sse::SseClient;

let mut client = SseClient::connect("http://localhost:8973").await?;

while let Some(event) = client.next().await {
    match event? {
        Event::Change { path, change_type } => {
            println!("Changed: {:?}", path);
        }
        Event::Quantum(data) => {
            let tree = decompress_quantum(&data)?;
        }
    }
}
```

## Use Cases

### 1. Live Development Dashboard
```javascript
// Real-time project statistics
eventSource.addEventListener('stats', (e) => {
    updateDashboard(JSON.parse(e.data));
});
```

### 2. AI-Powered Code Assistant
```python
# Stream changes to AI for real-time analysis
async for event in tree_sse.stream():
    if event.type == 'change':
        await ai.analyze_change(event.path, event.delta)
```

### 3. Distributed Build System
```rust
// Watch for changes, trigger builds
client.on_change(|change| {
    if change.path.ends_with(".rs") {
        build_queue.push(change.path);
    }
});
```

### 4. Cloud Sync
```go
// Sync quantum deltas to cloud
for event := range sseClient.Events() {
    if event.Type == "quantum" {
        cloudStorage.PushDelta(event.Data)
    }
}
```

## Performance Optimizations

1. **Compression Cache**: Pre-compress common requests
2. **Delta Encoding**: Send only changes, not full trees
3. **Client State Tracking**: Remember what each client has
4. **Parallel Compression**: Use thread pool for quantum encoding
5. **Zero-Copy Streaming**: Direct memory to network

## Security

```bash
# Authentication
st --serve . --auth-token $SECRET_TOKEN

# HTTPS with auto-cert
st --serve . --https --domain tree.example.com

# IP allowlist
st --serve . --allow "192.168.1.0/24,10.0.0.0/8"
```

## Future Extensions

1. **WebSocket upgrade**: Bidirectional communication
2. **GraphQL endpoint**: Query specific parts
3. **gRPC streaming**: For high-performance clients
4. **P2P mode**: Clients share updates
5. **Blockchain integration**: Immutable change history

*"From filesystem to flight stream - Smart Tree takes off!"* 🚀🎸