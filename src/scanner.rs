//
// -----------------------------------------------------------------------------
//  WELCOME TO THE JUNGLE! ...The filesystem jungle, that is. 🌴
//
//  You've found scanner.rs, the intrepid explorer and engine room of st.
//  This module is the Indiana Jones of our codebase. It bravely dives into
//  the deepest, darkest directories, dodges `.gitignore` traps, inspects
//  every file for treasure (metadata), and reports back its findings.
//
//  So grab your hat, and let's go on an adventure!
//
//  Brought to you by The Cheet - making filesystem traversal a rock concert! 🥁🧻
// -----------------------------------------------------------------------------
//

use anyhow::Result;
use crate::scanner_safety::{ScannerSafetyLimits, ScannerSafetyTracker, estimate_node_size};
use globset::{Glob, GlobSet, GlobSetBuilder}; // For powerful gitignore-style pattern matching.
use regex::Regex; // For user-defined find patterns.
use std::collections::{HashMap, HashSet}; // Our trusty hash-based collections.
use std::fs; // Filesystem operations, the bread and butter here.
use std::io::{BufRead, BufReader}; // For efficient reading, especially for content search.
use std::path::{Path, PathBuf}; // Path manipulation is key.
use std::sync::mpsc; // For streaming results from a worker thread.
use std::time::SystemTime; // To know when files were last touched.
use walkdir::{DirEntry, WalkDir}; // The excellent `walkdir` crate does the actual directory walking.

// Unix-specific imports for richer metadata like permissions, UID, GID.
// On other platforms, we'll use sensible defaults.
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

/// # FileNode: The Ultimate Backstage Pass
///
/// Every file and directory we meet gets one of these. It's a VIP pass that
/// holds all the juicy details: its name, size, when it was last cool (modified),
/// and whether it's on the super-secret "ignored" list. It's the atom of our
/// `st` universe.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileNode {
    /// The full path to the file or directory. The source of truth for location!
    pub path: PathBuf,
    /// Is it a directory? `true` if yes, `false` if it's a file or symlink.
    pub is_dir: bool,
    /// Size of the file in bytes. For directories, this is often 0 or metadata-dependent.
    pub size: u64,
    /// File permissions (e.g., `rwxr-xr-x`). Stored as a u32, typically from Unix mode.
    pub permissions: u32,
    /// User ID of the owner (Unix-specific).
    pub uid: u32,
    /// Group ID of the owner (Unix-specific).
    pub gid: u32,
    /// Timestamp of the last modification. Tells us how fresh or ancient a file is.
    pub modified: SystemTime,
    /// Is it a symbolic link? `true` if yes. We handle these with care.
    pub is_symlink: bool,
    /// Is it a hidden file (e.g., starts with a `.` on Unix)?
    pub is_hidden: bool,
    /// Did we encounter a "Permission Denied" error when trying to access this?
    /// Important for gracefully handling parts of the filesystem we can't read.
    pub permission_denied: bool,
    /// Is this file or directory ignored based on `.gitignore` or default ignore rules?
    pub is_ignored: bool,
    /// The depth of this entry relative to the scan root (root is depth 0).
    pub depth: usize,
    /// The specific type of the file (e.g., RegularFile, Symlink, Executable).
    pub file_type: FileType,
    /// A category assigned based on extension or name, used for coloring and context.
    /// (e.g., Rust, Python, Image, Archive).
    pub category: FileCategory,
    /// For content search: Information about where matches were found
    /// `None` if no search was performed or no matches.
    pub search_matches: Option<SearchMatches>,
    /// The filesystem type this file resides on
    pub filesystem_type: FilesystemType,
}

/// Information about search matches within a file
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchMatches {
    /// First match position (line, column)
    pub first_match: (usize, usize),
    /// Total number of matches found
    pub total_count: usize,
    /// List of all match positions (line, column) - limited to prevent memory issues
    pub positions: Vec<(usize, usize)>,
    /// Whether the search was truncated due to too many matches
    pub truncated: bool,
    /// Line content for each match (line number, line content, column) - optional for compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_content: Option<Vec<(usize, String, usize)>>,
}

/// # FileType: Distinguishing Different Kinds of Filesystem Objects
///
/// This enum helps us categorize entries beyond just "file" or "directory".
/// It's especially useful on Unix-like systems where you have sockets, pipes, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum FileType {
    Directory,   // A folder, a container of other things.
    RegularFile, // Your everyday, garden-variety file.
    Symlink,     // A pointer to another file or directory.
    Executable,  // A file that can be run (has execute permissions).
    Socket,      // A Unix domain socket.
    Pipe,        // A named pipe (FIFO).
    BlockDevice, // A block special file (e.g., /dev/sda).
    CharDevice,  // A character special file (e.g., /dev/tty).
}

/// # FilesystemType: Identifying the underlying filesystem
///
/// This enum represents different filesystem types with single-character codes
/// for compact display. The mapping is designed to be memorable and intuitive.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FilesystemType {
    Ext4,    // '4' - The most common Linux filesystem
    Ext3,    // '3' - Older ext filesystem
    Ext2,    // '2' - Even older ext filesystem
    Xfs,     // 'X' - XFS filesystem
    Btrfs,   // 'B' - Btrfs (B-tree filesystem)
    Zfs,     // 'Z' - ZFS filesystem
    Ntfs,    // 'N' - Windows NTFS
    Fat32,   // 'F' - FAT32
    ExFat,   // 'E' - exFAT
    Apfs,    // 'A' - Apple File System
    Hfs,     // 'H' - HFS+ (older Mac)
    Nfs,     // 'R' - Remote NFS mount
    Smb,     // 'S' - SMB/CIFS network filesystem
    Tmpfs,   // 'T' - Temporary filesystem (RAM)
    Procfs,  // 'P' - /proc virtual filesystem
    Sysfs,   // 'Y' - /sys virtual filesystem
    Devfs,   // 'D' - /dev virtual filesystem
    Mem8,    // 'M' - MEM|8 filesystem (Coming soon - Quantum File System) - https://m8.is
    Unknown, // '?' - Unknown filesystem
}

impl FilesystemType {
    /// Get the single-character code for this filesystem type
    pub fn to_char(&self) -> char {
        match self {
            FilesystemType::Ext4 => '4',
            FilesystemType::Ext3 => '3',
            FilesystemType::Ext2 => '2',
            FilesystemType::Xfs => 'X',
            FilesystemType::Btrfs => 'B',
            FilesystemType::Zfs => 'Z',
            FilesystemType::Ntfs => 'N',
            FilesystemType::Fat32 => 'F',
            FilesystemType::ExFat => 'E',
            FilesystemType::Apfs => 'A',
            FilesystemType::Hfs => 'H',
            FilesystemType::Nfs => 'R',
            FilesystemType::Smb => 'S',
            FilesystemType::Tmpfs => 'T',
            FilesystemType::Procfs => 'P',
            FilesystemType::Sysfs => 'Y',
            FilesystemType::Devfs => 'D',
            FilesystemType::Mem8 => 'M',
            FilesystemType::Unknown => '?',
        }
    }

    /// Check if this is a virtual filesystem that should be skipped
    pub fn is_virtual(&self) -> bool {
        matches!(
            self,
            FilesystemType::Procfs
                | FilesystemType::Sysfs
                | FilesystemType::Devfs
                | FilesystemType::Tmpfs
        )
    }

    /// Check if this filesystem type should be shown by default
    /// (only "interesting" filesystems based on platform)
    pub fn should_show_by_default(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            matches!(
                self,
                FilesystemType::Ext4
                    | FilesystemType::Ext3
                    | FilesystemType::Xfs
                    | FilesystemType::Btrfs
                    | FilesystemType::Zfs
                    | FilesystemType::Nfs
                    | FilesystemType::Smb
                    | FilesystemType::Mem8
            )
        }
        #[cfg(target_os = "macos")]
        {
            matches!(
                self,
                FilesystemType::Apfs
                    | FilesystemType::Hfs
                    | FilesystemType::Nfs
                    | FilesystemType::Smb
                    | FilesystemType::Mem8
            )
        }
        #[cfg(target_os = "windows")]
        {
            matches!(
                self,
                FilesystemType::Ntfs
                    | FilesystemType::Fat32
                    | FilesystemType::ExFat
                    | FilesystemType::Mem8
            )
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            // Show all non-virtual filesystems on other platforms
            !self.is_virtual()
        }
    }
}

/// # FileCategory: Adding Semantic Flavor to Files
///
/// This enum provides a higher-level categorization based on common file extensions
/// or names. It's primarily used for display purposes, like coloring output,
/// and can also help in understanding the nature of a directory's contents.
/// Trish loves how this makes the tree output more intuitive!
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum FileCategory {
    // --- Programming Languages ---
    Rust,       // .rs
    Python,     // .py, .pyw, .pyx, .pyi
    JavaScript, // .js, .mjs, .cjs
    TypeScript, // .ts, .tsx
    Java,       // .java, .class, .jar
    C,          // .c, .h
    Cpp,        // .cpp, .cc, .cxx, .hpp, .hxx
    Go,         // .go
    Ruby,       // .rb
    PHP,        // .php - Not sure php is programming.
    Shell,      // .sh, .bash, .zsh, .fish

    // --- Markup & Data Formats ---
    Markdown, // .md, .markdown
    Html,     // .html, .htm
    Css,      // .css, .scss, .sass, .less
    Json,     // .json, .jsonc
    Yaml,     // .yaml, .yml
    Xml,      // .xml, .svg (SVG is XML-based)
    Toml,     // .toml
    Csv,      // .csv



    // --- Build Systems & Configuration ---
    Makefile,   // Makefile, makefile, GNUmakefile
    Dockerfile, // Dockerfile, .dockerfile
    GitConfig,  // .gitignore, .gitconfig, .gitmodules

    // --- Archives & Compressed Files ---
    Archive, // .zip, .tar, .gz, .bz2, .xz, .7z, .rar

    // --- Media Files ---
    Image, // .jpg, .jpeg, .png, .gif, .bmp, .ico, .webp
    Video, // .mp4, .avi, .mkv, .mov, .wmv, .flv, .webm
    Audio, // .mp3, .wav, .flac, .aac, .ogg, .wma

    // --- System & Binary Files ---
    SystemFile, // Special system files like swap.img, vmlinuz
    Binary,     // Executables, shared libraries (.exe, .dll, .so, .dylib, .o, .a)

    // --- Database ---
    Database, // .db, .sqlite, .mdb, .accdb, .dbf
    
    // --- Office & Documents ---
    Office,      // .doc, .docx, .odt
    Spreadsheet, // .xls, .xlsx, .ods, .csv
    PowerPoint,  // .ppt, .pptx, .odp
    Pdf,         // .pdf
    Ebook,       // .epub, .mobi, .azw
    
    // --- Text Variants ---
    Log,         // .log
    Config,      // .ini, .cfg, .conf, .env, .properties
    License,     // LICENSE, COPYING files
    Readme,      // README files
    Txt,         // .txt
    Rtf,         // .rtf
    
    // --- Security & Crypto ---
    Certificate, // .crt, .cert, .pem, .key
    Encrypted,   // .gpg, .pgp, .aes
    
    // --- Fonts ---
    Font,        // .ttf, .otf, .woff, .woff2
    
    // --- Virtual & Disk Images ---
    DiskImage,   // .img, .iso, .vdi, .vmdk, .vhd, .dd, .dmg
    
    // --- 3D & CAD ---
    Model3D,     // .obj, .stl, .dae, .fbx, .blend
    
    // --- Scientific & Data ---
    Jupyter,     // .ipynb
    RData,       // .rdata, .rds
    Matlab,      // .m, .mat
    
    // --- Web Assets ---
    WebAsset,    // .wasm, .map
    
    // --- Package & Dependencies ---
    Package,     // package.json, Cargo.toml, requirements.txt, etc.
    Lock,        // package-lock.json, Cargo.lock, yarn.lock
    
    // --- Testing ---
    Test,        // Files with test_, _test, .test, .spec patterns
    
    // --- Memory Files (Our special type!) ---
    Memory,      // .mem8, .m8 - MEM|8 memory files
    
    // --- Others ---
    Backup,      // .bak, .backup, ~
    Temp,        // .tmp, .temp, .swp
    Unknown,     // If we can't categorize it, it's a mysterious Unknown!
}

