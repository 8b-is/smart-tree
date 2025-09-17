// Unified Watcher - Master control for all context absorption and searching
// "The all-seeing eye of Smart Tree!" - Aye

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::context_absorber::ContextAbsorber;
use super::smart_background_searcher::{SmartBackgroundSearcher, SearchConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedWatcherConfig {
    pub project_name: String,
    pub watch_paths: Vec<String>,
    pub enable_absorption: bool,
    pub enable_search: bool,
    pub enable_logging: bool,
    pub auto_start: bool,
}

impl Default for UnifiedWatcherConfig {
    fn default() -> Self {
        Self {
            project_name: "smart-tree".to_string(),
            watch_paths: vec![
                "~/Documents/".to_string(),
                "~/.config/".to_string(),
                "~/Library/Application Support/Claude/".to_string(),
                "~/.cursor/".to_string(),
                "~/.vscode/".to_string(),
            ],
            enable_absorption: true,
            enable_search: true,
            enable_logging: true,
            auto_start: false,
        }
    }
}

pub struct UnifiedWatcher {
    config: UnifiedWatcherConfig,
    absorber: Option<Arc<Mutex<ContextAbsorber>>>,
    searcher: Option<Arc<Mutex<SmartBackgroundSearcher>>>,
    status: Arc<Mutex<WatcherStatus>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherStatus {
    pub is_running: bool,
    pub files_watched: usize,
    pub contexts_absorbed: usize,
    pub search_results_cached: usize,
    pub last_activity: Option<String>,
    pub watched_directories: Vec<String>,
}

impl UnifiedWatcher {
    pub fn new(config: UnifiedWatcherConfig) -> Result<Self> {
        let status = Arc::new(Mutex::new(WatcherStatus {
            is_running: false,
            files_watched: 0,
            contexts_absorbed: 0,
            search_results_cached: 0,
            last_activity: None,
            watched_directories: config.watch_paths.clone(),
        }));

        Ok(Self {
            config,
            absorber: None,
            searcher: None,
            status,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        println!("🚀 Starting Unified Watcher for project: {}", self.config.project_name);

        // Initialize activity logging if enabled
        if self.config.enable_logging {
            crate::activity_logger::ActivityLogger::init(Some("~/.st/watcher.jsonl".to_string()))?;
            crate::activity_logger::ActivityLogger::log_event(
                "watcher",
                "start",
                serde_json::json!({
                    "project": self.config.project_name,
                    "watch_paths": self.config.watch_paths,
                })
            )?;
        }

        // Expand watch paths
        let watch_paths: Vec<PathBuf> = self.config.watch_paths
            .iter()
            .map(|p| PathBuf::from(shellexpand::tilde(p).to_string()))
            .filter(|p| p.exists())
            .collect();

        // Start context absorber if enabled
        if self.config.enable_absorption {
            println!("🧽 Starting Context Absorber...");
            let mut absorber = ContextAbsorber::new(self.config.project_name.clone())?;
            absorber.start_watching()?;
            self.absorber = Some(Arc::new(Mutex::new(absorber)));
            println!("   ✅ Context Absorber active");
        }

        // Start smart searcher if enabled
        if self.config.enable_search {
            println!("🔍 Starting Smart Background Searcher...");
            let search_config = SearchConfig {
                max_lines_per_file: 1000,  // Limit for JSONL files
                smart_sampling: true,
                ..Default::default()
            };
            let mut searcher = SmartBackgroundSearcher::new(search_config)?;
            searcher.start_watching(watch_paths.clone())?;
            self.searcher = Some(Arc::new(Mutex::new(searcher)));
            println!("   ✅ Smart Searcher active");
        }

        // Update status
        if let Ok(mut status) = self.status.lock() {
            status.is_running = true;
            status.watched_directories = watch_paths.iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            status.last_activity = Some(format!("Started watching at {}", chrono::Utc::now()));
        }

        // Start monitoring thread
        self.start_monitor_thread();

        println!("\n✨ Unified Watcher is now active!");
        println!("📂 Watching {} directories", watch_paths.len());
        println!("🎯 Project: {}", self.config.project_name);

        Ok(())
    }

    fn start_monitor_thread(&self) {
        let status = self.status.clone();
        let absorber = self.absorber.clone();
        let _searcher = self.searcher.clone();

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(30));

                // Update status periodically
                if let Ok(mut stat) = status.lock() {
                    // Get absorbed context count
                    if let Some(abs) = &absorber {
                        if let Ok(abs_lock) = abs.lock() {
                            stat.contexts_absorbed = abs_lock.get_absorbed_contexts().len();
                        }
                    }

                    // Update last activity
                    stat.last_activity = Some(format!("Active at {}", chrono::Utc::now()));
                }
            }
        });
    }

    pub async fn stop(&mut self) -> Result<()> {
        println!("🛑 Stopping Unified Watcher...");

        // Stop absorber
        if let Some(abs) = &self.absorber {
            if let Ok(mut abs_lock) = abs.lock() {
                abs_lock.stop_watching();
            }
        }

        // Clear searcher cache
        if let Some(search) = &self.searcher {
            if let Ok(search_lock) = search.lock() {
                search_lock.clear_cache();
            }
        }

        // Update status
        if let Ok(mut status) = self.status.lock() {
            status.is_running = false;
            status.last_activity = Some(format!("Stopped at {}", chrono::Utc::now()));
        }

        // Log shutdown
        if self.config.enable_logging {
            crate::activity_logger::ActivityLogger::log_event(
                "watcher",
                "stop",
                serde_json::json!({
                    "project": self.config.project_name,
                })
            )?;
        }

        Ok(())
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Value>> {
        if let Some(searcher) = &self.searcher {
            if let Ok(search_lock) = searcher.lock() {
                let paths: Vec<PathBuf> = self.config.watch_paths
                    .iter()
                    .map(|p| PathBuf::from(shellexpand::tilde(p).to_string()))
                    .collect();

                let results = search_lock.search(query, paths).await;

                // Convert to JSON for MCP
                let json_results: Vec<Value> = results.into_iter()
                    .map(|r| serde_json::json!({
                        "file": r.file_path.to_string_lossy(),
                        "line": r.line_number,
                        "content": r.content,
                        "score": r.score,
                        "type": r.file_type,
                    }))
                    .collect();

                return Ok(json_results);
            }
        }
        Ok(Vec::new())
    }

    pub fn get_status(&self) -> WatcherStatus {
        self.status.lock().unwrap().clone()
    }
}

// MCP Tool Handler
pub async fn handle_unified_watcher(params: Value, _ctx: Arc<crate::mcp::McpContext>) -> Result<Value> {
    let action = params["action"].as_str().unwrap_or("status");

    // Use a static instance for the watcher
    static WATCHER: Lazy<Arc<Mutex<Option<UnifiedWatcher>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

    match action {
        "start" => {
            let project = params["project"]
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    // Try to detect project from current directory
                    std::env::current_dir()
                        .ok()
                        .and_then(|p| p.file_name().map(|n| n.to_os_string()))
                        .and_then(|n| n.to_str().map(|s| s.to_string()))
                        .unwrap_or_else(|| "unknown".to_string())
                });

            let watch_paths = params["paths"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_else(|| UnifiedWatcherConfig::default().watch_paths);

            let config = UnifiedWatcherConfig {
                project_name: project.to_string(),
                watch_paths,
                enable_absorption: params["enable_absorption"].as_bool().unwrap_or(true),
                enable_search: params["enable_search"].as_bool().unwrap_or(true),
                enable_logging: params["enable_logging"].as_bool().unwrap_or(true),
                auto_start: false,
            };

            let mut watcher = UnifiedWatcher::new(config)?;
            watcher.start().await?;

            let status = watcher.get_status();

            // Store the watcher
            *WATCHER.lock().unwrap() = Some(watcher);

            Ok(serde_json::json!({
                "status": "started",
                "project": project,
                "watching": status.watched_directories,
                "features": {
                    "absorption": params["enable_absorption"].as_bool().unwrap_or(true),
                    "search": params["enable_search"].as_bool().unwrap_or(true),
                    "logging": params["enable_logging"].as_bool().unwrap_or(true),
                },
                "message": format!("🚀 Unified Watcher active for '{}'", project)
            }))
        }

        "stop" => {
            if let Some(mut watcher) = WATCHER.lock().unwrap().take() {
                watcher.stop().await?;
                Ok(serde_json::json!({
                    "status": "stopped",
                    "message": "Watcher stopped successfully"
                }))
            } else {
                Ok(serde_json::json!({
                    "status": "not_running",
                    "message": "No watcher is currently running"
                }))
            }
        }

        "search" => {
            let query = params["query"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing query parameter"))?;

            if let Some(watcher) = WATCHER.lock().unwrap().as_ref() {
                let results = watcher.search(query).await?;
                Ok(serde_json::json!({
                    "query": query,
                    "results": results,
                    "count": results.len(),
                }))
            } else {
                Ok(serde_json::json!({
                    "error": "Watcher not running",
                    "message": "Start the watcher first with action: 'start'"
                }))
            }
        }

        "status" => {
            if let Some(watcher) = WATCHER.lock().unwrap().as_ref() {
                let status = watcher.get_status();
                Ok(serde_json::json!({
                    "running": status.is_running,
                    "files_watched": status.files_watched,
                    "contexts_absorbed": status.contexts_absorbed,
                    "search_results_cached": status.search_results_cached,
                    "last_activity": status.last_activity,
                    "watched_directories": status.watched_directories,
                }))
            } else {
                Ok(serde_json::json!({
                    "running": false,
                    "message": "No watcher configured"
                }))
            }
        }

        _ => Err(anyhow::anyhow!("Unknown action: {}. Valid actions: start, stop, search, status", action))
    }
}

use once_cell::sync::Lazy;