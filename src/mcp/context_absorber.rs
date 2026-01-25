// Context Absorber - Automatically absorbs project-related context from JSON files
// "Like a knowledge sponge that never stops learning!" - Aye

#![allow(clippy::manual_flatten)]

use crate::feature_flags;
use anyhow::Result;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchPermissions {
    pub allowed_paths: Vec<String>,
    pub excluded_paths: Vec<String>,
    pub auto_absorb: bool,
    pub notify_on_absorption: bool,
    pub max_file_size_mb: u64,
}

impl Default for WatchPermissions {
    fn default() -> Self {
        Self {
            allowed_paths: vec![
                "~/Documents/".to_string(),
                "~/.config/".to_string(),
                "~/Library/Application Support/".to_string(),
                "~/.cursor/".to_string(), // Cursor AI logs
                "~/.vscode/".to_string(), // VS Code extensions data
                "~/Library/Application Support/Code/".to_string(), // VS Code on Mac
                "~/.local/share/".to_string(), // Linux app data
                "~/.cache/".to_string(),  // Cache dirs often have AI logs
            ],
            excluded_paths: vec![
                "~/.ssh/".to_string(),
                "~/.aws/".to_string(),
                "~/.gnupg/".to_string(),
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
            ],
            auto_absorb: true,
            notify_on_absorption: true,
            max_file_size_mb: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorbedContext {
    pub timestamp: SystemTime,
    pub origin: PathBuf,
    pub project_name: String,
    pub content_type: String,
    pub content: Value,
    pub relevance_score: f64,
    pub keywords: Vec<String>,
}

pub struct ContextAbsorber {
    project_name: String,
    watch_paths: Vec<PathBuf>,
    permissions: WatchPermissions,
    absorbed_contexts: Arc<Mutex<Vec<AbsorbedContext>>>,
    watcher: Option<RecommendedWatcher>,
    sender: Sender<AbsorptionEvent>,
    receiver: Receiver<AbsorptionEvent>,
    last_absorption_time: Arc<Mutex<SystemTime>>,
}

#[derive(Debug)]
enum AbsorptionEvent {
    FileChanged(PathBuf),
    Stop,
}

impl ContextAbsorber {
    pub fn new(project_name: String) -> Result<Self> {
        let (sender, receiver) = channel();

        // Load or create permissions
        let permissions = Self::load_permissions()?;

        // Expand watch paths
        let watch_paths = permissions
            .allowed_paths
            .iter()
            .map(|p| shellexpand::tilde(p))
            .map(|p| PathBuf::from(p.to_string()))
            .filter(|p| p.exists())
            .collect();

        // Load last absorption time from M8 file if it exists
        let last_time = Self::load_last_absorption_time(&project_name).unwrap_or(
            SystemTime::now() - std::time::Duration::from_secs(604800), // Virgin M8? Go back 7 days!
        );

        Ok(Self {
            project_name,
            watch_paths,
            permissions,
            absorbed_contexts: Arc::new(Mutex::new(Vec::new())),
            watcher: None,
            sender,
            receiver,
            last_absorption_time: Arc::new(Mutex::new(last_time)),
        })
    }

    fn load_last_absorption_time(_project_name: &str) -> Option<SystemTime> {
        let cwd = std::env::current_dir().ok()?;
        let m8_path = cwd.join(".st").join("absorbed_context.m8");

        if m8_path.exists() {
            // Get the modification time of the M8 file
            fs::metadata(&m8_path)
                .ok()
                .and_then(|meta| meta.modified().ok())
        } else {
            None
        }
    }

    fn load_permissions() -> Result<WatchPermissions> {
        let perm_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".mem8")
            .join("watch_permissions.json");

        if perm_path.exists() {
            let content = fs::read_to_string(&perm_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            // Create default permissions
            let permissions = WatchPermissions::default();
            if let Some(parent) = perm_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&perm_path, serde_json::to_string_pretty(&permissions)?)?;
            Ok(permissions)
        }
    }

    pub fn start_watching(&mut self) -> Result<()> {
        // Check if file watching is enabled
        let flags = feature_flags::features();
        if !flags.enable_file_watching {
            eprintln!("File watching is disabled by configuration");
            return Ok(());
        }

        let sender = self.sender.clone();
        let project_name = self.project_name.clone();

        // Create watcher
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Create(_) => {
                            // NEW FILE CREATED!
                            for path in event.paths {
                                let ext = path.extension().and_then(|s| s.to_str());
                                // Watch JSON, JSONL, and Markdown files!
                                if ext == Some("json")
                                    || ext == Some("jsonl")
                                    || ext == Some("md")
                                    || ext == Some("markdown")
                                {
                                    println!("🆕 New file detected: {}", path.display());
                                    // For new files, always check them
                                    if let Ok(content) = fs::read_to_string(&path) {
                                        // Use smart project detection instead of simple contains
                                        if crate::mcp::smart_project_detector::contains_project_reference(&content, &project_name) {
                                            println!("   📎 Contains project reference! Absorbing...");
                                            let _ = sender.send(AbsorptionEvent::FileChanged(path));
                                        } else {
                                            println!("   ⏭️  No project references found");
                                        }
                                    }
                                }
                            }
                        }
                        EventKind::Modify(_) => {
                            // EXISTING FILE MODIFIED
                            for path in event.paths {
                                let ext = path.extension().and_then(|s| s.to_str());
                                if ext == Some("json")
                                    || ext == Some("jsonl")
                                    || ext == Some("md")
                                    || ext == Some("markdown")
                                {
                                    // Check if file might contain project name using smart detection
                                    if let Ok(content) = fs::read_to_string(&path) {
                                        if crate::mcp::smart_project_detector::contains_project_reference(&content, &project_name) {
                                            let _ = sender.send(AbsorptionEvent::FileChanged(path));
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            },
            Config::default(),
        )?;

        // Watch all configured paths
        for path in &self.watch_paths {
            if path.exists() {
                watcher.watch(path, RecursiveMode::Recursive)?;
                println!("👁️  Watching: {}", path.display());
            }
        }

        self.watcher = Some(watcher);

        // Start absorption thread
        self.start_absorption_thread();

        // Do initial scan of existing files (only those modified since last absorption)
        self.initial_scan()?;

        Ok(())
    }

    fn initial_scan(&self) -> Result<()> {
        println!("🔍 Initial scan for files modified since last absorption...");

        let last_time = *self.last_absorption_time.lock().unwrap();
        let mut files_to_check = Vec::new();

        // Scan watch paths for relevant files
        for watch_path in &self.watch_paths {
            if watch_path.is_dir() {
                // Find JSON, JSONL, and MD files
                let patterns = vec!["*.json", "*.jsonl", "*.md", "*.markdown"];
                for pattern in patterns {
                    let glob_pattern = format!("{}/{}", watch_path.display(), pattern);
                    if let Ok(paths) = glob::glob(&glob_pattern) {
                        for path_result in paths {
                            if let Ok(path) = path_result {
                                // Check modification time
                                if let Ok(metadata) = fs::metadata(&path) {
                                    if let Ok(modified) = metadata.modified() {
                                        if modified > last_time {
                                            files_to_check.push(path);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Also check subdirectories (one level deep for performance)
                    let recursive_pattern = format!("{}/*/{}", watch_path.display(), pattern);
                    if let Ok(paths) = glob::glob(&recursive_pattern) {
                        for path_result in paths.take(100) {
                            // Limit to avoid scanning too much
                            if let Ok(path) = path_result {
                                if let Ok(metadata) = fs::metadata(&path) {
                                    if let Ok(modified) = metadata.modified() {
                                        if modified > last_time {
                                            files_to_check.push(path);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        println!(
            "📊 Found {} files modified since last absorption",
            files_to_check.len()
        );

        // Process files that might contain project references
        let mut absorbed_count = 0;
        for path in files_to_check {
            // Quick check if file might contain project reference
            if let Ok(content) = fs::read_to_string(&path) {
                if crate::mcp::smart_project_detector::contains_project_reference(
                    &content,
                    &self.project_name,
                ) {
                    println!("   📎 Absorbing: {}", path.display());
                    let _ = self.sender.send(AbsorptionEvent::FileChanged(path));
                    absorbed_count += 1;
                }
            }
        }

        println!(
            "✅ Initial scan complete! Absorbed {} files",
            absorbed_count
        );

        // Update last absorption time
        *self.last_absorption_time.lock().unwrap() = SystemTime::now();

        Ok(())
    }

    fn start_absorption_thread(&mut self) {
        // Create a new channel for this thread
        let (tx, rx) = channel();
        self.sender = tx; // Update sender with new channel

        let project_name = self.project_name.clone();
        let contexts = self.absorbed_contexts.clone();
        let permissions = self.permissions.clone();

        thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                match event {
                    AbsorptionEvent::FileChanged(path) => {
                        if let Ok(context) = Self::absorb_file(&path, &project_name, &permissions) {
                            if permissions.notify_on_absorption {
                                println!("🧽 Absorbed context from: {}", path.display());
                                println!("   Relevance: {:.2}", context.relevance_score);
                            }

                            // Store context
                            if let Ok(mut ctx_lock) = contexts.lock() {
                                ctx_lock.push(context.clone());
                            }

                            // Write to M8 file
                            let _ = Self::append_to_m8(&context);
                        }
                    }
                    AbsorptionEvent::Stop => break,
                }
            }
        });
    }

    fn absorb_file(
        path: &Path,
        project_name: &str,
        permissions: &WatchPermissions,
    ) -> Result<AbsorbedContext> {
        // Check file size
        let metadata = fs::metadata(path)?;
        if metadata.len() > permissions.max_file_size_mb * 1024 * 1024 {
            return Err(anyhow::anyhow!("File too large"));
        }

        // Check if path is excluded
        for excluded in &permissions.excluded_paths {
            if path
                .to_string_lossy()
                .contains(excluded.trim_start_matches('*'))
            {
                return Err(anyhow::anyhow!("Path is excluded"));
            }
        }

        // Read file content
        let content = fs::read_to_string(path)?;

        // Determine file type and parse accordingly
        let ext = path.extension().and_then(|s| s.to_str());
        let (parsed_content, content_type) = match ext {
            Some("json") => {
                // Parse as JSON
                let json: Value = serde_json::from_str(&content)?;
                let relevant = Self::extract_relevant_content(&json, project_name);
                (relevant, Self::detect_content_type(&json))
            }
            Some("jsonl") => {
                // Parse as JSONL (JSON Lines - one JSON object per line)
                let relevant = Self::extract_jsonl_content(&content, project_name)?;
                (relevant, "jsonl_stream".to_string())
            }
            Some("md") | Some("markdown") => {
                // Parse Markdown - extract relevant sections
                let relevant = Self::extract_markdown_content(&content, project_name);
                (relevant, "markdown_document".to_string())
            }
            _ => {
                // Treat as plain text
                let relevant = Self::extract_text_content(&content, project_name);
                (relevant, "text_file".to_string())
            }
        };

        let keywords = Self::extract_keywords(&parsed_content);
        let relevance_score = Self::calculate_relevance(&parsed_content, project_name);

        Ok(AbsorbedContext {
            timestamp: SystemTime::now(),
            origin: path.to_path_buf(),
            project_name: project_name.to_string(),
            content_type,
            content: parsed_content,
            relevance_score,
            keywords,
        })
    }

    fn extract_relevant_content(json: &Value, project_name: &str) -> Value {
        let mut relevant = serde_json::json!({});

        // Recursively find all mentions of project name
        Self::find_mentions(json, project_name, &mut relevant);

        relevant
    }

    fn find_mentions(json: &Value, needle: &str, result: &mut Value) {
        match json {
            Value::Object(map) => {
                for (key, value) in map {
                    if key.contains(needle) || value.to_string().contains(needle) {
                        result[key] = value.clone();
                    }
                    Self::find_mentions(value, needle, result);
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    if item.to_string().contains(needle) {
                        if let Value::Array(ref mut res_arr) = result["mentions"] {
                            res_arr.push(item.clone());
                        } else {
                            result["mentions"] = serde_json::json!([item.clone()]);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn extract_keywords(content: &Value) -> Vec<String> {
        let mut keywords = HashSet::new();
        let text = content.to_string();

        // Simple keyword extraction (can be improved with NLP)
        for word in text.split_whitespace() {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
            if clean.len() > 4 && !STOP_WORDS.contains(&clean.to_lowercase().as_str()) {
                keywords.insert(clean.to_string());
            }
        }

        keywords.into_iter().collect()
    }

    fn calculate_relevance(content: &Value, project_name: &str) -> f64 {
        let text = content.to_string();
        let mentions = text.matches(project_name).count();
        let total_words = text.split_whitespace().count();

        if total_words == 0 {
            return 0.0;
        }

        // Simple relevance: mentions per 100 words, capped at 1.0
        ((mentions as f64 / total_words as f64) * 100.0).min(1.0)
    }

    fn detect_content_type(json: &Value) -> String {
        // Try to detect what kind of JSON this is - Including AI assistants!
        if json.get("conversations").is_some() {
            "claude_conversation".to_string()
        } else if json.get("messages").is_some() && json.get("model").is_some() {
            // Cursor AI chat format
            "cursor_ai_chat".to_string()
        } else if json.get("cells").is_some() && json.get("metadata").is_some() {
            // Jupyter notebook with AI interactions
            "jupyter_notebook".to_string()
        } else if json.get("entries").is_some() || json.get("chats").is_some() {
            // VS Code Copilot Chat or other VS Code AI extensions
            "vscode_ai_chat".to_string()
        } else if json.get("workspaceFolders").is_some() {
            "vscode_workspace".to_string()
        } else if json.get("dependencies").is_some() {
            "package_json".to_string()
        } else if json.get("config").is_some() {
            "configuration".to_string()
        } else if json.get("prompts").is_some() || json.get("completions").is_some() {
            // GitHub Copilot suggestions log
            "copilot_suggestions".to_string()
        } else {
            "generic_json".to_string()
        }
    }

    fn extract_jsonl_content(content: &str, project_name: &str) -> Result<Value> {
        let mut relevant_lines = Vec::new();

        // Parse each line as JSON
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Try to parse the line as JSON
            if let Ok(json) = serde_json::from_str::<Value>(line) {
                // Check if this line mentions the project
                if json.to_string().contains(project_name) {
                    relevant_lines.push(json);
                }
            }
        }

        Ok(serde_json::json!({
            "jsonl_entries": relevant_lines,
            "total_relevant": relevant_lines.len()
        }))
    }

    fn extract_markdown_content(content: &str, project_name: &str) -> Value {
        let mut sections = Vec::new();
        let mut current_section = String::new();
        let mut in_relevant_section = false;

        for line in content.lines() {
            // Check if line mentions project
            if line.contains(project_name) {
                in_relevant_section = true;
            }

            // Capture headers and relevant content
            if line.starts_with('#') || in_relevant_section {
                current_section.push_str(line);
                current_section.push('\n');

                // If we have a good chunk, save it
                if current_section.len() > 500 {
                    sections.push(current_section.clone());
                    current_section.clear();
                    in_relevant_section = false;
                }
            }
        }

        // Save any remaining content
        if !current_section.is_empty() {
            sections.push(current_section);
        }

        serde_json::json!({
            "markdown_sections": sections,
            "mentions_count": content.matches(project_name).count()
        })
    }

    fn extract_text_content(content: &str, project_name: &str) -> Value {
        // Extract lines that mention the project
        let relevant_lines: Vec<String> = content
            .lines()
            .filter(|line| line.contains(project_name))
            .map(|s| s.to_string())
            .collect();

        // Also get some context around mentions
        let mut context_snippets = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if line.contains(project_name) {
                let start = i.saturating_sub(2);
                let end = (i + 3).min(lines.len());
                let snippet = lines[start..end].join("\n");
                context_snippets.push(snippet);
            }
        }

        serde_json::json!({
            "relevant_lines": relevant_lines,
            "context_snippets": context_snippets,
            "total_mentions": content.matches(project_name).count()
        })
    }

    fn append_to_m8(context: &AbsorbedContext) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let st_dir = cwd.join(".st");
        let m8_path = st_dir.join("absorbed_context.m8");

        // Ensure directory exists
        fs::create_dir_all(&st_dir)?;

        // Append context to M8 file
        let mut existing = if m8_path.exists() {
            let content = fs::read_to_string(&m8_path)?;
            if content.is_empty() {
                Vec::new()
            } else {
                serde_json::from_str::<Vec<AbsorbedContext>>(&content).unwrap_or_default()
            }
        } else {
            Vec::new()
        };

        existing.push(context.clone());

        // Keep only last 100 contexts
        if existing.len() > 100 {
            let skip_count = existing.len().saturating_sub(100);
            existing = existing.into_iter().skip(skip_count).collect();
        }

        fs::write(&m8_path, serde_json::to_string_pretty(&existing)?)?;

        Ok(())
    }

    pub fn stop_watching(&mut self) {
        let _ = self.sender.send(AbsorptionEvent::Stop);
        self.watcher = None;
    }

    pub fn get_absorbed_contexts(&self) -> Vec<AbsorbedContext> {
        self.absorbed_contexts
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}

// Common stop words to ignore
const STOP_WORDS: &[&str] = &[
    "the", "and", "for", "with", "this", "that", "from", "into", "over", "under", "about",
    "through", "between", "after", "before", "during",
];

// MCP tool integration
pub async fn handle_context_absorber(params: Value) -> Result<Value> {
    let action = params["action"].as_str().unwrap_or("status");
    let project_name = params["project_name"].as_str().unwrap_or("smart-tree");

    match action {
        "start" => {
            let mut absorber = ContextAbsorber::new(project_name.to_string())?;
            absorber.start_watching()?;

            Ok(serde_json::json!({
                "status": "started",
                "project": project_name,
                "watching_paths": absorber.watch_paths,
                "message": format!("🧽 Context absorber started for '{}'", project_name)
            }))
        }
        "status" => {
            let cwd = std::env::current_dir()?;
            let m8_path = cwd.join(".st").join("absorbed_context.m8");

            let count = if m8_path.exists() {
                let content = fs::read_to_string(&m8_path)?;
                serde_json::from_str::<Vec<AbsorbedContext>>(&content)
                    .map(|v| v.len())
                    .unwrap_or(0)
            } else {
                0
            };

            Ok(serde_json::json!({
                "status": "ready",
                "project": project_name,
                "absorbed_contexts": count,
                "m8_file": m8_path.to_string_lossy()
            }))
        }
        "configure" => {
            let perm_path = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".mem8")
                .join("watch_permissions.json");

            Ok(serde_json::json!({
                "permissions_file": perm_path.to_string_lossy(),
                "current_permissions": WatchPermissions::default(),
                "message": "Edit the permissions file to configure watching"
            }))
        }
        _ => Err(anyhow::anyhow!("Unknown action: {}", action)),
    }
}