/// # TreeStats: The Final Scoreboard
///
/// After the concert is over, this is where we see how we did. It's the
/// scoreboard that tracks total files, total directories, the biggest hits
/// (largest files), and more. It's the answer to "So, how was the show?"
#[derive(Debug, Default)]
pub struct TreeStats {
    /// Total number of files encountered (excluding directories).
    pub total_files: u64,
    /// Total number of directories encountered.
    pub total_dirs: u64,
    /// Total size of all files (in bytes).
    pub total_size: u64,
    /// A map of file extensions to their counts (e.g., {"rs": 10, "toml": 2}).
    pub file_types: HashMap<String, u64>,
    /// Top N largest files found (path and size). N is usually 10.
    pub largest_files: Vec<(u64, PathBuf)>,
    /// Top N newest files found (path and modification time).
    pub newest_files: Vec<(SystemTime, PathBuf)>,
    /// Top N oldest files found (path and modification time).
    pub oldest_files: Vec<(SystemTime, PathBuf)>,
}

impl TreeStats {
    /// Updates the statistics based on a newly processed `FileNode`.
    /// This method is called for each non-permission-denied node.
    pub fn update_file(&mut self, node: &FileNode) {
        if node.is_dir {
            self.total_dirs += 1;
        } else {
            // It's a file!
            self.total_files += 1;
            self.total_size += node.size;

            // Track file extensions for type distribution.
            if let Some(ext) = node.path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    *self.file_types.entry(ext_str.to_string()).or_insert(0) += 1;
                }
            }

            // --- Update Top N Lists ---
            // These lists are kept sorted and truncated to maintain a fixed size (e.g., top 10).

            // Update largest files: Add, sort by size (desc), truncate.
            self.largest_files.push((node.size, node.path.clone()));
            self.largest_files.sort_by(|a, b| b.0.cmp(&a.0)); // Largest first
            self.largest_files.truncate(10); // Keep only the top 10

            // Update newest files: Add, sort by modification time (desc), truncate.
            self.newest_files.push((node.modified, node.path.clone()));
            self.newest_files.sort_by(|a, b| b.0.cmp(&a.0)); // Newest first
            self.newest_files.truncate(10);

            // Update oldest files: Add, sort by modification time (asc), truncate.
            self.oldest_files.push((node.modified, node.path.clone()));
            self.oldest_files.sort_by(|a, b| a.0.cmp(&b.0)); // Oldest first
            self.oldest_files.truncate(10);
        }
    }
}

/// # ScannerConfig: The Rider for our Rock Star Scanner
///
/// This is the list of demands for our scanner. "Don't show me hidden files,"
/// "I only want to see files bigger than a tour bus," "Ignore the messy backstage
/// area (`.gitignore`)." We build this from the user's command-line arguments
/// to make sure the scanner puts on the exact show the user wants to see.
#[derive(Default, Clone)]
pub struct ScannerConfig {
    /// Maximum depth to traverse into subdirectories.
    pub max_depth: usize,
    /// Should symbolic links be followed? (Currently always `false`).
    pub follow_symlinks: bool,
    /// Should `.gitignore` files be respected?
    pub respect_gitignore: bool,
    /// Should hidden files (starting with `.`) be shown?
    pub show_hidden: bool,
    /// Should ignored files/directories be shown (usually in brackets)?
    pub show_ignored: bool,
    /// An optional regex pattern to filter files/directories by name.
    pub find_pattern: Option<Regex>,
    /// An optional file extension to filter by (e.g., "rs").
    pub file_type_filter: Option<String>,
    /// Optional entry type filter ("f" for files, "d" for directories).
    pub entry_type_filter: Option<String>,
    /// Optional minimum file size filter.
    pub min_size: Option<u64>,
    /// Optional maximum file size filter.
    pub max_size: Option<u64>,
    /// Optional filter for files newer than a specific date.
    pub newer_than: Option<SystemTime>,
    /// Optional filter for files older than a specific date.
    pub older_than: Option<SystemTime>,
    /// Should the scanner use its built-in list of default ignore patterns
    /// (like `node_modules`, `__pycache__`, `target/`)?
    pub use_default_ignores: bool,
    /// An optional keyword to search for within file contents.
    pub search_keyword: Option<String>,
    /// Should filesystem type indicators be shown?
    pub show_filesystems: bool,
    /// Sort field for results (name, size, date, type)
    pub sort_field: Option<String>,
    /// Limit results to top N entries (useful with sort)
    pub top_n: Option<usize>,
    /// Include actual line content in search results (for AI/MCP use)
    pub include_line_content: bool,
}

// --- Default Ignore Patterns: The "Please Don't Play These Songs" List ---
// Every band has songs they'd rather not play. This is our list of files and
// directories (`node_modules`, `target/`, etc.) that we usually skip to keep
// the show clean and focused on the hits. A tidy tree is a happy tree!
const DEFAULT_IGNORE_PATTERNS: &[&str] = &[
    // Version control systems (but not all hidden dirs like .ssh)
    ".git",
    ".svn",
    ".hg",
    ".bzr",
    "_darcs",
    // Python artifacts
    "__pycache__",
    "*.pyc",
    "*.pyo",
    "*.pyd",
    ".Python",
    ".pytest_cache",
    ".tox",
    ".coverage",
    "*.egg-info",
    ".eggs",
    // Node.js / JavaScript artifacts
    "node_modules",
    ".npm",
    ".yarn",
    ".pnpm-store",
    "bower_components",
    ".next",
    ".nuxt",
    // General cache directories often found in projects
    ".cache", // Common cache dir name
    // Virtual environments
    "venv",
    "env",
    "ENV",
    "virtualenv",
    ".venv",
    ".env",
    "conda-meta",
    // Build/compilation artifacts from various languages/systems
    "target", // Rust
    "build",
    "dist",
    "out",
    "bin",
    "obj", // Common build output dirs
    "*.o",
    "*.a",
    "*.so",
    "*.dll",
    "*.dylib", // Object files, libraries
    // Package manager caches/data
    ".cargo",
    ".rustup", // Rust
    ".gem",
    ".bundle", // Ruby
    // IDEs and editor-specific files/directories
    ".idea",
    ".vscode",
    ".vs", // Common IDE metadata
    "*.swp",
    "*.swo",
    "*~", // Vim/editor backup/swap files
    ".project",
    ".classpath",
    ".settings", // Eclipse/Java
    // Development tool caches
    ".mypy_cache",
    ".ruff_cache",
    ".hypothesis",
    ".pytest_cache",
    ".tox",
    ".coverage",
    ".sass-cache",
    // OS-specific junk files
    ".DS_Store",    // macOS
    "Thumbs.db",    // Windows
    "desktop.ini",  // Windows
    "$RECYCLE.BIN", // Windows recycle bin
    // Common temporary file/directory names and patterns
    "tmp",
    "temp",
    ".tmp",
    ".temp",
    "*.tmp",
    "*.temp",
    // More cache directories
    ".sass-cache", // Sass CSS preprocessor
    "__MACOSX",    // macOS archive metadata
    // System directories that are almost never useful to traverse deeply from a user's project root.
    // These are more aggressively ignored if `st` is run on `/`.
    // "proc", "sys", "dev", "lost+found", "mnt", "media", // Handled by DEFAULT_SYSTEM_PATHS
    // Other common ignores
    ".vagrant",
    ".terraform",
];

// Default paths that are almost always too noisy or problematic to scan,
// especially if `st` is run from `/` or a very high-level directory.
// These are typically mount points for virtual filesystems or system-critical areas.
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
    "/snap", // Common mount points or special dirs
];

// Specific individual files (absolute paths) that should always be ignored
// due to their special nature (e.g., virtual files representing system memory).
const DEFAULT_IGNORE_FILES: &[&str] = &[
    "/proc/kcore",    // Virtual file representing physical memory, can be huge & slow.
    "/proc/kmsg",     // Kernel messages, can be an infinite stream.
    "/proc/kallsyms", // Kernel symbols, can be large.
];

