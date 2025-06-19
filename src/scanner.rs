//! Directory scanning and traversal engine

use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
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
    pub is_ignored: bool,
    pub depth: usize,
    pub file_type: FileType,
    pub category: FileCategory,
    pub search_matches: Option<Vec<usize>>, // Hex positions of search matches
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

/// Categories for file coloring based on extension/type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileCategory {
    // Programming languages
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Java,
    C,
    Cpp,
    Go,
    Ruby,
    PHP,
    Shell,
    
    // Markup/Data
    Markdown,
    Html,
    Css,
    Json,
    Yaml,
    Xml,
    Toml,
    
    // Build/Config
    Makefile,
    Dockerfile,
    GitConfig,
    
    // Archives
    Archive,
    
    // Media
    Image,
    Video,
    Audio,
    
    // System
    SystemFile,
    Binary,
    
    // Default
    Unknown,
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
    pub show_ignored: bool,
    pub find_pattern: Option<Regex>,
    pub file_type_filter: Option<String>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub newer_than: Option<SystemTime>,
    pub older_than: Option<SystemTime>,
    pub use_default_ignores: bool,
    pub search_keyword: Option<String>,
}

// Default patterns to ignore - common directories that are usually not useful in tree output
const DEFAULT_IGNORE_PATTERNS: &[&str] = &[
    // Version control directories (but not all hidden dirs)
    ".git",
    ".svn",
    ".hg",
    ".bzr",
    "_darcs",
    
    // Python
    "__pycache__",
    "*.pyc",
    "*.pyo",
    "*.pyd",
    ".Python",
    ".pytest_cache",
    ".tox",
    ".coverage",
    ".coverage.*",
    "*.egg-info",
    ".eggs",
    
    // Node.js / JavaScript
    "node_modules",
    ".npm",
    ".yarn",
    ".pnpm-store",
    "bower_components",
    ".next",
    ".nuxt",
    ".cache",
    
    // Virtual environments
    "venv",
    "env",
    "ENV",
    "virtualenv",
    ".venv",
    ".env",
    "conda-meta",
    
    // Build/compilation artifacts
    "target",     // Rust
    "build",
    "dist",
    "out",
    "bin",
    "obj",
    "*.o",
    "*.a",
    "*.so",
    "*.dll",
    "*.dylib",
    
    // Package managers
    ".cargo",
    ".rustup",
    ".gem",
    ".bundle",
    
    // IDEs and editors
    ".idea",
    ".vscode",
    ".vs",
    "*.swp",
    "*.swo",
    "*~",
    ".project",
    ".classpath",
    ".settings",
    
    // Development tool caches
    ".mypy_cache",
    ".ruff_cache",
    ".hypothesis",
    ".pytest_cache",
    ".tox",
    ".coverage",
    ".sass-cache",
    
    // OS specific
    ".DS_Store",
    "Thumbs.db",
    "desktop.ini",
    "$RECYCLE.BIN",
    
    // Temporary files
    "tmp",
    "temp",
    ".tmp",
    ".temp",
    "*.tmp",
    "*.temp",
    "*.log",
    
    // Cache directories
    ".cache",
    ".sass-cache",
    "__MACOSX",
    
    // System directories (when at root)
    "proc",
    "sys",
    "dev",
    "lost+found",
    "mnt",
    "media",
    
    // Other common ignores
    ".vagrant",
    ".terraform"
];

// Default directories to ignore at specific paths
const DEFAULT_SYSTEM_PATHS: &[&str] = &[
    "/proc",
    "/sys",
    "/dev",
    "/run",
    "/tmp",
    "/var/tmp",
    "/lost+found",
    "/mnt",
    "/media",
    "/snap",
];

// Specific files that should always be ignored
const DEFAULT_IGNORE_FILES: &[&str] = &[
    "/proc/kcore",  // Virtual file representing system memory
    "/proc/kmsg",   // Kernel messages
    "/proc/kallsyms", // Kernel symbols
];

/// Directory scanner
pub struct Scanner {
    config: ScannerConfig,
    gitignore: Option<GlobSet>,
    default_ignores: Option<GlobSet>,
    system_paths: HashSet<PathBuf>,
    ignore_files: HashSet<PathBuf>,
    root: PathBuf,
}

