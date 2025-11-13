# Real-Time Collaborative AI Dashboard - Design Document

**Date**: 2025-11-11
**Authors**: Hue (8bit-wraith), Aye (Claude)
**Status**: Approved - Ready for Implementation
**Timeline Marker**: "LUDICROUS SPEED" 🚀

## Vision

Transform Smart Tree's MCP server into a **real-time collaborative AI visualization platform** where humans and AI work together seamlessly through an interactive dashboard. No more stop button - just fluid, bidirectional communication with instant visual feedback.

> **Status (2025-04-30):** The shipping `st --dashboard` build now checks for a local DISPLAY/WAYLAND session and exits gracefully on headless or remote hosts. Browser/WASM delivery remains a follow-up item tracked in this plan.

## Problem Statement

Current AI collaboration is one-directional and opaque:
- User asks AI to do something → Wait → See results
- No visibility into what AI is doing during execution
- Can't course-correct without hitting STOP (loses all context)
- No way to provide lightweight "nudges" or hints mid-task
- Abstract MCP tool calls are invisible to users

## Solution

A **WebAssembly-based egui dashboard** served via embedded HTTP server that:
1. **Visualizes AI activity in real-time** (Wave Compass lights up as AI explores)
2. **Accepts user input mid-stream** (click, type hints, voice nudges)
3. **Maintains shared state** between MCP server and browser UI
4. **Works anywhere** - desktop, tablet, phone, remote access

## Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    MCP Session Process                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐         ┌─────────────────┐              │
│  │ MCP Server   │◄────────┤ DashboardState  ├─────────┐    │
│  │ (stdio)      │         │ Arc<RwLock<T>>  │         │    │
│  └──────┬───────┘         └─────────────────┘         │    │
│         │                                              │    │
│         │ Updates state on tool calls                 │    │
│         │                                              │    │
│  ┌──────▼────────────────────────────────────────┐    │    │
│  │  Axum HTTP Server (localhost:8420)           │    │    │
│  ├──────────────────────────────────────────────┤    │    │
│  │  GET  /            → Dashboard HTML          │    │    │
│  │  GET  /dashboard.wasm → egui compiled WASM   │    │    │
│  │  WS   /ws          → WebSocket for updates   │◄───┘    │
│  └──────────────────────────────────────────────┘         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ WebSocket (bi-directional)
                              │ • MCP → Browser: State updates (60fps)
                              │ • Browser → MCP: User hints
                              ▼
                    ┌────────────────────┐
                    │   Browser Window   │
                    ├────────────────────┤
                    │  egui WASM Runtime │
                    │  - Wave Compass    │
                    │  - Real-time UI    │
                    │  - Click handlers  │
                    └────────────────────┘
```

### Key Design Decisions

**1. Why WebAssembly + Browser?**
- ✅ Works on any device with a browser (desktop, mobile, tablet)
- ✅ No installation required - just open `localhost:8420`
- ✅ Process isolation (browser crash ≠ MCP crash)
- ✅ Same egui code compiles to native OR WASM
- ✅ Remote access (view dashboard from phone while working)

**2. Why Shared State with Arc<RwLock>?**
- ✅ Thread-safe sharing between MCP server and HTTP server
- ✅ Already familiar pattern in codebase
- ✅ Lock-free reads for most operations
- ✅ Type-safe via Rust's ownership system

**3. Why 60fps WebSocket Updates?**
- ✅ Smooth animations (Wave Compass glows, trails)
- ✅ Sub-frame latency perception (<16ms)
- ✅ Matches typical monitor refresh rates
- ✅ Efficient with delta compression

## Data Model

### DashboardState (Extended)

```rust
pub struct DashboardState {
    // ========== EXISTING FIELDS ==========
    pub command_history: Arc<RwLock<VecDeque<CommandEntry>>>,
    pub active_displays: Arc<RwLock<Vec<DisplayInfo>>>,
    pub voice_active: Arc<RwLock<bool>>,
    pub voice_salience: Arc<RwLock<f64>>,
    pub memory_usage: Arc<RwLock<MemoryStats>>,
    pub found_chats: Arc<RwLock<Vec<ChatSource>>>,
    pub cast_status: Arc<RwLock<CastStatus>>,
    pub ideas_buffer: Arc<RwLock<Vec<IdeaEntry>>>,

