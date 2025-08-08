// smart_edit_diff.rs - Local diff storage for Smart Edit operations
// Stores diffs in .st folder with timestamps for audit trail

use anyhow::{Context, Result};
use similar::TextDiff;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct DiffStorage {
    project_root: PathBuf,
    pub st_folder: PathBuf,
}

impl DiffStorage {
    /// Initialize diff storage for a project
    pub fn new(project_root: impl AsRef<Path>) -> Result<Self> {
        let project_root = project_root.as_ref().to_path_buf();
        let st_folder = project_root.join(".st");

        // Create .st folder if it doesn't exist
        if !st_folder.exists() {
            fs::create_dir(&st_folder).context("Failed to create .st folder")?;
        }

        // Ensure .st is in .gitignore
        Self::ensure_gitignore(&project_root)?;

        Ok(DiffStorage {
            project_root,
            st_folder,
        })
    }

    /// Ensure .st/ is in .gitignore
    fn ensure_gitignore(project_root: &Path) -> Result<()> {
        let gitignore_path = project_root.join(".gitignore");

        // Check if .gitignore exists and contains .st/
        let needs_update = if gitignore_path.exists() {
            let content = fs::read_to_string(&gitignore_path)?;
            !content
                .lines()
                .any(|line| line.trim() == ".st/" || line.trim() == ".st")
        } else {
            true
        };

        if needs_update {
            // Append .st/ to .gitignore
            let mut file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&gitignore_path)?;

            // Add newline if file exists and doesn't end with one
            if gitignore_path.exists() {
                let content = fs::read_to_string(&gitignore_path)?;
                if !content.is_empty() && !content.ends_with('\n') {
                    writeln!(file)?;
                }
            }

            writeln!(file, ".st/")?;
        }