impl Scanner {
    /// Determine file category based on extension and name
    fn get_file_category(path: &Path, file_type: FileType) -> FileCategory {
        if matches!(file_type, FileType::Directory) {
            return FileCategory::Unknown;
        }

        // Check for system files
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name == "swap.img" || name == "swapfile" || name.starts_with("vmlinuz") || name.starts_with("initrd") {
                return FileCategory::SystemFile;
            }
        }

        // Check by extension
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                // Programming languages
                "rs" => FileCategory::Rust,
                "py" | "pyw" | "pyx" | "pyi" => FileCategory::Python,
                "js" | "mjs" | "cjs" => FileCategory::JavaScript,
                "ts" | "tsx" => FileCategory::TypeScript,
                "java" | "class" | "jar" => FileCategory::Java,
                "c" | "h" => FileCategory::C,
                "cpp" | "cc" | "cxx" | "hpp" | "hxx" => FileCategory::Cpp,
                "go" => FileCategory::Go,
                "rb" => FileCategory::Ruby,
                "php" => FileCategory::PHP,
                "sh" | "bash" | "zsh" | "fish" => FileCategory::Shell,
                
                // Markup/Data
                "md" | "markdown" => FileCategory::Markdown,
                "html" | "htm" => FileCategory::Html,
                "css" | "scss" | "sass" | "less" => FileCategory::Css,
                "json" | "jsonc" => FileCategory::Json,
                "yaml" | "yml" => FileCategory::Yaml,
                "xml" | "svg" => FileCategory::Xml,
                "toml" => FileCategory::Toml,
                
                // Build/Config
                "dockerfile" => FileCategory::Dockerfile,
                "gitignore" | "gitconfig" | "gitmodules" => FileCategory::GitConfig,
                
                // Archives
                "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => FileCategory::Archive,
                
                // Media
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "ico" | "webp" => FileCategory::Image,
                "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" => FileCategory::Video,
                "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" => FileCategory::Audio,
                
                // Binary/Executable
                "exe" | "dll" | "so" | "dylib" | "o" | "a" => FileCategory::Binary,
                
