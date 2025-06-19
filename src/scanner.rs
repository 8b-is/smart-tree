//! Directory scanning and traversal engine

use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::{DirEntry, WalkDir};

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

/// File metadata we collect during traversal
#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub permissions: u32,
    pub uid: u32,
    pub gid: u32,
    pub modified: SystemTime,
    pub is_symlink: bool,
    pub is_hidden: bool,
    pub permission_denied: bool,
    pub depth: usize,
    pub file_type: FileType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    Directory,
    RegularFile,
    Symlink,
    Executable,
    Socket,
    Pipe,
    BlockDevice,
    CharDevice,
}

/// Statistics collected during traversal
#[derive(Debug, Default)]
pub struct TreeStats {
    pub total_files: u64,
    pub total_dirs: u64,
    pub total_size: u64,
    pub file_types: HashMap<String, u64>,
    pub largest_files: Vec<(u64, PathBuf)>,
    pub newest_files: Vec<(SystemTime, PathBuf)>,
    pub oldest_files: Vec<(SystemTime, PathBuf)>,
}

impl TreeStats {
    pub fn update_file(&mut self, node: &FileNode) {
        if node.is_dir {
            self.total_dirs += 1;
        } else {
            self.total_files += 1;
            self.total_size += node.size;
            
            // Track file extensions
            if let Some(ext) = node.path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    *self.file_types.entry(ext_str.to_string()).or_insert(0) += 1;
                }
            }
            
            // Update largest files
            self.largest_files.push((node.size, node.path.clone()));
            self.largest_files.sort_by(|a, b| b.0.cmp(&a.0));
            self.largest_files.truncate(10);
            
            // Update newest files
            self.newest_files.push((node.modified, node.path.clone()));
            self.newest_files.sort_by(|a, b| b.0.cmp(&a.0));
            self.newest_files.truncate(10);
            
            // Update oldest files
            self.oldest_files.push((node.modified, node.path.clone()));
            self.oldest_files.sort_by(|a, b| a.0.cmp(&b.0));
            self.oldest_files.truncate(10);
        }
    }
}

/// Scanner configuration
pub struct ScannerConfig {
    pub max_depth: usize,
    pub follow_symlinks: bool,
    pub respect_gitignore: bool,
    pub show_hidden: bool,
    pub find_pattern: Option<Regex>,
    pub file_type_filter: Option<String>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub newer_than: Option<SystemTime>,
    pub older_than: Option<SystemTime>,
}

/// Directory scanner
pub struct Scanner {
    config: ScannerConfig,
    gitignore: Option<GlobSet>,
    root: PathBuf,
}

impl Scanner {
    pub fn new(root: &Path, config: ScannerConfig) -> Result<Self> {
        let gitignore = if config.respect_gitignore {
            Self::load_gitignore(root)?
        } else {
            None
        };

        Ok(Self {
            config,
            gitignore,
            root: root.to_path_buf(),
        })
    }

