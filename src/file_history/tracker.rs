//! File history tracker implementation
//!
//! 🎸 The Cheet says: "Track it, log it, never forget it!"

use super::*;
use anyhow::Context;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// File history log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Unix timestamp
    pub timestamp: u64,
    /// Target file path
    pub file_path: String,
    /// Operation performed
    pub operation: FileOperation,
    /// Operation context
    pub context: OperationContext,
    /// AI agent identifier
    pub agent: String,
    /// Session ID for grouping related operations
    pub session_id: String,
}

/// File history tracker
pub struct FileHistoryTracker {
    config: FileHistoryConfig,
    /// Cache of current log files
    log_cache: Arc<Mutex<HashMap<String, Vec<LogEntry>>>>,
}

impl FileHistoryTracker {
    /// Create new tracker with default config
    pub fn new() -> Result<Self> {
        Self::with_config(FileHistoryConfig::default())
    }

    /// Create tracker with custom config
    pub fn with_config(config: FileHistoryConfig) -> Result<Self> {
        if config.auto_create {
            fs::create_dir_all(&config.base_dir)
                .context("Failed to create file history directory")?;
        }

        Ok(Self {
            config,
            log_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Log a file operation
    pub fn log_operation(
        &self,
        file_path: &Path,
        operation: FileOperation,
        context: OperationContext,
        agent: &str,
        session_id: &str,
    ) -> Result<()> {
        let (time_bucket, timestamp) = get_time_bucket();
        let project_id = get_project_id(file_path)?;

        let entry = LogEntry {
            timestamp,
            file_path: file_path.to_string_lossy().to_string(),
            operation,
            context,
            agent: agent.to_string(),
            session_id: session_id.to_string(),
        };

        // Get log file path
        let log_dir = self.config.base_dir.join(&project_id);
        if self.config.auto_create {
            fs::create_dir_all(&log_dir)?;
        }

        let log_file = log_dir.join(format!("{}.flg", time_bucket));

        // Append to log file
        self.append_to_log(&log_file, &entry)?;

        // Update cache
        if let Ok(mut cache) = self.log_cache.lock() {
            let key = format!("{}/{}", project_id, time_bucket);
            cache.entry(key).or_insert_with(Vec::new).push(entry);
        }

        Ok(())
    }

    /// Append entry to log file
    fn append_to_log(&self, log_file: &Path, entry: &LogEntry) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)?;

        // Write as JSON lines format
        let json = serde_json::to_string(entry)?;
        writeln!(file, "{}", json)?;

        Ok(())
    }

    /// Track a file read operation
    pub fn track_read(&self, file_path: &Path, agent: &str, session_id: &str) -> Result<String> {
        let hash = hash_file(file_path)?;
        let size = fs::metadata(file_path)?.len() as usize;

        let context = OperationContext::new(FileOperation::Read)
            .with_bytes(size)
            .with_hashes(Some(hash.clone()), Some(hash.clone()));

        self.log_operation(file_path, FileOperation::Read, context, agent, session_id)?;
        Ok(hash)
    }

    /// Track a file write operation with smart operation detection
    pub fn track_write(
        &self,
        file_path: &Path,
        old_content: Option<&str>,
        new_content: &str,
        agent: &str,
        session_id: &str,
    ) -> Result<FileOperation> {
        let old_hash = old_content.map(|c| {
            let mut hasher = Sha256::new();
            hasher.update(c.as_bytes());
            format!("{:x}", hasher.finalize())
        });

        let mut hasher = Sha256::new();
        hasher.update(new_content.as_bytes());
        let new_hash = format!("{:x}", hasher.finalize());

        // Suggest best operation
        let operation = suggest_operation(old_content, new_content, self.config.prefer_append);

        let bytes_affected = match operation {
            FileOperation::Append => new_content.len() - old_content.map(|s| s.len()).unwrap_or(0),
            FileOperation::Create => new_content.len(),
            _ => new_content.len(),
        };

        let context = OperationContext::new(operation)
            .with_bytes(bytes_affected)
            .with_hashes(old_hash, Some(new_hash));

        self.log_operation(file_path, operation, context, agent, session_id)?;
        Ok(operation)
    }

    /// Get history for a specific file
    pub fn get_file_history(&self, file_path: &Path) -> Result<Vec<LogEntry>> {
        let project_id = get_project_id(file_path)?;
        let log_dir = self.config.base_dir.join(&project_id);

        if !log_dir.exists() {
            return Ok(Vec::new());
        }

        let mut all_entries = Vec::new();
        let target_path = file_path.to_string_lossy();

        // Read all log files in project directory
        for entry in fs::read_dir(&log_dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("flg") {
                let contents = fs::read_to_string(entry.path())?;
                for line in contents.lines() {
                    if let Ok(log_entry) = serde_json::from_str::<LogEntry>(line) {
                        if log_entry.file_path == target_path {
                            all_entries.push(log_entry);
                        }
                    }
                }
            }
        }

        // Sort by timestamp
        all_entries.sort_by_key(|e| e.timestamp);
        Ok(all_entries)
    }

    /// Get project summary
    pub fn get_project_summary(&self, project_path: &Path) -> Result<ProjectSummary> {
        let project_id = get_project_id(project_path)?;
        let log_dir = self.config.base_dir.join(&project_id);

        if !log_dir.exists() {
            return Ok(ProjectSummary::default());
        }

        let mut summary = ProjectSummary::default();
        let mut file_ops: HashMap<String, Vec<FileOperation>> = HashMap::new();

        // Read all log files
        for entry in fs::read_dir(&log_dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("flg") {
                let contents = fs::read_to_string(entry.path())?;
                for line in contents.lines() {
                    if let Ok(log_entry) = serde_json::from_str::<LogEntry>(line) {
                        summary.total_operations += 1;

                        file_ops
                            .entry(log_entry.file_path.clone())
                            .or_default()
                            .push(log_entry.operation);

                        summary
                            .operation_counts
                            .entry(log_entry.operation)
                            .and_modify(|c| *c += 1)
                            .or_insert(1);
                    }
                }
            }
        }

        summary.files_modified = file_ops.len();
        Ok(summary)
    }
}

/// Project summary statistics
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub total_operations: usize,
    pub files_modified: usize,
    pub operation_counts: HashMap<FileOperation, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_time_bucket() {
        let (bucket, _) = get_time_bucket();
        assert_eq!(bucket.len(), 13); // YYYYMMDD_HHMM
    }

    #[test]
    fn test_tracker_basic() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = FileHistoryConfig {
            base_dir: temp_dir.path().to_path_buf(),
            auto_create: true,
            prefer_append: true,
        };

        let tracker = FileHistoryTracker::with_config(config)?;
        let test_file = temp_dir.path().join("test.txt");

        // Create file and track
        fs::write(&test_file, "hello")?;
        let op = tracker.track_write(&test_file, None, "hello", "test-agent", "session-1")?;
        assert_eq!(op, FileOperation::Create);

        // Append and track
        let op = tracker.track_write(
            &test_file,
            Some("hello"),
            "hello world",
            "test-agent",
            "session-1",
        )?;
        assert_eq!(op, FileOperation::Append);

        Ok(())
    }
}