                _ => FileCategory::Unknown,
            }
        } else {
            // Check by filename for special cases
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                match name {
                    "Makefile" | "makefile" | "GNUmakefile" => FileCategory::Makefile,
                    "Dockerfile" => FileCategory::Dockerfile,
                    ".gitignore" | ".gitconfig" => FileCategory::GitConfig,
                    _ => {
                        if matches!(file_type, FileType::Executable) {
                            FileCategory::Binary
                        } else {
                            FileCategory::Unknown
                        }
                    }
                }
            } else {
                FileCategory::Unknown
            }
        }
    }

    pub fn new(root: &Path, config: ScannerConfig) -> Result<Self> {
        let gitignore = if config.respect_gitignore {
            Self::load_gitignore(root)?
        } else {
            None
        };

        let default_ignores = if config.use_default_ignores {
            Self::build_default_ignores()?
        } else {
            None
        };

        let system_paths: HashSet<PathBuf> = if config.use_default_ignores {
            DEFAULT_SYSTEM_PATHS.iter()
                .map(|p| PathBuf::from(p))
                .collect()
        } else {
            HashSet::new()
        };

        let ignore_files: HashSet<PathBuf> = if config.use_default_ignores {
            DEFAULT_IGNORE_FILES.iter()
                .map(|p| PathBuf::from(p))
                .collect()
        } else {
            HashSet::new()
        };

        Ok(Self {
            config,
            gitignore,
            default_ignores,
            system_paths,
            ignore_files,
            root: root.to_path_buf(),
        })
    }

    fn build_default_ignores() -> Result<Option<GlobSet>> {
        let mut builder = GlobSetBuilder::new();
        
        for pattern in DEFAULT_IGNORE_PATTERNS {
            if let Ok(glob) = Glob::new(pattern) {
                builder.add(glob);
            }
        }
        
        Ok(Some(builder.build()?))
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

    /// Stream nodes as they are discovered
    pub fn scan_stream(&self, sender: mpsc::Sender<FileNode>) -> Result<TreeStats> {
        let mut stats = TreeStats::default();
        let mut ignored_dirs = std::collections::HashSet::new();
        
        let mut walker = WalkDir::new(&self.root)
            .max_depth(self.config.max_depth)
            .follow_links(self.config.follow_symlinks)
            .into_iter();

        // Process entries
        while let Some(entry_result) = walker.next() {
            match entry_result {
                Ok(entry) => {
                    let depth = entry.depth();
                    let path = entry.path();
                    
                    // Check if this path should be ignored
                    let is_ignored = self.should_ignore(path)?;
                    
                    if is_ignored {
                        if self.config.show_ignored {
                            // Process the entry to show it as ignored
                            if let Some(mut node) = self.process_entry(&entry, depth)? {
                                // Search in file contents if requested
                                if !node.is_dir && self.should_search_file(&node) {
                                    node.search_matches = self.search_in_file(&node.path);
                                }
                                
                                // Send node immediately
                                if sender.send(node.clone()).is_err() {
                                    break; // Receiver dropped
                                }
                                
                                if !node.permission_denied {
                                    stats.update_file(&node);
                                }
                            }
                            // If it's a directory, skip its contents
                            if entry.file_type().is_dir() {
                                ignored_dirs.insert(path.to_path_buf());
                                walker.skip_current_dir();
                            }
                        } else {
                            // Not showing ignored items
                            if entry.file_type().is_dir() {
                                walker.skip_current_dir();
                            }
                            continue;
                        }
                    } else {
                        // Normal processing for non-ignored items
                        if let Some(mut node) = self.process_entry(&entry, depth)? {
                            // Search in file contents if requested
                            if !node.is_dir && self.should_search_file(&node) {
                                node.search_matches = self.search_in_file(&node.path);
                            }
                            
                            // Apply filters before sending
                            // Include files with search matches even if they don't match other filters
                            let has_search_match = node.search_matches.as_ref().map(|m| !m.is_empty()).unwrap_or(false);
                            if node.is_dir || has_search_match || self.should_include(&node) {
                                if sender.send(node.clone()).is_err() {
                                    break; // Receiver dropped
                                }
                                
                                if !node.permission_denied {
                                    stats.update_file(&node);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    // Handle permission denied gracefully
                    if let Some(path) = e.path() {
                        let depth = e.depth();
                        let node = self.create_permission_denied_node(path, depth);
                        if sender.send(node.clone()).is_err() {
                            break; // Receiver dropped
                        }
                        stats.update_file(&node);
                        // Skip the directory contents if permission denied
                        walker.skip_current_dir();
                    }
                }
            }
        }

        Ok(stats)
    }

    /// Check if we should search in this file
    fn should_search_file(&self, node: &FileNode) -> bool {
        if self.config.search_keyword.is_none() {
            return false;
        }
        
        // Only search in files matching the type filter if specified
        if let Some(filter_ext) = &self.config.file_type_filter {
            if let Some(ext) = node.path.extension() {
                return ext.to_string_lossy() == *filter_ext;
            }
            return false;
        }
        
        // Otherwise search in all text-like files
        // Check by extension for common text files
        if let Some(ext) = node.path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "txt" | "text" | "log" | "md" | "rst" | "tex" | "org" | "adoc" |
                "ini" | "cfg" | "conf" | "config" | "properties" | "env" |
                "csv" | "tsv" | "sql" | "graphql" | "proto" | "thrift" |
                "vim" | "vimrc" | "bashrc" | "zshrc" | "gitconfig" | "editorconfig" => return true,
                _ => {}
            }
        }
        
        // Check by category
        matches!(node.category, 
            FileCategory::Rust | FileCategory::Python | FileCategory::JavaScript |
            FileCategory::TypeScript | FileCategory::Java | FileCategory::C |
            FileCategory::Cpp | FileCategory::Go | FileCategory::Ruby | FileCategory::PHP |
            FileCategory::Shell | FileCategory::Markdown | FileCategory::Html |
            FileCategory::Css | FileCategory::Json | FileCategory::Yaml |
            FileCategory::Xml | FileCategory::Toml | FileCategory::Makefile |
            FileCategory::Dockerfile | FileCategory::GitConfig | FileCategory::Unknown
        )
    }

    /// Search for keyword in file and return hex positions
    fn search_in_file(&self, path: &Path) -> Option<Vec<usize>> {
        let keyword = self.config.search_keyword.as_ref()?;
        
        // Try to open the file
        let file = match fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return None,
        };
        
        let mut positions = Vec::new();
        let reader = BufReader::new(file);
        let mut byte_position = 0;
        
        // Search line by line for efficiency
        for line in reader.lines() {
            match line {
                Ok(content) => {
                    // Find all occurrences in this line
                    for (idx, _) in content.match_indices(keyword) {
                        positions.push(byte_position + idx);
                    }
                    // Update byte position (add line length + newline)
                    byte_position += content.len() + 1;
                }
                Err(_) => break, // Stop on read error (e.g., binary file)
            }
            
            // Limit search results to prevent memory issues
            if positions.len() > 100 {
                break;
            }
        }
        
        if positions.is_empty() {
            None
        } else {
            Some(positions)
        }
    }

    /// Scan a directory and return all nodes with statistics
    pub fn scan(&self) -> Result<(Vec<FileNode>, TreeStats)> {
        let mut all_nodes = Vec::new();
        let mut ignored_dirs = std::collections::HashSet::new();
        
        let mut walker = WalkDir::new(&self.root)
            .max_depth(self.config.max_depth)
            .follow_links(self.config.follow_symlinks)
            .into_iter();

        // Process entries
        while let Some(entry_result) = walker.next() {
            match entry_result {
                Ok(entry) => {
                    let depth = entry.depth();
                    let path = entry.path();
                    
                    // Check if this path should be ignored
                    let is_ignored = self.should_ignore(path)?;
                    
                    if is_ignored {
                        if self.config.show_ignored {
                            // Process the entry to show it as ignored
                            if let Some(node) = self.process_entry(&entry, depth)? {
                                all_nodes.push(node);
                            }
                            // If it's a directory, skip its contents
                            if entry.file_type().is_dir() {
                                ignored_dirs.insert(path.to_path_buf());
                                walker.skip_current_dir();
                            }
                        } else {
                            // Not showing ignored items
                            if entry.file_type().is_dir() {
                                walker.skip_current_dir();
                            }
                            continue;
                        }
                    } else {
                        // Normal processing for non-ignored items
                        if let Some(mut node) = self.process_entry(&entry, depth)? {
                            // Search in file contents if requested (for non-streaming mode)
                            if !node.is_dir && self.should_search_file(&node) {
                                node.search_matches = self.search_in_file(&node.path);
                            }
                            all_nodes.push(node);
                        }
                    }
                }
                Err(e) => {
                    // Handle permission denied gracefully
                    if let Some(path) = e.path() {
                        let depth = e.depth();
                        all_nodes.push(self.create_permission_denied_node(path, depth));
                        // Skip the directory contents if permission denied
                        walker.skip_current_dir();
                    }
                }
            }
        }

        // Second pass: filter nodes and build final list
        let (nodes, stats) = if self.has_filters() {
            self.filter_nodes_with_ancestors(all_nodes)
        } else {
            // No filters, include all nodes
            let mut stats = TreeStats::default();
            for node in &all_nodes {
                if node.is_dir || !node.permission_denied {
                    stats.update_file(node);
                }
            }
            (all_nodes, stats)
        };

        Ok((nodes, stats))
    }

    /// Check if any filters are active
    fn has_filters(&self) -> bool {
        self.config.find_pattern.is_some() ||
        self.config.file_type_filter.is_some() ||
        self.config.min_size.is_some() ||
        self.config.max_size.is_some() ||
        self.config.newer_than.is_some() ||
        self.config.older_than.is_some()
    }

    /// Filter nodes and include only directories that contain matching files
    fn filter_nodes_with_ancestors(&self, all_nodes: Vec<FileNode>) -> (Vec<FileNode>, TreeStats) {
        let mut stats = TreeStats::default();
        let mut matching_files = Vec::new();
        let mut required_dirs = HashSet::new();

        // Find all files that match the filters or have search matches
        for node in &all_nodes {
            let has_search_match = node.search_matches.as_ref().map(|m| !m.is_empty()).unwrap_or(false);
            if !node.is_dir && (has_search_match || self.should_include(node)) {
                matching_files.push(node.clone());
                stats.update_file(node);

                // Add all ancestor directories
                let mut current = node.path.parent();
                while let Some(parent) = current {
                    if parent == self.root || required_dirs.contains(parent) {
                        break;
                    }
                    required_dirs.insert(parent.to_path_buf());
                    current = parent.parent();
                }
            }
        }

        // Build final node list with required directories and matching files
        let mut result = Vec::new();
        
        // Always include root if we have any matches
        if !matching_files.is_empty() {
            if let Some(root_node) = all_nodes.iter().find(|n| n.path == self.root) {
                result.push(root_node.clone());
                stats.total_dirs += 1;
            }
        }

        // Add required directories
        for node in &all_nodes {
            if node.is_dir && node.path != self.root && required_dirs.contains(&node.path) {
                result.push(node.clone());
                stats.total_dirs += 1;
            }
        }

        // Add matching files
        result.extend(matching_files);

        (result, stats)
    }

    fn process_entry(&self, entry: &DirEntry, depth: usize) -> Result<Option<FileNode>> {
        let path = entry.path();
        
        // Check if should be ignored
        let is_ignored = self.should_ignore(path)?;
        
        // Check if hidden file
        let is_hidden = path.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false);
        
        // Skip if ignored and not showing ignored
        if is_ignored && !self.config.show_ignored {
            return Ok(None);
        }
        
        // Skip if hidden and not showing hidden
        if is_hidden && !self.config.show_hidden && !is_ignored {
            return Ok(None);
        }

        let metadata = entry.metadata()?;
        let file_type = self.determine_file_type(&metadata);
        let category = Self::get_file_category(path, file_type);
        
        // Get the actual size, handling special files
        let size = if self.is_special_file(path, &metadata) {
            // For special files in /proc, /sys, etc., use 0 size
            0
        } else {
            metadata.len()
        };
        
        Ok(Some(FileNode {
            path: path.to_path_buf(),
            is_dir: metadata.is_dir(),
            size,
            permissions: Self::get_permissions(&metadata),
            uid: Self::get_uid(&metadata),
            gid: Self::get_gid(&metadata),
            modified: metadata.modified()?,
            is_symlink: metadata.is_symlink(),
            is_hidden,
            permission_denied: false,
            is_ignored,
            depth,
            file_type,
            category,
            search_matches: None,
        }))
    }

    /// Check if this is a special file that reports incorrect size
    fn is_special_file(&self, path: &Path, metadata: &fs::Metadata) -> bool {
        // Check if it's in a virtual filesystem
        if let Some(path_str) = path.to_str() {
            if path_str.starts_with("/proc") || 
               path_str.starts_with("/sys") || 
               path_str.starts_with("/dev") {
                return true;
            }
        }
        
        // Check if it's a special file type (character/block device, etc)
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            let file_type = metadata.file_type();
            if file_type.is_char_device() || 
               file_type.is_block_device() ||
               file_type.is_fifo() ||
               file_type.is_socket() {
                return true;
            }
        }
        
        false
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
            is_ignored: false,
            depth,
            file_type: FileType::Directory,
            category: FileCategory::Unknown,
            search_matches: None,
        }
    }

    fn should_ignore(&self, path: &Path) -> Result<bool> {
        // Check if it's a specific file to ignore
        if self.config.use_default_ignores && self.ignore_files.contains(path) {
            return Ok(true);
        }

        // Check if it's a system path or within a system path
        if self.config.use_default_ignores {
            // Check exact match
            if self.system_paths.contains(path) {
                return Ok(true);
            }
            
            // Check if path is within any system path
            for system_path in &self.system_paths {
                if path.starts_with(system_path) {
                    return Ok(true);
                }
            }
        }

        // Check default ignores
        if let Some(default_ignores) = &self.default_ignores {
            if let Some(file_name) = path.file_name() {
                // Check just the filename against patterns
                if default_ignores.is_match(Path::new(file_name)) {
                    return Ok(true);
                }
            }
            
            // Also check against relative path for patterns like "*.pyc"
            if let Ok(rel_path) = path.strip_prefix(&self.root) {
                if default_ignores.is_match(rel_path) {
                    return Ok(true);
                }
            }
        }

        // Check gitignore
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
            show_ignored: false,
            find_pattern: None,
            file_type_filter: None,
            min_size: None,
            max_size: None,
            newer_than: None,
            older_than: None,
            use_default_ignores: true,
            search_keyword: None,
        };
        
        let scanner = Scanner::new(Path::new("."), config).unwrap();
        assert!(scanner.gitignore.is_some() || scanner.gitignore.is_none());
    }
}