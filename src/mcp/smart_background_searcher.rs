// Smart Background Searcher - Intelligent file content searching with limits
// "Like ripgrep but knows when to stop reading!" - Aye

use anyhow::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub max_lines_per_file: usize, // Default: 1000 for JSONL, 5000 for others
    pub max_file_size_mb: u64,     // Skip files larger than this
    pub search_timeout_ms: u64,    // Timeout per file
    pub fuzzy_threshold: i64,      // Fuzzy match score threshold
    pub smart_sampling: bool,      // Sample large files intelligently
    pub watch_patterns: Vec<String>, // File patterns to watch
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            max_lines_per_file: 1000,
            max_file_size_mb: 50,
            search_timeout_ms: 500,
            fuzzy_threshold: 50,
            smart_sampling: true,
            watch_patterns: vec![
                "*.json".to_string(),
                "*.jsonl".to_string(),
                "*.md".to_string(),
                "*.log".to_string(),
                "*.txt".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub content: String,
    pub score: i64,
    pub context: Vec<String>, // Lines around the match
    pub file_type: String,
    pub timestamp: std::time::SystemTime,
}

pub struct SmartBackgroundSearcher {
    config: SearchConfig,
    search_index: Arc<Mutex<HashMap<PathBuf, Vec<SearchResult>>>>,
    watcher: Option<RecommendedWatcher>,
    sender: Sender<SearchEvent>,
}

enum SearchEvent {
    Search { query: String, paths: Vec<PathBuf> },
    FileChanged(PathBuf),
    Stop,
}

impl SmartBackgroundSearcher {
    pub fn new(config: SearchConfig) -> Result<Self> {
        let (sender, receiver) = channel();
        let search_index = Arc::new(Mutex::new(HashMap::new()));

        // Start background search thread
        let index_clone = search_index.clone();
        let config_clone = config.clone();

        thread::spawn(move || {
            Self::search_worker(receiver, index_clone, config_clone);
        });

        Ok(Self {
            config,
            search_index,
            watcher: None,
            sender,
        })
    }

    fn search_worker(
        receiver: Receiver<SearchEvent>,
        index: Arc<Mutex<HashMap<PathBuf, Vec<SearchResult>>>>,
        config: SearchConfig,
    ) {
        let fuzzy_matcher = SkimMatcherV2::default();

        while let Ok(event) = receiver.recv() {
            match event {
                SearchEvent::Search { query, paths } => {
                    for path in paths {
                        if let Ok(results) =
                            Self::search_file(&path, &query, &config, &fuzzy_matcher)
                        {
                            if !results.is_empty() {
                                if let Ok(mut idx) = index.lock() {
                                    idx.insert(path, results);
                                }
                            }
                        }
                    }
                }
                SearchEvent::FileChanged(path) => {
                    // Re-index changed file
                    if let Ok(mut idx) = index.lock() {
                        idx.remove(&path);
                    }
                }
                SearchEvent::Stop => break,
            }
        }
    }

    fn search_file(
        path: &Path,
        query: &str,
        config: &SearchConfig,
        matcher: &SkimMatcherV2,
    ) -> Result<Vec<SearchResult>> {
        let start = Instant::now();
        let mut results = Vec::new();

        // Check file size
        let metadata = fs::metadata(path)?;
        if metadata.len() > config.max_file_size_mb * 1024 * 1024 {
            return Ok(results); // Skip large files
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let file_type = Self::detect_file_type(ext);

        // Determine max lines based on file type
        let max_lines = match ext {
            "jsonl" => config.max_lines_per_file,
            "log" => config.max_lines_per_file,
            _ => config.max_lines_per_file * 5, // Allow more for regular files
        };

        let mut line_number = 0;
        let mut lines_buffer: Vec<String> = Vec::with_capacity(5);

        for line_result in reader.lines() {
            // Check timeout
            if start.elapsed().as_millis() > config.search_timeout_ms as u128 {
                break;
            }

            line_number += 1;
            if line_number > max_lines {
                if config.smart_sampling {
                    // Smart sampling: read every Nth line after limit
                    if line_number % 10 != 0 {
                        continue;
                    }
                } else {
                    break;
                }
            }

            if let Ok(line) = line_result {
                // Keep a rolling buffer for context
                lines_buffer.push(line.clone());
                if lines_buffer.len() > 5 {
                    lines_buffer.remove(0);
                }

                // Try fuzzy matching
                if let Some(score) = matcher.fuzzy_match(&line, query) {
                    if score >= config.fuzzy_threshold {
                        // For JSONL, try to parse and extract relevant fields
                        let content = if ext == "jsonl" {
                            Self::extract_jsonl_content(&line).unwrap_or(line.clone())
                        } else {
                            line.clone()
                        };

                        results.push(SearchResult {
                            file_path: path.to_path_buf(),
                            line_number,
                            content,
                            score,
                            context: lines_buffer.clone(),
                            file_type: file_type.clone(),
                            timestamp: std::time::SystemTime::now(),
                        });
                    }
                }

                // Also check for exact substring match (case insensitive)
                if line.to_lowercase().contains(&query.to_lowercase()) {
                    let content = if ext == "jsonl" {
                        Self::extract_jsonl_content(&line).unwrap_or(line.clone())
                    } else {
                        line.clone()
                    };

                    results.push(SearchResult {
                        file_path: path.to_path_buf(),
                        line_number,
                        content,
                        score: 100, // High score for exact match
                        context: lines_buffer.clone(),
                        file_type: file_type.clone(),
                        timestamp: std::time::SystemTime::now(),
                    });
                }
            }
        }

        Ok(results)
    }

    fn extract_jsonl_content(line: &str) -> Option<String> {
        // Try to parse JSONL and extract meaningful content
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            // Extract common AI assistant fields
            let mut parts = Vec::new();

            if let Some(msg) = json.get("message").and_then(|v| v.as_str()) {
                parts.push(msg.to_string());
            }
            if let Some(prompt) = json.get("prompt").and_then(|v| v.as_str()) {
                parts.push(format!("Prompt: {}", prompt));
            }
            if let Some(response) = json.get("response").and_then(|v| v.as_str()) {
                parts.push(format!("Response: {}", response));
            }
            if let Some(content) = json.get("content").and_then(|v| v.as_str()) {
                parts.push(content.to_string());
            }

            if !parts.is_empty() {
                return Some(parts.join(" | "));
            }
        }
        None
    }