/// # Scanner: The Rock Star of our Show
///
/// BEHOLD! The `Scanner` itself! This is the main act. It takes the config,
/// the ignore lists, and a path, and it puts on a spectacular show of directory
/// traversal. It's fast, it's smart, and it knows all the best moves.
pub struct Scanner {
    /// The configuration for this scanning operation.
    config: ScannerConfig,
    /// Compiled `GlobSet` from `.gitignore` files, if respected and found.
    gitignore: Option<GlobSet>,
    /// Compiled `GlobSet` from our `DEFAULT_IGNORE_PATTERNS`.
    default_ignores: Option<GlobSet>,
    /// A set of absolute system paths to ignore (e.g., /proc, /sys).
    system_paths: HashSet<PathBuf>,
    /// A set of specific absolute file paths to ignore (e.g., /proc/kcore).
    ignore_files: HashSet<PathBuf>,
    /// The root path from which the scan originates.
    root: PathBuf,
    /// Safety limits to prevent crashes on large directories
    safety_limits: ScannerSafetyLimits,
}

impl Scanner {
    /// Returns the canonicalized root path of the scanner
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Quick scan for basic project analysis - lighter weight than full scan
    /// Returns only basic stats and key files for faster integration
    pub fn quick_scan(&self) -> Result<(Vec<FileNode>, TreeStats)> {
        let mut config = self.config.clone();
        config.max_depth = 3; // Limit depth for quick scan
        
        let quick_scanner = Scanner::new(&self.root, config)?;
        quick_scanner.scan()
    }

    /// Find files modified within a specific time range
    /// Useful for finding recent activity in projects
    pub fn find_recent_files(&self, hours_ago: u64) -> Result<Vec<FileNode>> {
        let cutoff_time = std::time::SystemTime::now() 
            - std::time::Duration::from_secs(hours_ago * 3600);
        
        let (nodes, _) = self.scan()?;
        Ok(nodes.into_iter()
            .filter(|node| !node.is_dir && node.modified > cutoff_time)
            .collect())
    }

    /// Get key project files (build configs, main files, etc.)
    /// Returns a filtered list of important files for project analysis
    pub fn find_key_files(&self) -> Result<Vec<FileNode>> {
        let (nodes, _) = self.scan()?;
        
        let important_patterns = [
            "main.rs", "lib.rs", "mod.rs",
            "package.json", "Cargo.toml", "requirements.txt", "pyproject.toml",
            "README.md", "LICENSE", "Makefile", "CMakeLists.txt",
            "index.js", "app.js", "server.js", "main.js",
            "main.py", "__init__.py", "setup.py",
            "go.mod", "main.go",
            "pom.xml", "build.gradle", "build.xml",
            ".gitignore", "docker-compose.yml", "Dockerfile",
        ];

        Ok(nodes.into_iter()
            .filter(|node| {
                if node.is_dir {
                    return false;
                }
                
                let file_name = node.path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                important_patterns.iter().any(|&pattern| file_name == pattern)
            })
            .collect())
    }

    /// ## `get_file_category`
    /// Determines a `FileCategory` for a given path and `FileType`.
    /// This function uses a series of heuristics based on file extensions and common names
    /// to classify files into broad categories, useful for display and understanding content.
    /// It's like a quick identification guide for files!
    fn get_file_category(path: &Path, file_type: FileType) -> FileCategory {
        // Directories don't get a specific content category here; their content defines them.
        if matches!(file_type, FileType::Directory) {
            return FileCategory::Unknown;
        }

        // First, check for some very specific system file names.
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name == "swap.img"
                || name == "swapfile"
                || name.starts_with("vmlinuz")
                || name.starts_with("initrd")
            {
                return FileCategory::SystemFile;
            }
        }

        // Primary categorization is by file extension.
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                // --- Programming Languages ---
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
                "sh" | "bash" | "zsh" | "fish" | "ps1" | "bat" | "cmd" => FileCategory::Shell,

                // --- Markup/Data ---
                "md" | "markdown" => FileCategory::Markdown,
                "html" | "htm" => FileCategory::Html,
                "css" | "scss" | "sass" | "less" => FileCategory::Css,
                "json" | "jsonc" | "geojson" => FileCategory::Json,
                "yaml" | "yml" => FileCategory::Yaml,
                "xml" | "svg" | "plist" | "kml" | "gpx" => FileCategory::Xml, // SVG and others are XML-based
                "toml" => FileCategory::Toml,

                // --- Build/Config (some are also by name) ---
                "dockerfile" => FileCategory::Dockerfile, // Extension variant
                // .gitignore, .gitconfig are usually by name, handled below

                // --- Archives ---
                "zip" | "tar" | "gz" | "tgz" | "bz2" | "tbz2" | "xz" | "txz" | "7z" | "rar" => FileCategory::Archive,