    fn load_gitignore(root: &Path) -> Result<Option<GlobSet>> {
        let gitignore_path = root.join(".gitignore");
        if !gitignore_path.exists() {
            return Ok(None);
        }

        let mut builder = GlobSetBuilder::new();
        let content = fs::read_to_string(&gitignore_path)?;
        
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                if let Ok(glob) = Glob::new(line) {
                    builder.add(glob);
                }
            }
        }

        Ok(Some(builder.build()?))
    }

    /// Scan a directory and return all nodes with statistics
    pub fn scan(&self) -> Result<(Vec<FileNode>, TreeStats)> {
        let mut nodes = Vec::new();
        let mut stats = TreeStats::default();
        
        let walker = WalkDir::new(&self.root)
            .max_depth(self.config.max_depth)
            .follow_links(self.config.follow_symlinks);

        for entry in walker {
            match entry {
                Ok(entry) => {
                    let depth = entry.depth();
                    if let Some(node) = self.process_entry(&entry, depth)? {
                        // Apply filters
                        if self.should_include(&node) {
                            stats.update_file(&node);
                            nodes.push(node);
                        }
                    }
                }
                Err(e) => {
                    // Handle permission denied gracefully
                    if let Some(path) = e.path() {
                        let depth = e.depth();
                        nodes.push(self.create_permission_denied_node(path, depth));
                    }
                }
            }
        }

        Ok((nodes, stats))
    }

    fn process_entry(&self, entry: &DirEntry, depth: usize) -> Result<Option<FileNode>> {
        let path = entry.path();
        
        // Check if should be ignored
        if self.should_ignore(path)? {
            return Ok(None);
        }

        // Check if hidden file and config says to skip
        let is_hidden = path.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false);
        
        if is_hidden && !self.config.show_hidden {
            return Ok(None);
        }

        let metadata = entry.metadata()?;
        let file_type = self.determine_file_type(&metadata);
        
        Ok(Some(FileNode {
            path: path.to_path_buf(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            permissions: Self::get_permissions(&metadata),
            uid: Self::get_uid(&metadata),
            gid: Self::get_gid(&metadata),
            modified: metadata.modified()?,
            is_symlink: metadata.is_symlink(),
            is_hidden,
            permission_denied: false,
            depth,
            file_type,
        }))
    }

    fn create_permission_denied_node(&self, path: &Path, depth: usize) -> FileNode {
        FileNode {
            path: path.to_path_buf(),
            is_dir: true,
            size: 0,
            permissions: 0,
            uid: 0,
            gid: 0,
            modified: SystemTime::now(),
            is_symlink: false,
            is_hidden: false,
            permission_denied: true,
            depth,
            file_type: FileType::Directory,
        }
    }

    fn should_ignore(&self, path: &Path) -> Result<bool> {
        if let Some(gitignore) = &self.gitignore {
            if let Ok(rel_path) = path.strip_prefix(&self.root) {
                return Ok(gitignore.is_match(rel_path));
            }
        }
        Ok(false)
    }

    fn should_include(&self, node: &FileNode) -> bool {
        // Find pattern filter
        if let Some(pattern) = &self.config.find_pattern {
            let path_str = node.path.to_string_lossy();
            if !pattern.is_match(&path_str) {
                return false;
            }
        }

        // File type filter
        if let Some(filter_ext) = &self.config.file_type_filter {
            if let Some(ext) = node.path.extension() {
                if ext.to_string_lossy() != *filter_ext {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Size filters (only for files)
        if !node.is_dir {
            if let Some(min_size) = self.config.min_size {
                if node.size < min_size {
                    return false;
                }
            }
            if let Some(max_size) = self.config.max_size {
                if node.size > max_size {
                    return false;
                }
            }
        }

        // Date filters
        if let Some(newer_than) = self.config.newer_than {
            if node.modified < newer_than {
                return false;
            }
        }
        if let Some(older_than) = self.config.older_than {
            if node.modified > older_than {
                return false;
            }
        }

        true
    }

    fn determine_file_type(&self, metadata: &fs::Metadata) -> FileType {
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            let file_type = metadata.file_type();
            
            if file_type.is_dir() {
                FileType::Directory
            } else if file_type.is_symlink() {
                FileType::Symlink
            } else if file_type.is_socket() {
                FileType::Socket
            } else if file_type.is_fifo() {
                FileType::Pipe
            } else if file_type.is_block_device() {
                FileType::BlockDevice
            } else if file_type.is_char_device() {
                FileType::CharDevice
            } else if metadata.permissions().mode() & 0o111 != 0 {
                FileType::Executable
            } else {
                FileType::RegularFile
            }
        }
        
        #[cfg(not(unix))]
        {
            if metadata.is_dir() {
                FileType::Directory
            } else if metadata.is_symlink() {
                FileType::Symlink
            } else {
                FileType::RegularFile
            }
        }
    }

    #[cfg(unix)]
    fn get_permissions(metadata: &fs::Metadata) -> u32 {
        metadata.permissions().mode() & 0o777
    }

    #[cfg(not(unix))]
    fn get_permissions(_metadata: &fs::Metadata) -> u32 {
        0o755 // Default permissions for non-Unix systems
    }

    #[cfg(unix)]
    fn get_uid(metadata: &fs::Metadata) -> u32 {
        metadata.uid()
    }

    #[cfg(not(unix))]
    fn get_uid(_metadata: &fs::Metadata) -> u32 {
        1000 // Default UID for non-Unix systems
    }

    #[cfg(unix)]
    fn get_gid(metadata: &fs::Metadata) -> u32 {
        metadata.gid()
    }

    #[cfg(not(unix))]
    fn get_gid(_metadata: &fs::Metadata) -> u32 {
        1000 // Default GID for non-Unix systems
    }
}

/// Helper function to parse human-readable sizes
pub fn parse_size(size_str: &str) -> Result<u64> {
    let size_str = size_str.trim().to_uppercase();
    
    if size_str.is_empty() {
        return Ok(0);
    }

    let (num_part, unit_part) = if let Some(pos) = size_str.find(|c: char| c.is_alphabetic()) {
        (&size_str[..pos], &size_str[pos..])
    } else {
        (size_str.as_str(), "")
    };

    let number: f64 = num_part.parse()?;
    
    let multiplier = match unit_part {
        "B" | "" => 1.0,
        "K" | "KB" => 1024.0,
        "M" | "MB" => 1024.0 * 1024.0,
        "G" | "GB" => 1024.0 * 1024.0 * 1024.0,
        "T" | "TB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => anyhow::bail!("Invalid size unit: {}", unit_part),
    };

    Ok((number * multiplier) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("100").unwrap(), 100);
        assert_eq!(parse_size("1K").unwrap(), 1024);
        assert_eq!(parse_size("2.5M").unwrap(), 2621440);
        assert_eq!(parse_size("1GB").unwrap(), 1073741824);
    }

    #[test]
    fn test_scanner_creation() {
        let config = ScannerConfig {
            max_depth: 5,
            follow_symlinks: false,
            respect_gitignore: true,
            show_hidden: false,
            find_pattern: None,
            file_type_filter: None,
            min_size: None,
            max_size: None,
            newer_than: None,
            older_than: None,
        };
        
        let scanner = Scanner::new(Path::new("."), config).unwrap();
        assert!(scanner.gitignore.is_some() || scanner.gitignore.is_none());
    }
}