    fn detect_file_type(ext: &str) -> String {
        match ext {
            "json" => "json".to_string(),
            "jsonl" => "jsonl_stream".to_string(),
            "md" | "markdown" => "markdown".to_string(),
            "log" => "log_file".to_string(),
            "txt" => "text_file".to_string(),
            _ => "unknown".to_string(),
        }
    }

    pub async fn search(&self, query: &str, paths: Vec<PathBuf>) -> Vec<SearchResult> {
        // Send search request to background thread
        let _ = self.sender.send(SearchEvent::Search {
            query: query.to_string(),
            paths: paths.clone(),
        });

        // Wait a bit for results
        thread::sleep(Duration::from_millis(self.config.search_timeout_ms));

        // Collect results from index
        let mut all_results = Vec::new();
        if let Ok(idx) = self.search_index.lock() {
            for path in paths {
                if let Some(results) = idx.get(&path) {
                    all_results.extend(results.clone());
                }
            }
        }

        // Sort by score
        all_results.sort_by(|a, b| b.score.cmp(&a.score));
        all_results
    }

    pub fn start_watching(&mut self, watch_paths: Vec<PathBuf>) -> Result<()> {
        let sender = self.sender.clone();
        let config = self.config.clone();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                        for path in event.paths {
                            // Check if file matches our watch patterns
                            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                                let should_watch = config.watch_patterns.iter().any(|pattern| {
                                    pattern.ends_with(&format!("*.{}", ext))
                                        || pattern == &format!("*.{}", ext)
                                });

                                if should_watch {
                                    println!("🔍 File changed, re-indexing: {}", path.display());
                                    let _ = sender.send(SearchEvent::FileChanged(path));
                                }
                            }
                        }
                    }
                }
            },
            Config::default(),
        )?;

        // Watch specified paths
        for path in &watch_paths {
            if path.exists() {
                watcher.watch(path, RecursiveMode::Recursive)?;
                println!("👁️  Watching for changes in: {}", path.display());
            }
        }

        self.watcher = Some(watcher);

        // Do initial indexing of existing files
        self.initial_index(watch_paths)?;

        Ok(())
    }

    fn initial_index(&self, watch_paths: Vec<PathBuf>) -> Result<()> {
        println!("🔍 Initial indexing of watched directories...");

        for watch_path in watch_paths {
            if watch_path.is_dir() {
                // Find all matching files in the directory
                for pattern in &self.config.watch_patterns {
                    let glob_pattern = format!("{}/{}", watch_path.display(), pattern);
                    if let Ok(paths) = glob::glob(&glob_pattern) {
                        let files: Vec<PathBuf> = paths
                            .filter_map(|p| p.ok())
                            .filter(|p| p.is_file())
                            .collect();

                        if !files.is_empty() {
                            println!("  Found {} {} files", files.len(), pattern);
                            // Trigger initial search/index for these files
                            let _ = self.sender.send(SearchEvent::Search {
                                query: String::new(), // Empty query for initial indexing
                                paths: files,
                            });
                        }
                    }
                }
            }
        }

        println!("✅ Initial indexing complete!");
        Ok(())
    }

    pub fn get_cached_results(&self, path: &Path) -> Vec<SearchResult> {
        if let Ok(idx) = self.search_index.lock() {
            idx.get(path).cloned().unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    pub fn clear_cache(&self) {
        if let Ok(mut idx) = self.search_index.lock() {
            idx.clear();
        }
    }
}

// Integration with MCP
pub async fn handle_smart_search(params: Value) -> Result<Value> {
    let query = params["query"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing query parameter"))?;

    let paths: Vec<PathBuf> = params["paths"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(PathBuf::from)
                .collect()
        })
        .unwrap_or_else(|| vec![std::env::current_dir().unwrap_or_default()]);

    let config = SearchConfig::default();
    let searcher = SmartBackgroundSearcher::new(config)?;

    let results = searcher.search(query, paths).await;

    // Format results for MCP
    let formatted: Vec<Value> = results
        .into_iter()
        .take(20) // Limit results
        .map(|r| {
            serde_json::json!({
                "file": r.file_path.to_string_lossy(),
                "line": r.line_number,
                "content": r.content,
                "score": r.score,
                "type": r.file_type,
                "context": r.context,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "results": formatted,
        "count": formatted.len(),
        "message": format!("Found {} matches for '{}'", formatted.len(), query)
    }))
}