                // --- Media ---
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "ico" | "webp" | "tiff" | "tif"
                | "heic" | "heif" => FileCategory::Image,
                "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "mpeg" | "mpg" => {
                    FileCategory::Video
                }
                "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" => FileCategory::Audio,

                // --- Binary/Executable (some overlap with system, but these are common distributable/object formats) ---
                "exe" | "dll" | "so" | "dylib" | "o" | "a" | "lib" | "msi" | "deb" | "rpm"
                | "app" => FileCategory::Binary,

                // --- Database Files ---
                "db" | "sqlite" | "sqlitedb" | "sqlite3" | "db3" | "db4" | "db5" | "mdb" | "accdb" | "dbf" => FileCategory::Database,
                
                // --- Office & Documents ---
                "doc" | "docx" | "odt" | "rtf" => FileCategory::Office,
                "xls" | "xlsx" | "ods" | "csv" | "tsv" => FileCategory::Spreadsheet,
                "ppt" | "pptx" | "odp" => FileCategory::PowerPoint,
                "pdf" => FileCategory::Pdf,
                "epub" | "mobi" | "azw" | "azw3" | "fb2" => FileCategory::Ebook,
                
                // --- Text & Config Files ---
                "txt" | "text" => FileCategory::Txt,
                "log" => FileCategory::Log,
                "ini" | "cfg" | "conf" | "config" | "properties" | "env" => FileCategory::Config,
                
                // --- Security & Crypto ---
                "crt" | "cert" | "pem" | "key" | "pub" | "cer" | "der" => FileCategory::Certificate,
                "gpg" | "pgp" | "aes" | "enc" | "asc" => FileCategory::Encrypted,
                
                // --- Fonts ---
                "ttf" | "otf" | "woff" | "woff2" | "eot" | "fon" | "fnt" => FileCategory::Font,
                
                // --- Disk Images ---
                "img" | "vdi" | "vmdk" | "vhd" | "vhdx" | "dd" | "hdd" | "qcow" | "qcow2" => FileCategory::DiskImage,
                "iso" | "dmg" => FileCategory::DiskImage, // These can be both archives and disk images, but treating as disk images
                
                // --- 3D & CAD ---
                "obj" | "stl" | "dae" | "fbx" | "blend" | "3ds" | "ply" | "gltf" | "glb" => FileCategory::Model3D,
                
                // --- Scientific & Data ---
                "ipynb" => FileCategory::Jupyter,
                "rdata" | "rds" | "rda" => FileCategory::RData,
                "m" | "mat" | "mlx" => FileCategory::Matlab,
                
                // --- Web Assets ---
                "wasm" | "map" | "sourcemap" => FileCategory::WebAsset,
                
                // --- Memory Files (MEM|8!) ---
                "mem8" | "m8" => FileCategory::Memory,
                
                // --- Backup & Temp ---
                "bak" | "backup" | "old" | "orig" => FileCategory::Backup,
                "tmp" | "temp" | "swp" | "swo" | "swn" => FileCategory::Temp,
                
                _ => FileCategory::Unknown // Extension not recognized
            }
        } else {
            // No extension, or extension parsing failed. Try common filenames.
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Check for test files
                if name.starts_with("test_") || name.ends_with("_test") || 
                   name.contains(".test.") || name.contains(".spec.") {
                    return FileCategory::Test;
                }
                
                // Check for specific filenames
                match name {
                    "Makefile" | "makefile" | "GNUmakefile" => FileCategory::Makefile,
                    "Dockerfile" => FileCategory::Dockerfile,
                    ".gitignore" | ".gitconfig" | ".gitattributes" | ".gitmodules" => FileCategory::GitConfig,
                    "LICENSE" | "LICENCE" | "COPYING" => FileCategory::License,
                    "README" | "README.md" | "README.txt" | "README.rst" => FileCategory::Readme,
                    "package.json" | "Cargo.toml" | "requirements.txt" | "pyproject.toml" | 
                    "pom.xml" | "build.gradle" | "go.mod" | "composer.json" => FileCategory::Package,
                    "package-lock.json" | "Cargo.lock" | "yarn.lock" | "pnpm-lock.yaml" | 
                    "poetry.lock" | "Gemfile.lock" => FileCategory::Lock,
                    _ => {
                        // Check for backup files ending with ~
                        if name.ends_with('~') {
                            FileCategory::Backup
                        } else if matches!(file_type, FileType::Executable) {
                            FileCategory::Binary
                        } else {
                            FileCategory::Unknown
                        }
                    }
                }
            } else {
                FileCategory::Unknown // Path has no filename component (should be rare for actual files).
            }
        }
    }

    /// ## `Scanner::new` - Constructor
    ///
    /// Creates a new `Scanner` instance. This involves:
    /// 1. Storing the provided `config` and `root` path.
    /// 2. Loading and compiling `.gitignore` patterns if `config.respect_gitignore` is true.
    /// 3. Compiling the `DEFAULT_IGNORE_PATTERNS` if `config.use_default_ignores` is true.
    /// 4. Initializing sets of system paths and specific files to always ignore.
    ///
    /// This setup prepares the scanner for efficient `should_ignore` checks during traversal.
    pub fn new(root: &Path, config: ScannerConfig) -> Result<Self> {
        // Canonicalize the root path to get the absolute path
        // If canonicalize fails (e.g., path doesn't exist), fall back to absolute path
        let canonical_root = root
            .canonicalize()
            .or_else(|_| std::env::current_dir().map(|cwd| cwd.join(root)))
            .unwrap_or_else(|_| root.to_path_buf());

        // Load .gitignore patterns from the root directory if requested.
        let gitignore = if config.respect_gitignore {
            Self::load_gitignore(&canonical_root)? // This can return None if no .gitignore or error.
        } else {
            None // Not respecting .gitignore.
        };

        // Build the GlobSet for default ignore patterns if requested.
        let default_ignores = if config.use_default_ignores {
            Self::build_default_ignores()? // This can return None if patterns are invalid (unlikely for defaults).
        } else {
            None // Not using default ignores.st
        };

        // Initialize the set of system paths to ignore (e.g., /proc, /sys).
        let system_paths: HashSet<PathBuf> = if config.use_default_ignores {
            DEFAULT_SYSTEM_PATHS
                .iter()
                .map(PathBuf::from) // Convert string slices to PathBufs
                .collect() // Collect into a HashSet for quick lookups.
        } else {
            HashSet::new() // Empty set if not using default ignores.
        };

        // Initialize the set of specific files to ignore (e.g., /proc/kcore).
        let ignore_files: HashSet<PathBuf> = if config.use_default_ignores {
            DEFAULT_IGNORE_FILES.iter().map(PathBuf::from).collect()
        } else {
            HashSet::new()
        };

        // Determine appropriate safety limits based on the path
        let safety_limits = if canonical_root == PathBuf::from(&std::env::var("HOME").unwrap_or_default()) {
            // Home directory needs special care
            ScannerSafetyLimits::for_home_directory()
        } else if canonical_root.starts_with("/") && canonical_root.components().count() <= 2 {
            // Root or near-root paths need limits
            ScannerSafetyLimits::for_home_directory()
        } else {
            // Regular directories can use default limits
            ScannerSafetyLimits::default()
        };
        
        Ok(Self {
            config,
            gitignore,
            default_ignores,
            system_paths,
            ignore_files,
            root: canonical_root, // Store a copy of the root path.
            safety_limits,
        })
    }

    /// ## `build_default_ignores`
    ///
    /// Compiles the `DEFAULT_IGNORE_PATTERNS` array into a `GlobSet` for efficient matching.
    /// This `GlobSet` is used to quickly check if a file/directory name matches any of the
    /// common patterns we want to ignore by default (like `node_modules`, `target/`).
    /// Returns `Ok(Some(GlobSet))` on success, or `Ok(None)` if no patterns (should not happen),
    /// or an `Err` if glob compilation fails (very unlikely for our hardcoded patterns).
    fn build_default_ignores() -> Result<Option<GlobSet>> {
        let mut builder = GlobSetBuilder::new(); // Start with an empty builder.

        // Add each default pattern to the builder.
        for pattern_str in DEFAULT_IGNORE_PATTERNS {
            // Glob::new can fail if the pattern is malformed, but ours should be fine.
            if let Ok(glob) = Glob::new(pattern_str) {
                builder.add(glob);
            }
            // Silently ignore malformed default patterns, though this shouldn't occur.
        }

        // Build the GlobSet from the accumulated patterns.
        // This can fail if, for example, the set is empty or patterns are incompatible,
        // but again, highly unlikely for our predefined set.
        Ok(Some(builder.build()?))
    }

    /// ## `load_gitignore`
    ///
    /// Reads the `.gitignore` file from the specified `root` directory (if it exists)
    /// and compiles its patterns into a `GlobSet`.
    /// Lines starting with `#` (comments) and empty lines are ignored.
    /// Returns `Ok(Some(GlobSet))` if `.gitignore` is found and parsed,
    /// `Ok(None)` if no `.gitignore` file exists, or an `Err` on I/O or parsing issues.
    fn load_gitignore(root: &Path) -> Result<Option<GlobSet>> {
        let gitignore_path = root.join(".gitignore"); // Construct path to .gitignore.
        if !gitignore_path.exists() {
            return Ok(None); // No .gitignore file found, nothing to load.
        }

        let mut builder = GlobSetBuilder::new();
        // Read the entire .gitignore file, handling non-UTF-8 content gracefully
        let content = match fs::read(&gitignore_path) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
            Err(e) => {
                eprintln!("Warning: Could not read .gitignore at {:?}: {}", gitignore_path, e);
                return Ok(None);
            }
        };

        // Process each line of the .gitignore file.
        for line in content.lines() {
            let trimmed_line = line.trim(); // Remove leading/trailing whitespace.
                                            // Ignore empty lines and lines that are comments (start with '#').
            if !trimmed_line.is_empty() && !trimmed_line.starts_with('#') {
                // Attempt to compile the line as a glob pattern.
                // If successful, add it to our GlobSet builder.
                if let Ok(glob) = Glob::new(trimmed_line) {
                    builder.add(glob);
                }
                // Malformed patterns in user's .gitignore are silently skipped.
            }
        }

        // Build the final GlobSet from all valid patterns.
        Ok(Some(builder.build()?))
    }

    /// Stream nodes as they are discovered
    /// This version of scan is optimized for the `--stream` flag.
    /// It sends `FileNode` objects through the `sender` channel as soon as they are processed.
    /// This allows the formatter to start displaying output immediately, which is great for large directories.
    /// Returns the final `TreeStats` once the scan is complete.
    pub fn scan_stream(&self, sender: mpsc::Sender<FileNode>) -> Result<TreeStats> {
        let mut stats = TreeStats::default();

        // When searching, we need to collect all nodes first to determine which directories to show
        if self.config.search_keyword.is_some() {
            // Use the non-streaming scan and then send results in order
            let (nodes, stats) = self.scan()?;
            for node in nodes {
                if sender.send(node).is_err() {
                    break; // Receiver disconnected
                }
            }
            return Ok(stats);
        }

        // Initialize safety tracker for streaming mode
        let safety_tracker = ScannerSafetyTracker::new(self.safety_limits.clone());

        // Original streaming logic for non-search cases
        let mut walker = WalkDir::new(&self.root)
            .max_depth(self.config.max_depth)
            .follow_links(self.config.follow_symlinks)
            .into_iter();

        // Loop through each entry provided by WalkDir.
        while let Some(entry_result) = walker.next() {
            // Check safety limits
            if let Err(safety_error) = safety_tracker.should_continue() {
                eprintln!("⚠️  {}", safety_error);
                eprintln!("   Use --max-depth or scan a more specific directory");
                break;
            }
            
            match entry_result {
                Ok(entry) => {
                    // Successfully read a directory entry.
                    let depth = entry.depth();
                    let path = entry.path();

                    // Determine if this entry should be ignored based on various rules.
                    let is_ignored_by_rules = self.should_ignore(path)?;

                    if is_ignored_by_rules {
                        // The entry matches an ignore rule.
                        if self.config.show_ignored {
                            // If we're showing ignored items, process it but mark as ignored.
                            if let Some(mut node) =
                                self.process_entry(&entry, depth, is_ignored_by_rules)?
                            {
                                // Perform content search if applicable, even for ignored files being shown.
                                if !node.is_dir && self.should_search_file(&node) {
                                    node.search_matches = self.search_in_file(&node.path);
                                }

                                // Track node for safety limits
                                safety_tracker.add_file(estimate_node_size(node.path.to_string_lossy().len()));
                                
                                // Send the (ignored) node through the channel.
                                if sender.send(node.clone()).is_err() {
                                    break; // Receiver has disconnected, stop scanning.
                                }

                                // Update stats for ignored items if they aren't permission-denied.
                                // This ensures `show_ignored` gives a full picture.
                                if !node.permission_denied {
                                    stats.update_file(&node);
                                }
                            }
                            // If this ignored item is a directory, tell WalkDir not to descend into it.
                            if entry.file_type().is_dir() {
                                // `ignored_dirs.insert(path.to_path_buf());` // Not strictly needed if just skipping.
                                walker.skip_current_dir();
                            }
                        } else {
                            // We are *not* showing ignored items, and this one is ignored.
                            // If it's a directory, skip its contents. Otherwise, just continue.
                            if entry.file_type().is_dir() {
                                walker.skip_current_dir();
                            }
                            // `continue;` // Implicitly done by not processing further.
                        }
                    } else {
                        // The entry is NOT ignored by rules. Process it normally.
                        if let Some(mut node) = self.process_entry(&entry, depth, false)? {
                            // `is_ignored` is false here
                            // Perform content search if applicable.
                            if !node.is_dir && self.should_search_file(&node) {
                                node.search_matches = self.search_in_file(&node.path);
                            }

                            // Apply filters (size, date, type, find pattern).
                            // A file is included if it's a directory, or it matches filters, or it has a search match.
                            let has_search_match = node
                                .search_matches
                                .as_ref()
                                .is_some_and(|m| m.total_count > 0);

                            // If we have a search keyword, only include files with matches
                            let should_include_file = if self.config.search_keyword.is_some() {
                                has_search_match
                            } else {
                                self.should_include(&node)
                            };

                            if node.is_dir || should_include_file {
                                // Track node for safety limits
                                safety_tracker.add_file(estimate_node_size(node.path.to_string_lossy().len()));
                                
                                // Send the processed node through the channel.
                                if sender.send(node.clone()).is_err() {
                                    break; // Receiver disconnected.
                                }

                                // Update statistics for included, non-permission-denied items.
                                if !node.permission_denied {
                                    stats.update_file(&node);
                                }
                            }
                        } else {
                            // process_entry returned None, which means this is a hidden entry and show_hidden is false
                            // If it's a directory, we need to skip its contents
                            if entry.file_type().is_dir() {
                                walker.skip_current_dir();
                            }
                        }
                    }
                }
                Err(e) => {
                    // An error occurred trying to access a directory entry (e.g., permission denied).
                    if let Some(path) = e.path() {
                        let depth = e.depth();

                        // Check if this is a "directory contents" error vs "directory entry" error.
                        // If this is a permission error, it's likely we already processed the directory
                        // entry successfully but can't read its contents. In that case, skip creating
                        // a duplicate node since we already marked the original as permission_denied.
                        let is_contents_error = e.io_error().map_or(false, |io_err| {
                            io_err.kind() == std::io::ErrorKind::PermissionDenied
                        });

                        if !is_contents_error {
                            // Create a special node representing the permission-denied entry.
                            let node = self.create_permission_denied_node(path, depth);
                            safety_tracker.add_file(estimate_node_size(node.path.to_string_lossy().len()));
                            
                            if sender.send(node.clone()).is_err() {
                                break; // Receiver disconnected.
                            }
                            // Still update stats (e.g., directory count) for permission-denied entries if shown.
                            stats.update_file(&node);
                        }

                        // Tell WalkDir not to try to descend into this unreadable directory.
                        walker.skip_current_dir();
                    }
                    // If the error is not path-specific, it might be logged or ignored depending on severity.
                    // For now, we primarily handle path-specific errors like permission issues.
                }
            }
        }
        // Scan complete, return the accumulated statistics.
        Ok(stats)
    }

    /// ## `should_search_file`
    /// This function is called before `search_in_file` to decide if it's worth attempting a search.
    /// It checks if a search keyword is configured and if the file is likely text-based.
    fn should_search_file(&self, node: &FileNode) -> bool {
        // No search keyword? No search.
        if self.config.search_keyword.is_none() {
            return false;
        }

        // If there's a file type filter, only search files that match it
        if let Some(ref filter_ext) = self.config.file_type_filter {
            if let Some(ext) = node.path.extension() {
                if ext.to_str() != Some(filter_ext) {
                    return false;
                }
            } else {
                // No extension, doesn't match filter
                return false;
            }
        }

        // Skip directories, symlinks, and special files.
        if node.is_dir || node.is_symlink || node.permission_denied {
            return false;
        }

        // Skip binary and system files based on category.
        matches!(
            node.category,
            FileCategory::Rust
                | FileCategory::Python
                | FileCategory::JavaScript
                | FileCategory::TypeScript
                | FileCategory::Java
                | FileCategory::C
                | FileCategory::Cpp
                | FileCategory::Go
                | FileCategory::Ruby
                | FileCategory::PHP
                | FileCategory::Shell
                | FileCategory::Markdown
                | FileCategory::Html
                | FileCategory::Css
                | FileCategory::Json
                | FileCategory::Yaml
                | FileCategory::Xml
                | FileCategory::Toml
                | FileCategory::Makefile
                | FileCategory::Dockerfile
                | FileCategory::GitConfig
        )
    }

    /// ## `search_in_file`
    ///
    /// Searches for the configured keyword within a file and returns match information.
    /// Returns line and column positions for each match, up to a reasonable limit.
    /// The search is case-sensitive. Optionally includes the actual line content.
    fn search_in_file(&self, path: &Path) -> Option<SearchMatches> {
        // Ensure there's a keyword to search for.
        let keyword = self.config.search_keyword.as_ref()?;
        if keyword.is_empty() {
            return None;
        }

        // Attempt to open the file for reading.
        let file = match fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return None,
        };

        let mut positions = Vec::new();
        let mut line_content_vec = Vec::new();
        let reader = BufReader::new(file);
        let mut line_number = 1;
        let mut first_match: Option<(usize, usize)> = None;
        let mut total_count = 0;

        // Read and process the file line by line.
        for line_result in reader.lines() {
            match line_result {
                Ok(line_content) => {
                    // Find all occurrences of the keyword in the current line.
                    let mut line_has_match = false;
                    let mut first_column_in_line = None;
                    
                    for (column_index, _) in line_content.match_indices(keyword) {
                        total_count += 1;
                        line_has_match = true;

                        // Column numbers are 1-based for user display
                        let match_pos = (line_number, column_index + 1);

                        if first_match.is_none() {
                            first_match = Some(match_pos);
                        }
                        
                        if first_column_in_line.is_none() {
                            first_column_in_line = Some(column_index + 1);
                        }

                        // Only store first 100 positions to prevent memory issues
                        if positions.len() < 100 {
                            positions.push(match_pos);
                        }

                        // Stop processing this file if we've found too many matches
                        if total_count > 100 {
                            let line_content_option = if self.config.include_line_content {
                                Some(line_content_vec)
                            } else {
                                None
                            };
                            
                            return Some(SearchMatches {
                                first_match: first_match.unwrap(),
                                total_count,
                                positions,
                                truncated: true,
                                line_content: line_content_option,
                            });
                        }
                    }
                    
                    // If this line has matches and we're including content, add it
                    if line_has_match && self.config.include_line_content && line_content_vec.len() < 100 {
                        line_content_vec.push((
                            line_number, 
                            line_content.clone(), 
                            first_column_in_line.unwrap()
                        ));
                    }
                    
                    line_number += 1;
                }
                Err(_) => {
                    // Invalid UTF-8 or other error, stop searching this file
                    break;
                }
            }
        }

        // Return matches if any were found
        first_match.map(|first| {
            let line_content_option = if self.config.include_line_content && !line_content_vec.is_empty() {
                Some(line_content_vec)
            } else {
                None
            };
            
            SearchMatches {
                first_match: first,
                total_count,
                positions,
                truncated: false,
                line_content: line_content_option,
            }
        })
    }

    /// ## `scan` - The Full Scan (Non-Streaming)
    ///
    /// Performs a complete directory scan, collecting all `FileNode`s that meet the criteria
    /// (not ignored, or shown if ignored, and pass filters if any).
    /// This method first traverses the entire directory structure defined by `config.max_depth`,
    /// creating `FileNode` objects for each entry. It then performs a second pass if filters
    /// are active to ensure that directories are only included if they (or their subdirectories)
    /// contain files that match the filters.
    /// Returns a tuple: `(Vec<FileNode>, TreeStats)`.
    /// ## `scan` - The "Scan-It-All-Then-Sort-It-Out" Method
    ///
    /// This is the classic way to scan. It's a two-act show:
    /// 1. **Act I**: Walk through every single file and directory, collecting a huge list of `FileNode`s.
    /// 2. **Act II**: If there are filters, go through that huge list and pick out only the ones that
    ///    match, making sure to keep their parent directories so the tree still makes sense.
    ///    It's thorough and great for when you need the whole picture before making decisions.
    pub fn scan(&self) -> Result<(Vec<FileNode>, TreeStats)> {
        let mut all_nodes_collected = Vec::new(); // Stores all nodes initially encountered.
                                                  // `ignored_dirs` was here, but its primary use with `skip_current_dir` is within the loop.
                                                  // If we need to track them for other reasons post-loop, it could be reinstated.

        // Initialize safety tracker
        let safety_tracker = ScannerSafetyTracker::new(self.safety_limits.clone());

        let mut walker = WalkDir::new(&self.root)
            .max_depth(self.config.max_depth)
            .follow_links(self.config.follow_symlinks)
            .into_iter();

        while let Some(entry_result) = walker.next() {
            // Check safety limits
            if let Err(safety_error) = safety_tracker.should_continue() {
                eprintln!("⚠️  {}", safety_error);
                eprintln!("   Use --max-depth, --stream mode, or scan a more specific directory");
                break;
            }
            
            match entry_result {
                Ok(entry) => {
                    let depth = entry.depth();
                    let path = entry.path();
                    let is_ignored_by_rules = self.should_ignore(path)?;

                    if is_ignored_by_rules {
                        if self.config.show_ignored {
                            // Process and add the ignored entry.
                            if let Some(mut node) = self.process_entry(&entry, depth, true)? {
                                if !node.is_dir && self.should_search_file(&node) {
                                    node.search_matches = self.search_in_file(&node.path);
                                }
                                safety_tracker.add_file(estimate_node_size(node.path.to_string_lossy().len()));
                                all_nodes_collected.push(node);
                            }
                            if entry.file_type().is_dir() {
                                walker.skip_current_dir(); // Don't descend into ignored dirs if showing them.
                            }
                        } else {
                            // Not showing ignored, and it's a directory: skip its contents.
                            if entry.file_type().is_dir() {
                                walker.skip_current_dir();
                            }
                            // If it's a file, it's simply skipped by not adding to `all_nodes_collected`.
                        }
                    } else {
                        // Not ignored by rules, process normally.
                        if let Some(mut node) = self.process_entry(&entry, depth, false)? {
                            if !node.is_dir && self.should_search_file(&node) {
                                node.search_matches = self.search_in_file(&node.path);
                            }
                            all_nodes_collected.push(node);
                        } else {
                            // process_entry returned None, which means this is a hidden entry and show_hidden is false
                            // If it's a directory, we need to skip its contents
                            if entry.file_type().is_dir() {
                                walker.skip_current_dir();
                            }
                        }
                    }
                }
                Err(e) => {
                    // Handle errors like permission denied.
                    if let Some(path) = e.path() {
                        let depth = e.depth();
                        all_nodes_collected.push(self.create_permission_denied_node(path, depth));
                        if e.io_error().is_some_and(|io_err| {
                            io_err.kind() == std::io::ErrorKind::PermissionDenied
                        }) {
                            walker.skip_current_dir(); // Skip unreadable directory.
                        }
                    }
                }
            }
        }

        // If filters are active, we need a second pass to ensure directories are only included
        // if they contain (or lead to) matching files.
        // Also, calculate stats based on the *final* list of nodes.
        let (final_nodes, final_stats) = if self.has_active_filters() {
            self.filter_nodes_and_calculate_stats(all_nodes_collected)
        } else {
            // No filters, so all collected nodes are final. Calculate stats on them.
            let mut stats = TreeStats::default();
            for node in &all_nodes_collected {
                // Only update stats for non-permission-denied items, or items that are directories.
                // (Permission denied files usually have size 0 and aren't "counted" in the same way).
                if !node.permission_denied || node.is_dir {
                    stats.update_file(node);
                }
            }
            (all_nodes_collected, stats)
        };

        // Apply sorting and top-N filtering if requested
        let sorted_nodes = self.apply_sorting_and_limit(final_nodes);
        
        Ok((sorted_nodes, final_stats))
    }

    /// ## `has_active_filters`
    ///
    /// Helper function to quickly check if any of the primary filtering criteria
    /// (find pattern, type, size, date) are currently set in the configuration.
    /// This determines if the second filtering pass (`filter_nodes_and_calculate_stats`) is needed.
    /// Note: `search_keyword` is handled slightly differently; it can make a file appear
    /// even if other filters would exclude it, so it's part of `should_include` logic.
    fn has_active_filters(&self) -> bool {
        self.config.find_pattern.is_some()
            || self.config.file_type_filter.is_some()
            || self.config.entry_type_filter.is_some()
            || self.config.min_size.is_some()
            || self.config.max_size.is_some()
            || self.config.newer_than.is_some()
            || self.config.older_than.is_some()
            || self.config.search_keyword.is_some() // Now search_keyword is also a filter
    }

    /// ## `filter_nodes_and_calculate_stats` (Formerly `filter_nodes_with_ancestors`)
    ///
    /// This crucial function takes all nodes collected during the initial traversal
    /// and filters them based on the `ScannerConfig`. It ensures that:
    /// 1. Files are included if they directly match all active filters OR if they contain a search match.
    /// 2. Directories are included if they themselves match a `--find` pattern OR
    ///    if they are an ancestor of an included file.
    ///    It then calculates `TreeStats` based on this final, filtered list of nodes.
    ///    This replaces the older `filter_nodes_with_ancestors` to integrate stat calculation
    ///    and clarify the logic for directory inclusion with `--find`.
    fn filter_nodes_and_calculate_stats(
        &self,
        all_nodes_collected: Vec<FileNode>,
    ) -> (Vec<FileNode>, TreeStats) {
        let mut final_stats = TreeStats::default();
        let mut included_files_and_matching_dirs = Vec::new(); // Files that pass filters, and Dirs that match --find
        let mut required_ancestor_dirs = HashSet::new(); // Ancestors of included_files

        // --- Pass 1: Identify matching files and directories that directly match --find ---
        for node in &all_nodes_collected {
            if node.permission_denied {
                // Skip permission denied entries for filtering logic
                continue;
            }

            let has_search_match = node
                .search_matches
                .as_ref()
                .is_some_and(|m| m.total_count > 0);

            if node.is_dir {
                // For directories, only the --find pattern applies directly.
                // Other filters (size, date, type) don't apply to directories themselves.
                if self
                    .config
                    .find_pattern
                    .as_ref()
                    .is_some_and(|p| p.is_match(&node.path.to_string_lossy()))
                {
                    included_files_and_matching_dirs.push(node.clone());
                    // Add ancestors of this directly matched directory
                    let mut current = node.path.parent();
                    while let Some(parent_path) = current {
                        if parent_path == self.root || required_ancestor_dirs.contains(parent_path)
                        {
                            break;
                        }
                        required_ancestor_dirs.insert(parent_path.to_path_buf());
                        current = parent_path.parent();
                    }
                }
            } else {
                // For files, check if it passes all filters OR has a search match.
                // If we have a search keyword, ONLY include files with search matches
                if self.config.search_keyword.is_some() {
                    if has_search_match {
                        // Even with search matches, the file must still pass other filters
                        if self.should_include(node) {
                            included_files_and_matching_dirs.push(node.clone());
                            // Add all ancestors of this matching file to `required_ancestor_dirs`.
                            let mut current = node.path.parent();
                            while let Some(parent_path) = current {
                                // Stop if we reach the root or an already added ancestor.
                                if parent_path == self.root
                                    || required_ancestor_dirs.contains(parent_path)
                                {
                                    break;
                                }
                                required_ancestor_dirs.insert(parent_path.to_path_buf());
                                current = parent_path.parent();
                            }
                        }
                    }
                } else {
                    // No search keyword, use normal filtering
                    if has_search_match || self.should_include(node) {
                        included_files_and_matching_dirs.push(node.clone());
                        // Add all ancestors of this matching file to `required_ancestor_dirs`.
                        let mut current = node.path.parent();
                        while let Some(parent_path) = current {
                            // Stop if we reach the root or an already added ancestor.
                            if parent_path == self.root
                                || required_ancestor_dirs.contains(parent_path)
                            {
                                break;
                            }
                            required_ancestor_dirs.insert(parent_path.to_path_buf());
                            current = parent_path.parent();
                        }
                    }
                }
            }
        }

        // --- Pass 2: Build the final list of nodes ---
        let mut final_node_list = Vec::new();
        let mut added_paths = HashSet::new(); // To prevent duplicates if a dir is both an ancestor and matches --find

        // Always add the root node if there's anything to show.
        if !included_files_and_matching_dirs.is_empty() {
            if let Some(root_node) = all_nodes_collected.iter().find(|n| n.path == self.root) {
                if added_paths.insert(root_node.path.clone()) {
                    final_node_list.push(root_node.clone());
                }
            }
        }

        // Add required ancestor directories and directly matching directories from `all_nodes_collected`.
        for node in &all_nodes_collected {
            if node.permission_denied {
                // Also include permission denied nodes if they are part of the path
                if (required_ancestor_dirs.contains(&node.path)
                    || node.path == self.root && !final_node_list.is_empty())
                    && added_paths.insert(node.path.clone())
                {
                    final_node_list.push(node.clone());
                }
                continue;
            }

            if node.is_dir {
                // Is it a required ancestor OR a directory that itself matched --find?
                let is_find_match = self
                    .config
                    .find_pattern
                    .as_ref()
                    .is_some_and(|p| p.is_match(&node.path.to_string_lossy()));
                if (required_ancestor_dirs.contains(&node.path)
                    || (is_find_match && node.path != self.root))
                    && added_paths.insert(node.path.clone())
                {
                    final_node_list.push(node.clone());
                }
            }
        }

        // Add the files that passed filters or had search matches.
        for node in included_files_and_matching_dirs {
            // If it's a directory, it was already handled above (if it matched --find).
            // If it's a file, add it now.
            if !node.is_dir {
                if added_paths.insert(node.path.clone()) {
                    final_node_list.push(node);
                }
            } else {
                // It's a directory that matched --find
                if added_paths.insert(node.path.clone()) {
                    final_node_list.push(node);
                }
            }
        }

        // Sort the final list by path for consistent output.
        final_node_list.sort_by(|a, b| a.path.cmp(&b.path));

        // --- Pass 3: Calculate stats on the final_node_list ---
        for node in &final_node_list {
            // Update stats, ensuring not to double-count or miscount permission-denied entries.
            if !node.permission_denied || node.is_dir {
                // Dirs (even denied) contribute to dir count.
                final_stats.update_file(node);
            }
        }

        (final_node_list, final_stats)
    }

    /// ## `process_entry`
    ///
    /// Converts a `walkdir::DirEntry` into our `FileNode` struct.
    /// This involves fetching metadata, determining file type, category, hidden status, etc.
    /// It also incorporates the `is_ignored_by_rules` status passed to it.
    /// Returns `Ok(Some(FileNode))` on success, `Ok(None)` if the entry should be skipped
    /// (e.g., hidden and not showing hidden), or an `Err` if metadata access fails.
    /// The `is_ignored_by_rules` parameter tells this function if `should_ignore` already determined this node is ignored.
    fn process_entry(
        &self,
        entry: &DirEntry,
        depth: usize,
        is_ignored_by_rules: bool,
    ) -> Result<Option<FileNode>> {
        let path = entry.path();

        // Determine if the file is hidden (starts with '.').
        let is_hidden = path
            .file_name()
            .and_then(|name_osstr| name_osstr.to_str()) // Convert OsStr to &str
            .is_some_and(|name_str| name_str.starts_with('.'));

        // Skip if hidden and we are not configured to show hidden files,
        // UNLESS it's an ignored item that we *are* configured to show (is_ignored_by_rules = true, config.show_ignored = true).
        // The `is_ignored_by_rules` flag takes precedence for display if `config.show_ignored` is true.
        if is_hidden && !self.config.show_hidden && !is_ignored_by_rules {
            // If it's a directory, we need to tell walkdir to skip its contents.
            if entry.file_type().is_dir() {
                // This is tricky because `process_entry` doesn't have `walker` to call `skip_current_dir()`.
                // The caller (`scan` or `scan_stream`) handles `skip_current_dir` based on `should_ignore`
                // and hidden status before calling `process_entry` or by checking the returned node.
                // For now, returning None signals to the caller that this node (and its children if a dir)
                // should not be further processed or added, unless `show_ignored` logic overrides.
            }
            return Ok(None); // Skip this hidden entry.
        }

        // Try to get metadata for the entry. This can fail (e.g., permission denied).
        let metadata = match entry.metadata() {
            Ok(md) => md,
            Err(_e) => {
                // If metadata fails, it's likely a permission issue or a broken symlink.
                // We create a special "permission_denied_node" in the calling `scan`/`scan_stream` methods
                // because they have access to `walker.skip_current_dir()`.
                // Here, we can't fully form that node, so we might return an error or a partial node.
                // For simplicity, if metadata fails here, we treat it as an inaccessible entry.
                // The main scan loops handle creating a FileNode for permission denied errors from WalkDir.
                // This specific call path implies WalkDir *could* read the entry but metadata() failed.
                // This is less common than WalkDir itself erroring.
                // Let's assume the main loops catch this via `Err(e)` from `walker.next()`.
                // If `process_entry` is called on an entry that `WalkDir` gave Ok for, but `metadata()` fails,
                // it's an edge case. We'll return a basic node marked as permission denied.
                return Ok(Some(self.create_permission_denied_node(path, depth)));
            }
        };

        let file_type = self.determine_file_type(&metadata);
        let category = Self::get_file_category(path, file_type);

        // Determine the size. For special virtual files (like in /proc or /sys),
        // reported size can be misleading (e.g., 0 or huge). We mark these as size 0.
        let size = if self.is_special_virtual_file(path, &metadata) {
            0
        } else {
            metadata.len()
        };

        // Check if this is a directory that we can't read the contents of
        let permission_denied = if metadata.is_dir() {
            // Try to read the directory to see if we have permission
            std::fs::read_dir(path).is_err()
        } else {
            false
        };

        Ok(Some(FileNode {
            path: path.to_path_buf(),
            is_dir: metadata.is_dir(),
            size,
            permissions: Self::get_permissions(&metadata),
            uid: Self::get_uid(&metadata),
            gid: Self::get_gid(&metadata),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH), // Fallback for modified time
            is_symlink: metadata.file_type().is_symlink(), // Use file_type() for symlink check
            is_hidden,
            permission_denied, // Set based on whether we can read directory contents
            is_ignored: is_ignored_by_rules, // Use the pre-determined ignore status.
            depth,
            file_type,
            category,
            search_matches: None, // Search matches are added later by the caller if needed.
            filesystem_type: Self::get_filesystem_type(path),
        }))
    }

    /// ## `get_filesystem_type`
    ///
    /// Detects the filesystem type for a given path using statfs on Unix systems
    #[cfg(unix)]
    fn get_filesystem_type(path: &Path) -> FilesystemType {
        use libc::statfs;
        use std::ffi::CString;
        use std::mem;

        // Filesystem magic numbers from statfs.h
        const EXT4_SUPER_MAGIC: i64 = 0xef53;
        const XFS_SUPER_MAGIC: i64 = 0x58465342;
        const BTRFS_SUPER_MAGIC: i64 = 0x9123683e;
        const ZFS_SUPER_MAGIC: i64 = 0x2fc12fc1;
        const NTFS_SB_MAGIC: i64 = 0x5346544e;
        const MSDOS_SUPER_MAGIC: i64 = 0x4d44; // FAT
        const EXFAT_SUPER_MAGIC: i64 = 0x2011bab0;
        const APFS_SUPER_MAGIC: i64 = 0x42535041; // 'APFS'
        const HFS_SUPER_MAGIC: i64 = 0x482b; // HFS+
        const NFS_SUPER_MAGIC: i64 = 0x6969;
        const SMB_SUPER_MAGIC: i64 = 0x517b;
        const TMPFS_MAGIC: i64 = 0x01021994;
        const PROC_SUPER_MAGIC: i64 = 0x9fa0;
        const SYSFS_MAGIC: i64 = 0x62656572;
        const DEVFS_SUPER_MAGIC: i64 = 0x1373;

        let path_cstr = match CString::new(path.to_string_lossy().as_bytes()) {
            Ok(s) => s,
            Err(_) => return FilesystemType::Unknown,
        };

        let mut stat_buf: libc::statfs = unsafe { mem::zeroed() };
        let result = unsafe { statfs(path_cstr.as_ptr(), &mut stat_buf) };

        if result != 0 {
            // statfs failed, fall back to path-based detection for virtual filesystems
            if let Some(path_str) = path.to_str() {
                if path_str.starts_with("/proc") {
                    return FilesystemType::Procfs;
                } else if path_str.starts_with("/sys") {
                    return FilesystemType::Sysfs;
                } else if path_str.starts_with("/dev") {
                    return FilesystemType::Devfs;
                }
            }
            return FilesystemType::Unknown;
        }

        // Check for Mem8 filesystem by looking for .mem8 marker files
        if path.join(".mem8").exists() || path.to_string_lossy().contains("mem8") {
            return FilesystemType::Mem8;
        }

        match stat_buf.f_type as i64 {
            EXT4_SUPER_MAGIC => FilesystemType::Ext4, // TODO: Distinguish ext2/3/4
            XFS_SUPER_MAGIC => FilesystemType::Xfs,
            BTRFS_SUPER_MAGIC => FilesystemType::Btrfs,
            ZFS_SUPER_MAGIC => FilesystemType::Zfs,
            NTFS_SB_MAGIC => FilesystemType::Ntfs,
            MSDOS_SUPER_MAGIC => FilesystemType::Fat32,
            EXFAT_SUPER_MAGIC => FilesystemType::ExFat,
            APFS_SUPER_MAGIC => FilesystemType::Apfs,
            HFS_SUPER_MAGIC => FilesystemType::Hfs,
            NFS_SUPER_MAGIC => FilesystemType::Nfs,
            SMB_SUPER_MAGIC => FilesystemType::Smb,
            TMPFS_MAGIC => FilesystemType::Tmpfs,
            PROC_SUPER_MAGIC => FilesystemType::Procfs,
            SYSFS_MAGIC => FilesystemType::Sysfs,
            DEVFS_SUPER_MAGIC => FilesystemType::Devfs,
            _ => FilesystemType::Unknown,
        }
    }

    #[cfg(not(unix))]
    fn get_filesystem_type(_path: &Path) -> FilesystemType {
        // On non-Unix systems, we can't easily detect filesystem type
        FilesystemType::Unknown
    }

    /// ## `is_virtual_filesystem`
    ///
    /// Checks if a path is on a virtual filesystem
    fn is_virtual_filesystem(path: &Path) -> bool {
        Self::get_filesystem_type(path).is_virtual()
    }

    /// ## `is_special_virtual_file`
    ///
    /// Checks if a file is likely a special virtual file (e.g., in /proc, /sys, /dev)
    /// where reported metadata like size might be zero, misleading, or cause issues if read.
    /// This helps in deciding to report size as 0 for such files.
    #[allow(unused_variables)]
    fn is_special_virtual_file(&self, path: &Path, metadata: &fs::Metadata) -> bool {
        // Check if the path starts with known virtual filesystem prefixes.
        if let Some(path_str) = path.to_str() {
            if path_str.starts_with("/proc/")
                || path_str.starts_with("/sys/")
                || path_str.starts_with("/dev/")
            {
                return true;
            }
        }

        // Check for specific problematic files by absolute path.
        if self.ignore_files.contains(path) {
            // Uses the pre-built HashSet of specific problem files.
            return true;
        }

        // On Unix, check for special file types like character devices, block devices, FIFOs, sockets.
        // These often have size 0 or non-standard size reporting.
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt; // For is_char_device(), is_block_device(), etc.
            let ft = metadata.file_type();
            if ft.is_char_device() || ft.is_block_device() || ft.is_fifo() || ft.is_socket() {
                return true;
            }
        }

        false // Not determined to be a special virtual file by these checks.
    }

    /// ## `create_permission_denied_node`
    ///
    /// Helper to create a `FileNode` representing an entry (usually a directory)
    /// that could not be accessed due to permission errors.
    /// These nodes are marked specially so formatters can indicate the issue.
    fn create_permission_denied_node(&self, path: &Path, depth: usize) -> FileNode {
        FileNode {
            path: path.to_path_buf(),
            is_dir: true, // Assume it's a directory, as that's common for permission errors during traversal.
            size: 0,      // No size info available.
            permissions: 0, // No permission info.
            uid: 0,       // No UID info.
            gid: 0,       // No GID info.
            modified: SystemTime::UNIX_EPOCH, // Default timestamp.
            is_symlink: false,
            is_hidden: false,        // Cannot determine if hidden.
            permission_denied: true, // Mark as permission denied.
            is_ignored: false,       // Not ignored by rules, but inaccessible.
            depth,
            file_type: FileType::Directory, // Assume directory.
            category: FileCategory::Unknown,
            search_matches: None,
            filesystem_type: Self::get_filesystem_type(path),
        }
    }

    /// ## `should_ignore` - The Bouncer at the Club Door
    ///
    /// This function is our tough-but-fair bouncer. It checks every file and
    /// directory against our lists (`.gitignore`, default ignores, etc.).
    /// "Sorry, `node_modules`, you're not on the list tonight."
    /// It's the first line of defense against clutter.
    fn should_ignore(&self, path: &Path) -> Result<bool> {
        // --- Rule 0: Never ignore the root path itself ---
        // If the user explicitly asks to scan a directory, we should show it
        // even if it would normally be ignored (e.g., scanning 'target' directory)
        if path == self.root {
            return Ok(false);
        }

        // --- Rule 1: Check against specific, always-ignored files (absolute paths) ---
        if self.config.use_default_ignores && self.ignore_files.contains(path) {
            return Ok(true); // Matches a specific problematic file.
        }

        // --- Rule 2: ALWAYS skip virtual filesystems like /proc, /sys, /dev ---
        // These are checked regardless of use_default_ignores because they're not real files
        // and can cause issues (huge fake sizes, hangs, etc.)
        if Self::is_virtual_filesystem(path) {
            return Ok(true);
        }

        // --- Rule 3: Check against other system paths if using default ignores ---
        if self.config.use_default_ignores {
            // Check for exact match of a system path.
            if self.system_paths.contains(path) {
                return Ok(true);
            }
            // Check if the current path is a child of any registered system path.
            for system_root_path in &self.system_paths {
                if path.starts_with(system_root_path) {
                    return Ok(true); // It's inside /tmp, /var/tmp, etc.
                }
            }
        }

        // --- Rule 3: Check against default ignore patterns (GlobSet) ---
        // These patterns usually match file/directory names or relative paths within a project.
        if let Some(ref default_ignore_set) = self.default_ignores {
            // Check if the simple file/directory name matches any default pattern.
            // (e.g., "node_modules" will match `path/to/project/node_modules`)
            if let Some(file_name) = path.file_name() {
                if default_ignore_set.is_match(Path::new(file_name)) {
                    return Ok(true);
                }
            }
            // Also check the path relative to the scan root against default patterns.
            // This handles patterns like "*.pyc" or "build/outputs/".
            if let Ok(relative_path_to_root) = path.strip_prefix(&self.root) {
                if default_ignore_set.is_match(relative_path_to_root) {
                    return Ok(true);
                }
            }
        }

        // --- Rule 4: Check against .gitignore patterns (GlobSet) ---
        // These patterns are always relative to the root of the scan (where .gitignore is located).
        if let Some(ref gitignore_set) = self.gitignore {
            if let Ok(relative_path_to_root) = path.strip_prefix(&self.root) {
                if gitignore_set.is_match(relative_path_to_root) {
                    return Ok(true); // Matches a .gitignore pattern.
                }
            }
            // If strip_prefix fails (path is not under root), it can't match .gitignore relative patterns.
        }

        // If none of the above rules triggered, the path is not ignored.
        Ok(false)
    }

    /// ## `should_include` - The Velvet Rope
    ///
    /// Once a file gets past the bouncer (`should_ignore`), it has to get past
    /// the velvet rope. This function checks if the file meets the specific criteria
    /// for this party: "Are you a `.rs` file? Are you bigger than 1MB?"
    /// Only the coolest files that match all the rules get in.
    fn should_include(&self, node: &FileNode) -> bool {
        // --- Filter by --find pattern (applies to both files and directories) ---
        if let Some(ref find_regex_pattern) = self.config.find_pattern {
            // Convert path to string for regex matching. Lossy conversion is acceptable for matching.
            let path_str = node.path.to_string_lossy();
            if !find_regex_pattern.is_match(&path_str) {
                return false; // Path doesn't match the --find pattern.
            }
        }
        
        // --- Filter by entry type (--entry-type) ---
        if let Some(ref entry_type) = self.config.entry_type_filter {
            match entry_type.as_str() {
                "f" => {
                    if node.is_dir {
                        return false; // Looking for files only, but this is a directory
                    }
                }
                "d" => {
                    if !node.is_dir {
                        return false; // Looking for directories only, but this is a file
                    }
                }
                _ => {} // Should not happen due to clap validation
            }
        }

        // --- Filters below only apply to files, not directories ---
        if !node.is_dir {
            // --- Filter by file extension (--type) ---
            if let Some(ref required_extension) = self.config.file_type_filter {
                match node
                    .path
                    .extension()
                    .and_then(|ext_osstr| ext_osstr.to_str())
                {
                    Some(file_ext_str) => {
                        if !file_ext_str.eq_ignore_ascii_case(required_extension) {
                            return false; // Extension doesn't match.
                        }
                    }
                    None => return false, // File has no extension, so cannot match.
                }
            }

            // --- Filter by minimum size (--min-size) ---
            if let Some(min_allowed_size) = self.config.min_size {
                if node.size < min_allowed_size {
                    return false; // File is too small.
                }
            }

            // --- Filter by maximum size (--max-size) ---
            if let Some(max_allowed_size) = self.config.max_size {
                if node.size > max_allowed_size {
                    return false; // File is too large.
                }
            }
        } // End of file-only filters

        // --- Date filters (apply to both files and directories based on their modification time) ---
        // --- Filter by newer_than date (--newer-than) ---
        if let Some(min_modification_date) = self.config.newer_than {
            if node.modified < min_modification_date {
                return false; // Entry is older than required.
            }
        }

        // --- Filter by older_than date (--older-than) ---
        if let Some(max_modification_date) = self.config.older_than {
            if node.modified > max_modification_date {
                return false; // Entry is newer than allowed.
            }
        }

        // If all applicable filters passed (or no filters were active for a category), include the node.
        true
    }

    /// ## `determine_file_type` (Helper for `process_entry`)
    ///
    /// Examines `fs::Metadata` to determine a more specific `FileType`
    /// than just `is_dir` or `is_file`. On Unix, this can identify symlinks,
    /// sockets, FIFOs, block/char devices, and executables (by permission).
    /// On non-Unix, it's simpler (dir, symlink, or regular file).
    fn determine_file_type(&self, metadata: &fs::Metadata) -> FileType {
        #[cfg(unix)] // Unix-specific detailed file type detection
        {
            use std::os::unix::fs::FileTypeExt; // For is_socket, is_fifo, etc.
            let ft = metadata.file_type(); // Get the rich FileType from metadata.

            if ft.is_dir() {
                FileType::Directory
            } else if ft.is_symlink() {
                // Check symlink before other types, as it can point to them.
                FileType::Symlink
            } else if ft.is_socket() {
                FileType::Socket
            } else if ft.is_fifo() {
                // Named pipe
                FileType::Pipe
            } else if ft.is_block_device() {
                FileType::BlockDevice
            } else if ft.is_char_device() {
                FileType::CharDevice
            // Check for executable permission (any of user, group, other execute bits are set).
            // This applies to regular files that are not dirs, symlinks, or other special types.
            } else if ft.is_file() && (metadata.permissions().mode() & 0o111 != 0) {
                FileType::Executable
            } else {
                // If none of the above, it's a regular (non-executable) file.
                FileType::RegularFile
            }
        }

        #[cfg(not(unix))] // Simpler detection for non-Unix platforms
        {
            if metadata.is_dir() {
                FileType::Directory
            } else if metadata.file_type().is_symlink() {
                // `is_symlink()` is part of stable `fs::FileType`
                FileType::Symlink
            } else {
                // No easy cross-platform way to check executable bit without external crates or OS-specific calls.
                // So, on non-Unix, we don't distinguish Executable from RegularFile here.
                FileType::RegularFile
            }
        }
    }

    // --- Platform-Dependent Metadata Helpers ---
    // These provide a consistent way to get permissions, UID, and GID,
    // with sensible defaults for non-Unix systems where these concepts might not directly apply
    // or be easily accessible via standard Rust fs::Metadata.

    #[cfg(unix)]
    fn get_permissions(metadata: &fs::Metadata) -> u32 {
        // On Unix, get the mode and mask it to get the permission bits (e.g., 0o755).
        metadata.permissions().mode() & 0o777
    }
    #[cfg(not(unix))]
    fn get_permissions(_metadata: &fs::Metadata) -> u32 {
        0o755 // A common default permission (rwxr-xr-x) for non-Unix.
    }

    #[cfg(unix)]
    fn get_uid(metadata: &fs::Metadata) -> u32 {
        metadata.uid() // Get User ID from metadata.
    }
    #[cfg(not(unix))]
    fn get_uid(_metadata: &fs::Metadata) -> u32 {
        1000 // Common default UID placeholder for non-Unix.
    }

    #[cfg(unix)]
    fn get_gid(metadata: &fs::Metadata) -> u32 {
        metadata.gid() // Get Group ID from metadata.
    }
    #[cfg(not(unix))]
    fn get_gid(_metadata: &fs::Metadata) -> u32 {
        0
    }

    /// Apply sorting and optional top-N limit to the results
    fn apply_sorting_and_limit(&self, mut nodes: Vec<FileNode>) -> Vec<FileNode> {
        // If no sort field specified, return as-is
        let sort_field = match &self.config.sort_field {
            Some(field) => field,
            None => return nodes,
        };

        // Sort based on the field
        match sort_field.as_str() {
            "name" | "a-to-z" => {
                // Sort by name alphabetically (A to Z)
                nodes.sort_by(|a, b| {
                    let name_a = a.path.file_name().unwrap_or_default().to_string_lossy();
                    let name_b = b.path.file_name().unwrap_or_default().to_string_lossy();
                    name_a.cmp(&name_b)
                });
            }
            "z-to-a" => {
                // Sort by name reverse alphabetically (Z to A)
                nodes.sort_by(|a, b| {
                    let name_a = a.path.file_name().unwrap_or_default().to_string_lossy();
                    let name_b = b.path.file_name().unwrap_or_default().to_string_lossy();
                    name_b.cmp(&name_a)
                });
            }
            "size" | "largest" => {
                // Sort by size descending (largest first)
                nodes.sort_by(|a, b| b.size.cmp(&a.size));
            }
            "smallest" => {
                // Sort by size ascending (smallest first)
                nodes.sort_by(|a, b| a.size.cmp(&b.size));
            }
            "date" | "newest" => {
                // Sort by modification time descending (newest first)
                nodes.sort_by(|a, b| b.modified.cmp(&a.modified));
            }
            "oldest" => {
                // Sort by modification time ascending (oldest first)
                nodes.sort_by(|a, b| a.modified.cmp(&b.modified));
            }
            "type" => {
                // Sort by file extension, then by name
                nodes.sort_by(|a, b| {
                    let ext_a = a.path.extension().unwrap_or_default().to_string_lossy();
                    let ext_b = b.path.extension().unwrap_or_default().to_string_lossy();
                    match ext_a.cmp(&ext_b) {
                        std::cmp::Ordering::Equal => {
                            let name_a = a.path.file_name().unwrap_or_default().to_string_lossy();
                            let name_b = b.path.file_name().unwrap_or_default().to_string_lossy();
                            name_a.cmp(&name_b)
                        }
                        other => other,
                    }
                });
            }
            _ => {
                // Unknown sort field, don't sort
                eprintln!("Warning: Unknown sort field '{}', ignoring", sort_field);
            }
        }

        // Apply top-N limit if specified
        if let Some(limit) = self.config.top_n {
            nodes.truncate(limit);
        }

        nodes
    }
} // end impl Scanner