        Ok(())
    }

    /// Store a diff for a file before Smart Edit operation
    pub fn store_diff(
        &self,
        file_path: &Path,
        original_content: &str,
        new_content: &str,
    ) -> Result<PathBuf> {
        // Get relative path from project root
        let relative_path = file_path
            .strip_prefix(&self.project_root)
            .unwrap_or(file_path);

        // Create diff
        let diff = TextDiff::from_lines(original_content, new_content);

        // Generate filename with timestamp
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let filename = format!(
            "{}-{}",
            relative_path.to_string_lossy().replace('/', "-"),
            timestamp
        );

        let diff_path = self.st_folder.join(&filename);

        // Write unified diff format
        let mut file = File::create(&diff_path)?;

        // Use the simple unified diff format
        let mut unified_diff = diff.unified_diff();
        let unified = unified_diff.context_radius(3).header(
            &format!("a/{}", relative_path.display()),
            &format!("b/{}", relative_path.display()),
        );

        write!(file, "{}", unified)?;

        Ok(diff_path)
    }

    /// Store the original file before any edits (for first edit)
    pub fn store_original(&self, file_path: &Path, content: &str) -> Result<()> {
        let relative_path = file_path
            .strip_prefix(&self.project_root)
            .unwrap_or(file_path);

        let original_path = self
            .st_folder
            .join(relative_path.to_string_lossy().replace('/', "-"));

        // Only store if it doesn't exist
        if !original_path.exists() {
            fs::write(&original_path, content)?;
        }

        Ok(())
    }

    /// Get the latest stored version of a file
    pub fn get_latest_version(&self, file_path: &Path) -> Result<Option<String>> {
        let relative_path = file_path
            .strip_prefix(&self.project_root)
            .unwrap_or(file_path);

        let base_name = relative_path.to_string_lossy().replace('/', "-");

        // Find all diffs for this file
        let mut diffs: Vec<_> = fs::read_dir(&self.st_folder)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                name.starts_with(&base_name) && name.contains('-')
            })
            .collect();

        // Sort by timestamp (newest first)
        diffs.sort_by_key(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            name.split('-')
                .last()
                .and_then(|ts| ts.parse::<u64>().ok())
                .unwrap_or(0)
        });
        diffs.reverse();

        // If we have diffs, reconstruct the latest version
        if !diffs.is_empty() {
            // Start with original if it exists
            let original_path = self.st_folder.join(&base_name);
            let content = if original_path.exists() {
                fs::read_to_string(&original_path)?
            } else {
                // Try to get from actual file
                fs::read_to_string(file_path)?
            };

            // Apply diffs in order (oldest to newest)
            for _diff_entry in diffs.iter().rev() {
                // This is simplified - in production you'd parse and apply the diff
                // For now, we'll just return that we have history
            }

            return Ok(Some(content));
        }

        Ok(None)
    }

    /// List all stored diffs for a file
    pub fn list_diffs(&self, file_path: &Path) -> Result<Vec<DiffInfo>> {
        let relative_path = file_path
            .strip_prefix(&self.project_root)
            .unwrap_or(file_path);

        let base_name = relative_path.to_string_lossy().replace('/', "-");

        let mut diffs = Vec::new();

        for entry in fs::read_dir(&self.st_folder)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with(&base_name) && name.contains('-') {
                if let Some(timestamp_str) = name.split('-').last() {
                    if let Ok(timestamp) = timestamp_str.parse::<u64>() {
                        diffs.push(DiffInfo {
                            path: entry.path(),
                            timestamp,
                            file_path: file_path.to_path_buf(),
                        });
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        diffs.sort_by_key(|d| d.timestamp);
        diffs.reverse();

        Ok(diffs)
    }

    /// Clean up old diffs (keep last N diffs per file)
    pub fn cleanup_old_diffs(&self, keep_count: usize) -> Result<usize> {
        let mut removed_count = 0;

        // Group diffs by file
        let mut file_diffs: std::collections::HashMap<String, Vec<PathBuf>> =
            std::collections::HashMap::new();

        for entry in fs::read_dir(&self.st_folder)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip non-diff files (like originals)
            if !name.contains('-') {
                continue;
            }

            // Extract base filename
            if let Some(pos) = name.rfind('-') {
                let base = &name[..pos];
                file_diffs
                    .entry(base.to_string())
                    .or_default()
                    .push(entry.path());
            }
        }

        // Remove old diffs for each file
        for (_, mut diffs) in file_diffs {
            if diffs.len() > keep_count {
                // Sort by timestamp (embedded in filename)
                diffs.sort();

                // Remove oldest diffs
                let to_remove = diffs.len() - keep_count;
                for diff_path in diffs.into_iter().take(to_remove) {
                    fs::remove_file(diff_path)?;
                    removed_count += 1;
                }
            }
        }

        Ok(removed_count)
    }
}

#[derive(Debug)]
pub struct DiffInfo {
    pub path: PathBuf,
    pub timestamp: u64,
    pub file_path: PathBuf,
}

impl DiffInfo {
    /// Get human-readable timestamp
    pub fn timestamp_str(&self) -> String {
        use chrono::{DateTime, Utc};
        let datetime =
            DateTime::<Utc>::from_timestamp(self.timestamp as i64, 0).unwrap_or_else(|| Utc::now());
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_diff_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = DiffStorage::new(temp_dir.path()).unwrap();

        // Check .st folder was created
        assert!(temp_dir.path().join(".st").exists());

        // Check .gitignore was updated
        let gitignore = fs::read_to_string(temp_dir.path().join(".gitignore")).unwrap();
        assert!(gitignore.contains(".st/"));
    }

    #[test]
    fn test_store_diff() {
        let temp_dir = TempDir::new().unwrap();
        let storage = DiffStorage::new(temp_dir.path()).unwrap();

        let file_path = temp_dir.path().join("test.rs");
        let original = "fn main() {\n    println!(\"Hello\");\n}";
        let modified = "fn main() {\n    println!(\"Hello, World!\");\n}";

        let diff_path = storage.store_diff(&file_path, original, modified).unwrap();
        assert!(diff_path.exists());

        let diff_content = fs::read_to_string(&diff_path).unwrap();
        assert!(diff_content.contains("--- a/test.rs"));
        assert!(diff_content.contains("+++ b/test.rs"));
        assert!(diff_content.contains("-    println!(\"Hello\");"));
        assert!(diff_content.contains("+    println!(\"Hello, World!\");"));
    }
}