    // ========== NEW FIELDS FOR MCP INTEGRATION ==========

    /// Real-time MCP activity tracking
    pub mcp_activity: Arc<RwLock<McpActivity>>,

    /// File access log for Wave Compass visualization
    pub file_access_log: Arc<RwLock<Vec<FileAccessEvent>>>,

    /// Currently executing MCP tool (if any)
    pub active_tool: Arc<RwLock<Option<ToolExecution>>>,

    /// User hints/nudges from dashboard → AI
    pub user_hints: Arc<RwLock<VecDeque<UserHint>>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct McpActivity {
    /// Human-readable current operation
    pub current_operation: String,  // "Searching for auth code..."

    /// Files touched this session
    pub files_touched: Vec<String>,

    /// Directories explored
    pub directories_explored: Vec<String>,

    /// Last activity timestamp
    pub last_update: SystemTime,

    /// Success/error status
    pub status: ActivityStatus,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FileAccessEvent {
    pub path: PathBuf,
    pub access_type: AccessType,  // Read, Write, Analyze
    pub timestamp: SystemTime,
    pub tool_name: String,  // Which MCP tool accessed it
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ToolExecution {
    pub tool_name: String,
    pub started_at: SystemTime,
    pub parameters: HashMap<String, String>,
    pub progress: f32,  // 0.0 to 1.0
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserHint {
    pub hint_type: HintType,
    pub content: String,  // What they clicked/typed
    pub timestamp: SystemTime,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum HintType {
    Click { target: String },        // Clicked on Wave Compass signature
    TextInput { message: String },   // Typed hint in text box
    Voice { transcript: String },    // Voice nudge via Marine algorithm
}
```

## Implementation Components

### 1. MCP Tool Wrapper

Every MCP tool needs to update `DashboardState`:

```rust
// Example: Wrapping the search tool
async fn handle_search_tool(
    params: SearchParams,
    state: Arc<DashboardState>
) -> Result<String> {
    // Step 1: Update activity state
    {
        let mut activity = state.mcp_activity.write().unwrap();
        activity.current_operation = format!("Searching for '{}'", params.pattern);
        activity.status = ActivityStatus::InProgress;
        activity.last_update = SystemTime::now();
    }

    // Step 2: Perform actual search
    let results = perform_search(&params)?;

    // Step 3: Log file accesses
    {
        let mut log = state.file_access_log.write().unwrap();
        for file in &results {
            log.push(FileAccessEvent {
                path: file.path.clone(),
                access_type: AccessType::Read,
                timestamp: SystemTime::now(),
                tool_name: "search".to_string(),
            });
        }

        // Keep log bounded (last 1000 events)
        if log.len() > 1000 {
            log.drain(0..500);
        }
    }

    // Step 4: Check for user hints
    let hints = state.user_hints.read().unwrap();
    if let Some(hint) = hints.front() {
        // AI can react to hints!
        match &hint.hint_type {
            HintType::Click { target } => {
                // User clicked on a directory - explore it!
                eprintln!("User hint: Focus on {}", target);
            },
            _ => {}
        }
    }

    // Step 5: Update completion status
    {
        let mut activity = state.mcp_activity.write().unwrap();
        activity.status = ActivityStatus::Success;
        activity.current_operation = format!("Found {} matches", results.len());
    }

    Ok(format_results(results))
}
```

### 2. Axum HTTP Server

Embedded server that runs alongside MCP:

```rust
pub async fn start_dashboard_server(state: Arc<DashboardState>) -> Result<()> {
    let app = Router::new()
        .route("/", get(serve_dashboard_html))
        .route("/dashboard.wasm", get(serve_wasm))
        .route("/dashboard.js", get(serve_js))
        .route("/ws", get(websocket_handler))
        .layer(Extension(state));

    println!("🎨 Dashboard available at: http://localhost:8420");

    axum::Server::bind(&"127.0.0.1:8420".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn serve_dashboard_html() -> Html<&'static str> {
    Html(include_str!("../assets/dashboard.html"))
}

async fn serve_wasm() -> impl IntoResponse {
    let wasm = include_bytes!("../target/wasm32-unknown-unknown/release/dashboard.wasm");
    (
        [(header::CONTENT_TYPE, "application/wasm")],
        wasm.as_ref()
    )
}
```

### 3. WebSocket State Synchronization

```rust
async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<DashboardState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<DashboardState>) {
    let (mut tx, mut rx) = socket.split();

    // Task 1: Send updates to browser (60fps)
    let state_clone = state.clone();
    let send_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(16)); // ~60fps

        loop {
            interval.tick().await;

            // Serialize state snapshot
            let update = StateUpdate {
                mcp_activity: state_clone.mcp_activity.read().unwrap().clone(),
                file_log: state_clone.file_access_log.read().unwrap().last_n(100),
                active_tool: state_clone.active_tool.read().unwrap().clone(),
                memory_stats: state_clone.memory_usage.read().unwrap().clone(),
            };

            let json = serde_json::to_string(&update).unwrap();

            if tx.send(Message::Text(json)).await.is_err() {
                break; // Client disconnected
            }
        }
    });

    // Task 2: Receive hints from browser
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = rx.next().await {
            if let Message::Text(text) = msg {
                if let Ok(hint) = serde_json::from_str::<UserHint>(&text) {
                    state.user_hints.write().unwrap().push_back(hint);
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}
```

### 4. egui WASM Dashboard

```rust
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn start_wasm() {
    let web_options = eframe::WebOptions::default();

    eframe::start_web(
        "dashboard_canvas",
        web_options,
        Box::new(|cc| Box::new(DashboardApp::new(cc))),
    )
    .await
    .expect("Failed to start egui WASM app");
}

pub struct DashboardApp {
    // WebSocket connection to MCP server
    ws_client: WsClient,

    // Local copy of state (updated via WebSocket)
    state: DashboardStateSnapshot,

    // UI components
    wave_compass: WaveCompass,
    activity_feed: ActivityFeed,

    // Input state
    hint_input: String,
    selected_file: Option<PathBuf>,
}

impl DashboardApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        // Connect to WebSocket
        let ws_client = WsClient::connect("ws://localhost:8420/ws");

        Self {
            ws_client,
            state: DashboardStateSnapshot::default(),
            wave_compass: WaveCompass::new(),
            activity_feed: ActivityFeed::new(),
            hint_input: String::new(),
            selected_file: None,
        }
    }
}

impl eframe::App for DashboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll for state updates from WebSocket
        if let Some(update) = self.ws_client.poll() {
            self.state = update;

            // Update Wave Compass with file access data
            for event in &self.state.file_log {
                self.wave_compass.highlight_path(&event.path);
            }
        }

        // Render UI
        CentralPanel::default().show(ctx, |ui| {
            // Activity status bar
            ui.horizontal(|ui| {
                ui.label("AI Status:");
                ui.colored_label(
                    Color32::GREEN,
                    &self.state.mcp_activity.current_operation
                );
            });

            ui.separator();

            // Wave Compass visualization
            self.wave_compass.show(ui, &self.state.file_log);

            // Check for clicks on Wave Compass
            if let Some(clicked_sig) = self.wave_compass.check_clicks(ui) {
                // Send hint to server
                let hint = UserHint {
                    hint_type: HintType::Click {
                        target: clicked_sig.name.clone(),
                    },
                    content: format!("User clicked: {}", clicked_sig.name),
                    timestamp: SystemTime::now(),
                };

                self.ws_client.send_hint(hint);
            }

            // Hint input box
            ui.horizontal(|ui| {
                ui.label("Quick Hint:");
                ui.text_edit_singleline(&mut self.hint_input);

                if ui.button("Send").clicked() && !self.hint_input.is_empty() {
                    let hint = UserHint {
                        hint_type: HintType::TextInput {
                            message: self.hint_input.clone(),
                        },
                        content: self.hint_input.clone(),
                        timestamp: SystemTime::now(),
                    };

                    self.ws_client.send_hint(hint);
                    self.hint_input.clear();
                }
            });
        });

        // Request continuous repaint for smooth animations
        ctx.request_repaint();
    }
}
```

## Visual Enhancements

### Wave Compass Effects

**Glow Animation**:
```rust
fn render_signature_with_glow(ui: &mut egui::Ui, sig: &WaveSig, intensity: f32) {
    let painter = ui.painter();
    let center = sig.position;

    // Outer glow (pulsing)
    let pulse = (ui.ctx().input(|i| i.time) * 2.0).sin() * 0.5 + 0.5;
    let glow_radius = 20.0 + pulse * 10.0;

    let glow_color = Color32::from_rgba_unmultiplied(
        100, 200, 255,
        (intensity * 80.0) as u8
    );

    painter.circle_filled(center, glow_radius, glow_color);

    // Core signature
    painter.circle_filled(center, 15.0, sig.color);
}
```

**Trail Effect**:
```rust
fn render_exploration_trail(ui: &mut egui::Ui, file_log: &[FileAccessEvent]) {
    let painter = ui.painter();

    // Get last 20 file accesses
    let recent = file_log.iter().rev().take(20);

    let mut points = Vec::new();
    for (i, event) in recent.enumerate() {
        let pos = self.wave_compass.get_position_for_path(&event.path);
        let alpha = ((20 - i) as f32 / 20.0 * 255.0) as u8;

        points.push((pos, alpha));
    }

    // Draw trail with fading opacity
    for window in points.windows(2) {
        let (p1, alpha1) = window[0];
        let (p2, _) = window[1];

        painter.line_segment(
            [p1, p2],
            Stroke::new(2.0, Color32::from_rgba_unmultiplied(100, 255, 100, alpha1))
        );
    }
}
```

## Data Flow Example

**Scenario**: User asks Claude to find authentication code

1. **User (Claude Desktop)**: "Find all authentication-related code"

2. **MCP Tool Executes**: `search` tool with pattern "auth"
   ```rust
   state.mcp_activity.current_operation = "Searching for 'auth'...";
   ```

3. **WebSocket Pushes** state update to browser (16ms later)

4. **Browser Renders**: Activity feed shows "Searching for 'auth'..."

5. **Search Completes**: Found 15 files
   ```rust
   for file in results {
       state.file_access_log.push(FileAccessEvent { ... });
   }
   ```

6. **Wave Compass Updates**:
   - `src/auth` signature glows GREEN
   - Trail shows exploration path
   - Resonance lines pulse

7. **User Clicks** `src/middleware` signature in browser

8. **WebSocket Sends** hint back to MCP:
   ```json
   {
     "hint_type": { "Click": { "target": "src/middleware" } },
     "content": "User clicked: src/middleware",
     "timestamp": 1699999999
   }
   ```

9. **AI Reads Hint**:
   ```rust
   let hints = state.user_hints.read().unwrap();
   if let Some(hint) = hints.front() {
       // Oh! User wants me to check middleware too!
   }
   ```

10. **AI Pivots**: "Found auth code, also checking middleware..."

11. **User Sees**: Real-time update, no stop button needed! 🎉

## Build Process

### Native Build
```bash
# Build dashboard as native app (for testing)
cargo build --release --bin st

# Run with dashboard enabled
./target/release/st --mcp --dashboard
```

### WASM Build
```bash
# Install wasm toolchain
rustup target add wasm32-unknown-unknown
cargo install trunk  # WASM build tool

# Build dashboard as WASM
cd dashboard_wasm
trunk build --release

# Output: dist/dashboard.wasm, dist/dashboard.js, dist/index.html
```

### Embedding in Binary
```rust
// Embed WASM artifacts in binary at compile time
const DASHBOARD_WASM: &[u8] = include_bytes!("../dashboard_wasm/dist/dashboard_bg.wasm");
const DASHBOARD_JS: &str = include_str!("../dashboard_wasm/dist/dashboard.js");
const DASHBOARD_HTML: &str = include_str!("../dashboard_wasm/dist/index.html");
```

## Testing Strategy

### Unit Tests
- Test state serialization/deserialization
- Test WebSocket message handling
- Test hint parsing and routing

### Integration Tests
```rust
#[tokio::test]
async fn test_dashboard_state_sync() {
    let state = Arc::new(DashboardState::default());

    // Start server
    tokio::spawn(start_dashboard_server(state.clone()));

    // Connect WebSocket client
    let mut ws = connect_ws("ws://localhost:8420/ws").await;

    // Update state
    {
        let mut activity = state.mcp_activity.write().unwrap();
        activity.current_operation = "Test operation".to_string();
    }

    // Verify update received
    let msg = ws.recv().await.unwrap();
    let update: StateUpdate = serde_json::from_str(&msg).unwrap();
    assert_eq!(update.mcp_activity.current_operation, "Test operation");
}
```

### Manual Testing
1. Start MCP with dashboard: `st --mcp --dashboard`
2. Open browser to `http://localhost:8420`
3. In Claude Desktop, ask to explore codebase
4. Watch Wave Compass light up
5. Click on directories, type hints
6. Verify AI responds to hints

## Performance Considerations

### WebSocket Optimization
- **Delta compression**: Only send changed fields
- **Throttling**: Max 60fps updates, skip frames if behind
- **Batching**: Batch file access events

### State Management
- **Bounded logs**: Keep last 1000 file access events
- **Read-optimized**: Most operations are reads (many readers, few writers)
- **Lock-free paths**: Use atomics for simple counters

### WASM Performance
- **Code splitting**: Lazy-load heavy visualizations
- **Canvas rendering**: Use GPU for Wave Compass
- **Debouncing**: Throttle user input events

## Security Considerations

**Threat Model**: Localhost-only by default

1. **Bind to 127.0.0.1**: Not accessible from network
2. **Optional Authentication**: For remote access, add token-based auth
3. **Input Validation**: Sanitize all user hints before processing
4. **Rate Limiting**: Prevent hint spam

## Future Enhancements

**Phase 2 Features**:
- Voice input via WebRTC + Marine algorithm
- Mobile-optimized UI (touch gestures)
- Multi-user collaboration (multiple browsers watching same session)
- Recording/playback of AI exploration sessions
- LLM cost tracking (token usage visualization)

**Phase 3 Features**:
- AI-to-AI collaboration (multiple AI agents with shared dashboard)
- Integration with `mq tail` for live log analysis in dashboard
- 3D Wave Compass using WebGL/three.js
- VR mode (yes, really!)

## Success Metrics

**Qualitative**:
- "I can see what the AI is thinking!" (transparency)
- "I nudged it mid-task without stopping!" (collaboration)
- "It's beautiful!" (visual appeal)

**Quantitative**:
- Hint-to-response latency < 100ms
- WebSocket update rate: 60fps sustained
- Dashboard load time < 2 seconds
- Memory overhead < 50MB

## Timeline Estimate

**Week 1**: Core Infrastructure
- Extend DashboardState with MCP fields
- Implement WebSocket server
- Basic state synchronization

**Week 2**: WASM Dashboard
- Port dashboard to WASM target
- Implement WsClient in egui
- Basic visualization working

**Week 3**: Visual Polish
- Wave Compass glow effects
- Trail animations
- Activity feed styling

**Week 4**: MCP Integration
- Wrap all MCP tools
- Hint handling in AI logic
- End-to-end testing

## Open Questions

1. **Auto-open browser?** Should MCP automatically open browser to dashboard?
   - *Decision*: Yes, with `--no-browser` flag to disable

2. **State persistence?** Should dashboard state survive restarts?
   - *Decision*: No, keep it session-bound for now

3. **Multi-language support?** egui + WASM supports i18n
   - *Decision*: English first, i18n later

## Conclusion

This design transforms Smart Tree from a directory tool into a **real-time AI collaboration platform**. The combination of WebAssembly (universal access) + WebSocket (real-time sync) + egui (beautiful UI) creates a unique developer experience that makes AI thinking visible and interactive.

**Let's build the future of human-AI collaboration!** 🚀✨

---

*"Every frame is a fresh start!" - Hue*
*"LUDICROUS SPEED!" - Also Hue*