/// # `parse_size` - The Universal Translator for Sizes
///
/// This handy function takes something a human understands, like "2.5M", and
/// translates it into something a computer understands (2,621,440 bytes).
/// It's like having a Babel fish for file sizes. Why should we have to do
/// that math when the computer can do it for us?
pub fn parse_size(size_str: &str) -> Result<u64> {
    let size_str = size_str.trim().to_uppercase();
    if size_str.is_empty() {
        return Err(anyhow::anyhow!("Empty size string"));
    }

    // Find the first alphabetic character which marks the start of the unit.
    let unit_start_index = size_str
        .find(|c: char| c.is_alphabetic())
        .unwrap_or(size_str.len());
    let (num_part_str, unit_part) = size_str.split_at(unit_start_index);

    // Trim any space from the number part before parsing.
    let num_part_str = num_part_str.trim();

    if num_part_str.is_empty() {
        return Err(anyhow::anyhow!("Missing number for size string"));
    }

    let num: f64 = match num_part_str.parse() {
        Ok(n) => n,
        Err(e) => return Err(anyhow::anyhow!("Invalid number '{}': {}", num_part_str, e)),
    };

    // Check for negative numbers.
    if num.is_sign_negative() {
        return Err(anyhow::anyhow!("Size cannot be negative: {}", num));
    }

    let multiplier = match unit_part {
        "K" | "KB" => 1024.0,
        "M" | "MB" => 1024.0 * 1024.0,
        "G" | "GB" => 1024.0 * 1024.0 * 1024.0,
        "T" | "TB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        "B" | "" => 1.0,
        _ => return Err(anyhow::anyhow!("Invalid size unit: '{}'", unit_part)),
    };

    Ok((num * multiplier) as u64)
}

// --- Unit Tests: Ensuring Our Scanner Behaves ---
// Aye, even the most brilliant code needs tests to keep it honest!
// These tests cover some basic functionality of the scanner.
#[cfg(test)]
mod tests {
    use super::*; // Import everything from the parent module (scanner.rs).

    #[test]
    fn test_parse_size_valid_inputs() {
        assert_eq!(parse_size("100").unwrap(), 100);
        assert_eq!(parse_size("100B").unwrap(), 100);
        assert_eq!(parse_size("1k").unwrap(), 1024);
        assert_eq!(parse_size("1K").unwrap(), 1024);
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("2.5M").unwrap(), (2.5 * 1024.0 * 1024.0) as u64);
        assert_eq!(parse_size("1GB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(
            parse_size("0.5T").unwrap(),
            (0.5 * 1024.0 * 1024.0 * 1024.0 * 1024.0) as u64
        );
        assert_eq!(parse_size("  2 MB  ").unwrap(), 2 * 1024 * 1024); // Test with whitespace
    }

    #[test]
    fn test_parse_size_invalid_inputs() {
        assert!(parse_size("100X").is_err());
        assert!(parse_size("garbage").is_err());
        assert!(parse_size("-100M").is_err());
        assert!(parse_size("1..5K").is_err());
    }

    #[test]
    fn test_parse_size_zero_and_empty() {
        assert_eq!(parse_size("0").unwrap(), 0);
        assert!(parse_size("").is_err());
        assert!(parse_size("  ").is_err());
    }

    // Basic test for Scanner creation. More comprehensive tests would involve
    // creating a temporary directory structure and verifying scan results.
    #[test]
    fn test_scanner_creation_defaults() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = ScannerConfig {
            max_depth: 5,
            follow_symlinks: false,
            respect_gitignore: true,
            show_hidden: false,
            show_ignored: false,
            find_pattern: None,
            file_type_filter: None,
            entry_type_filter: None,
            min_size: None,
            max_size: None,
            newer_than: None,
            older_than: None,
            use_default_ignores: true,
            search_keyword: None,
            show_filesystems: false,
            sort_field: None,
            top_n: None,
            include_line_content: false,
        };
        let scanner_result = Scanner::new(temp_dir.path(), config);
        assert!(scanner_result.is_ok());
    }
}
